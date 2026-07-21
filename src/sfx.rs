use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};

// ---------------------------------------------------------------------------
// WAV file generation (no external crates needed)
// ---------------------------------------------------------------------------

fn generate_wav_sine(freq: f32, duration_secs: f32, sample_rate: u32) -> Vec<u8> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let data_len = num_samples * 2; // 16-bit mono
    let file_size = 44 + data_len;

    let mut wav = Vec::with_capacity(file_size);

    // RIFF header
    wav.extend(b"RIFF");
    wav.extend(&(file_size as u32 - 8).to_le_bytes());
    wav.extend(b"WAVE");

    // fmt chunk
    wav.extend(b"fmt ");
    wav.extend(&16u32.to_le_bytes()); // chunk size
    wav.extend(&1u16.to_le_bytes());  // PCM
    wav.extend(&1u16.to_le_bytes());  // mono
    wav.extend(&sample_rate.to_le_bytes());
    wav.extend(&(sample_rate * 2).to_le_bytes()); // byte rate
    wav.extend(&2u16.to_le_bytes());  // block align
    wav.extend(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend(b"data");
    wav.extend(&(data_len as u32).to_le_bytes());

    // PCM samples (16-bit signed)
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let envelope = {
            let attack = (i as f32 / (sample_rate as f32 * 0.01)).min(1.0);
            let release = ((num_samples - i) as f32 / (sample_rate as f32 * 0.02)).min(1.0);
            attack.min(release) * 0.30
        };
        let sample = (t * freq * std::f32::consts::TAU).sin() * envelope;
        let amplitude = (sample * 32767.0) as i16;
        wav.extend(&amplitude.to_le_bytes());
    }

    wav
}

fn generate_wav_sweep(start_freq: f32, end_freq: f32, duration_secs: f32, sample_rate: u32) -> Vec<u8> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let data_len = num_samples * 2;
    let file_size = 44 + data_len;

    let mut wav = Vec::with_capacity(file_size);

    wav.extend(b"RIFF");
    wav.extend(&(file_size as u32 - 8).to_le_bytes());
    wav.extend(b"WAVE");
    wav.extend(b"fmt ");
    wav.extend(&16u32.to_le_bytes());
    wav.extend(&1u16.to_le_bytes());
    wav.extend(&1u16.to_le_bytes());
    wav.extend(&sample_rate.to_le_bytes());
    wav.extend(&(sample_rate * 2).to_le_bytes());
    wav.extend(&2u16.to_le_bytes());
    wav.extend(&16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(&(data_len as u32).to_le_bytes());

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let freq = start_freq + (end_freq - start_freq) * (t / duration_secs).min(1.0);
        let envelope = {
            let attack = (i as f32 / (sample_rate as f32 * 0.01)).min(1.0);
            let release = ((num_samples - i) as f32 / (sample_rate as f32 * 0.02)).min(1.0);
            attack.min(release) * 0.30
        };
        let sample = (t * freq * std::f32::consts::TAU).sin() * envelope;
        let amplitude = (sample * 32767.0) as i16;
        wav.extend(&amplitude.to_le_bytes());
    }

    wav
}

// ---------------------------------------------------------------------------
// Asset handles resource
// ---------------------------------------------------------------------------

#[derive(Resource, Debug)]
pub struct SfxHandles {
    pub shoot: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub kill: Handle<AudioSource>,
    pub wave_start: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub victory: Handle<AudioSource>,
}

/// Write all sound-effect WAV files to disk, then load them via AssetServer.
pub fn setup_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dir = "assets/sounds";
    std::fs::create_dir_all(dir).ok();

    std::fs::write(format!("{dir}/shoot.wav"), &generate_wav_sine(880.0, 0.08, 44100)).ok();
    std::fs::write(format!("{dir}/hit.wav"), &generate_wav_sine(220.0, 0.06, 44100)).ok();
    std::fs::write(format!("{dir}/kill.wav"), &generate_wav_sweep(440.0, 880.0, 0.15, 44100)).ok();
    std::fs::write(format!("{dir}/wave_start.wav"), &generate_wav_sweep(440.0, 660.0, 0.3, 44100)).ok();
    std::fs::write(format!("{dir}/game_over.wav"), &generate_wav_sweep(440.0, 110.0, 0.5, 44100)).ok();
    std::fs::write(format!("{dir}/victory.wav"), &generate_wav_sweep(440.0, 880.0, 0.5, 44100)).ok();

    commands.insert_resource(SfxHandles {
        shoot: asset_server.load("sounds/shoot.wav"),
        hit: asset_server.load("sounds/hit.wav"),
        kill: asset_server.load("sounds/kill.wav"),
        wave_start: asset_server.load("sounds/wave_start.wav"),
        game_over: asset_server.load("sounds/game_over.wav"),
        victory: asset_server.load("sounds/victory.wav"),
    });
}

/// Play a one-shot sound effect.
pub fn play_sfx(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings::ONCE,
        Name::new("SfxPlayer"),
    ));
}

/// Called on OnEnter(GameOver).
pub fn play_game_over_sfx(mut commands: Commands, sfx: Res<SfxHandles>) {
    play_sfx(&mut commands, &sfx.game_over);
}

/// Called on OnEnter(Victory).
pub fn play_victory_sfx(mut commands: Commands, sfx: Res<SfxHandles>) {
    play_sfx(&mut commands, &sfx.victory);
}

/// Event for requesting sound playback from gameplay systems.
#[derive(Event, Debug)]
pub enum SfxRequest {
    Shoot,
    Hit,
    Kill,
    WaveStart,
}

/// Listens for SfxRequest events and plays matching sounds.
pub fn handle_sfx_requests(
    mut events: EventReader<SfxRequest>,
    mut commands: Commands,
    sfx: Res<SfxHandles>,
) {
    for event in events.read() {
        let handle = match event {
            SfxRequest::Shoot => &sfx.shoot,
            SfxRequest::Hit => &sfx.hit,
            SfxRequest::Kill => &sfx.kill,
            SfxRequest::WaveStart => &sfx.wave_start,
        };
        play_sfx(&mut commands, handle);
    }
}
