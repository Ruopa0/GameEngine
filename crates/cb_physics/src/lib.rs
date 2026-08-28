#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
use bevy::prelude::*;

pub struct CbPhysicsPlugin;

impl Plugin for CbPhysicsPlugin {
    fn build(&self, _app: &mut App) {
        // PhysicsPlugins::default() is added in cb_game/main.rs directly.
        // This crate holds shared physics utilities, query helpers, and
        // constants used by cb_movement and cb_weapons.
    }
}

/// Tag for surfaces that allow wall-running
#[derive(Component, Default)]
pub struct WallRunSurface;

/// Tag for surfaces that can be vaulted over (low obstacles)
#[derive(Component, Default)]
pub struct VaultSurface;

/// Tag for ledges that can be grabbed / mantled
#[derive(Component, Default)]
pub struct LedgeSurface;

/// Collision layers used across the engine.
/// We use plain u32 bitmasks instead of derive-PhysicsLayer
/// to avoid avian_derive cfg warnings until they patch the crate.
pub mod layers {
    pub const DEFAULT:    u32 = 1 << 0;
    pub const PLAYER:     u32 = 1 << 1;
    pub const PROJECTILE: u32 = 1 << 2;
    pub const TRIGGER:    u32 = 1 << 3;
}


