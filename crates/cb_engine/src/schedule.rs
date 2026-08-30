use bevy::prelude::*;

/// Custom schedule sets to enforce deterministic ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EngineSet {
    /// Gather all inputs from the user (keyboard, mouse, gamepad).
    InputCollection,
    /// Update HFSM, AI state, and determine desired movement.
    MovementUpdate,
    /// Apply movement to the physics character controller.
    PhysicsStep,
    /// Weapon processing, raycasts, and hit registration.
    WeaponProcess,
    /// Game logic (rules, score, etc.).
    GameRules,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(120.0));
        app.configure_sets(
            FixedUpdate,
            (
                EngineSet::InputCollection,
                EngineSet::MovementUpdate,
                EngineSet::PhysicsStep,
                EngineSet::WeaponProcess,
                EngineSet::GameRules,
            )
                .chain(),
        );
    }
}
