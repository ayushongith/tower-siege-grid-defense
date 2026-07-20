use bevy::prelude::*;

use crate::components::{Health, HealthBar, HitEffect, PathFollower};
use crate::AppState;

pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_health_bars,
                update_hit_effects,
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Update each enemy's health bar child to reflect current HP percentage.
fn update_health_bars(
    enemies: Query<(Entity, &Health, &Children), With<PathFollower>>,
    mut bars: Query<(&mut Sprite, &mut Transform), With<HealthBar>>,
) {
    for (_enemy_entity, health, children) in &enemies {
        for &child in children.iter() {
            if let Ok((mut sprite, mut transform)) = bars.get_mut(child) {
                let ratio = (health.current / health.max).clamp(0.0, 1.0);
                let color = if ratio > 0.6 {
                    Color::srgb(0.20, 0.85, 0.20)
                } else if ratio > 0.3 {
                    Color::srgb(0.85, 0.75, 0.15)
                } else {
                    Color::srgb(0.85, 0.15, 0.15)
                };
                sprite.color = color;
                if let Some(ref mut size) = sprite.custom_size {
                    size.x = 36.0 * ratio;
                }
                transform.translation.x = -18.0 + 18.0 * ratio;
            }
        }
    }
}

/// Tick hit-effect timers; fade alpha and despawn when expired.
fn update_hit_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut HitEffect, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut effect, mut transform, mut sprite) in &mut effects {
        effect.timer.tick(time.delta());
        if effect.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = effect.timer.fraction();
        // Expand slightly and fade out.
        let scale = 1.0 + 0.5 * frac;
        transform.scale = Vec3::splat(scale);
        sprite.color.set_alpha(1.0 - frac);
    }
}
