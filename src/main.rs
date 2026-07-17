//! Tower Siege: Grid Defense — project scaffold (Day 1, step 1).

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
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
