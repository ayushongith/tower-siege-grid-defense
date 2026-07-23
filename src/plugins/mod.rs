//! Gameplay plugins for Tower Siege (Day 1).
//!
//! Each plugin owns a bounded vertical slice:
//! - `MapPlugin`     — grid, path, static visuals
//! - `EnemyPlugin`   — spawn + path following
//! - `InputPlugin`   — keyboard / mouse → state & debug actions
//!
//! Splitting by domain keeps `main.rs` thin and makes Day 2 tower systems
//! a new plugin instead of a tangled free-function dump.

pub mod audio_plugin;
pub mod camera_plugin;
pub mod enemy_plugin;
pub mod input_plugin;
pub mod map_plugin;
pub mod projectile_plugin;
pub mod status_plugin;
pub mod tower_plugin;
pub mod ui_plugin;
pub mod visual_plugin;
pub mod wave_plugin;

pub use audio_plugin::AudioPlugin;
pub use camera_plugin::CameraPlugin;
pub use enemy_plugin::EnemyPlugin;
pub use input_plugin::InputPlugin;
pub use map_plugin::MapPlugin;
pub use projectile_plugin::ProjectilePlugin;
pub use status_plugin::StatusPlugin;
pub use tower_plugin::TowerPlugin;
pub use ui_plugin::UiPlugin;
pub use visual_plugin::VisualPlugin;
pub use wave_plugin::WavePlugin;
