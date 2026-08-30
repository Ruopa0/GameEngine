use super::{CharacterState, MovementState};

pub fn check_transitions(cs: &CharacterState, double_tap_crouch: bool) -> MovementState {
    let can_jump = cs.is_grounded || cs.coyote_timer > 0.0;
    let moving = cs.desired_direction.length_squared() > 0.01;

    // --- Prone state -----------------------------------------------------
    if cs.current == MovementState::Prone {
        if cs.wishes_jump && can_jump {
            // Stand up then jump -- brief transition through Idle
            return MovementState::Jump;
        } else if cs.wishes_crouch_tap && !double_tap_crouch {
            // Single tap C from prone -> go to crouch
            return MovementState::Crouch;
        } else if cs.wishes_sprint && moving {
            // Sprint to stand up and start running
            return MovementState::Sprint;
        } else {
            return MovementState::Prone;
        }
    }

    // --- Slide state -----------------------------------------------------
    if cs.current == MovementState::Slide {
        if cs.wishes_jump && can_jump {
            return MovementState::Jump;
        } else if cs.wishes_prone || cs.wishes_crouch_tap {
            // Second tap C or Prone button during slide -> drop to prone
            return MovementState::Prone;
        } else if cs.time_in_state > 0.8 {
            // Slide expired -> transition to Crouch
            return MovementState::Crouch;
        } else if !moving {
            // Stopped moving during slide -> crouch
            return MovementState::Crouch;
        } else {
            return MovementState::Slide;
        }
    }

    // --- Normal grounded transitions -------------------------------------
    if cs.wishes_jump && can_jump {
        MovementState::Jump
    } else if !cs.is_grounded {
        MovementState::Fall
    } else if cs.wishes_prone {
        // Dedicated prone button
        MovementState::Prone
    } else if cs.wishes_crouch_tap && moving {
        // Tap C while moving -> slide (which flows into prone)
        MovementState::Slide
    } else if double_tap_crouch && !moving {
        // Double-tap C while stationary -> prone directly
        MovementState::Prone
    } else if cs.wishes_crouch && cs.wishes_sprint && moving {
        // Sprint + crouch while moving -> slide
        MovementState::Slide
    } else if cs.wishes_crouch {
        MovementState::Crouch
    } else if cs.wishes_tac_sprint && moving {
        MovementState::TacSprint
    } else if cs.wishes_sprint && moving {
        MovementState::Sprint
    } else if moving {
        MovementState::Walk
    } else {
        MovementState::Idle
    }
}
