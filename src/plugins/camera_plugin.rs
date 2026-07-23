//! Camera pan and zoom.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::components::{CameraController, MainGameCamera};
use crate::AppState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraController>()
            .add_systems(
                Update,
                (camera_pan, camera_zoom)
                    .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
            );
    }
}

fn camera_pan(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    ctrl: Res<CameraController>,
    mut camera: Query<&mut Transform, With<MainGameCamera>>,
) {
    let mut delta = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        delta.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        delta.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        delta.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        delta.x += 1.0;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let move_delta = delta.normalize() * ctrl.pan_speed * time.delta_secs() / ctrl.zoom;
    for mut transform in &mut camera {
        transform.translation.x += move_delta.x;
        transform.translation.y += move_delta.y;
    }
}

fn camera_zoom(
    mut scroll: EventReader<MouseWheel>,
    mut ctrl: ResMut<CameraController>,
    mut camera: Query<&mut Projection, With<MainGameCamera>>,
) {
    let mut delta = 0.0f32;
    for ev in scroll.read() {
        delta += match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.1,
            MouseScrollUnit::Pixel => ev.y * 0.001,
        };
    }
    if delta.abs() < f32::EPSILON {
        return;
    }

    ctrl.zoom = (ctrl.zoom + delta).clamp(ctrl.min_zoom, ctrl.max_zoom);
    for mut projection in &mut camera {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 1.0 / ctrl.zoom;
        }
    }
}
