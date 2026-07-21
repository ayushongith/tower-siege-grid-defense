# Tower Siege: Grid Defense

A **2D tower defense game** built with **Rust** and the **Bevy 0.15** game engine. This repository is a complete 5-day build: a fully playable tower defense game with waves, towers, upgrades, audio, and game-over/victory states.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Build Notes](#build-notes)
- [Controls](#controls)
- [Architecture](#architecture)
- [Day 1 Feature Set](#day-1-feature-set)
- [Project Layout](#project-layout)
- [Verification](#verification)
- [Roadmap](#roadmap)
- [License](#license)

---

## Quick Start

Ensure you have the Rust toolchain installed (`rustup` + `cargo`).

```bash
git clone https://github.com/ayushongith/tower-siege-grid-defense.git
cd tower-siege-grid-defense
cargo run
```

For a smoother experience (optimized build):

```bash
cargo run --release
```

> **Note:** The first Bevy compilation can take several minutes as it builds the engine and its dependencies from source.

---

## Build Notes

### Default: GNU toolchain with `rust-lld`

This project is configured to build with the **`x86_64-pc-windows-gnu`** target and the **`rust-lld`** linker (see `.cargo/config.toml`). This works without Visual Studio installed.

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup component add rust-mingw --toolchain stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu run
```

### Alternative: MSVC toolchain

If you prefer the MSVC toolchain (the default on Windows), install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (C++ workload), then remove or edit `.cargo/config.toml` and run:

```bash
cargo run
```

---

## Controls

| Input | Action |
|-------|--------|
| **Enter** / **Space** | Start game from main menu |
| **Enter** / **Space** | Start game / restart after game over or victory / spawn Normal during play |
| **1** | Spawn a Normal enemy (manual override) |
| **2** | Spawn a Fast enemy |
| **3** | Spawn a Tank enemy |
| **4** | Select Arrow tower (50g, place with click) |
| **5** | Select Cannon tower (100g, place with click) |
| **U** | Upgrade selected existing tower |
| **S** | Sell selected existing tower (50% refund) |
| **Escape** | Clear tower selection / tower edit / toggle pause |
| **Left Click** | Place selected tower / select existing tower / debug-print grid cell |
| | **Waves auto-start when Playing begins; press Space/1/2/3 for manual spawns** |

---

## Architecture

The project follows Bevy's **Entity Component System (ECS)** pattern, organized into modular plugins:

| Module | Responsibility |
|--------|---------------|
| `main.rs` | App entry point, Bevy plugin registration, UI rendering (main menu, HUD, pause overlay), state transitions |
| `components.rs` | ECS component definitions (`Position`, `Health`, `Enemy`, `GridPosition`, etc.) |
| `resources.rs` | Global game resources (`GameStats`, `WaveManager`, `Map` with `TileType` grid) |
| `utils.rs` | Coordinate conversion utilities (`world_to_grid`, `grid_to_world`) |
| `plugins/map_plugin.rs` | Grid generation, path definition, tile rendering and coloring |
| `plugins/enemy_plugin.rs` | Enemy spawning, waypoint-based path following, lives on base-reach |
| `plugins/input_plugin.rs` | Keyboard and mouse input handling for all game states |
| `plugins/tower_plugin.rs` | Tower placement, targeting nearest enemy, projectile spawning |
| `plugins/projectile_plugin.rs` | Projectile movement, collision, damage application, gold rewards |
| `plugins/visual_plugin.rs` | Health bar updates, hit-effect particles, visual feedback |
| `plugins/wave_plugin.rs` | Wave definitions, timed spawning, state transitions, announcements |

### Game States

Three states drive the application flow:

```
MainMenu → Playing → Paused
              ↑         │
              └─────────┘
```

---

## Feature Set

### Day 1 — Foundation ✓
- **Bevy 0.15** app with custom window title ("Tower Siege: Grid Defense"), clear color, and 1280×720 resolution
- **State-driven UI**: Main menu, playing HUD (gold, lives, wave number, enemy count), and pause overlay
- **15×10 tile grid** centered in world space (`TILE_SIZE = 64` px)
- **Predefined winding path** with visual tile overlays distinguishing path, buildable, spawn (blue), and base (gold) tiles
- **Three enemy types** rendered as colored circles with outlines:
  - *Normal* — balanced speed and health
  - *Fast* — higher speed, lower health
  - *Tank* — slower, higher health
- **Smooth waypoint following** using linear interpolation along path segments

### Day 2 — Towers & Combat ✓
- **Two tower types**: Arrow (50g, fast fire) and Cannon (100g, heavy damage)
- **Tower placement**: select tower type (4/5), click any **Buildable** tile
- **Economy**: gold deducted on placement, **Occupied** tile state prevents stacking
- **Auto-targeting**: towers acquire the nearest enemy within range each frame
- **Projectile system**: homing projectiles fired from towers toward targets
- **Damage & kills**: projectiles apply damage on arrival; enemies award gold on death
- **Lives system**: enemies reaching the base decrement lives (shown in HUD)
- **Tower selection HUD**: displays active tower type in the status bar

### Day 3 — Wave System ✓
- **Auto-scaling wave generation**: each wave composed of Normal, Fast, and Tank enemies
- **Timed spawn schedule**: enemies spawn automatically at calculated intervals
- **Interleaved composition**: enemy types mixed within each wave for variety
- **Wave state machine**: Idle → Spawning → Complete, with automatic transitions
- **Inter-wave countdown**: 3-second delay between waves before auto-advancing
- **Wave announcements**: "Wave N!" on start, "Wave N complete!" on finish (2-second banner)
- **HUD progress**: displays spawned / total enemies for the current wave
- **Manual spawns preserved**: Space/1/2/3 still work for testing alongside waves

### Day 4 — Upgrades & Visuals ✓
- **Health bars**: colored bar above each enemy (green → yellow → red based on HP%)
- **Hit effects**: expanding circle particle on projectile impact with alpha fade
- **VisualPlugin**: centralized system for health bar updates and hit effect lifecycle
- **Tower upgrading**: select a tower by clicking, press **U** to upgrade (up to level 3)
- **Upgrade stats**: +25% damage, +10% range, -10% fire rate per level
- **Upgrade cost**: scales per level (base_cost × 0.75 × level)
- **Tower sell-back**: press **S** on selected tower to refund 50% of total investment
- **Selection ring**: translucent yellow circle highlights the selected tower
- **HUD hints**: shows upgrade cost and sell refund when a tower is selected
- **Turret parenting**: turret is child of tower base, despawns recursively on sell

### Day 5 — Release ✓
- **Game Over state**: triggers when lives reach 0, with full-screen overlay
- **Victory state**: triggers after clearing 10 waves, with congratulations screen
- **Restart flow**: Enter/Space from Game Over or Victory returns to MainMenu with all state reset
- **Cleanup system**: despawning all gameplay entities (enemies, towers, projectiles) on game end
- **Procedural audio**: six sound effects generated as WAV files at runtime (no external assets)
- **Event-driven SFX**: SfxRequest events trigger sounds for shooting, hits, kills, and wave starts
- **Enemy scaling**: HP scales +10% per wave, gold rewards scale +2 per wave
- **Balancing tuned**: starting gold 250, smoother wave composition curve, faster spawn intervals

---

## Project Layout

```
tower-siege-grid-defense/
├── Cargo.toml              # Project manifest and dependencies
├── Cargo.lock              # Dependency lockfile
├── README.md               # This file
├── .gitignore              # Git exclusion rules
├── .cargo/
│   └── config.toml         # Build target and linker configuration
└── src/
    ├── main.rs             # App entry point, plugins, UI, state management
    ├── components.rs       # ECS component definitions
    ├── resources.rs        # Global resources and game state
    ├── sfx.rs              # Procedural audio generation and playback
    ├── utils.rs            # Coordinate conversion helpers
    └── plugins/
        ├── mod.rs          # Plugin module exports
        ├── map_plugin.rs           # Grid map generation and rendering
        ├── enemy_plugin.rs         # Enemy spawning and path-following
        ├── input_plugin.rs         # Input handling
        ├── tower_plugin.rs         # Tower placement, targeting, shooting, upgrades, sell-back
        ├── projectile_plugin.rs    # Projectile movement, damage, hit effects
        ├── visual_plugin.rs        # Health bars, hit-effect particles, visual feedback
        └── wave_plugin.rs          # Wave definitions, timed spawning, transitions
```

---

## Verification

When you run `cargo run`, confirm the following:

- [x] A 1280×720 window opens titled "Tower Siege: Grid Defense"
- [x] The main menu displays with controls information
- [x] Pressing **Enter** or **Space** transitions to the playing state
- [x] A visible 15×10 grid is rendered with a winding path
- [x] A **Spawn** marker (blue) and **Base** marker (gold) are visible
- [x] Pressing **Space**, **1**, **2**, or **3** spawns enemies that traverse the path smoothly
- [x] Enemies disappear upon reaching the base (decrements lives, logged to terminal)
- [x] **Escape** pauses and resumes the game
- [x] **4** selects Arrow tower, **5** selects Cannon tower (shown in HUD)
- [x] Click a **Buildable** tile to place a tower (gold deducted, tile turns Occupied)
- [x] Towers automatically target and fire projectiles at nearby enemies
- [x] Enemies die from projectile damage and award gold
- [x] Left-clicking a non-buildable tile prints its grid cell and type
- [x] On entering Playing, Wave 1 automatically starts (5 Normal enemies)
- [x] Enemies spawn one at a time with a visible timer interval
- [x] "Wave 1!" announcement banner appears briefly at wave start
- [x] After all enemies are spawned, state transitions to Complete → Idle
- [x] After 3 seconds in Idle, Wave 2 starts with more enemies (mixed types)
- [x] "Wave 1 complete!" and "Wave 2!" banners appear at transitions
- [x] HUD shows "Enemies: spawned/total" progress for the current wave
- [x] Enemies have colored health bars above them (green → yellow → red)
- [x] Projectile impacts show a brief expanding circle hit effect
- [x] Click an occupied tile to select its tower (yellow ring appears)
- [x] Press **U** to upgrade: damage, range, and fire rate improve; gold is deducted
- [x] Press **S** to sell: tower and turret disappear, tile becomes Buildable, 50% refunded
- [x] HUD shows upgrade cost and sell refund when a tower is selected
- [x] When lives reach 0, Game Over screen appears with restart prompt
- [x] After clearing 10 waves, Victory screen appears with congratulations
- [x] Press Enter/Space on Game Over or Victory to return to MainMenu
- [x] Restarting resets all state: gold, lives, waves, map tiles, and despawns all entities
- [x] Shooting, hit, kill, and wave start sounds play during gameplay
- [x] Game Over and Victory have distinct sound effects
- [x] Enemy HP increases ~10% per wave (tougher late-game enemies)

---

## Roadmap

### Day 1 ✓ (Complete)
Foundation: grid, path, enemies, game states, HUD

### Day 2 ✓ (Complete)
Towers, projectiles, economy, lives, tower types

### Day 3 ✓ (Complete)
Auto-scaling wave system with timed spawns, wave UI, mixed compositions, wave state machine

### Day 4 ✓ (Complete)
Tower upgrades, sell-back, health bars, hit effects, selection ring, HUD integration

### Day 5 ✓ (Complete)
Game over / victory states, procedural audio, balancing, restart flow, release

---

## License

Educational / portfolio project. Use freely.
