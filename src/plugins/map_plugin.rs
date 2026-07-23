//! MapPlugin — grid, path, tile visuals with sync.

use bevy::prelude::*;

use crate::components::{BaseMarker, GridPosition, MapDecoration, MapTile, PathWaypointMarker, SpawnMarker};
use crate::resources::{load_map_by_index, tile_color, GameSettings, Map, TileType};
use crate::utils::{grid_to_world, TILE_SIZE};
use crate::AppState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .add_systems(Startup, setup_initial_map)
            .add_systems(OnEnter(AppState::Playing), respawn_map_visuals)
            .add_systems(
                Update,
                sync_tile_visuals
                    .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
            );
    }
}

fn setup_initial_map(mut commands: Commands, settings: Res<GameSettings>) {
    let map = load_map_by_index(settings.map_index);
    commands.insert_resource(map);
}

/// Rebuild map visuals when entering Playing (handles restart / map change).
fn respawn_map_visuals(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut map: ResMut<Map>,
    existing: Query<Entity, Or<(With<MapTile>, With<MapDecoration>, With<SpawnMarker>, With<BaseMarker>, With<PathWaypointMarker>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    *map = load_map_by_index(settings.map_index);
    map.dirty = true;

    for e in &existing {
        commands.entity(e).despawn_recursive();
    }

    spawn_map_visuals(&mut commands, &map, &mut meshes, &mut materials);
}

pub fn spawn_map_visuals(
    commands: &mut Commands,
    map: &Map,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let tile_visual = TILE_SIZE - 2.0;

    for row in 0..map.height {
        for col in 0..map.width {
            let tile = map.get_tile(col, row).expect("in-range");
            let world = grid_to_world(col, row, map.width, map.height);

            commands.spawn((
                MapTile,
                GridPosition { col, row },
                Sprite {
                    color: tile_color(tile),
                    custom_size: Some(Vec2::splat(tile_visual)),
                    ..default()
                },
                Transform::from_translation(world.extend(0.0)),
                Name::new(format!("Tile_{col}_{row}")),
            ));
        }
    }

    for window in map.path.windows(2) {
        let a = window[0];
        let b = window[1];
        let mid = (a + b) * 0.5;
        let delta = b - a;
        let length = delta.length().max(1.0);
        let angle = delta.y.atan2(delta.x);

        commands.spawn((
            MapDecoration,
            Sprite {
                color: Color::srgb(0.62, 0.42, 0.22),
                custom_size: Some(Vec2::new(length, 10.0)),
                ..default()
            },
            Transform::from_translation(mid.extend(1.0))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }

    for (i, point) in map.path.iter().enumerate() {
        commands.spawn((
            PathWaypointMarker,
            MapDecoration,
            Mesh2d(meshes.add(Circle::new(5.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.90, 0.55)))),
            Transform::from_translation(point.extend(2.0)),
            Name::new(format!("Waypoint_{i}")),
        ));
    }

    if let Some(start) = map.path.first().copied() {
        commands.spawn((
            SpawnMarker,
            MapDecoration,
            Mesh2d(meshes.add(Circle::new(22.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                0.25, 0.70, 1.00, 0.85,
            )))),
            Transform::from_translation(start.extend(3.0)),
        ));
    }

    if let Some(end) = map.path.last().copied() {
        commands.spawn((
            BaseMarker,
            MapDecoration,
            Mesh2d(meshes.add(Circle::new(24.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                0.95, 0.75, 0.20, 0.90,
            )))),
            Transform::from_translation(end.extend(3.0)),
        ));
    }

    info!("Map '{}' spawned: {}×{}", map.name, map.width, map.height);
}

fn sync_tile_visuals(
    mut map: ResMut<Map>,
    mut tiles: Query<(&GridPosition, &mut Sprite), With<MapTile>>,
) {
    if !map.dirty {
        return;
    }
    map.dirty = false;

    for (grid, mut sprite) in &mut tiles {
        if let Some(tile) = map.get_tile(grid.col, grid.row) {
            sprite.color = tile_color(tile);
        }
    }
}

pub fn mark_tile(map: &mut Map, col: usize, row: usize, tile: TileType) {
    map.dirty = true;
    map.set_tile(col, row, tile);
}
