/// Weapon fire and reload systems -- run in FixedUpdate.

use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;


use crate::components::{FireRate, Magazine, Spread, RecoilPattern, FireMode};

/// Marker event -- emitted when a shot is confirmed this tick.
/// ballistics::process_hitscan listens to this.
#[derive(Message)]
pub struct ShotFiredEvent {
    pub shooter:    Entity,
    pub weapon:     Entity,
    pub origin:     Vec3,
    pub direction:  Vec3,
    pub spread_rad: f32,
    pub is_local:   bool,
    pub projectile_speed: Option<f32>,
}

pub fn weapon_fire_system(
    time:      Res<Time>,
    mut query: Query<(
        Entity,
        &crate::components::WeaponConfig,
        &mut FireRate,
        &mut Magazine,
        &mut Spread,
        &mut RecoilPattern,
        &GlobalTransform,
        Option<&ChildOf>,
    )>,
    mut ev_shot: MessageWriter<ShotFiredEvent>,
) {
    let dt = time.delta_secs();

    for (entity, weapon_cfg, mut fire_rate, mut mag, mut spread, mut recoil, gtf, parent) in query.iter_mut() {
        fire_rate.tick(dt);
        mag.tick(dt);
        spread.current = (spread.current - spread.recovery_rate * dt).max(spread.base);

        if let FireMode::Burst(rounds) = weapon_cfg.fire_mode {
            if fire_rate.trigger_just && fire_rate.burst_remaining == 0 {
                fire_rate.burst_remaining = rounds;
            }
        }

        let should_fire = match weapon_cfg.fire_mode {
            FireMode::FullAuto  => fire_rate.trigger_held && fire_rate.can_fire(),
            FireMode::SemiAuto  => fire_rate.trigger_just, // NO cooldown for semi-auto
            FireMode::Burst(_)  => fire_rate.burst_remaining > 0 && fire_rate.can_fire(),
        };

        if should_fire && mag.try_fire() {
            fire_rate.consume();
            if fire_rate.burst_remaining > 0 {
                fire_rate.burst_remaining -= 1;
            }

            // Advance recoil pattern
            let pattern_delta = if recoil.index < recoil.pattern.len() {
                let d = recoil.pattern[recoil.index];
                recoil.index = (recoil.index + 1).min(recoil.pattern.len() - 1);
                d
            } else {
                *recoil.pattern.last().unwrap_or(&Vec2::ZERO)
            };

            recoil.kick += pattern_delta;

            // Grow spread
            spread.current = (spread.current + spread.per_shot).min(spread.max);

            // Compute gun muzzle origin (aligned with the first-person gun muzzle tip at center-bottom)
            let forward = gtf.forward();
            let up = gtf.up();
            let muzzle_origin = gtf.translation() + (*up * -0.21) + (*forward * 0.74);

            // Compute shot direction aiming toward the crosshair focal point (100m ahead)
            let crosshair_focal_point = gtf.translation() + *forward * 100.0;
            let shot_direction = (crosshair_focal_point - muzzle_origin).normalize_or_zero();

            let shooter_entity = parent.map(|p| p.get()).unwrap_or(entity);
            ev_shot.write(ShotFiredEvent {
                shooter:   shooter_entity,
                weapon:    entity,
                origin:    muzzle_origin,
                direction: shot_direction,
                spread_rad: spread.current,
                is_local:  true,
                projectile_speed: weapon_cfg.projectile_speed,
            });
        }

        // Reset recoil index when not firing
        if !fire_rate.trigger_held {
            if recoil.index > 0 { recoil.index = recoil.index.saturating_sub(1); }
            recoil.kick = recoil.kick.lerp(Vec2::ZERO, (recoil.recovery * dt).min(1.0));
        }

        // Consume the just_pressed event so it isn't processed twice
        fire_rate.trigger_just = false;
    }
}





