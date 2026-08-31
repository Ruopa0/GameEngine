use crate::fsm::{CharacterState, MovementState};
use crate::parkour::ParkourSense;
/// Kinematic Character Controller -- maps CharacterState to bevy_tnua inputs.
///
/// Reads the FSM state and feeds TnuaBuiltinWalk + TnuaBuiltinJump each frame.
/// Jump height is boosted by momentum.flow.
use bevy::prelude::*;
use bevy_tnua::prelude::*;

use avian3d::prelude::GravityScale;

#[derive(bevy_tnua::prelude::TnuaScheme)]
#[scheme(basis = bevy_tnua::builtins::TnuaBuiltinWalk)]
pub enum CharacterScheme {
    Jump(bevy_tnua::builtins::TnuaBuiltinJump),
}

pub fn apply_movement(
    mut query: Query<(
        &CharacterState,
        &ParkourSense,
        &mut TnuaController<CharacterScheme>,
        &TnuaConfig<CharacterScheme>,
        &mut GravityScale,
        &mut avian3d::prelude::Collider,
    )>,
    mut configs: ResMut<Assets<CharacterSchemeConfig>>,
) {
    for (state, sense, mut controller, config_handle, mut gravity, mut collider) in query.iter_mut() {
        controller.initiate_action_feeding();
        // --- Speed from state (tuned to 1/4th for tactical grounded pace) ---
        let base_speed: f32 = match state.current {
            MovementState::TacSprint => 3.75, // was 15.0
            MovementState::Sprint => 2.25,    // was 9.0
            MovementState::Walk => 1.25,      // was 5.0
            MovementState::Slide => 1.75,     // was 7.0
            MovementState::Crouch => 0.40,    // was 1.25
            MovementState::Prone => 0.35,     // was 1.6
            MovementState::Idle => 0.0,
            MovementState::Vault => 1.0,      // was 3.5
            MovementState::Mantle => 0.6,     // was 2.0
            MovementState::LedgeGrab => 0.0,
            _ => 0.75,
        };

        let flow_bonus = (state.momentum.flow * 0.5) / 16.0;
        let speed = base_speed + flow_bonus;

        let mut desired_velocity = state.desired_direction.normalize_or_zero() * speed;

        // Baseline physics tuning (high acceleration for crisp ground friction and no sliding)
        let mut air_accel = 25.0;
        let mut accel = 180.0; // High acceleration/stopping power for immediate stop
        let mut free_fall_gravity = 12.0; // Snappy downward pull
        gravity.0 = 2.5; // Strong gravity scale

        let mut float_height = 0.85;
        let mut capsule_height = 1.0;
        
        if state.current == MovementState::Crouch || state.current == MovementState::Slide {
            float_height = 0.45;
            capsule_height = 0.2;
        }
        if state.current == MovementState::Prone {
            capsule_height = 0.0; // Acts as a sphere of radius 0.35
            float_height = 0.35; // Exactly the radius, so it touches the ground
        }
        
        // Update physical collider size on state change
        if state.current != state.previous {
            *collider = avian3d::prelude::Collider::capsule(0.35, capsule_height);
        }

        if state.current == MovementState::Slide {
            accel = 8.0; // Controlled momentum slide
        }

        if state.current == MovementState::Vault || state.current == MovementState::Mantle {
            let target = sense
                .vault_target
                .or(sense.ledge_point)
                .unwrap_or(Vec3::ZERO);
            if target != Vec3::ZERO {
                let t = state.time_in_state;
                let horizontal_dir = state.desired_direction.normalize_or_zero();

                // 3-Phase Kinematic Arc
                if t < 0.15 {
                    // Phase 1: Launch (Up and forward)
                    desired_velocity = horizontal_dir * speed + Vec3::Y * 1.5;
                    air_accel = 25.0;
                } else if t < 0.40 {
                    // Phase 2: Apex float
                    desired_velocity = horizontal_dir * speed;
                    air_accel = 20.0;
                    gravity.0 = 2.0;
                    free_fall_gravity = 5.0;
                } else {
                    // Phase 3: Finish (Normal fall)
                    desired_velocity = horizontal_dir * speed;
                }
            }
        }

        // --- Mutate Tnua Config --------------------------------------
        if let Some(config) = configs.get_mut(&config_handle.0) {
            config.basis.acceleration = accel;
            config.basis.air_acceleration = air_accel;
            config.basis.free_fall_extra_gravity = free_fall_gravity;
            config.basis.float_height = float_height;

            // Jump tuning: realistic height (1.2m) with strong gravity descent
            config.jump.height = 1.2;
            config.jump.shorten_extra_gravity = 15.0;
        }

        // --- Walk basis ----------------------------------------------
        controller.basis = bevy_tnua::builtins::TnuaBuiltinWalk {
            desired_motion: desired_velocity,
            desired_forward: Dir3::new(state.desired_direction).ok(),
        };

        // --- Jump action ---------------------------------------------
        if state.wishes_jump
            || state.current == MovementState::Jump
        {
            controller.action(CharacterScheme::Jump(
                bevy_tnua::builtins::TnuaBuiltinJump {
                    allow_in_air: false,
                    ..Default::default()
                },
            ));
        }
    }
}
