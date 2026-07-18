//! Tower Siege: Grid Defense — Day 1 foundation.
//!
//! Day 1 milestone:
//! - Visible 15×10 grid with a winding path
//! - Spawn / Base markers
//! - Space spawns enemies that walk the full path
//! - ESC pauses; MainMenu → Playing via Enter/Space
//!
//! Architecture notes:
//! - Plugins own domains (map, enemy, input)
//! - `AppState` gates which systems run
//! - Resources hold singleton data (`Map`, `GameStats`, `WaveManager`)
//! - Components hold per-entity data

mod components;
mod plugins;
mod resources;
mod utils;

use bevy::prelude::*;

use components::TowerSelection;
use plugins::{EnemyPlugin, InputPlugin, MapPlugin, ProjectilePlugin, TowerPlugin};
use resources::{GameStats, WaveManager};

// ---------------------------------------------------------------------------
// Application states
// ---------------------------------------------------------------------------

/// High-level flow control. Systems use `run_if(in_state(...))` so pause is
/// free (Playing systems simply stop scheduling) without scattered bools.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

fn main() {
    App::new()
        // --- Window / engine defaults ---------------------------------------
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tower Siege: Grid Defense".into(),
                        resolution: (1280., 720.).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                // Nearest filtering keeps chunky 2D tiles crisp when scaled.
                .set(ImagePlugin::default_nearest()),
        )
        // Dark green-gray battlefield backdrop.
        .insert_resource(ClearColor(Color::srgb(0.12, 0.16, 0.12)))
        // --- Global resources ------------------------------------------------
        .init_resource::<GameStats>()
        .init_resource::<WaveManager>()
        // --- States ----------------------------------------------------------
        .init_state::<AppState>()
        // --- Domain plugins --------------------------------------------------
        .add_plugins((MapPlugin, EnemyPlugin, InputPlugin, TowerPlugin, ProjectilePlugin))
        // --- Bootstrap systems ----------------------------------------------
        .add_systems(Startup, (setup_camera, setup_menu_ui))
        .add_systems(OnEnter(AppState::Playing), hide_menu_ui)
        .add_systems(OnEnter(AppState::MainMenu), show_menu_ui)
        .add_systems(OnEnter(AppState::Paused), show_paused_banner)
        .add_systems(OnExit(AppState::Paused), hide_paused_banner)
        .add_systems(
            Update,
            update_hud.run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
        )
        .run();
}

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

/// Fixed orthographic camera centered on the map origin.
/// 15×10 tiles × 64px ≈ 960×640; 1280×720 leaves comfortable padding.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        // Explicit identity transform at origin — map is already centered.
        Transform::from_xyz(0.0, 0.0, 1000.0),
        Name::new("MainCamera"),
    ));
}

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct PauseBanner;

#[derive(Component)]
struct HudText;

fn setup_menu_ui(mut commands: Commands) {
    // Root UI node covering the window.
    commands
        .spawn((
            MenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.05, 0.82)),
            Name::new("MenuRoot"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tower Siege: Grid Defense"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.92, 0.80)),
            ));
            parent.spawn((
                Text::new("Day 2 — Towers & Combat"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.78, 0.65)),
            ));
            parent.spawn((
                Text::new("Press ENTER or SPACE to start"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
            parent.spawn((
                Text::new(
                    "SPACE/1/2/3 spawn · 4=Arrow(50g) · 5=Cannon(100g) · LMB place tower · ESC pause",
                ),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.70, 0.65)),
            ));
        });

    // HUD (hidden until Playing via text content updates; always present).
    commands.spawn((
        HudText,
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.90, 0.90, 0.90)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(16.0),
            ..default()
        },
        Visibility::Hidden,
        Name::new("HudText"),
    ));

    // Pause banner (toggled on pause enter/exit).
    commands
        .spawn((
            PauseBanner,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            Visibility::Hidden,
            Name::new("PauseBanner"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED — press ESC to resume"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
        });
}

fn hide_menu_ui(mut query: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

fn show_menu_ui(mut query: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Visible;
    }
}

fn show_paused_banner(mut query: Query<&mut Visibility, With<PauseBanner>>) {
    for mut vis in &mut query {
        *vis = Visibility::Visible;
    }
}

fn hide_paused_banner(mut query: Query<&mut Visibility, With<PauseBanner>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

fn update_hud(
    stats: Res<GameStats>,
    waves: Res<WaveManager>,
    tower_sel: Res<TowerSelection>,
    mut hud: Query<(&mut Text, &mut Visibility), With<HudText>>,
) {
    for (mut text, mut vis) in &mut hud {
        *vis = Visibility::Visible;
        let tower_hint = match tower_sel.selected {
            Some(t) => format!(" [placing {:?} tower - click buildable tile]", t),
            None => "".to_string(),
        };
        *text = Text::new(format!(
            "Gold: {}   Lives: {}   Wave: {}   Spawned: {}{}",
            stats.gold, stats.lives, stats.current_wave, waves.enemies_spawned, tower_hint
        ));
    }
}
