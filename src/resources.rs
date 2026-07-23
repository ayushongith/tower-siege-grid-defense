//! Global game resources.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{EnemyType, TowerType};

// ---------------------------------------------------------------------------
// Difficulty / game mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn starting_gold(self) -> u32 {
        match self {
            Difficulty::Easy => 350,
            Difficulty::Normal => 250,
            Difficulty::Hard => 150,
        }
    }

    pub fn starting_lives(self) -> u32 {
        match self {
            Difficulty::Easy => 30,
            Difficulty::Normal => 20,
            Difficulty::Hard => 10,
        }
    }

    pub fn spawn_rate_mult(self) -> f32 {
        match self {
            Difficulty::Easy => 1.25,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.75,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GameMode {
    #[default]
    Campaign,
    Endless,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Campaign => "Campaign",
            GameMode::Endless => "Endless",
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub difficulty: Difficulty,
    pub mode: GameMode,
    pub map_index: usize,
    pub endless: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            mode: GameMode::Campaign,
            map_index: 0,
            endless: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Economy / run stats
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone)]
pub struct GameStats {
    pub gold: u32,
    pub lives: u32,
    pub kills: u32,
    pub towers_built: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            gold: 250,
            lives: 20,
            kills: 0,
            towers_built: 0,
        }
    }
}

impl GameStats {
    pub fn apply_difficulty(&mut self, difficulty: Difficulty) {
        self.gold = difficulty.starting_gold();
        self.lives = difficulty.starting_lives();
        self.kills = 0;
        self.towers_built = 0;
    }
}

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TileType {
    #[default]
    Buildable,
    Path,
    Occupied,
    Spawn,
    Base,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDefinition {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub grid_path: Vec<(usize, usize)>,
}

#[derive(Resource, Debug, Clone)]
pub struct Map {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    pub path: Vec<Vec2>,
    pub dirty: bool,
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
            if self.tiles[i] != tile {
                self.tiles[i] = tile;
                self.dirty = true;
            }
        }
    }

    pub fn from_definition(def: &MapDefinition) -> Self {
        let width = def.width;
        let height = def.height;
        let mut tiles = vec![TileType::Buildable; width * height];

        for window in def.grid_path.windows(2) {
            let (c0, r0) = window[0];
            let (c1, r1) = window[1];
            paint_orthogonal(&mut tiles, width, height, c0, r0, c1, r1);
        }

        if let Some(&(sc, sr)) = def.grid_path.first() {
            tiles[sr * width + sc] = TileType::Spawn;
        }
        if let Some(&(bc, br)) = def.grid_path.last() {
            tiles[br * width + bc] = TileType::Base;
        }

        let path: Vec<Vec2> = def
            .grid_path
            .iter()
            .map(|(c, r)| crate::utils::grid_to_world(*c, *r, width, height))
            .collect();

        Self {
            name: def.name.clone(),
            width,
            height,
            tiles,
            path,
            dirty: false,
        }
    }

    pub fn generate_day1() -> Self {
        Self::from_definition(&MapDefinition {
            name: "Classic".into(),
            width: Self::WIDTH,
            height: Self::HEIGHT,
            grid_path: vec![
                (0, 4), (2, 4), (2, 7), (5, 7), (5, 2), (8, 2), (8, 8), (11, 8), (11, 3),
                (13, 3), (13, 6), (14, 6),
            ],
        })
    }
}

pub fn load_map_by_index(index: usize) -> Map {
    let maps = available_maps();
    if let Some(def) = maps.get(index) {
        Map::from_definition(def)
    } else {
        Map::generate_day1()
    }
}

pub fn available_maps() -> Vec<MapDefinition> {
    let mut maps = Vec::new();

    for file in ["maps/classic.ron", "maps/zigzag.ron", "maps/arena.ron"] {
        if let Ok(text) = std::fs::read_to_string(format!("assets/{file}")) {
            if let Ok(def) = ron::from_str::<MapDefinition>(&text) {
                maps.push(def);
                continue;
            }
        }
    }

    if maps.is_empty() {
        maps.push(MapDefinition {
            name: "Classic".into(),
            width: Map::WIDTH,
            height: Map::HEIGHT,
            grid_path: vec![
                (0, 4), (2, 4), (2, 7), (5, 7), (5, 2), (8, 2), (8, 8), (11, 8), (11, 3),
                (13, 3), (13, 6), (14, 6),
            ],
        });
        maps.push(MapDefinition {
            name: "Zigzag".into(),
            width: Map::WIDTH,
            height: Map::HEIGHT,
            grid_path: vec![
                (0, 1), (3, 1), (3, 8), (6, 8), (6, 2), (9, 2), (9, 7), (12, 7), (12, 4),
                (14, 4),
            ],
        });
        maps.push(MapDefinition {
            name: "Arena".into(),
            width: Map::WIDTH,
            height: Map::HEIGHT,
            grid_path: vec![
                (0, 5), (4, 5), (4, 1), (7, 1), (7, 9), (10, 9), (10, 3), (13, 3), (13, 6),
                (14, 6),
            ],
        });
    }

    maps
}

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
    }
}

pub fn tile_color(tile: TileType) -> Color {
    match tile {
        TileType::Buildable => Color::srgb(0.35, 0.55, 0.32),
        TileType::Path => Color::srgb(0.45, 0.32, 0.18),
        TileType::Occupied => Color::srgb(0.30, 0.30, 0.30),
        TileType::Spawn => Color::srgb(0.20, 0.55, 0.85),
        TileType::Base => Color::srgb(0.75, 0.55, 0.15),
    }
}

// ---------------------------------------------------------------------------
// Waves
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WaveState {
    #[default]
    Idle,
    Spawning,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WaveModifier {
    #[default]
    None,
    Fast,
    Armored,
    BonusGold,
}

impl WaveModifier {
    pub fn label(self) -> &'static str {
        match self {
            WaveModifier::None => "",
            WaveModifier::Fast => "Fast Wave!",
            WaveModifier::Armored => "Armored Wave!",
            WaveModifier::BonusGold => "Bonus Gold Wave!",
        }
    }

    pub fn for_wave(wave: u32) -> Self {
        match wave % 5 {
            0 if wave > 0 => WaveModifier::BonusGold,
            2 => WaveModifier::Fast,
            3 => WaveModifier::Armored,
            _ => WaveModifier::None,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct WaveManager {
    pub current_wave: u32,
    pub state: WaveState,
    pub spawn_timer: Timer,
    pub spawn_queue: Vec<EnemyType>,
    pub spawn_index: usize,
    pub total_enemies: u32,
    pub enemies_spawned: u32,
    pub enemies_alive: u32,
    pub interwave_timer: Timer,
    pub auto_start_next: bool,
    pub modifier: WaveModifier,
    pub campaign_victory_wave: u32,
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
            modifier: WaveModifier::None,
            campaign_victory_wave: 10,
        }
    }
}

impl WaveManager {
    pub fn generate_composition(wave: u32, modifier: WaveModifier) -> Vec<EnemyType> {
        let mut queue = Vec::new();

        if wave == 5 || wave == 10 {
            queue.push(EnemyType::Boss);
        }

        let normal_count = 3 + wave;
        let fast_count = if wave >= 2 { wave.saturating_sub(1) } else { 0 };
        let tank_count = if wave >= 3 { wave.saturating_sub(2) } else { 0 };
        let armored_count = if wave >= 4 { wave / 2 } else { 0 };
        let swarm_count = if wave >= 3 { wave } else { 0 };

        let total = normal_count + fast_count + tank_count + armored_count + swarm_count;
        let mut n = 0u32;
        let mut f = 0u32;
        let mut t = 0u32;
        let mut a = 0u32;
        let mut s = 0u32;

        for i in 0..total {
            if modifier == WaveModifier::Armored && a < armored_count.max(2) {
                queue.push(EnemyType::Armored);
                a += 1;
            } else if i % 7 == 0 && s < swarm_count {
                queue.push(EnemyType::Swarm);
                s += 1;
            } else if i % 5 == 0 && t < tank_count {
                queue.push(EnemyType::Tank);
                t += 1;
            } else if i % 3 == 1 && f < fast_count {
                queue.push(EnemyType::Fast);
                f += 1;
            } else if a < armored_count {
                queue.push(EnemyType::Armored);
                a += 1;
            } else if n < normal_count {
                queue.push(EnemyType::Normal);
                n += 1;
            } else if f < fast_count {
                queue.push(EnemyType::Fast);
                f += 1;
            } else if s < swarm_count {
                queue.push(EnemyType::Swarm);
                s += 1;
            }
        }

        if modifier == WaveModifier::Fast {
            for _ in 0..(wave / 2 + 1) {
                queue.push(EnemyType::Fast);
            }
        }

        queue
    }

    pub fn spawn_interval(wave: u32, difficulty: Difficulty) -> f32 {
        let base = (1.8 - (wave as f32 - 1.0) * 0.10).max(0.35);
        base * difficulty.spawn_rate_mult()
    }
}

// ---------------------------------------------------------------------------
// Audio settings
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master: f32,
    pub sfx: f32,
    pub music: f32,
    pub music_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 0.8,
            sfx: 0.9,
            music: 0.5,
            music_enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Save data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveData {
    pub high_score: u32,
    pub best_wave: u32,
    pub total_kills: u32,
    pub games_played: u32,
    pub unlocked_towers: Vec<TowerType>,
    pub audio: AudioSettings,
}

impl SaveData {
    pub const PATH: &'static str = "assets/save.json";

    pub fn load() -> Self {
        std::fs::read_to_string(Self::PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(Self::default_new)
    }

    pub fn save(&self) {
        std::fs::create_dir_all("assets").ok();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(Self::PATH, json).ok();
        }
    }

    fn default_new() -> Self {
        Self {
            high_score: 0,
            best_wave: 0,
            total_kills: 0,
            games_played: 0,
            unlocked_towers: TowerType::ALL.to_vec(),
            audio: AudioSettings::default(),
        }
    }

    pub fn record_run(&mut self, wave: u32, kills: u32, score: u32) {
        self.games_played += 1;
        self.total_kills += kills;
        self.best_wave = self.best_wave.max(wave);
        self.high_score = self.high_score.max(score);
        self.save();
    }
}

#[derive(Resource, Debug, Clone)]
pub struct RunScore {
    pub value: u32,
}

impl Default for RunScore {
    fn default() -> Self {
        Self { value: 0 }
    }
}
