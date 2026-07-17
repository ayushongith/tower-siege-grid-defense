//! Core ECS components for Tower Siege: Grid Defense (Day 1).
//!
//! Components are pure data. Behaviour lives in systems (plugins).
//! This separation keeps the architecture scalable for towers, projectiles,
//! and combat systems in later days.

// Several components are scaffolding for Days 2–5; silence until wired up.
#![allow(dead_code)]

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Spatial / motion
// ---------------------------------------------------------------------------

/// World-space position (Bevy `Transform` is the render source of truth;
/// we keep `Position` in sync for gameplay queries that should not depend on
/// render hierarchy).
#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Linear velocity in world units per second. Reserved for projectiles / knockback
/// on later days; enemies primarily use path following instead.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec2);

/// Hit points. Day 1 does not apply damage yet, but the component is ready
/// so Day 2 towers can reduce `current` without a data-model rewrite.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }
}

// ---------------------------------------------------------------------------
// Grid
// ---------------------------------------------------------------------------

/// Discrete tile coordinate on the map grid.
/// `col` grows right, `row` grows up (matches Bevy's Y-up world).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

// ---------------------------------------------------------------------------
// Enemies
// ---------------------------------------------------------------------------

/// High-level enemy archetype. Stats are duplicated onto `Enemy` at spawn time
/// so runtime tuning / buffs do not require looking up a global table every frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
}

impl EnemyType {
    pub fn base_speed(self) -> f32 {
        match self {
            EnemyType::Normal => 80.0,
            EnemyType::Fast => 140.0,
            EnemyType::Tank => 45.0,
        }
    }

    pub fn base_health(self) -> f32 {
        match self {
            EnemyType::Normal => 50.0,
            EnemyType::Fast => 30.0,
            EnemyType::Tank => 150.0,
        }
    }

    pub fn gold_value(self) -> u32 {
        match self {
            EnemyType::Normal => 10,
            EnemyType::Fast => 15,
            EnemyType::Tank => 25,
        }
    }

    pub fn color(self) -> Color {
        match self {
            EnemyType::Normal => Color::srgb(0.90, 0.20, 0.20),
            EnemyType::Fast => Color::srgb(1.00, 0.55, 0.15),
            EnemyType::Tank => Color::srgb(0.55, 0.15, 0.15),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyType::Normal => 16.0,
            EnemyType::Fast => 12.0,
            EnemyType::Tank => 22.0,
        }
    }
}

/// Path-following enemy state.
///
/// Movement model (Day 1):
/// - `waypoint_index` = index of the *current segment start* in `Map.path`
/// - `progress` ∈ [0, 1) = interpolation factor toward the next waypoint
///
/// This is more stable than pure velocity steering for grid paths and makes
/// future features (slow debuffs, path re-routing) easier to reason about.
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub speed: f32,
    pub waypoint_index: usize,
    pub progress: f32,
    pub gold_value: u32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        Self {
            enemy_type,
            speed: enemy_type.base_speed(),
            waypoint_index: 0,
            progress: 0.0,
            gold_value: enemy_type.gold_value(),
        }
    }
}

/// Marker: this entity should be advanced along `Map.path` by the enemy plugin.
#[derive(Component, Debug, Default)]
pub struct PathFollower;

// ---------------------------------------------------------------------------
// Map visuals (markers so we can query tiles later for placement rules)
// ---------------------------------------------------------------------------

/// Marker for a single map tile entity.
#[derive(Component, Debug)]
pub struct MapTile;

/// Marker for the spawn pad visual.
#[derive(Component, Debug)]
pub struct SpawnMarker;

/// Marker for the base / exit visual.
#[derive(Component, Debug)]
pub struct BaseMarker;

/// Marker for path waypoint dots (debug / readability).
#[derive(Component, Debug)]
pub struct PathWaypointMarker;
