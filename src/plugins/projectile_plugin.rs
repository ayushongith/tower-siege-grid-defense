use bevy::prelude::*;

use crate::components::{Enemy, Health, HitEffect, PathFollower, Position, Projectile};
use crate::sfx::SfxRequest;
use crate::resources::{GameStats, WaveManager};
use crate::AppState;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_projectiles, apply_projectile_damage)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn move_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile, &mut Position)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, projectile, mut position) in &mut projectiles {
        let current = transform.translation.truncate();
        let dir = (projectile.target_pos - current).normalize_or_zero();
        let step = dir * projectile.speed * dt;

        transform.translation.x += step.x;
        transform.translation.y += step.y;
        position.0 = transform.translation.truncate();

        if current.distance(projectile.target_pos) < 10.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn apply_projectile_damage(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform)>,
    mut enemies: Query<(Entity, &mut Health, &Transform, &Enemy), With<PathFollower>>,
    mut stats: ResMut<GameStats>,
    mut waves: ResMut<WaveManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sfx: EventWriter<SfxRequest>,
) {
    for (proj_entity, projectile, proj_transform) in &projectiles {
        let proj_pos = proj_transform.translation.truncate();
        if proj_pos.distance(projectile.target_pos) > 5.0 {
            continue;
        }

        for (enemy_entity, mut health, enemy_transform, enemy) in &mut enemies {
            let enemy_pos = enemy_transform.translation.truncate();
            if proj_pos.distance(enemy_pos) > 25.0 {
                continue;
            }

            health.current -= projectile.damage;
            commands.entity(proj_entity).despawn();

            sfx.send(SfxRequest::Hit);

            // Spawn hit effect at impact point.
            let hit_color = if health.current <= 0.0 {
                Color::srgb(1.0, 0.90, 0.30)
            } else {
                Color::srgb(1.0, 1.0, 1.0)
            };
            commands.spawn((
                HitEffect {
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                },
                Mesh2d(meshes.add(Circle::new(8.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(hit_color))),
                Transform::from_translation(enemy_pos.extend(20.0)),
                Name::new("HitEffect"),
            ));

            if health.current <= 0.0 {
                stats.gold += enemy.gold_value;
                waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
                sfx.send(SfxRequest::Kill);
                info!(
                    "Enemy {:?} killed! +{} gold (total: {})",
                    enemy.enemy_type, enemy.gold_value, stats.gold
                );
                commands.entity(enemy_entity).despawn_recursive();
            }
            break;
        }
    }
}