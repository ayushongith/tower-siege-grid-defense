use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{EnemyType, GridPosition, TowerEditTarget, TowerSelection, TowerType};
use crate::plugins::enemy_plugin::SpawnEnemyRequest;
use crate::plugins::tower_plugin::{find_tower_placement, spawn_tower};
use crate::resources::{GameStats, Map, WaveManager};
use crate::utils::world_to_grid;
use crate::AppState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TowerSelection>()
            .add_systems(
                Update,
                (
                    handle_main_menu_input.run_if(in_state(AppState::MainMenu)),
                    (
                        handle_escape_and_tower_select,
                        handle_spawn_keys,
                        handle_mouse_playing,
                    )
                        .chain()
                        .run_if(in_state(AppState::Playing)),
                    handle_paused_input.run_if(in_state(AppState::Paused)),
                ),
            );
    }
}

fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        info!("MainMenu → Playing");
        next_state.set(AppState::Playing);
    }
}

fn handle_escape_and_tower_select(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut tower_sel: ResMut<TowerSelection>,
    mut edit_target: ResMut<TowerEditTarget>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if tower_sel.selected.is_some() {
            tower_sel.selected = None;
            info!("Tower selection cleared");
        } else if edit_target.entity.is_some() {
            edit_target.entity = None;
            info!("Tower edit deselected");
        } else {
            info!("Playing → Paused");
            next_state.set(AppState::Paused);
        }
    }

    if keys.just_pressed(KeyCode::Digit4) {
        tower_sel.selected = Some(TowerType::Arrow);
        edit_target.entity = None;
        info!("Selected Arrow tower (cost: {})", TowerType::Arrow.cost());
    }
    if keys.just_pressed(KeyCode::Digit5) {
        tower_sel.selected = Some(TowerType::Cannon);
        edit_target.entity = None;
        info!("Selected Cannon tower (cost: {})", TowerType::Cannon.cost());
    }
}

fn handle_spawn_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut spawn_request: ResMut<SpawnEnemyRequest>,
    mut waves: ResMut<WaveManager>,
) {
    if keys.just_pressed(KeyCode::Space) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Normal;
        waves.enemies_spawned += 1;
        waves.enemies_alive += 1;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Normal;
        waves.enemies_spawned += 1;
        waves.enemies_alive += 1;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Fast;
        waves.enemies_spawned += 1;
        waves.enemies_alive += 1;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Tank;
        waves.enemies_spawned += 1;
        waves.enemies_alive += 1;
    }
}

fn handle_mouse_playing(
    mouse: Res<ButtonInput<MouseButton>>,
    tower_sel: Res<TowerSelection>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map: ResMut<Map>,
    mut stats: ResMut<GameStats>,
    mut edit_target: ResMut<TowerEditTarget>,
    towers: Query<(Entity, &GridPosition)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Clear tower edit selection on any click.
    edit_target.entity = None;

    if let Some(tower_type) = tower_sel.selected {
        if let Some((col, row)) = find_tower_placement(&windows, &camera_q, &*map) {
            spawn_tower(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut map,
                &mut stats,
                col,
                row,
                tower_type,
            );
        }
    } else {
        // Click on Occupied tile → select existing tower for upgrade/sell.
        if let Some((col, row)) = find_tower_placement(&windows, &camera_q, &*map) {
            let tile = map.get_tile(col, row);
            if tile == Some(crate::resources::TileType::Occupied) {
                for (entity, grid) in &towers {
                    if grid.col == col && grid.row == row {
                        edit_target.entity = Some(entity);
                        edit_target.col = col;
                        edit_target.row = row;
                        info!("Selected tower at ({col}, {row})");
                        return;
                    }
                }
            }
            debug_print_grid_click(&windows, &camera_q, &*map);
        }
    }
}

fn handle_paused_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        info!("Paused → Playing");
        next_state.set(AppState::Playing);
    }
}

fn debug_print_grid_click(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
    map: &Map,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    let Ok(world) = camera.viewport_to_world_2d(cam_transform, cursor) else {
        warn!("Failed to convert cursor to world position");
        return;
    };

    match world_to_grid(world, map.width, map.height) {
        Some((col, row)) => {
            let tile = map.get_tile(col, row);
            info!("Mouse click → grid ({col}, {row}), tile={tile:?}, world={world}");
        }
        None => {
            info!("Mouse click outside map at world={world}");
        }
    }
}