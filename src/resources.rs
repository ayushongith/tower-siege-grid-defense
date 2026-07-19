//! Global game resources (singleton state shared across systems).
//!
//! Resources are the right tool when:
//! - Exactly one instance should exist (map, economy, wave schedule)
//! - Many systems need read/write access without an entity query
//!
//! Prefer components when state is per-entity (HP, transform, enemy type).

// Wave / placement helpers are partially unused until later days.
#![allow(dead_code)]

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Economy / run stats
// ---------------------------------------------------------------------------

/// Player-facing run statistics.
#[derive(Resource, Debug, Clone)]
pub struct GameStats {
    pub gold: u32,
    pub lives: u32,
    pub current_wave: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            gold: 200,
            lives: 20,
            current_wave: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

/// Semantic type of a single grid cell.
///
/// Day 1 uses Path / Buildable / Spawn / Base for visuals and future placement.
/// `Occupied` is reserved for Day 2 tower placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TileType {
    #[default]
    Buildable,
    Path,
    Occupied,
    Spawn,
    Base,
}

/// Authoritative map layout and precomputed world-space path.
///
/// `tiles` is row-major: index = `row * width + col`.
/// `path` is a sequence of world positions enemies lerp between.
#[derive(Resource, Debug, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    /// World-space waypoints (already converted via `grid_to_world`).
    pub path: Vec<Vec2>,
}

impl Map {
    pub const WIDTH: usize = 15;
    pub const HEIGHT: usize = 10;

    pub fn tile_index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    pub fn get_tile(&self, col: usize, row: usize) -> Option<TileType> {
        if col >= self.width || row >= self.height {
            return None;
        }
        Some(self.tiles[self.tile_index(col, row)])
    }

    pub fn set_tile(&mut self, col: usize, row: usize, tile: TileType) {
        if col < self.width && row < self.height {
            let i = self.tile_index(col, row);
            self.tiles[i] = tile;
        }
    }

    /// Build the Day 1 demo map: 15×10 grid with a winding left→right path.
    pub fn generate_day1() -> Self {
        let width = Self::WIDTH;
        let height = Self::HEIGHT;
        let mut tiles = vec![TileType::Buildable; width * height];

        // Grid waypoints (col, row). Row 0 is the bottom of the map (Y-up).
        // Winding route with 15 corners / nodes for smooth multi-segment travel.
        let grid_path: Vec<(usize, usize)> = vec![
            (0, 4),   // Spawn
            (2, 4),
            (2, 7),
            (5, 7),
            (5, 2),
            (8, 2),
            (8, 8),
            (11, 8),
            (11, 3),
            (13, 3),
            (13, 6),
            (14, 6),  // Base
        ];

        // Paint path tiles between consecutive waypoints (axis-aligned only).
        for window in grid_path.windows(2) {
            let (c0, r0) = window[0];
            let (c1, r1) = window[1];
            paint_orthogonal(&mut tiles, width, height, c0, r0, c1, r1);
        }

        // Mark endpoints after painting so they override Path.
        let (spawn_c, spawn_r) = grid_path[0];
        let (base_c, base_r) = *grid_path.last().unwrap();
        tiles[spawn_r * width + spawn_c] = TileType::Spawn;
        tiles[base_r * width + base_c] = TileType::Base;

        // Convert grid nodes → world centers for enemy following.
        let path: Vec<Vec2> = grid_path
            .iter()
            .map(|(c, r)| crate::utils::grid_to_world(*c, *r, width, height))
            .collect();

        Self {
            width,
            height,
            tiles,
            path,
        }
    }
}

/// Fill tiles along an axis-aligned segment (inclusive).
fn paint_orthogonal(
    tiles: &mut [TileType],
    width: usize,
    height: usize,
    c0: usize,
    r0: usize,
    c1: usize,
    r1: usize,
) {
    if r0 == r1 {
        let row = r0;
        let (min_c, max_c) = if c0 <= c1 { (c0, c1) } else { (c1, c0) };
        for col in min_c..=max_c {
            if col < width && row < height {
                tiles[row * width + col] = TileType::Path;
            }
        }
    } else if c0 == c1 {
        let col = c0;
        let (min_r, max_r) = if r0 <= r1 { (r0, r1) } else { (r1, r0) };
        for row in min_r..=max_r {
            if col < width && row < height {
                tiles[row * width + col] = TileType::Path;
            }
        }
    } else {
        // Day 1 paths are orthogonal by construction; log if data is wrong.
        bevy::log::warn!(
            "Non-orthogonal path segment ({c0},{r0}) -> ({c1},{r1}); painting endpoints only"
        );
        if c0 < width && r0 < height {
            tiles[r0 * width + c0] = TileType::Path;
        }
        if c1 < width && r1 < height {
            tiles[r1 * width + c1] = TileType::Path;
        }
    }
}

// ---------------------------------------------------------------------------
// Waves (Day 3 — full auto-scaling wave system)
// ---------------------------------------------------------------------------

/// Wave lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WaveState {
    /// Between waves or at game start
    #[default]
    Idle,
    /// Actively spawning enemies on a timer
    Spawning,
    /// All enemies spawned, waiting for remaining enemies to be killed / exit
    Complete,
}

/// A single entry in a wave composition: how many of which enemy type.
#[derive(Debug, Clone, Copy)]
pub struct WaveEntry {
    pub enemy_type: crate::components::EnemyType,
    pub count: u32,
}

/// Tracks wave progression with full auto-spawn support.
#[derive(Resource, Debug, Clone)]
pub struct WaveManager {
    pub current_wave: u32,
    pub state: WaveState,
    /// Timer for spawning individual enemies within a wave
    pub spawn_timer: Timer,
    /// Current wave composition — flat list of enemy types to spawn in order
    pub spawn_queue: Vec<crate::components::EnemyType>,
    /// Index into `spawn_queue` for the next enemy
    pub spawn_index: usize,
    /// Total enemies in the current wave (for UI progress)
    pub total_enemies: u32,
    /// How many enemies have been spawned this session (all waves)
    pub enemies_spawned: u32,
    /// Currently alive enemy count (decremented on kill or base-reach)
    pub enemies_alive: u32,
    /// Timer between waves (countdown in Idle before auto-starting next wave)
    pub interwave_timer: Timer,
    /// Whether to automatically advance to the next wave
    pub auto_start_next: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 0,
            state: WaveState::Idle,
            spawn_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            spawn_queue: Vec::new(),
            spawn_index: 0,
            total_enemies: 0,
            enemies_spawned: 0,
            enemies_alive: 0,
            interwave_timer: Timer::from_seconds(3.0, TimerMode::Once),
            auto_start_next: true,
        }
    }
}

impl WaveManager {
    /// Generate the enemy composition for a given wave number (1-indexed).
    /// Difficulty scales automatically.
    pub fn generate_composition(wave: u32) -> Vec<crate::components::EnemyType> {
        let normal_count = 4 + wave;
        let fast_count = if wave >= 2 { wave - 1 } else { 0 };
        let tank_count = if wave >= 3 { wave - 2 } else { 0 };

        let mut queue = Vec::new();

        // Interleave enemy types for variety: sprinkle fast/tank among normals.
        // Pattern: N, F, N, T, N, F, N...
        let total = normal_count + fast_count + tank_count;
        let mut n_placed = 0u32;
        let mut f_placed = 0u32;
        let mut t_placed = 0u32;

        for i in 0..total {
            // Place a fast enemy every 3rd slot if still have fast enemies
            if i % 3 == 1 && f_placed < fast_count {
                queue.push(crate::components::EnemyType::Fast);
                f_placed += 1;
            } else if i % 5 == 0 && t_placed < tank_count {
                queue.push(crate::components::EnemyType::Tank);
                t_placed += 1;
            } else if n_placed < normal_count {
                queue.push(crate::components::EnemyType::Normal);
                n_placed += 1;
            } else if f_placed < fast_count {
                queue.push(crate::components::EnemyType::Fast);
                f_placed += 1;
            } else if t_placed < tank_count {
                queue.push(crate::components::EnemyType::Tank);
                t_placed += 1;
            }
        }

        queue
    }

    /// Compute spawn interval in seconds for a given wave (gets faster as waves progress).
    pub fn spawn_interval(wave: u32) -> f32 {
        (2.0 - (wave as f32 - 1.0) * 0.15).max(0.6)
    }
}
