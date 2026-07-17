# Tower Siege: Grid Defense

A **2D tower defense game** built with **Rust** and the **Bevy 0.15** game engine. This repository contains Day 1 of a planned 5-day build: a complete, runnable foundation featuring a visible grid map, enemy pathfinding, and core game state management.

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
| **Space** | Spawn a Normal enemy |
| **1** | Spawn a Normal enemy |
| **2** | Spawn a Fast enemy |
| **3** | Spawn a Tank enemy |
| **Escape** | Toggle pause / resume |
| **Left Click** | Debug-print grid cell under cursor (see terminal output) |

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
| `plugins/enemy_plugin.rs` | Enemy spawning, waypoint-based path following, despawning at base |
| `plugins/input_plugin.rs` | Keyboard and mouse input handling for all game states |

### Game States

Three states drive the application flow:

```
MainMenu → Playing → Paused
              ↑         │
              └─────────┘
```

---

## Day 1 Feature Set

- **Bevy 0.15** app with custom window title ("Tower Siege: Grid Defense"), clear color, and 1280×720 resolution
- **State-driven UI**: Main menu, playing HUD (gold, lives, wave number, enemy count), and pause overlay
- **15×10 tile grid** centered in world space (`TILE_SIZE = 64` px)
- **Predefined winding path** with visual tile overlays distinguishing path, buildable, spawn (blue), and base (gold) tiles
- **Three enemy types** rendered as colored circles with outlines:
  - *Normal* — balanced speed and health
  - *Fast* — higher speed, lower health
  - *Tank* — slower, higher health
- **Smooth waypoint following** using linear interpolation along path segments
- **Enemy lifecycle**: spawn at entrance → traverse path → despawn at base (logged to terminal)
- **Grid debug**: click any tile to print its type and coordinates
- **Clean plugin-based architecture** extensible for future days

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
    ├── utils.rs            # Coordinate conversion helpers
    └── plugins/
        ├── mod.rs          # Plugin module exports
        ├── map_plugin.rs   # Grid map generation and rendering
        ├── enemy_plugin.rs # Enemy spawning and path-following
        └── input_plugin.rs # Input handling
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
- [x] Enemies disappear upon reaching the base (logged to terminal)
- [x] **Escape** pauses and resumes the game
- [x] Left-clicking a tile prints its grid cell and type to the terminal

---

## Roadmap

### Day 1 ✓ (Complete)
Foundation: grid, path, enemies, game states, HUD

### Day 2 — Planned
- Tower placement on **Buildable** tiles (click to build)
- Gold cost system + `Occupied` tile state
- Projectile or hitscan damage system
- Lives system (decrement when enemies reach base)
- Multiple tower types (e.g., Arrow, Cannon)

### Day 3-5 — TBD
Wave system, upgrades, UI polish, audio, and more.

---

## License

Educational / portfolio project. Use freely.
