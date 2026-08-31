use bevy::prelude::*;

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct WeaponChest;

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct ChestOpened;

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WeaponChest>()
           .register_type::<ChestOpened>()
           .add_systems(Update, (handle_chest_interact, handle_weapon_pickup));
    }
}

#[derive(Component)]
pub struct WeaponPickup {
    pub config: cb_shared::components::WeaponConfig,
    pub fire: cb_shared::components::FireRate,
    pub mag: cb_shared::components::Magazine,
    pub spread: cb_shared::components::Spread,
    pub recoil: cb_shared::components::RecoilPattern,
}

pub fn handle_chest_interact(
    mut commands: Commands,
    input_opt: Option<Res<cb_input::PlayerInput>>,
    q_player: Query<&Transform, With<crate::player::Player>>,
    q_chests: Query<(Entity, &Transform), (With<WeaponChest>, Without<ChestOpened>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(input) = input_opt else { return };
    if !input.interact_just {
        return;
    }

    let Some(player_tf) = q_player.iter().next() else { return };

    let interact_dist = 3.0;
    for (chest_entity, chest_tf) in q_chests.iter() {
        if player_tf.translation.distance(chest_tf.translation) <= interact_dist {
            info!("Player opened chest with F key!");
            commands.entity(chest_entity).insert(ChestOpened);

            // Spawn the assault rifle drop above the intact chest
            let drop_mesh = meshes.add(Cuboid::new(0.8, 0.2, 0.2));
            let drop_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.9, 0.1),
                emissive: LinearRgba::new(0.2, 1.8, 0.2, 1.0),
                ..default()
            });

            let weapon_pickup = WeaponPickup {
                config: cb_shared::components::WeaponConfig {
                    name: "Assault Rifle",
                    fire_mode: cb_shared::components::FireMode::FullAuto,
                    damage: 35.0,
                    penetration: 0.6,
                    range: 150.0,
                    projectile_speed: Some(300.0),
                },
                fire: cb_shared::components::FireRate::new(750.0),
                mag: cb_shared::components::Magazine::new(30, 90, 2.0),
                spread: cb_shared::components::Spread::default(),
                recoil: cb_shared::components::RecoilPattern::default(),
            };

            commands.spawn((
                Name::new("WeaponDrop"),
                weapon_pickup,
                Mesh3d(drop_mesh),
                MeshMaterial3d(drop_mat),
                Transform::from_translation(chest_tf.translation + Vec3::Y * 0.8),
                avian3d::prelude::RigidBody::Kinematic,
                // Collider removed to reduce physics load
            ));

            // Spawn a small, tasteful burst of 4-6 spark particles
            let p_mesh = meshes.add(Cuboid::new(0.04, 0.04, 0.04));
            let p_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.85, 0.3),
                emissive: LinearRgba::new(4.0, 3.0, 0.8, 1.0),
                unlit: true,
                ..default()
            });

            let mut rng = fastrand::Rng::new();
            for _ in 0..5 {
                let dir = Vec3::new(
                    rng.f32() * 1.6 - 0.8,
                    rng.f32() * 1.5 + 1.0,
                    rng.f32() * 1.6 - 0.8,
                ).normalize_or_zero();
                let speed = rng.f32() * 2.0 + 1.5;

                commands.spawn((
                    Mesh3d(p_mesh.clone()),
                    MeshMaterial3d(p_mat.clone()),
                    Transform::from_translation(chest_tf.translation + Vec3::Y * 0.6),
                    crate::vfx::Particle {
                        velocity: dir * speed,
                        lifetime: rng.f32() * 0.3 + 0.2,
                        start_lifetime: 0.5,
                    },
                ));
            }

            break;
        }
    }
}

#[cfg(feature = "weapons")]
pub fn handle_weapon_pickup(
    mut commands: Commands,
    input_opt: Option<Res<cb_input::PlayerInput>>,
    mut q_player: Query<(&Transform, &mut cb_shared::components::WeaponInventory), With<crate::player::Player>>,
    mut q_camera: Query<(
        Entity,
        &mut cb_shared::components::WeaponConfig,
        &mut cb_shared::components::FireRate,
        &mut cb_shared::components::Magazine,
        &mut cb_shared::components::Spread,
        &mut cb_shared::components::RecoilPattern,
    ), With<crate::player::PlayerCamera>>,
    q_drops: Query<(Entity, &Transform, &WeaponPickup)>,
) {
    let Some(input) = input_opt else { return };
    if !input.interact_just { return }
    let Some((player_tf, mut inventory)) = q_player.iter_mut().next() else { return };
    let Some((_cam_entity, mut w_config, mut w_fire, mut w_mag, mut w_spread, mut w_recoil)) = q_camera.iter_mut().next() else { return };
    let pickup_radius = 2.5;
    for (drop_entity, drop_tf, pickup) in q_drops.iter() {
        if player_tf.translation.distance(drop_tf.translation) < pickup_radius {
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
            // Save currently equipped weapon
            let active_bundle = cb_weapons::components::WeaponBundle {
                config: w_config.clone(),
                fire: w_fire.clone(),
                mag: w_mag.clone(),
                spread: w_spread.clone(),
                recoil: w_recoil.clone(),
            };
            if inventory.active_slot == 1 { inventory.primary = Some(active_bundle); } else { inventory.secondary = Some(active_bundle); }
            // Put new one into inventory
            if inventory.primary.is_none() { inventory.primary = Some(new_bundle); inventory.active_slot = 1; }
            else if inventory.secondary.is_none() { inventory.secondary = Some(new_bundle); inventory.active_slot = 2; }
            else {
                if inventory.active_slot == 1 { inventory.primary = Some(new_bundle); } else { inventory.secondary = Some(new_bundle); }
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

#[cfg(not(feature = "weapons"))]
pub fn handle_weapon_pickup(
    mut _commands: Commands,
    _input_opt: Option<Res<cb_input::PlayerInput>>,
    _q_player: Query<(&Transform, &mut cb_shared::components::WeaponInventory), With<crate::player::Player>>, 
    _q_camera: Query<(
        Entity,
        &mut cb_shared::components::WeaponConfig,
        &mut cb_shared::components::FireRate,
        &mut cb_shared::components::Magazine,
        &mut cb_shared::components::Spread,
        &mut cb_shared::components::RecoilPattern,
    ), With<crate::player::PlayerCamera>>,
    _q_drops: Query<(Entity, &Transform, &WeaponPickup)>,
) {
    // Weapons feature disabled; do nothing.
}
