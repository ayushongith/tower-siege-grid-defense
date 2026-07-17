# Tower Siege: Grid Defense

Portfolio-oriented **2D tower defense** built with **Rust + Bevy 0.15**.

This repository currently contains **Day 1** of a 5-day build plan: a solid, runnable foundation where enemies spawn and walk a predefined path on a visible grid.

## Day 1 — Foundation

### How to run

```bash
cd tower_siege
cargo run
```

Release build (smoother):

```bash
cargo run --release
```

### Windows build notes

This repo includes `.cargo/config.toml` that targets **`x86_64-pc-windows-gnu`** with **`rust-lld`** (works without Visual Studio if you have the GNU Rust toolchain):

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup component add rust-mingw --toolchain stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu run
```

**MSVC alternative:** Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (C++ workload), remove or adjust `.cargo/config.toml`, then `cargo run` on the default MSVC toolchain.

First Bevy compile can take several minutes.

### Controls (Day 1)

| Input | Action |
|--------|--------|
| **Enter** or **Space** | Start game from main menu |
| **Space** | Spawn a Normal test enemy |
| **1** | Spawn Normal enemy |
| **2** | Spawn Fast enemy |
| **3** | Spawn Tank enemy |
| **Escape** | Pause / resume |
| **Left mouse** | Debug-print grid cell under cursor (see terminal log) |

### What Day 1 accomplishes

- Bevy 0.15 app with window title, clear color, and 1280×720 view
- `AppState`: `MainMenu` → `Playing` → `Paused`
- Clean ECS layout: `components`, `resources`, `utils`, domain **plugins**
- **15×10** grid map centered in world space (`TILE_SIZE = 64`)
- Winding **path** from left (Spawn) to right (Base) with painted tiles + overlays
- Enemies as colored circles with outline; **smooth waypoint following** (lerp along segments)
- Enemies despawn at the base with a log message (`reached base`)
- Resources ready for later days: `GameStats`, `WaveManager`, `Map` / `TileType`
- Grid helpers: `world_to_grid` / `grid_to_world`

### Project layout

```
tower_siege/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs
    ├── components.rs
    ├── resources.rs
    ├── utils.rs
    └── plugins/
        ├── mod.rs
        ├── map_plugin.rs
        ├── enemy_plugin.rs
        └── input_plugin.rs
```

### Milestone checklist

When you run `cargo run` you should see:

- [x] Visible grid with a clear winding path  
- [x] Spawn (blue) and Base (gold) markers  
- [x] **Space** spawns enemies that walk the full path smoothly  
- [x] **ESC** toggles pause  
- [x] Main menu → Playing transition  

## Next steps — Day 2 (planned)

- Tower placement on **Buildable** tiles (click to build)
- Gold costs + `Occupied` tiles
- Simple projectile or hitscan damage
- Lives decrease when enemies reach the base
- Basic tower types (e.g. Arrow / Cannon)

## License

Educational / portfolio project — use freely.
