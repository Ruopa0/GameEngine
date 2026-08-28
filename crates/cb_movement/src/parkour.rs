/// Parkour Detection — runs raycasts each FixedUpdate tick to detect
/// vault and mantle surfaces. Results are written into ParkourSense
/// so the FSM can make transitions.
///
/// Detection Grid (every tick while airborne or sprinting):
///   Forward shapeCast → vault / mantle candidate
///   Foot / head rayCasts → obstacle height classification

use bevy::prelude::*;
use avian3d::prelude::*;
use crate::fsm::{CharacterState, MovementState};

/// Results written from detection → FSM reads these next tick
#[derive(Component, Default)]
pub struct ParkourSense {
    pub vault_target:    Option<Vec3>,   // world-space snap point for vault
    pub mantle_target:   Option<Vec3>,   // world-space snap point for mantle
    pub ledge_point:     Option<Vec3>,   // ledge grab attachment
}

pub fn detect_parkour(
    mut query: Query<(
        &mut CharacterState,
        &mut ParkourSense,
        &Transform,
    )>,
    spatial: SpatialQuery,
) {
    for (cs, mut sense, transform) in query.iter_mut() {
        // Reset detections
        *sense = ParkourSense::default();

        let pos    = transform.translation;
        let fwd    = -transform.local_z().as_vec3(); // forward direction

        // Only run expensive raycasts when relevant
        let should_check = cs.current == MovementState::Sprint
            || cs.current == MovementState::Walk
            || cs.is_airborne();

        if !should_check { continue; }

        // ─── Forward obstacle detection ──────────────────────────────
        // Three rays at foot, waist, and head height classify obstacle
        let ray_origins = [
            pos + Vec3::Y * 0.3,   // foot — low obstacles
            pos + Vec3::Y * 1.0,   // waist
            pos + Vec3::Y * 1.8,   // head
        ];

        let mut hit_foot  = false;
        let mut hit_waist = false;
        let mut hit_head  = false;

        for (i, origin) in ray_origins.iter().enumerate() {
            if let Some(hit) = spatial.cast_ray(
                *origin,
                Dir3::new(fwd).unwrap_or(Dir3::NEG_Z),
                1.2,            // probe 1.2 m ahead
                true,
                &SpatialQueryFilter::default(),
            ) {
                let _ = hit; // hit recorded
                match i {
                    0 => hit_foot  = true,
                    1 => hit_waist = true,
                    2 => hit_head  = true,
                    _ => {}
                }
            }
        }

        // Classify obstacle:
        //   foot only            → step (auto-stepped by tnua)
        //   foot + waist         → vault candidate (< ~1.2 m)
        //   foot + waist + head  → mantle / ledge grab (> 1.2 m)
        if hit_foot && hit_waist && !hit_head {
            // Vault — snap point is the top of the obstacle
            sense.vault_target = Some(pos + fwd * 1.1 + Vec3::Y * 1.1);
        } else if hit_foot && hit_waist && hit_head {
            // Mantle / ledge grab
            sense.ledge_point = Some(pos + fwd * 1.1 + Vec3::Y * 2.1);
        }
    }
}

// ─── Extension trait so CharacterState can query airborne cleanly ────────────
impl CharacterState {
    pub fn is_airborne(&self) -> bool { self.current.is_airborne() }
}
