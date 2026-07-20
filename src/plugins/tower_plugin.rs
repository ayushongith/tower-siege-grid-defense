use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    GridPosition, Health, PathFollower, Position, Projectile, SelectionRing, Tower,
    TowerEditTarget, TowerLevel, TowerRangePreview, TowerType,
};
use crate::resources::{GameStats, Map, TileType};
use crate::utils::{grid_to_world, world_to_grid, TILE_SIZE};
use crate::AppState;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TowerEditTarget>()
            .add_systems(
                Update,
                (
                    tower_targeting,
                    tower_shooting,
                    update_range_preview,
                    upgrade_tower_system,
                    sell_tower_system,
                    update_tower_selection_ring,
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

/// Compute upgrade cost for a tower: base_cost * 0.75 * level.
pub fn upgrade_cost(level: u32, base_cost: u32) -> u32 {
    ((base_cost as f32) * 0.75 * level as f32).round() as u32
}

/// Handle 'U' key to upgrade the selected tower.
fn upgrade_tower_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut stats: ResMut<GameStats>,
    mut edit_target: ResMut<TowerEditTarget>,
    mut towers: Query<(&mut Tower, &mut TowerLevel, &GridPosition)>,
) {
    if !keys.just_pressed(KeyCode::KeyU) {
        return;
    }
    let Some(entity) = edit_target.entity else { return };

    let Ok((mut tower, mut level, _grid)) = towers.get_mut(entity) else {
        edit_target.entity = None;
        return;
    };

    if level.level >= level.max_level {
        info!("Tower already at max level {}", level.max_level);
        return;
    }

    let cost = upgrade_cost(level.level, tower.tower_type.cost());
    if stats.gold < cost {
        info!("Not enough gold to upgrade: need {}, have {}", cost, stats.gold);
        return;
    }

    stats.gold -= cost;
    level.total_invested += cost;
    level.level += 1;

    // Apply stat improvements.
    tower.damage += tower.tower_type.damage() * 0.25;
    tower.range += tower.tower_type.range() * 0.10;
    let new_rate = (tower.fire_rate * 0.90).max(0.3);
    tower.fire_rate = new_rate;
    tower.cooldown = Timer::from_seconds(new_rate, TimerMode::Repeating);

    info!(
        "Tower {:?} upgraded to level {}! damage={:.0}, range={:.0}, fire_rate={:.1}s",
        tower.tower_type, level.level, tower.damage, tower.range, tower.fire_rate
    );
}

/// Sell the selected tower: refund 50% of total invested, remove entity,
/// mark tile as Buildable.
fn sell_tower_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut stats: ResMut<GameStats>,
    mut edit_target: ResMut<TowerEditTarget>,
    mut map: ResMut<Map>,
    towers: Query<(&Tower, &TowerLevel, &GridPosition)>,
) {
    if !keys.just_pressed(KeyCode::KeyS) {
        return;
    }
    let Some(entity) = edit_target.entity else { return };
    let Ok((tower, _level, grid)) = towers.get(entity) else {
        edit_target.entity = None;
        return;
    };

    let total = _level.total_invested;
    let refund = (total as f32 * 0.50).round() as u32;
    stats.gold += refund;
    map.set_tile(grid.col, grid.row, TileType::Buildable);
    commands.entity(entity).despawn_recursive();
    info!(
        "Sold {:?} tower at ({}, {}), refunded {}g",
        tower.tower_type, grid.col, grid.row, refund
    );
    edit_target.entity = None;
}

/// Show/hide a selection ring sprite on the currently selected tower.
fn update_tower_selection_ring(
    mut commands: Commands,
    edit_target: Res<TowerEditTarget>,
    towers: Query<&Transform, With<Tower>>,
    rings: Query<Entity, With<SelectionRing>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Despawn previous ring if it exists.
    for ring_entity in &rings {
        commands.entity(ring_entity).despawn();
    }

    // If a tower is selected, spawn a new ring at its position.
    if let Some(entity) = edit_target.entity {
        if let Ok(transform) = towers.get(entity) {
            commands.spawn((
                SelectionRing,
                Mesh2d(meshes.add(Circle::new(30.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                    1.0, 1.0, 0.40, 0.35,
                )))),
                Transform::from_translation(Vec3::new(transform.translation.x, transform.translation.y, 7.0)),
                Name::new("SelectionRing"),
            ));
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
    let base = commands
        .spawn((
            Tower::new(tower_type),
            TowerLevel::new(tower_type.cost()),
            GridPosition { col, row },
            Sprite {
                color: tower_type.color(),
                custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
                ..default()
            },
            Transform::from_translation(world.extend(5.0)),
            Name::new(format!("Tower_{:?}_{}_{}", tower_type, col, row)),
        ))
        .id();

    // Turret circle on top (child of base — despawned with base on sell)
    let turret = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(height * 0.5))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.15, 0.15, 0.15)))),
            Transform::from_translation(Vec3::ZERO),
            Name::new("TowerTurret"),
        ))
        .id();
    commands.entity(base).add_child(turret);

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