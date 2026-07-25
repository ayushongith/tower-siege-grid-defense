use bevy::prelude::*;

use crate::components::{TowerEditTarget, TowerSelection, TowerType};
use crate::resources::{GameStats, LevelManager, WaveManager};
use crate::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (update_stats_panel, update_toolbar_buttons).run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::Playing), show_game_ui)
            .add_systems(OnExit(AppState::Playing), hide_game_ui);
    }
}

#[derive(Component)]
struct GameUiRoot;

#[derive(Component)]
struct StatsPanel;

#[derive(Component)]
struct TowerShopButton {
    tower_type: TowerType,
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            GameUiRoot,
            Visibility::Hidden,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Name::new("GameUiRoot"),
        ))
        .with_children(|parent| {
            parent.spawn((
                StatsPanel,
                Text::new("Gold: 0  Lives: 0  Lv1  Wave: 0/10  Kills: 0"),
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

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        height: Val::Px(72.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(8.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.10, 0.90)),
                    Name::new("BottomToolbar"),
                ))
                .with_children(|parent| {
                    for tt in TowerType::ALL {
                        parent
                            .spawn((
                                TowerShopButton { tower_type: tt },
                                Button,
                                Node {
                                    width: Val::Px(100.0),
                                    height: Val::Px(56.0),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.20, 0.9)),
                                Name::new(format!("TowerBtn_{:?}", tt)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(tt.label()),
                                    TextFont { font_size: 14.0, ..default() },
                                    TextColor(tt.color()),
                                ));
                                parent.spawn((
                                    Text::new(format!("{}g", tt.cost())),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(Color::srgb(0.90, 0.85, 0.40)),
                                ));
                                parent.spawn((
                                    Text::new(format!("[{}]", tt.hotkey_char())),
                                    TextFont { font_size: 10.0, ..default() },
                                    TextColor(Color::srgb(0.50, 0.50, 0.50)),
                                ));
                            });
                    }
                });
        });
}

fn show_game_ui(mut query: Query<&mut Visibility, With<GameUiRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Visible;
    }
}

fn hide_game_ui(mut query: Query<&mut Visibility, With<GameUiRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

fn update_stats_panel(
    stats: Res<GameStats>,
    waves: Res<WaveManager>,
    level: Res<LevelManager>,
    mut query: Query<&mut Text, With<StatsPanel>>,
) {
    for mut text in &mut query {
        text.0 = format!(
            "Gold: {}  Lives: {}  Lv{}  Wave: {}/{}  Kills: {}",
            stats.gold, stats.lives, level.current_level,
            waves.current_wave, waves.campaign_victory_wave, stats.kills,
        );
    }
}

fn update_toolbar_buttons(
    mut tower_sel: ResMut<TowerSelection>,
    mut edit_target: ResMut<TowerEditTarget>,
    stats: Res<GameStats>,
    mut buttons: Query<
        (&Interaction, &TowerShopButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, btn, mut bg) in &mut buttons {
        if *interaction == Interaction::Pressed {
            if tower_sel.selected == Some(btn.tower_type) {
                tower_sel.selected = None;
            } else {
                tower_sel.selected = Some(btn.tower_type);
                edit_target.entity = None;
            }
        }

        let is_selected = tower_sel.selected == Some(btn.tower_type);
        let can_afford = stats.gold >= btn.tower_type.cost();
        *bg = if is_selected {
            BackgroundColor(Color::srgba(0.30, 0.70, 0.40, 0.95))
        } else if can_afford {
            BackgroundColor(Color::srgba(0.15, 0.15, 0.20, 0.9))
        } else {
            BackgroundColor(Color::srgba(0.08, 0.08, 0.10, 0.7))
        };
    }
}
