use bevy::prelude::*;

use crate::components::{ChainLightningFx, Health, HealthBar, HitEffect, PathFollower};
use crate::AppState;

pub struct VisualPlugin;

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_health_bars,
                update_hit_effects,
                update_chain_fx,
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn update_health_bars(
    enemies: Query<(Entity, &Health, &Children), With<PathFollower>>,
    mut bars: Query<(&mut Sprite, &mut Transform), With<HealthBar>>,
) {
    for (_enemy_entity, health, children) in &enemies {
        for &child in children.iter() {
            if let Ok((mut sprite, mut transform)) = bars.get_mut(child) {
                let ratio = (health.current / health.max).clamp(0.0, 1.0);
                sprite.color = if ratio > 0.6 {
                    Color::srgb(0.20, 0.85, 0.20)
                } else if ratio > 0.3 {
                    Color::srgb(0.85, 0.75, 0.15)
                } else {
                    Color::srgb(0.85, 0.15, 0.15)
                };
                if let Some(ref mut size) = sprite.custom_size {
                    size.x = 36.0 * ratio;
                }
                transform.translation.x = -18.0 + 18.0 * ratio;
            }
        }
    }
}

fn update_hit_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut HitEffect, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut effect, mut transform, mat) in &mut effects {
        effect.timer.tick(time.delta());
        if effect.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = effect.timer.fraction();
        transform.scale = Vec3::splat(1.0 + 0.5 * frac);
        if let Some(m) = materials.get_mut(&mat.0) {
            m.color.set_alpha((1.0 - frac) * 0.9);
        }
    }
}

fn update_chain_fx(
    mut commands: Commands,
    time: Res<Time>,
    mut fx: Query<(Entity, &mut ChainLightningFx, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut effect, mat) in &mut fx {
        effect.timer.tick(time.delta());
        if effect.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(m) = materials.get_mut(&mat.0) {
            m.color.set_alpha(0.9 * (1.0 - effect.timer.fraction()));
        }
    }
}
