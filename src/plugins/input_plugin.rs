//! InputPlugin — keyboard / mouse → game intents for Day 1.
//!
//! We translate raw input into either:
//! - `NextState<AppState>` transitions (menu / play / pause)
//! - `SpawnEnemyRequest` flags consumed by EnemyPlugin
//!
//! Keeping input separate from enemy mesh spawning preserves a clean boundary
//! between "what the player asked for" and "how the world fulfills it".

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::EnemyType;
use crate::plugins::enemy_plugin::SpawnEnemyRequest;
use crate::resources::Map;
use crate::utils::world_to_grid;
use crate::AppState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_main_menu_input.run_if(in_state(AppState::MainMenu)),
                handle_playing_input.run_if(in_state(AppState::Playing)),
                handle_paused_input.run_if(in_state(AppState::Paused)),
            ),
        );
    }
}

/// Main menu: Enter or Space starts the match.
fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        info!("MainMenu → Playing");
        next_state.set(AppState::Playing);
    }
}

/// Playing controls:
/// - Space → spawn a test Normal enemy
/// - 1/2/3 → spawn Normal / Fast / Tank
/// - Escape → pause
/// - LMB → print grid coordinate under cursor
fn handle_playing_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut spawn_request: ResMut<SpawnEnemyRequest>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    map: Res<Map>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        info!("Playing → Paused");
        next_state.set(AppState::Paused);
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Normal;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Normal;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Fast;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        spawn_request.pending = true;
        spawn_request.enemy_type = EnemyType::Tank;
    }

    if mouse.just_pressed(MouseButton::Left) {
        debug_print_grid_click(&windows, &camera_q, &map);
    }
}

/// Paused: Escape resumes.
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
            info!(
                "Mouse click → grid ({col}, {row}), tile={tile:?}, world={world}"
            );
        }
        None => {
            info!("Mouse click outside map at world={world}");
        }
    }
}
