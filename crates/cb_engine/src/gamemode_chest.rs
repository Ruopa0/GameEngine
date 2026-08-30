use bevy::prelude::*;
use cb_weapons::health::EntityKilledEvent;

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct WeaponChest;

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EntityKilledEvent>()
           .register_type::<WeaponChest>()
           .add_systems(Update, (handle_chest_destroyed, handle_weapon_pickup));
    }
}

#[derive(Component)]
pub struct WeaponPickup {
    pub config: cb_weapons::components::WeaponConfig,
    pub fire: cb_weapons::components::FireRate,
    pub mag: cb_weapons::components::Magazine,
    pub spread: cb_weapons::components::Spread,
    pub recoil: cb_weapons::components::RecoilPattern,
}

pub fn handle_chest_destroyed(
    mut commands: Commands,
    mut killed_events: MessageReader<EntityKilledEvent>,
    q_chests: Query<&Transform, With<WeaponChest>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in killed_events.read() {
        if let Ok(transform) = q_chests.get(ev.entity) {
            let drop_mesh = meshes.add(Cuboid::new(0.8, 0.2, 0.2));
            let drop_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.9, 0.1),
                ..default()
            });

            let weapon_pickup = WeaponPickup {
                config: cb_weapons::components::WeaponConfig {
                    name: "Assault Rifle",
                    fire_mode: cb_weapons::components::FireMode::FullAuto,
                    damage: 35.0,
                    penetration: 0.6,
                    range: 150.0,
                    projectile_speed: Some(300.0),
                },
                fire: cb_weapons::components::FireRate::new(750.0), // fast fire rate
                mag: cb_weapons::components::Magazine::new(30, 90, 2.0), // big mag
                spread: cb_weapons::components::Spread::default(),
                recoil: cb_weapons::components::RecoilPattern::default(),
            };

            commands.spawn((
                Name::new("WeaponDrop"),
                weapon_pickup,
                Mesh3d(drop_mesh),
                MeshMaterial3d(drop_mat),
                Transform::from_translation(transform.translation + Vec3::Y * 1.0),
                avian3d::prelude::RigidBody::Dynamic,
                avian3d::prelude::Collider::cuboid(0.8, 0.2, 0.2),
            ));
        }
    }
}

pub fn handle_weapon_pickup(
    mut commands: Commands,
    input_opt: Option<Res<cb_input::PlayerInput>>,
    mut q_player: Query<(&Transform, &mut cb_weapons::components::WeaponInventory), With<crate::player::Player>>,
    mut q_camera: Query<(
        Entity, 
        &mut cb_weapons::components::WeaponConfig,
        &mut cb_weapons::components::FireRate,
        &mut cb_weapons::components::Magazine,
        &mut cb_weapons::components::Spread,
        &mut cb_weapons::components::RecoilPattern,
    ), With<crate::player::PlayerCamera>>,
    q_drops: Query<(Entity, &Transform, &WeaponPickup)>,
) {
    let Some(input) = input_opt else { return };
    if !input.interact_just {
        return;
    }
    
    let Some((player_tf, mut inventory)) = q_player.iter_mut().next() else { return };
    let Some((_cam_entity, mut w_config, mut w_fire, mut w_mag, mut w_spread, mut w_recoil)) = q_camera.iter_mut().next() else { return; };

    let pickup_radius = 2.5;

    for (drop_entity, drop_tf, pickup) in q_drops.iter() {
        if player_tf.translation.distance(drop_tf.translation) < pickup_radius {
            // Pick it up!
            info!("Picked up weapon: {}", pickup.config.name);

            // Construct bundle for the new weapon
            let new_bundle = cb_weapons::components::WeaponBundle {
                config: pickup.config.clone(),
                fire: cb_weapons::components::FireRate {
                    rpm: pickup.fire.rpm,
                    cooldown: pickup.fire.cooldown,
                    timer: 0.0,
                    trigger_held: false,
                    trigger_just: false,
                    burst_remaining: 0,
                },
                mag: cb_weapons::components::Magazine {
                    current: pickup.mag.max,
                    max: pickup.mag.max,
                    reserve: pickup.mag.reserve,
                    reload_time: pickup.mag.reload_time,
                    reload_timer: 0.0,
                    is_reloading: false,
                },
                spread: pickup.spread.clone(),
                recoil: pickup.recoil.clone(),
            };

            // Save the currently equipped weapon to inventory slot before overwriting it
            let active_bundle = cb_weapons::components::WeaponBundle {
                config: w_config.clone(),
                fire: w_fire.clone(),
                mag: w_mag.clone(),
                spread: w_spread.clone(),
                recoil: w_recoil.clone(),
            };
            if inventory.active_slot == 1 {
                inventory.primary = Some(active_bundle);
            } else {
                inventory.secondary = Some(active_bundle);
            }

            // Figure out where to put the new one
            if inventory.primary.is_none() {
                inventory.primary = Some(new_bundle);
                inventory.active_slot = 1;
            } else if inventory.secondary.is_none() {
                inventory.secondary = Some(new_bundle);
                inventory.active_slot = 2;
            } else {
                // Both full, drop current? No, let's just overwrite the active slot for now to keep it simple,
                // or we could spawn a drop entity for the old one. Overwriting active slot is fine for now.
                if inventory.active_slot == 1 {
                    inventory.primary = Some(new_bundle);
                } else {
                    inventory.secondary = Some(new_bundle);
                }
            }

            // Apply immediately
            let slot = inventory.active_slot;
            let target_bundle = if slot == 1 { inventory.primary.as_ref().unwrap() } else { inventory.secondary.as_ref().unwrap() };
            
            *w_config = target_bundle.config.clone();
            *w_fire = target_bundle.fire.clone();
            *w_mag = target_bundle.mag.clone();
            *w_spread = target_bundle.spread.clone();
            *w_recoil = target_bundle.recoil.clone();

            commands.entity(drop_entity).despawn();
            break;
        }
    }
}
