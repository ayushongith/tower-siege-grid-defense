//! Core ECS components for Tower Siege: Grid Defense.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Spatial / motion
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec2);

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

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Armor {
    pub reduction: f32,
}

// ---------------------------------------------------------------------------
// Grid
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

// ---------------------------------------------------------------------------
// Enemies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
    Armored,
    Swarm,
    Boss,
}

impl EnemyType {
    pub fn base_speed(self) -> f32 {
        match self {
            EnemyType::Normal => 80.0,
            EnemyType::Fast => 140.0,
            EnemyType::Tank => 45.0,
            EnemyType::Armored => 55.0,
            EnemyType::Swarm => 100.0,
            EnemyType::Boss => 35.0,
        }
    }

    pub fn base_health(self) -> f32 {
        match self {
            EnemyType::Normal => 50.0,
            EnemyType::Fast => 30.0,
            EnemyType::Tank => 150.0,
            EnemyType::Armored => 80.0,
            EnemyType::Swarm => 12.0,
            EnemyType::Boss => 800.0,
        }
    }

    pub fn gold_value(self) -> u32 {
        match self {
            EnemyType::Normal => 10,
            EnemyType::Fast => 15,
            EnemyType::Tank => 25,
            EnemyType::Armored => 20,
            EnemyType::Swarm => 4,
            EnemyType::Boss => 150,
        }
    }

    pub fn armor_reduction(self) -> f32 {
        match self {
            EnemyType::Armored => 0.40,
            EnemyType::Boss => 0.25,
            _ => 0.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            EnemyType::Normal => Color::srgb(0.90, 0.20, 0.20),
            EnemyType::Fast => Color::srgb(1.00, 0.55, 0.15),
            EnemyType::Tank => Color::srgb(0.55, 0.15, 0.15),
            EnemyType::Armored => Color::srgb(0.50, 0.50, 0.60),
            EnemyType::Swarm => Color::srgb(0.85, 0.35, 0.85),
            EnemyType::Boss => Color::srgb(0.70, 0.10, 0.70),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyType::Normal => 16.0,
            EnemyType::Fast => 12.0,
            EnemyType::Tank => 22.0,
            EnemyType::Armored => 18.0,
            EnemyType::Swarm => 8.0,
            EnemyType::Boss => 32.0,
        }
    }

    pub fn sides(self) -> u32 {
        match self {
            EnemyType::Fast | EnemyType::Swarm => 4,
            EnemyType::Tank | EnemyType::Armored => 6,
            EnemyType::Boss => 8,
            EnemyType::Normal => 0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub base_speed: f32,
    pub speed: f32,
    pub waypoint_index: usize,
    pub progress: f32,
    pub gold_value: u32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        let base_speed = enemy_type.base_speed();
        Self {
            enemy_type,
            base_speed,
            speed: base_speed,
            waypoint_index: 0,
            progress: 0.0,
            gold_value: enemy_type.gold_value(),
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct PathFollower;

// ---------------------------------------------------------------------------
// Status effects
// ---------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct SlowDebuff {
    pub factor: f32,
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct BurnDebuff {
    pub dps: f32,
    pub timer: Timer,
    pub tick: Timer,
}

#[derive(Component, Debug)]
pub struct StunDebuff {
    pub timer: Timer,
}

// ---------------------------------------------------------------------------
// Map visuals
// ---------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct MapTile;

#[derive(Component, Debug)]
pub struct SpawnMarker;

#[derive(Component, Debug)]
pub struct BaseMarker;

#[derive(Component, Debug)]
pub struct PathWaypointMarker;

#[derive(Component, Debug)]
pub struct MapDecoration;

// ---------------------------------------------------------------------------
// Towers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TowerType {
    Arrow,
    Cannon,
    Slow,
    Sniper,
    Tesla,
}

impl TowerType {
    pub const ALL: [TowerType; 5] = [
        TowerType::Arrow,
        TowerType::Cannon,
        TowerType::Slow,
        TowerType::Sniper,
        TowerType::Tesla,
    ];

    pub fn cost(self) -> u32 {
        match self {
            TowerType::Arrow => 50,
            TowerType::Cannon => 100,
            TowerType::Slow => 75,
            TowerType::Sniper => 120,
            TowerType::Tesla => 150,
        }
    }

    pub fn range(self) -> f32 {
        match self {
            TowerType::Arrow => 150.0,
            TowerType::Cannon => 120.0,
            TowerType::Slow => 130.0,
            TowerType::Sniper => 250.0,
            TowerType::Tesla => 140.0,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            TowerType::Arrow => 15.0,
            TowerType::Cannon => 40.0,
            TowerType::Slow => 8.0,
            TowerType::Sniper => 55.0,
            TowerType::Tesla => 18.0,
        }
    }

    pub fn fire_rate(self) -> f32 {
        match self {
            TowerType::Arrow => 1.0,
            TowerType::Cannon => 2.0,
            TowerType::Slow => 1.2,
            TowerType::Sniper => 3.0,
            TowerType::Tesla => 1.5,
        }
    }

    pub fn splash_radius(self) -> f32 {
        match self {
            TowerType::Cannon => 55.0,
            _ => 0.0,
        }
    }

    pub fn chain_count(self) -> u32 {
        match self {
            TowerType::Tesla => 3,
            _ => 0,
        }
    }

    pub fn chain_range(self) -> f32 {
        match self {
            TowerType::Tesla => 80.0,
            _ => 0.0,
        }
    }

    pub fn applies_slow(self) -> bool {
        matches!(self, TowerType::Slow)
    }

    pub fn slow_factor(self) -> f32 {
        if self.applies_slow() { 0.45 } else { 1.0 }
    }

    pub fn slow_duration(self) -> f32 {
        if self.applies_slow() { 2.5 } else { 0.0 }
    }

    pub fn color(self) -> Color {
        match self {
            TowerType::Arrow => Color::srgb(0.20, 0.75, 0.40),
            TowerType::Cannon => Color::srgb(0.75, 0.30, 0.20),
            TowerType::Slow => Color::srgb(0.25, 0.55, 0.90),
            TowerType::Sniper => Color::srgb(0.55, 0.55, 0.65),
            TowerType::Tesla => Color::srgb(0.45, 0.85, 0.95),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TowerType::Arrow => "Arrow",
            TowerType::Cannon => "Cannon",
            TowerType::Slow => "Slow",
            TowerType::Sniper => "Sniper",
            TowerType::Tesla => "Tesla",
        }
    }

    pub fn hotkey(self) -> KeyCode {
        match self {
            TowerType::Arrow => KeyCode::Digit4,
            TowerType::Cannon => KeyCode::Digit5,
            TowerType::Slow => KeyCode::Digit6,
            TowerType::Sniper => KeyCode::Digit7,
            TowerType::Tesla => KeyCode::Digit8,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Tower {
    pub tower_type: TowerType,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
    pub cooldown: Timer,
    pub target_entity: Option<Entity>,
    pub targeting_pos: Vec2,
}

impl Tower {
    pub fn new(tower_type: TowerType) -> Self {
        Self {
            tower_type,
            range: tower_type.range(),
            damage: tower_type.damage(),
            fire_rate: tower_type.fire_rate(),
            cooldown: Timer::from_seconds(tower_type.fire_rate(), TimerMode::Repeating),
            target_entity: None,
            targeting_pos: Vec2::ZERO,
        }
    }
}

#[derive(Component, Debug)]
pub struct TowerTurret;

#[derive(Component, Debug)]
pub struct TowerRangePreview;

#[derive(Component, Debug)]
pub struct TowerPlacementGhost;

// ---------------------------------------------------------------------------
// Projectiles
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub target: Option<Entity>,
    pub speed: f32,
    pub damage: f32,
    pub splash_radius: f32,
    pub chain_remaining: u32,
    pub chain_range: f32,
    pub tower_type: TowerType,
    pub source_tower: Option<Entity>,
}

// ---------------------------------------------------------------------------
// Selection / upgrades
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct TowerSelection {
    pub selected: Option<TowerType>,
}

#[derive(Component, Debug)]
pub struct HealthBar;

#[derive(Component, Debug, Clone)]
pub struct TowerLevel {
    pub level: u32,
    pub max_level: u32,
    pub total_invested: u32,
}

impl TowerLevel {
    pub fn new(base_cost: u32) -> Self {
        Self {
            level: 1,
            max_level: 3,
            total_invested: base_cost,
        }
    }
}

#[derive(Component, Debug)]
pub struct HitEffect {
    pub timer: Timer,
}

#[derive(Resource, Debug, Default)]
pub struct TowerEditTarget {
    pub entity: Option<Entity>,
    pub col: usize,
    pub row: usize,
}

#[derive(Component, Debug)]
pub struct SelectionRing;

#[derive(Component, Debug)]
pub struct ChainLightningFx {
    pub timer: Timer,
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct MainGameCamera;

#[derive(Resource, Debug)]
pub struct CameraController {
    pub pan_speed: f32,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            pan_speed: 420.0,
            zoom: 1.0,
            min_zoom: 0.5,
            max_zoom: 2.5,
        }
    }
}
