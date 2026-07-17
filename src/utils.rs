//! Shared math helpers for grid ↔ world conversion.
//!
//! The map is centered on the world origin so the default 2D camera frames it
//! without extra offset math. Tile centers (not corners) are the gameplay
//! positions for enemies, towers, and mouse picking.

#![allow(dead_code)]

use bevy::prelude::*;

/// Edge length of one tile in world units (pixels at 1:1 ortho scale).
pub const TILE_SIZE: f32 = 64.0;

/// Convert discrete grid coordinates to the world-space center of that tile.
///
/// - `col` increases to the right
/// - `row` increases upward (Bevy Y-up)
/// - Map is centered at `(0, 0)`
pub fn grid_to_world(col: usize, row: usize, map_width: usize, map_height: usize) -> Vec2 {
    let origin_x = -(map_width as f32 * TILE_SIZE) * 0.5 + TILE_SIZE * 0.5;
    let origin_y = -(map_height as f32 * TILE_SIZE) * 0.5 + TILE_SIZE * 0.5;
    Vec2::new(
        origin_x + col as f32 * TILE_SIZE,
        origin_y + row as f32 * TILE_SIZE,
    )
}

/// Convert a world position to grid coordinates.
///
/// Returns `None` when the point lies outside the map bounds.
pub fn world_to_grid(
    world: Vec2,
    map_width: usize,
    map_height: usize,
) -> Option<(usize, usize)> {
    let origin_x = -(map_width as f32 * TILE_SIZE) * 0.5;
    let origin_y = -(map_height as f32 * TILE_SIZE) * 0.5;

    let local_x = world.x - origin_x;
    let local_y = world.y - origin_y;

    if local_x < 0.0 || local_y < 0.0 {
        return None;
    }

    let col = (local_x / TILE_SIZE).floor() as isize;
    let row = (local_y / TILE_SIZE).floor() as isize;

    if col < 0 || row < 0 {
        return None;
    }

    let col = col as usize;
    let row = row as usize;

    if col >= map_width || row >= map_height {
        return None;
    }

    Some((col, row))
}

/// Half-extents of the full map in world units (useful for camera framing).
pub fn map_half_extents(map_width: usize, map_height: usize) -> Vec2 {
    Vec2::new(
        map_width as f32 * TILE_SIZE * 0.5,
        map_height as f32 * TILE_SIZE * 0.5,
    )
}
