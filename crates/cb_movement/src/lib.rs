#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
/// cb_movement — Movement plugin and public API.
///
/// Exports:
///   MovementPlugin  — add to App
///   fsm::CharacterState  — add to player entity
///
/// Systems run in FixedUpdate (authoritative) and Update (input / camera).

pub mod momentum;
pub mod fsm;
pub mod kcc;
pub mod parkour;

use bevy::prelude::*;
use bevy_tnua::TnuaUserControlsSystems;
use fsm::{CharacterState, MovementState};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<CharacterState>()
            .register_type::<MovementState>()
            // FSM ticks every FixedUpdate — deterministic, prediction-safe
            .add_systems(FixedUpdate, fsm::update_fsm)
            // KCC must run in TnuaUserControlsSystemSet — that's tnua's contract
            .add_systems(Update, kcc::apply_movement.in_set(TnuaUserControlsSystems))
            // Parkour detection (raycasts) runs in FixedUpdate before FSM
            .add_systems(FixedUpdate, parkour::detect_parkour.before(fsm::update_fsm));
    }
}


