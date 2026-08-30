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
        // --- Speed from state ----------------------------------------
        let base_speed: f32 = match state.current {
            MovementState::TacSprint => 15.0,
            MovementState::Sprint => 9.0,
            MovementState::Walk => 5.0,
            MovementState::Slide => 7.0,
            MovementState::Crouch => 1.25,
            MovementState::Prone => 1.6,
            MovementState::Idle => 0.0,
            MovementState::Vault => 3.5,
            MovementState::Mantle => 2.0,
            MovementState::LedgeGrab => 0.0,
            _ => 2.75,
        };

        let flow_bonus = (state.momentum.flow * 2.0) / 16.0;
        let speed = base_speed + flow_bonus;

        let mut desired_velocity = state.desired_direction.normalize_or_zero() * speed;

        // Baseline physics tuning (high acceleration for crisp ground friction and stopping power)
        let mut air_accel = 15.0;
        let mut accel = 60.0;
        let mut free_fall_gravity = 3.5;
        gravity.0 = 0.5;

        let mut float_height = 0.85;
        let mut capsule_height = 1.0;
        
        if state.current == MovementState::Crouch || state.current == MovementState::Slide {
            float_height = 0.45;
            capsule_height = 0.2;
        }
        if state.current == MovementState::Prone {
            float_height = 0.15; // flat on the ground
            capsule_height = 0.01;
        }
        
        // Update physical collider size on state change
        if state.current != state.previous {
            *collider = avian3d::prelude::Collider::capsule(0.35, capsule_height);
        }

        if state.current == MovementState::Slide {
            accel = 2.0; // Controlled momentum slide
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
                    desired_velocity = horizontal_dir * speed + Vec3::Y * 2.25;
                    air_accel = 25.0;
                } else if t < 0.40 {
                    // Phase 2: Apex float
                    desired_velocity = horizontal_dir * speed;
                    air_accel = 20.0;
                    gravity.0 = 0.5; // hover slightly
                    free_fall_gravity = 0.0;
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

            config.jump.height = 3.6;
            config.jump.shorten_extra_gravity = 5.0;
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
