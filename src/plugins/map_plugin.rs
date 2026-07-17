//! MapPlugin — builds the grid, path data, and static scenery for Day 1.
//!
//! ECS decisions:
//! - `Map` is a **Resource** (one authoritative layout).
//! - Each tile is an **entity** with a `Sprite` so we can later tint / replace
//!   individual cells when towers occupy them (Day 2).
//! - Path waypoint dots are separate entities for readability; they do not
//!   participate in gameplay queries.

use bevy::prelude::*;

use crate::components::{BaseMarker, MapTile, PathWaypointMarker, SpawnMarker};
use crate::resources::{Map, TileType};
use crate::utils::{grid_to_world, TILE_SIZE};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // Insert map data immediately so other plugins can depend on it.
        // Spawn visuals once at Startup (not OnEnter(Playing)) so resuming from
        // Paused does not duplicate every tile entity.
        app.insert_resource(Map::generate_day1())
            .add_systems(Startup, spawn_map_visuals);
    }
}

/// Colors chosen for clear Day 1 readability (dark, muted battlefield).
fn tile_color(tile: TileType) -> Color {
    match tile {
        TileType::Buildable => Color::srgb(0.35, 0.55, 0.32), // light green
        TileType::Path => Color::srgb(0.45, 0.32, 0.18),      // brown
        TileType::Occupied => Color::srgb(0.30, 0.30, 0.30),
        TileType::Spawn => Color::srgb(0.20, 0.55, 0.85),     // blue
        TileType::Base => Color::srgb(0.75, 0.55, 0.15),      // gold
    }
}

/// Spawn one sprite per tile + path decorations when entering Playing.
fn spawn_map_visuals(
    mut commands: Commands,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Slight inset so grid lines (gaps) read between tiles.
    let tile_visual = TILE_SIZE - 2.0;

    for row in 0..map.height {
        for col in 0..map.width {
            let tile = map
                .get_tile(col, row)
                .expect("tile coordinates are in-range by construction");
            let world = grid_to_world(col, row, map.width, map.height);

            commands.spawn((
                MapTile,
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

    // Path center-line segments (thin brown quads between waypoints).
    for window in map.path.windows(2) {
        let a = window[0];
        let b = window[1];
        let mid = (a + b) * 0.5;
        let delta = b - a;
        let length = delta.length().max(1.0);
        let angle = delta.y.atan2(delta.x);

        commands.spawn((
            Sprite {
                color: Color::srgb(0.62, 0.42, 0.22),
                custom_size: Some(Vec2::new(length, 10.0)),
                ..default()
            },
            Transform::from_translation(mid.extend(1.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            Name::new("PathSegment"),
        ));
    }

    // Waypoint dots for visual clarity.
    for (i, point) in map.path.iter().enumerate() {
        commands.spawn((
            PathWaypointMarker,
            Mesh2d(meshes.add(Circle::new(5.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.90, 0.55)))),
            Transform::from_translation(point.extend(2.0)),
            Name::new(format!("Waypoint_{i}")),
        ));
    }

    // Spawn marker (larger ring feel via circle mesh).
    if let Some(start) = map.path.first().copied() {
        commands.spawn((
            SpawnMarker,
            Mesh2d(meshes.add(Circle::new(22.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                0.25, 0.70, 1.00, 0.85,
            )))),
            Transform::from_translation(start.extend(3.0)),
            Name::new("SpawnMarker"),
        ));
    }

    // Base marker.
    if let Some(end) = map.path.last().copied() {
        commands.spawn((
            BaseMarker,
            Mesh2d(meshes.add(Circle::new(24.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
                0.95, 0.75, 0.20, 0.90,
            )))),
            Transform::from_translation(end.extend(3.0)),
            Name::new("BaseMarker"),
        ));
    }

    info!(
        "Map spawned: {}×{} tiles, {} path waypoints",
        map.width,
        map.height,
        map.path.len()
    );
}
