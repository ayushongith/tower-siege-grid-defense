//! Audio Plugin — background music and sound effects mixer.

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;

use crate::resources::AudioSettings;
use crate::AppState;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_music)
            .add_systems(OnExit(AppState::Playing), stop_music)
            .add_systems(Update, update_music_volume);
    }
}

#[derive(Component)]
struct BackgroundMusic;

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("music/background.mp3")),
        PlaybackSettings::LOOP,
        BackgroundMusic,
    ));
}

fn stop_music(mut commands: Commands, q_music: Query<Entity, With<BackgroundMusic>>) {
    for entity in &q_music {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_music_volume(
    settings: Res<AudioSettings>,
    q_music: Query<&AudioSink, With<BackgroundMusic>>,
) {
    if settings.is_changed() {
        if let Ok(sink) = q_music.get_single() {
            let volume = if settings.music_enabled {
                settings.master * settings.music
            } else {
                0.0
            };
            sink.set_volume(volume);
        }
    }
}
