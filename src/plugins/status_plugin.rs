//! Status effect systems — slow, burn, stun.

use bevy::prelude::*;

use crate::components::{BurnDebuff, Enemy, PathFollower, SlowDebuff, StunDebuff};
use crate::resources::{GameStats, WaveManager};
use crate::AppState;

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_slow_debuffs,
                apply_burn_debuffs,
                tick_stun_debuffs,
                cleanup_expired_debuffs,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}


fn apply_slow_debuffs(
    mut enemies: Query<(&mut Enemy, Option<&SlowDebuff>)>,
) {
    for (mut enemy, slow) in &mut enemies {
        enemy.speed = if let Some(s) = slow {
            enemy.base_speed * s.factor
        } else {
            enemy.base_speed
        };
    }
}

fn apply_burn_debuffs(
    time: Res<Time>,
    mut commands: Commands,
    mut stats: ResMut<GameStats>,
    mut waves: ResMut<WaveManager>,
    mut enemies: Query<(Entity, &mut crate::components::Health, &Enemy, &mut BurnDebuff)>,
) {
    for (entity, mut health, enemy, mut burn) in &mut enemies {
        burn.timer.tick(time.delta());
        burn.tick.tick(time.delta());

        if burn.tick.just_finished() {
            health.current -= burn.dps * burn.tick.duration().as_secs_f32();
            if health.current <= 0.0 {
                stats.gold += enemy.gold_value;
                stats.kills += 1;
                waves.enemies_alive = waves.enemies_alive.saturating_sub(1);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn tick_stun_debuffs(time: Res<Time>, mut stuns: Query<&mut StunDebuff>) {
    for mut stun in &mut stuns {
        stun.timer.tick(time.delta());
    }
}

fn cleanup_expired_debuffs(
    mut commands: Commands,
    slows: Query<(Entity, &SlowDebuff)>,
    burns: Query<(Entity, &BurnDebuff)>,
    stuns: Query<(Entity, &StunDebuff)>,
) {
    for (e, s) in &slows {
        if s.timer.finished() {
            commands.entity(e).remove::<SlowDebuff>();
        }
    }
    for (e, b) in &burns {
        if b.timer.finished() {
            commands.entity(e).remove::<BurnDebuff>();
        }
    }
    for (e, s) in &stuns {
        if s.timer.finished() {
            commands.entity(e).remove::<StunDebuff>();
        }
    }
}

/// Returns true if enemy is stunned (should not move).
pub fn is_stunned(entity: Entity, stuns: &Query<&StunDebuff>) -> bool {
    stuns.get(entity).map(|s| !s.timer.finished()).unwrap_or(false)
}

pub fn apply_slow(commands: &mut Commands, entity: Entity, factor: f32, duration: f32) {
    commands.entity(entity).insert(SlowDebuff {
        factor,
        timer: Timer::from_seconds(duration, TimerMode::Once),
    });
}

pub fn apply_burn(commands: &mut Commands, entity: Entity, dps: f32, duration: f32) {
    commands.entity(entity).insert(BurnDebuff {
        dps,
        timer: Timer::from_seconds(duration, TimerMode::Once),
        tick: Timer::from_seconds(0.5, TimerMode::Repeating),
    });
}

pub fn apply_stun(commands: &mut Commands, entity: Entity, duration: f32) {
    commands.entity(entity).insert(StunDebuff {
        timer: Timer::from_seconds(duration, TimerMode::Once),
    });
}
