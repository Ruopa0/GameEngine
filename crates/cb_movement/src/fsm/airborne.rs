use super::{CharacterState, MovementState};
use crate::parkour::ParkourSense;

pub fn check_transitions(cs: &CharacterState, sense: &ParkourSense) -> MovementState {
    let moving_forward = cs.desired_direction.length_squared() > 0.01;

    // Check for parkour interactions from air
    if moving_forward {
        if sense.vault_target.is_some() && cs.wishes_jump {
            return MovementState::Vault;
        } else if sense.ledge_point.is_some() && cs.wishes_jump {
            return MovementState::Mantle;
        }
    }

    if cs.is_grounded {
        MovementState::Idle // Grounded module will handle specific state next frame
    } else {
        if cs.current == MovementState::Jump { 
            MovementState::Jump 
        } else { 
            MovementState::Fall 
        }
    }
}
