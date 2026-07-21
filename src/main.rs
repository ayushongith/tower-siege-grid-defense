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
mod sfx;
mod utils;

use bevy::prelude::*;

use components::{
    HitEffect, PathFollower, Projectile, SelectionRing, Tower, TowerEditTarget, TowerLevel,
    TowerSelection,
};
use plugins::{
    wave_plugin::WaveAnnouncement, EnemyPlugin, InputPlugin, MapPlugin, ProjectilePlugin,
    TowerPlugin, VisualPlugin, WavePlugin,
};
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
    GameOver,
    Victory,
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
        .add_plugins((MapPlugin, EnemyPlugin, InputPlugin, TowerPlugin, ProjectilePlugin, VisualPlugin, WavePlugin))
        // --- Bootstrap systems ----------------------------------------------
        .add_event::<sfx::SfxRequest>()
        .add_systems(Startup, (setup_camera, setup_menu_ui, setup_game_over_ui, setup_victory_ui, sfx::setup_sfx))
        .add_systems(OnEnter(AppState::Playing), hide_menu_ui)
        .add_systems(OnEnter(AppState::MainMenu), show_menu_ui)
        .add_systems(OnEnter(AppState::Paused), show_paused_banner)
        .add_systems(OnExit(AppState::Paused), hide_paused_banner)
.add_systems(OnEnter(AppState::GameOver), (show_game_over_ui, cleanup_gameplay, sfx::play_game_over_sfx))
.add_systems(OnExit(AppState::GameOver), hide_game_over_ui)
.add_systems(OnEnter(AppState::Victory), (show_victory_ui, cleanup_gameplay, sfx::play_victory_sfx))
        .add_systems(OnExit(AppState::Victory), hide_victory_ui)
        .add_systems(
            Update,
            (
                update_hud,
                update_wave_announcement,
                detect_game_over,
                detect_victory,
                sfx::handle_sfx_requests,
            )
                .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
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

#[derive(Component)]
struct WaveAnnouncementText;

#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
struct VictoryUi;

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
                Text::new("Day 4 — Upgrades & Visuals"),
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
    "SPACE/1/2/3 spawn · 4=Arrow 5=Cannon · LMB place/select · U upgrade · S sell · ESC pause",
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

    // Wave announcement (shown briefly at wave start/complete).
    commands.spawn((
        WaveAnnouncementText,
        Text::new(""),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.90, 0.40)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            left: Val::Percent(50.0),
            ..default()
        },
        Name::new("WaveAnnouncement"),
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
    edit_target: Res<TowerEditTarget>,
    towers: Query<(&Tower, &TowerLevel)>,
    mut hud: Query<(&mut Text, &mut Visibility), With<HudText>>,
) {
    for (mut text, mut vis) in &mut hud {
        *vis = Visibility::Visible;
        let mut hints = String::new();

        if let Some(t) = tower_sel.selected {
            hints.push_str(&format!(" [placing {:?} tower - click buildable tile]", t));
        } else if let Some(entity) = edit_target.entity {
            if let Ok((tower, level)) = towers.get(entity) {
                hints.push_str(&format!(
                    " [select {:?} Lv{} - U upgrade({}g) | S sell({}g)]",
                    tower.tower_type,
                    level.level,
                    crate::plugins::tower_plugin::upgrade_cost(level.level, tower.tower_type.cost()),
                    (level.total_invested as f32 * 0.5).round() as u32,
                ));
            }
        }

        let wave_info = if waves.total_enemies > 0 {
            format!(
                "Enemies: {}/{}",
                (waves.spawn_index as u32).min(waves.total_enemies),
                waves.total_enemies
            )
        } else {
            "Enemies: 0".to_string()
        };
        *text = Text::new(format!(
            "Gold: {}   Lives: {}   Wave: {}   {}   Spawned: {}{}",
            stats.gold, stats.lives, waves.current_wave, wave_info, waves.enemies_spawned, hints
        ));
    }
}

fn update_wave_announcement(
    ann: Res<WaveAnnouncement>,
    mut query: Query<&mut Text, With<WaveAnnouncementText>>,
) {
    for mut text in &mut query {
        text.0 = ann.text.clone();
    }
}

// ---------------------------------------------------------------------------
// Game Over / Victory UI
// ---------------------------------------------------------------------------

fn setup_game_over_ui(mut commands: Commands) {
    commands
        .spawn((
            GameOverUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.05, 0.05, 0.88)),
            Visibility::Hidden,
            Name::new("GameOverRoot"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont { font_size: 64.0, ..default() },
                TextColor(Color::srgb(0.95, 0.20, 0.20)),
            ));
            parent.spawn((
                Text::new("Press ENTER or SPACE to return to menu"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
        });
}

fn show_game_over_ui(mut query: Query<&mut Visibility, With<GameOverUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Visible;
    }
}

fn hide_game_over_ui(mut query: Query<&mut Visibility, With<GameOverUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

fn setup_victory_ui(mut commands: Commands) {
    commands
        .spawn((
            VictoryUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.15, 0.05, 0.88)),
            Visibility::Hidden,
            Name::new("VictoryRoot"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("VICTORY!"),
                TextFont { font_size: 64.0, ..default() },
                TextColor(Color::srgb(0.30, 0.90, 0.30)),
            ));
            parent.spawn((
                Text::new("All waves survived! You defended the base."),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
            parent.spawn((
                Text::new("Press ENTER or SPACE to return to menu"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));
        });
}

fn show_victory_ui(mut query: Query<&mut Visibility, With<VictoryUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Visible;
    }
}

fn hide_victory_ui(mut query: Query<&mut Visibility, With<VictoryUi>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

/// Despawn all gameplay entities when game ends.
fn cleanup_gameplay(
    mut commands: Commands,
    enemies: Query<Entity, With<PathFollower>>,
    towers: Query<Entity, (With<Tower>, Without<PathFollower>)>,
    projectiles: Query<Entity, With<Projectile>>,
    hit_effects: Query<Entity, With<HitEffect>>,
    selection_rings: Query<Entity, With<SelectionRing>>,
) {
    for e in &enemies { commands.entity(e).despawn_recursive(); }
    for e in &towers { commands.entity(e).despawn_recursive(); }
    for e in &projectiles { commands.entity(e).despawn(); }
    for e in &hit_effects { commands.entity(e).despawn(); }
    for e in &selection_rings { commands.entity(e).despawn(); }
}

/// Transition to GameOver when lives reach 0.
fn detect_game_over(
    stats: Res<GameStats>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if stats.lives == 0 {
        info!("Game Over — lives depleted");
        next_state.set(AppState::GameOver);
    }
}

/// Transition to Victory after clearing enough waves.
fn detect_victory(
    waves: Res<WaveManager>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Victory after clearing wave 10 (after it enters Idle from Complete)
    if waves.current_wave >= 10 && waves.state == crate::resources::WaveState::Idle && waves.enemies_alive == 0 {
        info!("Victory — all 10 waves cleared!");
        next_state.set(AppState::Victory);
    }
}
