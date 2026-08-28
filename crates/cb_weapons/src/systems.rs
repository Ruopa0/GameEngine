/// Weapon fire and reload systems — run in FixedUpdate.

use bevy::prelude::*;


use crate::components::{FireRate, Magazine, Spread, RecoilPattern, FireMode};

/// Marker event — emitted when a shot is confirmed this tick.
/// ballistics::process_hitscan listens to this.
#[derive(Message)]
pub struct ShotFiredEvent {
    pub shooter:    Entity,
    pub origin:     Vec3,
    pub direction:  Vec3,
    pub spread_rad: f32,
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
    )>,
    mut ev_shot: MessageWriter<ShotFiredEvent>,
) {
    let dt = time.delta_secs();

    for (entity, weapon_cfg, mut fire_rate, mut mag, mut spread, mut recoil, gtf) in query.iter_mut() {
        fire_rate.tick(dt);
        mag.tick(dt);
        spread.current = (spread.current - spread.recovery_rate * dt).max(spread.base);

        if let FireMode::Burst(rounds) = weapon_cfg.fire_mode {
            if fire_rate.trigger_just && fire_rate.burst_remaining == 0 {
                fire_rate.burst_remaining = rounds;
            }
        }

        let should_fire = match weapon_cfg.fire_mode {
            FireMode::FullAuto  => fire_rate.trigger_held,
            FireMode::SemiAuto  => fire_rate.trigger_just,
            FireMode::Burst(_)  => fire_rate.burst_remaining > 0,
        };

        if should_fire && fire_rate.can_fire() && mag.try_fire() {
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

            // Compute shot direction from entity forward + spread
            let forward = gtf.forward();
            ev_shot.write(ShotFiredEvent {
                shooter:   entity,
                origin:    gtf.translation(),
                direction: *forward,
                spread_rad: spread.current,
            });
        }

        // Reset recoil index when not firing
        if !fire_rate.trigger_held {
            if recoil.index > 0 { recoil.index = recoil.index.saturating_sub(1); }
            recoil.kick = recoil.kick.lerp(Vec2::ZERO, (recoil.recovery * dt).min(1.0));
        }
    }
}





