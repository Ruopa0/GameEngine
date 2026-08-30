#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
/// cb_weapons -- Weapon system: ECS components, fire modes, ballistics, recoil.
///
/// Architecture:
///   WeaponConfig      -- static weapon definition (fire rate, damage, etc.)
///   Magazine          -- current ammo state
///   RecoilPattern     -- COD-style deterministic recoil sequence
///   Spread            -- bloom / accuracy model
///   WeaponPlugin      -- wires all weapon systems into Bevy

pub mod components;
pub mod systems;
pub mod ballistics;
pub mod recoil;
pub mod health;
pub mod viewmodel;

use bevy::prelude::*;

use components::*;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(viewmodel::ViewModelPlugin)
            .register_type::<WeaponConfig>()
            .register_type::<Magazine>()
            .register_type::<FireRate>()
            .register_type::<Spread>()
            .register_type::<Health>()
            .register_type::<health::ImmortalPlayer>()
            .register_type::<health::DespawnDelay>()
            .add_message::<systems::ShotFiredEvent>()
            .add_message::<ballistics::DamageEvent>()
            .add_message::<ballistics::HitVfxEvent>()
            .add_message::<health::EntityKilledEvent>()
            .add_systems(FixedUpdate, (
                systems::weapon_fire_system,
                recoil::recover_recoil,
                ballistics::process_hitscan,
                ballistics::process_projectiles,
                health::process_damage,
                health::update_despawn_delays,
            ).chain());
    }
}







