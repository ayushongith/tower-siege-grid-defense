use bevy::prelude::*;

use crate::components::{Tower, TowerEditTarget, TowerLevel, TowerSelection, TowerType};
use crate::plugins::tower_plugin;
use crate::resources::{GameStats, LevelManager, Map, TileType, WaveManager};
use crate::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    update_stats_panel,
                    update_toolbar_buttons,
                    update_tower_info_panel,
                    update_wave_progress,
                    handle_ui_upgrade_sell,
                )
                    .run_if(in_state(AppState::Playing)),
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

#[derive(Component)]
struct TowerInfoPanel;

#[derive(Component)]
struct TowerInfoTitle;

#[derive(Component)]
struct TowerInfoStats;

#[derive(Component)]
struct UpgradeBtn;

#[derive(Component)]
struct SellBtn;

#[derive(Component)]
struct WaveProgressBar;

#[derive(Component)]
struct WaveProgressFill;

#[derive(Component)]
struct WaveProgressText;

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
                    TowerInfoPanel,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(10.0),
                        bottom: Val::Px(90.0),
                        width: Val::Px(220.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.92)),
                    Visibility::Hidden,
                    Name::new("TowerInfoPanel"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TowerInfoTitle,
                        Text::new(""),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    parent.spawn((
                        TowerInfoStats,
                        Text::new(""),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                    ));
                    parent
                        .spawn((
                            UpgradeBtn,
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(34.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.20, 0.60, 0.30)),
                            Name::new("UpgradeButton"),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Upgrade"),
                                TextFont { font_size: 15.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                    parent
                        .spawn((
                            SellBtn,
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(34.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.70, 0.20, 0.20)),
                            Name::new("SellButton"),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Sell"),
                                TextFont { font_size: 15.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            parent.spawn((
                WaveProgressText,
                Text::new("Wave 0/0 | Remaining: 0"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.65)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(90.0),
                    left: Val::Percent(20.0),
                    width: Val::Percent(60.0),
                    ..default()
                },
                Name::new("WaveProgressText"),
            ));

            parent
                .spawn((
                    WaveProgressBar,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(80.0),
                        left: Val::Percent(20.0),
                        width: Val::Percent(60.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.6)),
                    Name::new("WaveProgressBar"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        WaveProgressFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.30, 0.85, 0.40)),
                        Name::new("WaveProgressFill"),
                    ));
                });

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

fn update_tower_info_panel(
    edit_target: Res<TowerEditTarget>,
    towers: Query<(&Tower, &TowerLevel)>,
    stats: Res<GameStats>,
    mut panel: Query<&mut Visibility, With<TowerInfoPanel>>,
    mut title: Query<&mut Text, (With<TowerInfoTitle>, Without<TowerInfoStats>)>,
    mut info_text: Query<&mut Text, (With<TowerInfoStats>, Without<TowerInfoTitle>)>,
    mut upgrade_btn: Query<&mut BackgroundColor, (With<UpgradeBtn>, Without<SellBtn>)>,
    mut upgrade_label: Query<&mut Text, (With<UpgradeBtn>, Without<SellBtn>)>,
) {
    if let Some(entity) = edit_target.entity {
        if let Ok((tower, tower_lv)) = towers.get(entity) {
            for mut vis in &mut panel {
                *vis = Visibility::Visible;
            }
            for mut t in &mut title {
                t.0 = format!("{:?} Tower Lv{}/{}", tower.tower_type, tower_lv.level, tower_lv.max_level);
            }
            for mut t in &mut info_text {
                t.0 = format!(
                    "Dmg: {:.0}  Range: {:.0}\nRate: {:.1}s  MaxLv: {}",
                    tower.damage, tower.range, tower.fire_rate, tower_lv.max_level,
                );
            }

            let is_max = tower_lv.level >= tower_lv.max_level;
            let cost = tower_plugin::upgrade_cost(tower_lv.level, tower.tower_type.cost());
            let can_afford = stats.gold >= cost;

            for mut bg in &mut upgrade_btn {
                *bg = if is_max {
                    BackgroundColor(Color::srgba(0.30, 0.30, 0.30, 0.8))
                } else if can_afford {
                    BackgroundColor(Color::srgb(0.20, 0.60, 0.30))
                } else {
                    BackgroundColor(Color::srgba(0.40, 0.20, 0.15, 0.8))
                };
            }
            for mut t in &mut upgrade_label {
                t.0 = if is_max {
                    "MAX LEVEL".to_string()
                } else {
                    format!("Upgrade ({}g)", cost)
                };
            }
            return;
        }
    }

    for mut vis in &mut panel {
        *vis = Visibility::Hidden;
    }
}

fn update_wave_progress(
    waves: Res<WaveManager>,
    level: Res<LevelManager>,
    mut fill: Query<&mut Node, With<WaveProgressFill>>,
    mut text: Query<&mut Text, With<WaveProgressText>>,
) {
    let target = level.target_wave();
    let progress = if target > 0 {
        (waves.current_wave as f32 / target as f32).min(1.0)
    } else {
        0.0
    };

    for mut node in &mut fill {
        node.width = Val::Percent(progress * 100.0);
    }

    let remaining = if waves.total_enemies > 0 {
        waves.total_enemies.saturating_sub(waves.enemies_spawned) + waves.enemies_alive
    } else {
        0
    };
    for mut t in &mut text {
        t.0 = format!("Wave {}/{} | Remaining: {}", waves.current_wave, target, remaining);
    }
}

fn handle_ui_upgrade_sell(
    buttons: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    upgrade_btn_q: Query<Entity, (With<UpgradeBtn>, Without<SellBtn>)>,
    sell_btn_q: Query<Entity, (With<SellBtn>, Without<UpgradeBtn>)>,
    mut stats: ResMut<GameStats>,
    mut edit_target: ResMut<TowerEditTarget>,
    mut towers: Query<(&mut Tower, &mut TowerLevel, &crate::components::GridPosition)>,
    mut map: ResMut<Map>,
    mut commands: Commands,
) {
    let upgrade_entity = upgrade_btn_q.iter().next();
    let sell_entity = sell_btn_q.iter().next();

    for (interaction, entity) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if Some(entity) == upgrade_entity {
            let Some(target) = edit_target.entity else { continue };
            let Ok((mut tower, mut level, _grid)) = towers.get_mut(target) else {
                edit_target.entity = None;
                continue;
            };
            if level.level >= level.max_level { continue; }
            let cost = tower_plugin::upgrade_cost(level.level, tower.tower_type.cost());
            if stats.gold < cost { continue; }
            stats.gold -= cost;
            level.total_invested += cost;
            level.level += 1;
            tower.damage += tower.tower_type.damage() * 0.25;
            tower.range += tower.tower_type.range() * 0.10;
            let new_rate = (tower.fire_rate * 0.90).max(0.3);
            tower.fire_rate = new_rate;
            tower.cooldown = Timer::from_seconds(new_rate, TimerMode::Repeating);
            info!("Upgraded tower to Lv{}", level.level);
        }

        if Some(entity) == sell_entity {
            let Some(target) = edit_target.entity else { continue };
            let Ok((_tower, level, grid)) = towers.get(target) else {
                edit_target.entity = None;
                continue;
            };
            let refund = (level.total_invested as f32 * 0.50).round() as u32;
            stats.gold += refund;
            map.set_tile(grid.col, grid.row, TileType::Buildable);
            commands.entity(target).despawn_recursive();
            edit_target.entity = None;
            info!("Sold tower for {}g", refund);
        }
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
