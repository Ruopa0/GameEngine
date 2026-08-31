/// Weapon ECS components -- all data-only, no logic.

use bevy::prelude::*;

// --- Fire Mode --------------------------------------------------------------

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerCombatant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum FireMode {
    #[default]
    SemiAuto,
    FullAuto,
    Burst(u8),
}

// --- Weapon Config (static, shared via Handle<WeaponData>) --------------------

/// Static weapon definition -- treated as a shared asset (clone-cheap)
#[derive(Component, Reflect, Clone)]
pub struct WeaponConfig {
    pub name: &'static str,
    pub fire_mode: FireMode,
    pub damage: f32,   // damage per bullet
    pub penetration: f32,   // 0..1 wall-penetration factor
    pub range: f32,   // max hitscan distance in metres
    pub projectile_speed: Option<f32>, // None = hitscan, Some = projectile m/s
}

impl Default for WeaponConfig {
    fn default() -> Self {
        Self {
            name: "Rifle",
            fire_mode: FireMode::FullAuto,
            damage: 25.0,
            penetration: 0.2,
            range: 200.0,
            projectile_speed: None, // hitscan
        }
    }
}

// --- Fire Rate --------------------------------------------------------------

#[derive(Component, Reflect, Clone)]
pub struct FireRate {
    pub rpm: f32,     // rounds per minute
    pub cooldown: f32,     // time between shots = 60/rpm
    pub timer: f32,     // current countdown
    pub trigger_held: bool,
    pub trigger_just: bool,    // just pressed (for semi / burst)
    pub burst_remaining: u8,      // shots remaining in current burst
}

impl FireRate {
    pub fn new(rpm: f32) -> Self {
        let cooldown = 60.0 / rpm;
        Self { rpm, cooldown, timer: 0.0, trigger_held: false, trigger_just: false, burst_remaining: 0 }
    }

    pub fn can_fire(&self) -> bool { self.timer <= 0.0 }
    pub fn consume(&mut self) { self.timer = self.cooldown; }
    pub fn tick(&mut self, dt: f32) { self.timer = (self.timer - dt).max(0.0); }
}

impl Default for FireRate { fn default() -> Self { Self::new(600.0) } }

// --- Magazine --------------------------------------------------------------

#[derive(Component, Reflect, Clone)]
pub struct Magazine {
    pub current: u32,
    pub max: u32,
    pub reserve: u32,
    pub reload_time: f32,  // seconds
    pub reload_timer: f32,  // counts down when reloading
    pub is_reloading: bool,
}

impl Magazine {
    pub fn new(capacity: u32, reserve: u32, reload_time: f32) -> Self {
        Self { current: capacity, max: capacity, reserve, reload_time, reload_timer: 0.0, is_reloading: false }
    }
    pub fn try_fire(&mut self) -> bool { if self.current > 0 && !self.is_reloading { self.current -= 1; true } else { false } }
    pub fn start_reload(&mut self) {
        if self.reserve > 0 && self.current < self.max && !self.is_reloading {
            self.is_reloading = true;
            self.reload_timer = self.reload_time;
        }
    }
    pub fn tick(&mut self, dt: f32) {
        if self.is_reloading {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                let needed = self.max - self.current;
                let refill = needed.min(self.reserve);
                self.current += refill;
                self.reserve -= refill;
                self.is_reloading = false;
            }
        }
    }
}

impl Default for Magazine { fn default() -> Self { Self::new(30, 90, 2.2) } }

// --- Spread / Bloom -----------------------------------------------------------

#[derive(Component, Reflect, Clone)]
pub struct Spread {
    /// Base accuracy cone (radians) when standing still and ADS
    pub base: f32,
    /// Current spread -- grows with each shot
    pub current: f32,
    /// Max spread cap
    pub max: f32,
    /// Growth per shot
    pub per_shot: f32,
    /// Recovery rate (rad/s back toward base)
    pub recovery_rate: f32,
}

impl Default for Spread {
    fn default() -> Self {
        Self { base: 0.002, current: 0.002, max: 0.06, per_shot: 0.005, recovery_rate: 0.08 }
    }
}

// --- Recoil Pattern -----------------------------------------------------------

/// COD-style pattern-based recoil -- each shot advances the index through a
/// fixed Vec2 sequence (yaw, pitch offsets). Player can counter-strafe to reset.
#[derive(Component, Reflect, Clone)]
pub struct RecoilPattern {
    /// Per-shot (yaw, pitch) deltas in radians
    pub pattern: Vec<Vec2>,
    /// Current position in the pattern (resets on stop-fire)
    pub index: usize,
    /// Visual-only kick (separate from actual aim vector -- client cosmetic)
    pub kick: Vec2,
    /// Recovery lerp speed when not firing
    pub recovery: f32,
}

impl Default for RecoilPattern {
    fn default() -> Self {
        Self {
            pattern: vec![
                Vec2::new(0.000, 0.003),
                Vec2::new(0.000, 0.004),
                Vec2::new(-0.001, 0.004),
                Vec2::new(-0.001, 0.005),
                Vec2::new(-0.002, 0.005),
                Vec2::new(-0.002, 0.004),
                Vec2::new(-0.001, 0.004),
                Vec2::new(0.001, 0.005),
                Vec2::new(0.002, 0.004),
                Vec2::new(0.002, 0.003),
            ],
            index: 0,
            kick: Vec2::ZERO,
            recovery: 8.0,
        }
    }
}

// --- Bundles -----------------------------------------------------------------

#[derive(Bundle, Default)]
pub struct WeaponBundle {
    pub config: WeaponConfig,
    pub fire: FireRate,
    pub mag: Magazine,
    pub spread: Spread,
    pub recoil: RecoilPattern,
}

impl WeaponBundle {
    pub fn clone_components(&self) -> Self {
        Self {
            config: self.config.clone(),
            fire: self.fire.clone(),
            mag: self.mag.clone(),
            spread: self.spread.clone(),
            recoil: self.recoil.clone(),
        }
    }
}

#[derive(Component, Default)]
pub struct WeaponInventory {
    pub primary: Option<WeaponBundle>,
    pub secondary: Option<WeaponBundle>,
    pub active_slot: u8,
    pub pending_slot: Option<u8>, // 1 for primary, 2 for secondary
}

/// Tag component to prevent entity from being despawned on lethal damage (e.g., player death screens)
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ImmortalPlayer;

/// Component attached to dead entities to delay despawning by 1.0 second
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DespawnDelay(pub Timer);

impl Default for DespawnDelay {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

// --- Health ------------------------------------------------------------------

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health { pub fn new(max: f32) -> Self { Self { current: max, max } } }
impl Default for Health { fn default() -> Self { Self::new(100.0) } }

// --- Projectile --------------------------------------------------------------

#[derive(Component, Reflect)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub penetration: f32,
    pub lifespan: f32,
    pub owner: Entity,
    pub is_local: bool,
}
