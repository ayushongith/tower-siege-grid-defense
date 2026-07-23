use bevy::prelude::*;

use crate::plugins::enemy_plugin::SpawnEnemyRequest;
use crate::resources::{GameSettings, WaveManager, WaveModifier, WaveState};
use crate::sfx::SfxRequest;
use crate::AppState;

/// Single-frame banner shown to the player (wave start / complete).
#[derive(Resource, Debug, Clone)]
pub struct WaveAnnouncement {
    pub text: String,
    pub timer: Timer,
}

impl Default for WaveAnnouncement {
    fn default() -> Self {
        Self {
            text: String::new(),
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveAnnouncement>()
            .add_systems(OnEnter(AppState::Playing), start_or_advance_wave)
            .add_systems(
                Update,
                (
                    wave_spawning_system,
                    wave_transition_system,
                    clear_announcement,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// When entering Playing, start wave 1 if no wave has been started yet.
fn start_or_advance_wave(
    mut waves: ResMut<WaveManager>,
    mut ann: ResMut<WaveAnnouncement>,
    mut sfx: EventWriter<SfxRequest>,
    settings: Res<GameSettings>,
) {
    if waves.state == WaveState::Idle && waves.current_wave == 0 {
        begin_wave(&mut waves, &mut ann, &settings);
        sfx.send(SfxRequest::WaveStart);
    }
}

fn begin_wave(waves: &mut WaveManager, ann: &mut WaveAnnouncement, settings: &GameSettings) {
    waves.current_wave += 1;
    waves.modifier = WaveModifier::for_wave(waves.current_wave);
    waves.state = WaveState::Spawning;
    waves.spawn_queue = WaveManager::generate_composition(waves.current_wave, waves.modifier);
    waves.spawn_index = 0;
    waves.total_enemies = waves.spawn_queue.len() as u32;
    waves.spawn_timer =
        Timer::from_seconds(WaveManager::spawn_interval(waves.current_wave, settings.difficulty), TimerMode::Repeating);
    ann.text = format!("Wave {}!", waves.current_wave);
    ann.timer.reset();
    info!(
        "Wave {} started: {} enemies",
        waves.current_wave, waves.total_enemies
    );
}

/// Timer-driven spawning: pop one enemy from the queue each tick.
fn wave_spawning_system(
    time: Res<Time>,
    mut waves: ResMut<WaveManager>,
    mut spawn_request: ResMut<SpawnEnemyRequest>,
) {
    if waves.state != WaveState::Spawning {
        return;
    }

    waves.spawn_timer.tick(time.delta());
    if !waves.spawn_timer.just_finished() {
        return;
    }

    if waves.spawn_index < waves.spawn_queue.len() {
        let enemy_type = waves.spawn_queue[waves.spawn_index];
        spawn_request.pending = true;
        spawn_request.enemy_type = enemy_type;
        waves.spawn_index += 1;
        waves.enemies_spawned += 1;
        waves.enemies_alive += 1;
    }
}

/// Transition between wave states:
/// - Spawning → Complete (when queue exhausted)
/// - Complete → Idle (when all enemies cleared)
/// - Idle → Spawning (auto-advance after interwave delay)
fn wave_transition_system(
    time: Res<Time>,
    mut waves: ResMut<WaveManager>,
    mut ann: ResMut<WaveAnnouncement>,
    mut sfx: EventWriter<SfxRequest>,
    settings: Res<GameSettings>,
) {
    match waves.state {
        WaveState::Spawning => {
            if waves.spawn_index >= waves.spawn_queue.len() && waves.spawn_queue.len() > 0 {
                waves.state = WaveState::Complete;
                ann.text = format!("Wave {} complete!", waves.current_wave);
                ann.timer.reset();
                info!("Wave {} spawning complete", waves.current_wave);
            }
        }
        WaveState::Complete => {
            if waves.enemies_alive == 0 {
                waves.state = WaveState::Idle;
                waves.interwave_timer.reset();
                info!("All enemies cleared. Next wave in {:.0}s", waves.interwave_timer.duration().as_secs_f32());
            }
        }
        WaveState::Idle => {
            if waves.auto_start_next && waves.interwave_timer.tick(time.delta()).finished() {
                begin_wave(&mut waves, &mut ann, &settings);
                sfx.send(SfxRequest::WaveStart);
            }
        }
    }
}

/// Clear announcement text after the timer expires (systems render the text
/// only while it's non-empty).
fn clear_announcement(time: Res<Time>, mut ann: ResMut<WaveAnnouncement>) {
    if !ann.text.is_empty() {
        ann.timer.tick(time.delta());
        if ann.timer.finished() {
            ann.text.clear();
        }
    }
}
