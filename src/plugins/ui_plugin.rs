use bevy::prelude::*;

use crate::components::TowerType;
use crate::resources::{GameStats, WaveManager};
use crate::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (update_stats_panel, update_wave_info_panel, handle_tower_shop_clicks)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(Component)]
struct TowerShopButton {
    tower_type: TowerType,
}

#[derive(Component)]
struct WaveInfoPanel;

#[derive(Component)]
struct StatsPanel;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        StatsPanel,
        Text::new("Gold: 0  Lives: 0"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.90, 0.90, 0.90)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Name::new("StatsPanel"),
    ));

    commands.spawn((
        WaveInfoPanel,
        Text::new("Wave: 0/10 | Enemies: 0/0"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.85, 0.85, 0.65)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(10.0),
            ..default()
        },
        Name::new("WaveInfoPanel"),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(150.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            Name::new("TowerShop"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tower Shop"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));

            for tower_type in TowerType::ALL {
                parent
                    .spawn((
                        TowerShopButton { tower_type },
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(tower_type.color()),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("{} ({})", tower_type.label(), tower_type.cost())),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::BLACK),
                        ));
                    });
            }
        });
}

fn update_stats_panel(stats: Res<GameStats>, mut query: Query<&mut Text, With<StatsPanel>>) {
    for mut text in &mut query {
        text.0 = format!("Gold: {}  Lives: {}", stats.gold, stats.lives);
    }
}

fn update_wave_info_panel(waves: Res<WaveManager>, mut query: Query<&mut Text, With<WaveInfoPanel>>) {
    for mut text in &mut query {
        text.0 = format!(
            "Wave: {}/{} | Enemies: {}/{}",
            waves.current_wave,
            waves.campaign_victory_wave,
            waves.enemies_spawned,
            waves.total_enemies
        );
    }
}

fn handle_tower_shop_clicks(
    mut interactions: Query<(&Interaction, &TowerShopButton), (Changed<Interaction>, With<Button>)>,
    mut tower_sel: ResMut<crate::components::TowerSelection>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            tower_sel.selected = Some(button.tower_type);
            info!("Selected tower from UI: {:?}", button.tower_type);
        }
    }
}
