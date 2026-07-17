//! Tower Siege: Grid Defense — Day 1 data layer (resources + grid utils).

mod components;
mod resources;
mod utils;

use bevy::prelude::*;

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
        .init_resource::<resources::GameStats>()
        .init_resource::<resources::WaveManager>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
