//! Recoil recovery -- visual kick lerps back to zero when trigger is released.
//! Actual aim correction is handled in systems.rs per-shot advancement.
use bevy::prelude::*;

use crate::components::RecoilPattern;

pub fn recover_recoil(time: Res<Time>, mut query: Query<&mut RecoilPattern>) {
    let dt = time.delta_secs();
    for mut recoil in query.iter_mut() {
        recoil.kick = recoil.kick.lerp(Vec2::ZERO, (recoil.recovery * dt).min(1.0));
    }
}



