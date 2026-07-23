use bevy::prelude::*;

use crate::components::{
    Armor, ChainLightningFx, Enemy, Health, HitEffect, PathFollower, Position, Projectile,
    TowerType,
};
use crate::plugins::status_plugin::{apply_burn, apply_slow};
use crate::resources::{GameStats, WaveManager};
use crate::sfx::SfxRequest;
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
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile, &mut Position)>,
    enemies: Query<&Transform, (With<PathFollower>, Without<Projectile>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut projectile, mut position) in &mut projectiles {
        let current = transform.translation.truncate();

        let target_pos = if let Some(target) = projectile.target {
            enemies
                .get(target)
                .map(|t| t.translation.truncate())
                .unwrap_or(current)
        } else {
            current
        };

        let to_target = target_pos - current;
        let dist = to_target.length();
        if dist < 8.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let step = to_target.normalize() * projectile.speed * dt;
        transform.translation.x += step.x;
        transform.translation.y += step.y;
        position.0 = transform.translation.truncate();
    }
}

fn apply_projectile_damage(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile, &Transform)>,
    mut enemies: Query<
        (Entity, &mut Health, &Transform, &Enemy, Option<&Armor>),
        With<PathFollower>,
    >,
    mut stats: ResMut<GameStats>,
    mut waves: ResMut<WaveManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sfx: EventWriter<SfxRequest>,
) {
    for (proj_entity, projectile, proj_transform) in &projectiles {
        let proj_pos = proj_transform.translation.truncate();

        let hit_pos = if let Some(target) = projectile.target {
            enemies
                .get(target)
                .map(|(_, _, tf, _, _)| tf.translation.truncate())
                .unwrap_or(proj_pos)
        } else {
            proj_pos
        };

        if proj_pos.distance(hit_pos) > 12.0 {
            continue;
        }

        let primary = projectile.target;
        let mut hit_entities: Vec<Entity> = Vec::new();

        if let Some(target) = primary {
            if enemies.get(target).is_ok() {
                hit_entities.push(target);
            }
        }

        if projectile.splash_radius > 0.0 {
            for (entity, _, tf, _, _) in enemies.iter() {
                if tf.translation.truncate().distance(hit_pos) <= projectile.splash_radius {
                    if !hit_entities.contains(&entity) {
                        hit_entities.push(entity);
                    }
                }
            }
        }

        if hit_entities.is_empty() {
            for (entity, _, tf, _, _) in enemies.iter() {
                if tf.translation.truncate().distance(proj_pos) <= 25.0 {
                    hit_entities.push(entity);
                    break;
                }
            }
        }

        commands.entity(proj_entity).despawn();
        sfx.send(SfxRequest::Hit);

        for entity in hit_entities {
            let Ok((_, mut health, enemy_tf, enemy, armor)) = enemies.get_mut(entity) else {
                continue;
            };

            let mut dmg = projectile.damage;
            if let Some(a) = armor {
                dmg *= 1.0 - a.reduction;
            }

            if projectile.splash_radius > 0.0 && Some(entity) != primary {
                dmg *= 0.55;
            }

            health.current -= dmg;
            spawn_hit_fx(
                &mut commands,
                &mut meshes,
                &mut materials,
                enemy_tf.translation.truncate(),
                health.current <= 0.0,
            );

            if projectile.tower_type.applies_slow() {
                apply_slow(
                    &mut commands,
                    entity,
                    projectile.tower_type.slow_factor(),
                    projectile.tower_type.slow_duration(),
                );
            }

            if health.current <= 0.0 {
                stats.gold += enemy.gold_value;
                stats.kills += 1;
                waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
                sfx.send(SfxRequest::Kill);
                commands.entity(entity).despawn_recursive();
            }
        }

        if projectile.chain_remaining > 0 && projectile.tower_type == TowerType::Tesla {
            chain_lightning(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut enemies,
                &mut stats,
                &mut waves,
                &mut sfx,
                hit_pos,
                primary,
                projectile.damage * 0.7,
                projectile.chain_remaining - 1,
                projectile.chain_range,
            );
        }
    }
}

fn chain_lightning(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    enemies: &mut Query<
        (Entity, &mut Health, &Transform, &Enemy, Option<&Armor>),
        With<PathFollower>,
    >,
    stats: &mut GameStats,
    waves: &mut WaveManager,
    sfx: &mut EventWriter<SfxRequest>,
    from: Vec2,
    exclude: Option<Entity>,
    damage: f32,
    chains_left: u32,
    range: f32,
) {
    let mut best: Option<(Entity, Vec2, f32)> = None;
    for (entity, _, tf, _, _) in enemies.iter() {
        if Some(entity) == exclude {
            continue;
        }
        let pos = tf.translation.truncate();
        let d = from.distance(pos);
        if d <= range {
            if best.map(|(_, _, bd)| d < bd).unwrap_or(true) {
                best = Some((entity, pos, d));
            }
        }
    }

    let Some((target, target_pos, _)) = best else { return };

    commands.spawn((
        ChainLightningFx {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        },
        Mesh2d(meshes.add(Circle::new(6.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
            0.5, 0.95, 1.0, 0.9,
        )))),
        Transform::from_translation(target_pos.extend(18.0)),
    ));

    if let Ok((_, mut health, _, enemy, armor)) = enemies.get_mut(target) {
        let mut dmg = damage;
        if let Some(a) = armor {
            dmg *= 1.0 - a.reduction;
        }
        health.current -= dmg;
        sfx.send(SfxRequest::Hit);

        if health.current <= 0.0 {
            stats.gold += enemy.gold_value;
            stats.kills += 1;
            waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
            sfx.send(SfxRequest::Kill);
            commands.entity(target).despawn_recursive();
        } else if chains_left > 0 {
            chain_lightning(
                commands,
                meshes,
                materials,
                enemies,
                stats,
                waves,
                sfx,
                target_pos,
                Some(target),
                damage * 0.85,
                chains_left - 1,
                range,
            );
        }
    }
}

fn spawn_hit_fx(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    kill: bool,
) {
    let color = if kill {
        Color::srgba(1.0, 0.90, 0.30, 0.9)
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.9)
    };
    commands.spawn((
        HitEffect {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        },
        Mesh2d(meshes.add(Circle::new(8.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_translation(pos.extend(20.0)),
    ));
}
