//! EnemyPlugin — spawn test creeps and advance them along `Map.path`.
//!
//! Movement is pure ECS: a system reads `Map` + `Time`, writes `Transform` /
//! `Position` / `Enemy` progress. No OOP "enemy.update()" methods.
//!
//! Day 1: manual Spacebar spawn only. WaveManager is updated for bookkeeping
//! so Day 2 can swap the input trigger for a wave schedule without redesign.

use bevy::prelude::*;

use crate::components::{Enemy, EnemyType, Health, PathFollower, Position};
use crate::resources::{GameStats, Map, WaveManager};
use crate::AppState;

/// Event-like resource flag set by InputPlugin when the player presses Space.
/// Using a tiny resource avoids coupling InputPlugin to mesh assets.
#[derive(Resource, Debug, Default)]
pub struct SpawnEnemyRequest {
    pub pending: bool,
    pub enemy_type: EnemyType,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnEnemyRequest>()
            .add_systems(
                Update,
                (
                    fulfill_spawn_requests,
                    move_path_followers,
                    sync_position_from_transform,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Consume spawn requests and create enemy entities at path start.
fn fulfill_spawn_requests(
    mut commands: Commands,
    mut request: ResMut<SpawnEnemyRequest>,
    waves: ResMut<WaveManager>,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !request.pending {
        return;
    }
    request.pending = false;

    let Some(start) = map.path.first().copied() else {
        warn!("Cannot spawn enemy: map path is empty");
        return;
    };

    let enemy_type = request.enemy_type;
    let radius = enemy_type.radius();
    let color = enemy_type.color();

    // Black outline: slightly larger disc behind the body (z-order).
    let outline = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(radius + 3.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.05, 0.05, 0.05)))),
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
            Name::new("EnemyOutline"),
        ))
        .id();

    let body = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(radius))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_translation(start.extend(10.0)),
            Position(start),
            Health::full(enemy_type.base_health()),
            Enemy::new(enemy_type),
            PathFollower,
            Name::new(format!("Enemy_{enemy_type:?}")),
        ))
        .id();

    // Parent outline under body so it follows automatically.
    commands.entity(body).add_children(&[outline]);

    info!(
        "Spawned {:?} enemy #{} at path start",
        enemy_type, waves.enemies_spawned
    );
}

/// Advance every `PathFollower` along path segments using speed × dt / segment length.
fn move_path_followers(
    time: Res<Time>,
    map: Res<Map>,
    mut commands: Commands,
    mut stats: ResMut<GameStats>,
    mut waves: ResMut<WaveManager>,
    mut query: Query<(Entity, &mut Transform, &mut Enemy), With<PathFollower>>,
) {
    let path = &map.path;
    if path.len() < 2 {
        return;
    }

    let dt = time.delta_secs();

    for (entity, mut transform, mut enemy) in &mut query {
        // Already past the last segment → arrived.
        if enemy.waypoint_index >= path.len() - 1 {
            stats.lives = stats.lives.saturating_sub(1);
            info!(
                "Enemy {:?} reached base! Lives remaining: {}",
                enemy.enemy_type, stats.lives
            );
            commands.entity(entity).despawn_recursive();
            waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
            continue;
        }

        // Move across consecutive segments if speed is high enough to finish one in a frame.
        let mut remaining_distance = enemy.speed * dt;

        while remaining_distance > 0.0 && enemy.waypoint_index < path.len() - 1 {
            let start = path[enemy.waypoint_index];
            let end = path[enemy.waypoint_index + 1];
            let segment = end - start;
            let segment_len = segment.length();

            if segment_len <= f32::EPSILON {
                enemy.waypoint_index += 1;
                enemy.progress = 0.0;
                continue;
            }

            let dist_along = enemy.progress * segment_len;
            let dist_left_on_segment = segment_len - dist_along;

            if remaining_distance < dist_left_on_segment {
                enemy.progress += remaining_distance / segment_len;
                remaining_distance = 0.0;
            } else {
                // Finish this segment and spill into the next.
                remaining_distance -= dist_left_on_segment;
                enemy.waypoint_index += 1;
                enemy.progress = 0.0;
            }
        }

        if enemy.waypoint_index >= path.len() - 1 {
            stats.lives = stats.lives.saturating_sub(1);
            info!(
                "Enemy {:?} reached base! Lives remaining: {}",
                enemy.enemy_type, stats.lives
            );
            waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
            // Snap to final point for one clean frame if needed, then remove.
            if let Some(last) = path.last() {
                transform.translation.x = last.x;
                transform.translation.y = last.y;
            }
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let start = path[enemy.waypoint_index];
        let end = path[enemy.waypoint_index + 1];
        let pos = start.lerp(end, enemy.progress);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        // Keep z stable for draw order above tiles.
        transform.translation.z = 10.0;
    }
}

/// Keep gameplay `Position` mirrored from `Transform` after movement.
/// Downstream systems (targeting, UI bars) can read `Position` without
/// pulling in full transform hierarchies.
fn sync_position_from_transform(mut query: Query<(&Transform, &mut Position), With<PathFollower>>) {
    for (transform, mut position) in &mut query {
        position.0 = transform.translation.truncate();
    }
}
