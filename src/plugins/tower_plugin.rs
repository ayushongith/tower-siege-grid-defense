use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    GridPosition, Health, PathFollower, Position, Projectile, SelectionRing, Tower,
    TowerEditTarget, TowerLevel, TowerPlacementGhost, TowerRangePreview, TowerSelection,
    TowerTurret, TowerType,
};
use crate::resources::{GameStats, Map, TileType};
use crate::sfx::SfxRequest;
use crate::utils::{grid_to_world, world_to_grid, TILE_SIZE};
use crate::AppState;

#[derive(Resource, Debug, Default)]
pub struct PlacementPreviewState {
    pub range_entity: Option<Entity>,
    pub ghost_entity: Option<Entity>,
}

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TowerEditTarget>()
            .init_resource::<PlacementPreviewState>()
            .add_systems(
                Update,
                (
                    tower_targeting,
                    rotate_turrets,
                    tower_shooting,
                    manage_placement_preview,
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
    enemies: Query<(Entity, &Transform), (With<PathFollower>, With<Health>)>,
) {
    for (mut tower, transform) in &mut towers {
        let tower_pos = transform.translation.truncate();
        let mut best: Option<(Entity, Vec2, f32)> = None;

        for (entity, enemy_transform) in &enemies {
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = tower_pos.distance(enemy_pos);
            if dist <= tower.range {
                if best.map(|(_, _, d)| dist < d).unwrap_or(true) {
                    best = Some((entity, enemy_pos, dist));
                }
            }
        }

        if let Some((entity, pos, _)) = best {
            tower.target_entity = Some(entity);
            tower.targeting_pos = pos;
        } else {
            tower.target_entity = None;
        }
    }
}

fn rotate_turrets(
    towers: Query<(&Tower, &Transform, &Children)>,
    mut turrets: Query<&mut Transform, (With<TowerTurret>, Without<Tower>)>,
) {
    let mut pending: Vec<(Entity, f32)> = Vec::new();
    for (tower, transform, children) in &towers {
        let dir = tower.targeting_pos - transform.translation.truncate();
        if dir.length_squared() < 1.0 {
            continue;
        }
        let angle = dir.y.atan2(dir.x);
        for &child in children.iter() {
            pending.push((child, angle));
        }
    }
    for (child, angle) in pending {
        if let Ok(mut t) = turrets.get_mut(child) {
            t.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn tower_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mut towers: Query<(Entity, &mut Tower, &Transform)>,
    enemies: Query<&Transform, With<PathFollower>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sfx: EventWriter<SfxRequest>,
) {
    for (tower_entity, mut tower, transform) in &mut towers {
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let Some(target) = tower.target_entity else { continue };
        let Ok(enemy_tf) = enemies.get(target) else {
            tower.target_entity = None;
            continue;
        };

        let tower_pos = transform.translation.truncate();
        let target_pos = enemy_tf.translation.truncate();
        if tower_pos.distance(target_pos) > tower.range {
            continue;
        }

        tower.targeting_pos = target_pos;
        let dir = (target_pos - tower_pos).normalize_or_zero();
        let proj_start = tower_pos + dir * 20.0;

        let (proj_radius, proj_color) = match tower.tower_type {
            TowerType::Arrow => (4.0, Color::srgb(0.30, 0.85, 0.50)),
            TowerType::Cannon => (7.0, Color::srgb(0.90, 0.40, 0.25)),
            TowerType::Slow => (5.0, Color::srgb(0.40, 0.70, 1.0)),
            TowerType::Sniper => (3.0, Color::srgb(0.85, 0.85, 0.95)),
            TowerType::Tesla => (6.0, Color::srgb(0.50, 0.95, 1.0)),
        };

        commands.spawn((
            Projectile {
                target: Some(target),
                speed: if tower.tower_type == TowerType::Sniper { 500.0 } else { 300.0 },
                damage: tower.damage,
                splash_radius: tower.tower_type.splash_radius(),
                chain_remaining: tower.tower_type.chain_count(),
                chain_range: tower.tower_type.chain_range(),
                tower_type: tower.tower_type,
                source_tower: Some(tower_entity),
            },
            Mesh2d(meshes.add(Circle::new(proj_radius))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(proj_color))),
            Transform::from_translation(proj_start.extend(15.0)),
            Position(proj_start),
            Name::new(format!("Projectile_{:?}", tower.tower_type)),
        ));
        sfx.send(SfxRequest::Shoot);
    }
}

fn manage_placement_preview(
    mut commands: Commands,
    tower_sel: Res<TowerSelection>,
    stats: Res<GameStats>,
    map: Res<Map>,
    mut preview: ResMut<PlacementPreviewState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut range_q: Query<
        (&mut Transform, &MeshMaterial2d<ColorMaterial>),
        (With<TowerRangePreview>, Without<TowerPlacementGhost>),
    >,
    mut ghost_q: Query<
        (&mut Transform, &mut Sprite),
        (With<TowerPlacementGhost>, Without<TowerRangePreview>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(tower_type) = tower_sel.selected else {
        despawn_preview(&mut commands, &mut preview);
        return;
    };

    if preview.range_entity.is_none() {
        let range_entity = commands
            .spawn((
                TowerRangePreview,
                Mesh2d(meshes.add(Circle::new(tower_type.range()))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                    0.3, 0.8, 0.4, 0.18,
                )))),
                Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ))
            .id();
        preview.range_entity = Some(range_entity);
    }

    if preview.ghost_entity.is_none() {
        let ghost_entity = commands
            .spawn((
                TowerPlacementGhost,
                Sprite {
                    color: Color::srgba(0.5, 0.9, 0.5, 0.55),
                    custom_size: Some(Vec2::splat(TILE_SIZE - 4.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 6.0)),
            ))
            .id();
        preview.ghost_entity = Some(ghost_entity);
    }

    let Some((col, row)) = find_tower_placement(&windows, &camera_q, &map) else {
        return;
    };

    let world = grid_to_world(col, row, map.width, map.height);
    let valid = map.get_tile(col, row) == Some(TileType::Buildable)
        && stats.gold >= tower_type.cost();

    let ghost_color = if valid {
        Color::srgba(0.3, 0.9, 0.4, 0.55)
    } else {
        Color::srgba(0.9, 0.2, 0.2, 0.55)
    };
    let range_color = if valid {
        Color::srgba(0.3, 0.8, 0.4, 0.18)
    } else {
        Color::srgba(0.9, 0.2, 0.2, 0.15)
    };

    if let Some(re) = preview.range_entity {
        if let Ok((mut tf, mat)) = range_q.get_mut(re) {
            tf.translation = world.extend(4.0);
            tf.scale = Vec3::splat(tower_type.range() / tower_type.range().max(1.0));
            if let Some(m) = materials.get_mut(&mat.0) {
                m.color = range_color;
            }
        }
    }

    if let Some(ge) = preview.ghost_entity {
        if let Ok((mut tf, mut sprite)) = ghost_q.get_mut(ge) {
            tf.translation = world.extend(6.0);
            sprite.color = ghost_color;
        }
    }
}

fn despawn_preview(commands: &mut Commands, preview: &mut PlacementPreviewState) {
    if let Some(e) = preview.range_entity.take() {
        commands.entity(e).despawn_recursive();
    }
    if let Some(e) = preview.ghost_entity.take() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn clear_placement_preview(commands: &mut Commands, preview: &mut PlacementPreviewState) {
    despawn_preview(commands, preview);
}

pub fn upgrade_cost(level: u32, base_cost: u32) -> u32 {
    ((base_cost as f32) * 0.75 * level as f32).round() as u32
}

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
        return;
    }

    let cost = upgrade_cost(level.level, tower.tower_type.cost());
    if stats.gold < cost {
        return;
    }

    stats.gold -= cost;
    level.total_invested += cost;
    level.level += 1;

    tower.damage += tower.tower_type.damage() * 0.25;
    tower.range += tower.tower_type.range() * 0.10;
    let new_rate = (tower.fire_rate * 0.90).max(0.3);
    tower.fire_rate = new_rate;
    tower.cooldown = Timer::from_seconds(new_rate, TimerMode::Repeating);
}

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
    let Ok((_tower, level, grid)) = towers.get(entity) else {
        edit_target.entity = None;
        return;
    };

    let refund = (level.total_invested as f32 * 0.50).round() as u32;
    stats.gold += refund;
    map.set_tile(grid.col, grid.row, TileType::Buildable);
    commands.entity(entity).despawn_recursive();
    edit_target.entity = None;
}

fn update_tower_selection_ring(
    mut commands: Commands,
    edit_target: Res<TowerEditTarget>,
    towers: Query<&Transform, With<Tower>>,
    rings: Query<Entity, With<SelectionRing>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ring_entity in &rings {
        commands.entity(ring_entity).despawn();
    }

    if let Some(entity) = edit_target.entity {
        if let Ok(transform) = towers.get(entity) {
            commands.spawn((
                SelectionRing,
                Mesh2d(meshes.add(Circle::new(30.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                    1.0, 1.0, 0.40, 0.35,
                )))),
                Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    transform.translation.y,
                    7.0,
                )),
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
        return false;
    }
    if map.get_tile(col, row) != Some(TileType::Buildable) {
        return false;
    }

    stats.gold -= tower_type.cost();
    stats.towers_built += 1;
    map.set_tile(col, row, TileType::Occupied);

    let world = grid_to_world(col, row, map.width, map.height);

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

    let (turret_mesh, turret_color, turret_rot) = match tower_type {
        TowerType::Arrow => (
            Mesh::from(RegularPolygon {
                circumcircle: Circle::new(12.0),
                sides: 3,
            }),
            Color::srgb(0.50, 0.50, 0.55),
            Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        ),
        TowerType::Cannon => (
            Mesh::from(RegularPolygon {
                circumcircle: Circle::new(12.0),
                sides: 8,
            }),
            Color::srgb(0.30, 0.25, 0.20),
            Quat::IDENTITY,
        ),
        TowerType::Slow => (
            Mesh::from(Circle::new(10.0)),
            Color::srgb(0.40, 0.70, 1.0),
            Quat::IDENTITY,
        ),
        TowerType::Sniper => (
            Mesh::from(RegularPolygon {
                circumcircle: Circle::new(14.0),
                sides: 4,
            }),
            Color::srgb(0.70, 0.70, 0.80),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        ),
        TowerType::Tesla => (
            Mesh::from(RegularPolygon {
                circumcircle: Circle::new(11.0),
                sides: 6,
            }),
            Color::srgb(0.60, 0.95, 1.0),
            Quat::IDENTITY,
        ),
    };

    let turret = commands
        .spawn((
            TowerTurret,
            Mesh2d(meshes.add(turret_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(turret_color))),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.5)).with_rotation(turret_rot),
        ))
        .id();
    commands.entity(base).add_child(turret);

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
