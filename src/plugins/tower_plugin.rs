use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    Health, PathFollower, Position, Projectile, Tower, TowerRangePreview, TowerType,
};
use crate::resources::{GameStats, Map, TileType};
use crate::utils::{grid_to_world, world_to_grid, TILE_SIZE};
use crate::AppState;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tower_targeting,
                tower_shooting,
                update_range_preview,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn tower_targeting(
    mut towers: Query<(&mut Tower, &Transform)>,
    enemies: Query<&Transform, (With<PathFollower>, With<Health>)>,
) {
    for (mut tower, transform) in &mut towers {
        let tower_pos = transform.translation.truncate();
        let mut closest: Option<Vec2> = None;
        let mut closest_dist = f32::MAX;

        for enemy_transform in &enemies {
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = tower_pos.distance(enemy_pos);
            if dist <= tower.range && dist < closest_dist {
                closest_dist = dist;
                closest = Some(enemy_pos);
            }
        }

        if let Some(pos) = closest {
            tower.targeting_pos = pos;
        }
    }
}

fn tower_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mut towers: Query<(Entity, &mut Tower, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (_entity, mut tower, transform) in &mut towers {
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let tower_pos = transform.translation.truncate();
        if tower_pos.distance(tower.targeting_pos) > tower.range {
            continue;
        }

        let target_pos = tower.targeting_pos;
        let dir = (target_pos - tower_pos).normalize_or_zero();
        let proj_start = tower_pos + dir * 20.0;

        let proj_color = match tower.tower_type {
            TowerType::Arrow => Color::srgb(0.30, 0.85, 0.50),
            TowerType::Cannon => Color::srgb(0.90, 0.40, 0.25),
        };
        let proj_radius = match tower.tower_type {
            TowerType::Arrow => 4.0,
            TowerType::Cannon => 7.0,
        };

        commands.spawn((
            Projectile {
                target_pos,
                speed: 300.0,
                damage: tower.damage,
            },
            Mesh2d(meshes.add(Circle::new(proj_radius))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(proj_color))),
            Transform::from_translation(proj_start.extend(15.0)),
            Position(proj_start),
            Name::new(format!("Projectile_{:?}", tower.tower_type)),
        ));
    }
}

fn update_range_preview(
    mut preview: Query<&mut Transform, With<TowerRangePreview>>,
    towers: Query<&Transform, (With<Tower>, Without<TowerRangePreview>)>,
) {
    for mut t in &mut preview {
        if let Some(tower_transform) = towers.iter().next() {
            t.translation = tower_transform.translation;
        }
    }
}

pub fn spawn_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    map: &mut Map,
    stats: &mut GameStats,
    col: usize,
    row: usize,
    tower_type: TowerType,
) -> bool {
    if stats.gold < tower_type.cost() {
        info!("Not enough gold: need {}, have {}", tower_type.cost(), stats.gold);
        return false;
    }

    let tile = map.get_tile(col, row);
    if tile != Some(TileType::Buildable) {
        info!("Cannot place tower on {:?} tile", tile);
        return false;
    }

    stats.gold -= tower_type.cost();
    map.set_tile(col, row, TileType::Occupied);

    let world = grid_to_world(col, row, map.width, map.height);
    let height = 24.0;

    // Base platform
    commands.spawn((
        Tower::new(tower_type),
        Sprite {
            color: tower_type.color(),
            custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
            ..default()
        },
        Transform::from_translation(world.extend(5.0)),
        Name::new(format!("Tower_{:?}_{}_{}", tower_type, col, row)),
    ));

    // Turret circle on top
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(height * 0.5))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.15, 0.15, 0.15)))),
        Transform::from_translation(world.extend(6.0)),
        Name::new("TowerTurret"),
    ));

    info!(
        "Placed {:?} tower at ({col}, {row}), gold remaining: {}",
        tower_type, stats.gold
    );
    true
}

pub fn find_tower_placement(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
    map: &Map,
) -> Option<(usize, usize)> {
    let Ok(window) = windows.get_single() else { return None };
    let Some(cursor) = window.cursor_position() else { return None };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return None };
    let Ok(world) = camera.viewport_to_world_2d(cam_transform, cursor) else {
        return None;
    };
    world_to_grid(world, map.width, map.height)
}