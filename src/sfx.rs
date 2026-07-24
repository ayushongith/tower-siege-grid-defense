use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Custom decodable audio source — raw PCM f32 samples, no file format needed.
// ---------------------------------------------------------------------------

#[derive(Asset, Debug, Clone, TypePath)]
pub struct SfxSource {
    samples: Arc<[f32]>,
    sample_rate: u32,
}

pub struct SfxDecoder {
    samples: Arc<[f32]>,
    sample_rate: u32,
    channels: u16,
    index: usize,
}

impl Iterator for SfxDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.index < self.samples.len() {
            let s = self.samples[self.index];
            self.index += 1;
            Some(s)
        } else {
            None
        }
    }
}

impl bevy::audio::Source for SfxDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.index))
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / self.sample_rate as f32 / self.channels as f32,
        ))
    }
}

impl bevy::audio::Decodable for SfxSource {
    type DecoderItem = f32;
    type Decoder = SfxDecoder;

    fn decoder(&self) -> Self::Decoder {
        SfxDecoder {
            samples: self.samples.clone(),
            sample_rate: self.sample_rate,
            channels: 1,
            index: 0,
        }
    }
}

fn gen_samples(freq: f32, duration_secs: f32, sample_rate: u32) -> Arc<[f32]> {
    let count = (sample_rate as f32 * duration_secs) as usize;
    let mut s = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / sample_rate as f32;
        let envelope = {
            let a = (i as f32 / (sample_rate as f32 * 0.01)).min(1.0);
            let r = ((count - i) as f32 / (sample_rate as f32 * 0.02)).min(1.0);
            a.min(r) * 0.30
        };
        s.push((t * freq * std::f32::consts::TAU).sin() * envelope);
    }
    s.into()
}

fn gen_sweep(start: f32, end: f32, duration_secs: f32, sr: u32) -> Arc<[f32]> {
    let count = (sr as f32 * duration_secs) as usize;
    let mut s = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / sr as f32;
        let freq = start + (end - start) * (t / duration_secs).min(1.0);
        let envelope = {
            let a = (i as f32 / (sr as f32 * 0.01)).min(1.0);
            let r = ((count - i) as f32 / (sr as f32 * 0.02)).min(1.0);
            a.min(r) * 0.30
        };
        s.push((t * freq * std::f32::consts::TAU).sin() * envelope);
    }
    s.into()
}

#[derive(Resource, Debug)]
pub struct SfxHandles {
    pub shoot: Handle<SfxSource>,
    pub hit: Handle<SfxSource>,
    pub kill: Handle<SfxSource>,
    pub wave_start: Handle<SfxSource>,
    pub game_over: Handle<SfxSource>,
    pub victory: Handle<SfxSource>,
}

pub fn setup_sfx(mut commands: Commands, mut assets: ResMut<Assets<SfxSource>>) {
    let sr = 44100;
    commands.insert_resource(SfxHandles {
        shoot: assets.add(SfxSource { samples: gen_samples(880.0, 0.08, sr), sample_rate: sr }),
        hit: assets.add(SfxSource { samples: gen_samples(220.0, 0.06, sr), sample_rate: sr }),
        kill: assets.add(SfxSource { samples: gen_sweep(440.0, 880.0, 0.15, sr), sample_rate: sr }),
        wave_start: assets.add(SfxSource { samples: gen_sweep(440.0, 660.0, 0.3, sr), sample_rate: sr }),
        game_over: assets.add(SfxSource { samples: gen_sweep(440.0, 110.0, 0.5, sr), sample_rate: sr }),
        victory: assets.add(SfxSource { samples: gen_sweep(440.0, 880.0, 0.5, sr), sample_rate: sr }),
    });
}

pub fn play_sfx(commands: &mut Commands, handle: &Handle<SfxSource>) {
    commands.spawn((
        AudioPlayer::<SfxSource>(handle.clone()),
        PlaybackSettings::ONCE,
        Name::new("SfxPlayer"),
    ));
}

pub fn play_game_over_sfx(mut commands: Commands, sfx: Res<SfxHandles>) {
    play_sfx(&mut commands, &sfx.game_over);
}

pub fn play_victory_sfx(mut commands: Commands, sfx: Res<SfxHandles>) {
    play_sfx(&mut commands, &sfx.victory);
}

#[derive(Event, Debug)]
pub enum SfxRequest {
    Shoot,
    Hit,
    Kill,
    WaveStart,
}

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
