/// Movement state machine — state definitions and FSM update system.
///
/// The HFSM has three top-level groups:
///   Grounded  →  Idle | Walk | Sprint | Crouch | Slide | Prone
///   Airborne  →  Jump | Fall
///   Interact  →  Vault | LedgeGrab | Mantle

use bevy::prelude::*;
use crate::momentum::Momentum;

pub mod grounded;
pub mod airborne;
pub mod interaction;

use serde::{Serialize, Deserialize};

// ─── State Enum ──────────────────────────────────────────────────────────────

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum MovementState {
    // Grounded
    #[default]
    Idle,
    Walk,
    Sprint,
    Crouch,
    Slide,
    Prone,

    // Airborne
    Jump,
    Fall,

    // Interaction (one-shot animations / transitions)
    Vault,
    LedgeGrab,
    Mantle,
}

impl MovementState {
    pub fn is_grounded(&self) -> bool {
        matches!(self, Self::Idle | Self::Walk | Self::Sprint | Self::Crouch | Self::Slide | Self::Prone)
    }

    pub fn is_airborne(&self) -> bool {
        matches!(self, Self::Jump | Self::Fall)
    }
}

// ─── Character State Component ──────────────────────────────────────────────

#[derive(Component, Reflect, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterState {
    pub current:           MovementState,
    pub previous:          MovementState,

    // Intent from the input layer
    pub desired_direction: Vec3,
    pub wishes_jump:       bool,
    pub wishes_sprint:     bool,
    pub wishes_crouch:     bool,
    pub wishes_crouch_tap: bool, // just-pressed C (for double-tap detection)
    pub wishes_prone:      bool, // ControlLeft

    // Grounding info (filled by tnua each frame)
    pub is_grounded:       bool,
    pub ground_normal:     Vec3,

    // Time trackers
    pub time_in_state:     f32,  // seconds in current state
    pub coyote_timer:      f32,  // counts down after leaving ground
    pub jump_buffer_timer: f32,  // 150ms pre-queue for jumps

    // Double-tap C detection for slide-to-prone
    pub crouch_tap_timer:  f32,  // counts down from 0.3s after first C tap

    // Momentum / flow
    pub momentum:          Momentum,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            current:           MovementState::Idle,
            previous:          MovementState::Idle,
            desired_direction: Vec3::ZERO,
            wishes_jump:       false,
            wishes_sprint:     false,
            wishes_crouch:     false,
            wishes_crouch_tap: false,
            wishes_prone:      false,
            is_grounded:       false,
            ground_normal:     Vec3::Y,
            time_in_state:     0.0,
            coyote_timer:      0.0,
            jump_buffer_timer: 0.0,
            crouch_tap_timer:  0.0,
            momentum:          Momentum::default(),
        }
    }
}

// Momentum is now imported from crate::momentum

// ─── FSM Update System ──────────────────────────────────────────────────────

use crate::parkour::ParkourSense;
use bevy_tnua::prelude::*;

pub fn update_fsm(
    time: Res<Time>,
    mut query: Query<(&mut CharacterState, &ParkourSense, &TnuaController<crate::kcc::CharacterScheme>)>,
) {
    let dt = time.delta_secs();

    for (mut cs, sense, controller) in query.iter_mut() {
        // Read grounding from Tnua
        cs.is_grounded = !controller.is_airborne().unwrap_or(true);

        // Tick momentum decay
        cs.momentum.tick(dt);

        // Track time in current state
        cs.time_in_state += dt;

        // Coyote time: count down after leaving ground
        if cs.is_grounded {
            cs.coyote_timer = 0.15; // reset
        } else {
            cs.coyote_timer = (cs.coyote_timer - dt).max(0.0);
        }

        // Jump buffer: user input sets wishes_jump, we set the buffer.
        // If the buffer > 0, they logically "wish to jump".
        if cs.wishes_jump {
            cs.jump_buffer_timer = 0.15;
            cs.wishes_jump = false; // consume the immediate flag, keep the timer
        } else {
            cs.jump_buffer_timer = (cs.jump_buffer_timer - dt).max(0.0);
        }
        
        // We will pass 'wishes_jump = true' to the FSM logic if the buffer is active
        let logically_wishes_jump = cs.jump_buffer_timer > 0.0;
        
        // Temporary override so submodules see it
        let old_wishes_jump = cs.wishes_jump;
        cs.wishes_jump = logically_wishes_jump;

        // ─── Double-tap C detection ──────────────────────────────────
        // If C was just tapped and the timer is still active from a previous tap → double-tap!
        let double_tap_crouch = cs.wishes_crouch_tap && cs.crouch_tap_timer > 0.0;

        // Update the tap timer
        if cs.wishes_crouch_tap && !double_tap_crouch {
            // First tap — start the window
            cs.crouch_tap_timer = 0.3;
        }
        cs.crouch_tap_timer = (cs.crouch_tap_timer - dt).max(0.0);

        let prev = cs.current;

        // Delegate to submodules based on current state group
        let next = if cs.current.is_grounded() {
            grounded::check_transitions(&cs, double_tap_crouch)
        } else if cs.current.is_airborne() {
            airborne::check_transitions(&cs, sense)
        } else {
            interaction::check_transitions(&cs, sense)
        };
        
        // Also allow ParkourSense to override from grounded if moving forward into an obstacle
        // e.g. Vaulting from the ground
        let moving_forward = cs.desired_direction.length_squared() > 0.01;
        let final_next = if cs.current.is_grounded() && moving_forward {
            if sense.vault_target.is_some() && cs.wishes_jump {
                MovementState::Vault
            } else if sense.ledge_point.is_some() && cs.wishes_jump {
                MovementState::Mantle
            } else {
                next
            }
        } else {
            next
        };

        // Apply transition
        if final_next != prev {
            // Track momentum gains on state entry
            match final_next {
                MovementState::Slide    => cs.momentum.add_slide(),
                MovementState::Vault    => cs.momentum.add_vault(),
                _ => {}
            }

            cs.previous = prev;
            cs.current  = final_next;
            cs.time_in_state = 0.0;
            
            // Consume the jump buffer if we actually jumped/vaulted
            if final_next == MovementState::Jump || final_next == MovementState::Vault || final_next == MovementState::Mantle {
                cs.jump_buffer_timer = 0.0;
            }
        }
        
        // Restore actual wishes_jump so it doesn't stay stuck
        cs.wishes_jump = old_wishes_jump;
    }
}
