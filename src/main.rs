//! Tower Siege: Grid Defense — Day 1 enemy pathing layer.

mod components;
mod plugins;
mod resources;
mod utils;

use bevy::prelude::*;

use plugins::{EnemyPlugin, MapPlugin};
use resources::{GameStats, WaveManager};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tower Siege: Grid Defense".into(),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.12, 0.16, 0.12)))
        .init_resource::<GameStats>()
        .init_resource::<WaveManager>()
        .init_state::<AppState>()
        .add_plugins((MapPlugin, EnemyPlugin))
        // Enter Playing immediately so enemy systems are active for later input.
        .add_systems(Startup, |mut next: ResMut<NextState<AppState>>| {
            next.set(AppState::Playing);
        })
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
        Name::new("MainCamera"),
    ));
}
