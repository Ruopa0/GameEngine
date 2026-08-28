use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// Scalar representation of "flow" — chaining tactical moves increases this.
/// It gates: animation speed multipliers, slide jump bonus, input buffer windows.
#[derive(Default, Debug, Reflect, Clone, PartialEq, Serialize, Deserialize)]
pub struct Momentum {
    /// 0.0 = no flow … 1.0 = full flow
    pub flow:           f32,
    /// Velocity carried through a parkour transition
    pub carry_velocity: Vec3,
}

impl Momentum {
    const DECAY_RATE:  f32 = 0.8; // flow lost per second when not chaining
    const GAIN_VAULT:  f32 = 0.25;
    const GAIN_SLIDE:  f32 = 0.15;

    pub fn tick(&mut self, dt: f32) {
        self.flow = (self.flow - Self::DECAY_RATE * dt).max(0.0);
    }

    pub fn add_vault(&mut self)    { self.flow = (self.flow + Self::GAIN_VAULT).min(1.0); }
    pub fn add_slide(&mut self)    { self.flow = (self.flow + Self::GAIN_SLIDE).min(1.0); }

    /// Extra jump height given by flow
    pub fn jump_bonus(&self) -> f32 { self.flow * 0.4 }
}
