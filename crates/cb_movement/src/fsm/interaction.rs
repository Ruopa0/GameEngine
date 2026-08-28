use super::{CharacterState, MovementState};
use crate::parkour::ParkourSense;

pub fn check_transitions(cs: &CharacterState, _sense: &ParkourSense) -> MovementState {
    // Interactions are duration-based or exit on ground contact
    let duration = match cs.current {
        MovementState::Vault => 0.4,
        MovementState::Mantle => 0.6,
        MovementState::LedgeGrab => Default::default(), // hold indefinitely until jump or drop
        _ => 0.0,
    };

    if cs.current == MovementState::LedgeGrab {
        if cs.wishes_jump {
            return MovementState::Mantle; // Transition to Mantle from LedgeGrab
        } else if cs.wishes_crouch {
            return MovementState::Fall; // Drop from ledge
        }
        return MovementState::LedgeGrab;
    }

    if cs.time_in_state >= duration {
        if cs.is_grounded {
            MovementState::Idle
        } else {
            MovementState::Fall
        }
    } else {
        cs.current
    }
}
