use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy::time::Timer;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

#[derive(Resource, Deref, DerefMut)]
pub struct DeathFallSpeed(pub f32);

use cb_movement::fsm::CharacterState;

//  "  "  "  Marker components  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  "  " 

use serde::{Serialize, Deserialize};

#[derive(Component, Reflect, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerCamera;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct FirstPersonWeapon;

pub fn spawn_player(commands: &mut Commands, transform: Transform) -> Entity {
    let starting_weapon = cb_shared::components::WeaponBundle {
        config: cb_shared::components::WeaponConfig {
            name: "Pistol",
            fire_mode: cb_shared::components::FireMode::SemiAuto,
            damage: 20.0,
            penetration: 0.2,
            range: 60.0,
            projectile_speed: Some(200.0),
        },
        fire: cb_shared::components::FireRate {
            rpm: 10000.0, // Fire rate as fast as player clicks
            cooldown: 0.0,
            timer: 0.0,
            trigger_held: false,
            trigger_just: false,
            burst_remaining: 0,
        },
        mag: cb_shared::components::Magazine::new(12, 48, 1.2),
        spread: cb_shared::components::Spread::default(),
        recoil: cb_shared::components::RecoilPattern::default(),
    };

    commands
        .spawn((
            (
                Player,
                transform,
                Visibility::default(),
            ),
            (
                // Physics (high friction & damping to prevent sliding)
                RigidBody::Dynamic,
                Collider::capsule(0.35, 1.0),   // radius 0.35, half-height 1.0
                LockedAxes::ROTATION_LOCKED,
                GravityScale(3.0),
                Friction::new(4.0).with_combine_rule(CoefficientCombine::Max),
                LinearDamping(2.0),
            ),
            (
                // Tnua
                TnuaController::<cb_movement::kcc::CharacterScheme>::default(),
                TnuaAvian3dSensorShape(Collider::cylinder(0.34, 0.0)),
                // Movement FSM
                CharacterState::default(),
            ),
            (
                // Health & Combat
                cb_shared::components::Health::new(100.0),
                cb_shared::components::ImmortalPlayer,
                cb_shared::components::PlayerCombatant,
                starting_weapon.clone_components(),
                cb_shared::components::WeaponInventory {
                    pending_slot: None,
                    primary: Some(starting_weapon.clone_components()),
                    secondary: None,
                    active_slot: 1,
                },
            )
        ))
        .with_children(|parent| {
            // FPS camera at eye level
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.75, 0.0),
                IsDefaultUiCamera,
                starting_weapon,
            ));
        })
        .id()
}

#[derive(Component, Reflect, Default, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[reflect(Component)]
pub struct RemotePlayer {
    pub user_id: u64,
    pub pitch: f32,
    pub active_weapon: u8,
}

#[derive(Component)]
pub struct RemotePlayerHead;

#[derive(Component)]
pub struct RemotePlayerGun;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IsDead {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Ragdoll;

#[derive(Component)]
#[derive(Message)]
pub struct PlayerRespawnedEvent;

#[derive(Component)]
pub struct RespawnReady;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
           .register_type::<PlayerCamera>()
           .register_type::<RemotePlayer>()
           .register_type::<IsDead>()
           .insert_resource(DeathFallSpeed(0.5))
            .add_message::<PlayerRespawnedEvent>()
           .add_message::<PlayerRespawnedEvent>()
            .add_systems(
                Update,
                (
                    crate::player_init::auto_add_player_components,
                    setup_player_mesh,
                    setup_remote_player_mesh,
                    update_remote_player_pitch,
                    update_remote_player_gun_visibility,
                    update_weapon_visuals,
                    handle_player_death,
                    update_player_respawn,
                    player_input,
                    camera_look,
                ),
            );
    }
}

#[cfg(feature = "weapons")]
pub fn handle_player_death(
    mut commands: Commands,
    mut killed_events: MessageReader<cb_weapons::health::EntityKilledEvent>,
    q_player: Query<Entity, (With<cb_shared::components::Health>, With<Player>, Without<IsDead>)>,
    mut q_weapon: Query<&mut Visibility, With<FirstPersonWeapon>>,
    mut q_camera: Query<&mut Transform, With<PlayerCamera>>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<Window>>,
    death_fall_speed: Res<DeathFallSpeed>,
) {
    // Original implementation unchanged
    for ev in killed_events.read() {
        if q_player.get(ev.entity).is_ok() {
            info!("Player was eliminated! Handling death...");
            commands.entity(ev.entity)
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(LinearVelocity(Vec3::new(0.0, -death_fall_speed.0, 0.0)))
                .insert(IsDead { timer: Timer::from_seconds(0.3, TimerMode::Once) });
            if let Ok(mut cam_tf) = q_camera.single_mut() {
                cam_tf.translation = Vec3::new(0.0, 0.25, 0.0);
                cam_tf.rotation = Quat::IDENTITY;
            }
            for mut vis in q_weapon.iter_mut() {
                *vis = Visibility::Hidden;
            }
            if let Ok(mut cursor) = cursor_options.single_mut() {
                cursor.grab_mode = bevy::window::CursorGrabMode::None;
                cursor.visible = true;
            }
        }
    }
}

#[cfg(not(feature = "weapons"))]
pub fn handle_player_death(
    _commands: Commands,
    _q_player: Query<Entity, (With<cb_shared::components::Health>, With<Player>, Without<IsDead>)>,
    _q_weapon: Query<&mut Visibility>,
    _q_camera: Query<&mut Transform>,
    _cursor_options: Query<&mut bevy::window::CursorOptions>,
    _death_fall_speed: Res<DeathFallSpeed>,
) {
    // Weapons feature disabled; no death handling via weapons events.
    // This stub simply does nothing.
}

pub fn update_player_respawn(
    mut commands: Commands,
    mut q_player: Query<(Entity, &mut Transform, &mut cb_shared::components::Health, Option<&mut Position>, Option<&mut LinearVelocity>, Option<&mut Rotation>), (With<Player>, With<IsDead>, Without<PlayerCamera>)>,
    q_spawns: Query<(&Transform, &crate::editor::serialization::SceneObject), (Without<Player>, Without<PlayerCamera>)>,
    mut q_weapon: Query<&mut Visibility, With<FirstPersonWeapon>>,
    mut q_camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut egui_contexts: Query<&mut bevy_egui::EguiContext, With<bevy::window::PrimaryWindow>>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<Window>>,
    mut ev_respawn: MessageWriter<PlayerRespawnedEvent>,
) {
    let Ok(mut ctx) = egui_contexts.single_mut() else { return };
    
    for (entity, mut tf, mut health, pos_opt, vel_opt, rot_opt) in q_player.iter_mut() {
        let mut clicked_respawn = false;

        bevy_egui::egui::Window::new("CodeBlue")
            .anchor(bevy_egui::egui::Align2::CENTER_CENTER, bevy_egui::egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx.get_mut(), |ui| {
                ui.label("Waiting to deploy...");
                if ui.button("Deploy").clicked() {
                    clicked_respawn = true;
                }
            });

        if !clicked_respawn {
            if let Ok(mut cursor) = cursor_options.single_mut() {
                if cursor.grab_mode != bevy::window::CursorGrabMode::None {
                    cursor.grab_mode = bevy::window::CursorGrabMode::None;
                    cursor.visible = true;
                }
            }
        }

        if clicked_respawn {
            info!("Player clicked respawn! Restoring player at spawn point.");
            health.current = health.max;
            for mut vis in q_weapon.iter_mut() {
                *vis = Visibility::Inherited;
            }

            // Restore camera to eye level and upright
            if let Ok(mut cam_tf) = q_camera.single_mut() {
                cam_tf.translation = Vec3::new(0.0, 0.75, 0.0);
                cam_tf.rotation = Quat::IDENTITY;
            }

            let mut available_spawns = Vec::new();
            for (spawn_tf, scene_obj) in q_spawns.iter() {
                if scene_obj.object_type == "spawn_point" {
                    available_spawns.push(spawn_tf.translation + Vec3::Y * 0.5);
                }
            }
            
            let spawn_pos = if !available_spawns.is_empty() {
                let idx = fastrand::usize(..available_spawns.len());
                available_spawns[idx]
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };

            // Restore position
            tf.translation = spawn_pos;
            tf.rotation = Quat::IDENTITY;
            if let Some(mut p) = pos_opt {
                p.0 = spawn_pos;
            }
            if let Some(mut v) = vel_opt {
                v.0 = Vec3::ZERO;
            }
            if let Some(mut r) = rot_opt {
                r.0 = Quat::IDENTITY;
            }

            commands.entity(entity)
                .remove::<IsDead>()
                .remove::<avian3d::prelude::Sleeping>()
                .insert(avian3d::prelude::Collider::capsule(0.35, 1.0))
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(AngularVelocity::ZERO)
                .insert(bevy_tnua::prelude::TnuaController::<cb_movement::kcc::CharacterScheme>::default());

            if let Ok(mut cursor) = cursor_options.single_mut() {
                cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
                cursor.visible = false;
            }

            ev_respawn.write(PlayerRespawnedEvent);
        }
    }
}

pub fn setup_player_mesh(
    mut commands: Commands,
    q_new_cameras: Query<Entity, (With<PlayerCamera>, Added<PlayerCamera>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for cam_entity in q_new_cameras.iter() {
        // Dark gunmetal body material
        let body_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.13, 0.15),
            metallic: 0.85,
            perceptual_roughness: 0.25,
            ..default()
        });

        // Accent material
        let accent_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.65, 0.85),
            emissive: LinearRgba::new(0.02, 0.25, 0.35, 1.0),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        });

        let frame_mesh = meshes.add(Cuboid::new(0.065, 0.10, 0.28));
        let barrel_mesh = meshes.add(Cuboid::new(0.045, 0.05, 0.22));
        let slide_mesh = meshes.add(Cuboid::new(0.060, 0.045, 0.26));
        let grip_mesh = meshes.add(Cuboid::new(0.055, 0.13, 0.09));

        let weapon_root = commands
            .spawn((
                Name::new("FirstPersonWeapon"),
                FirstPersonWeapon,
                Transform::from_xyz(0.0, -0.22, -0.42),
                Visibility::default(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(frame_mesh),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));
                parent.spawn((
                    Mesh3d(barrel_mesh),
                    MeshMaterial3d(accent_mat.clone()),
                    Transform::from_xyz(0.0, 0.025, -0.14),
                ));
                parent.spawn((
                    Mesh3d(slide_mesh),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.0, 0.045, 0.01),
                ));
                parent.spawn((
                    Mesh3d(grip_mesh),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.0, -0.07, 0.05).with_rotation(Quat::from_rotation_x(0.22)),
                ));
            })
            .id();

        commands.entity(cam_entity).add_child(weapon_root);
    }
}

pub fn setup_remote_player_mesh(
    mut commands: Commands,
    q_new_remote: Query<(Entity, &RemotePlayer), Added<RemotePlayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, remote_player) in q_new_remote.iter() {
        let user_color = crate::editor::user_color::get_user_color_bevy(remote_player.user_id);

        let body_mat = materials.add(StandardMaterial {
            base_color: user_color,
            metallic: 0.2,
            perceptual_roughness: 0.5,
            ..default()
        });

        let visor_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.10, 0.12),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        });

        let gun_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.15, 0.17),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        });

        let body_mesh = meshes.add(Capsule3d::new(0.35, 1.0));
        let head_mesh = meshes.add(Sphere::new(0.24));
        let visor_mesh = meshes.add(Cuboid::new(0.28, 0.12, 0.14));
        let gun_mesh = meshes.add(Cuboid::new(0.08, 0.10, 0.38));

        commands.entity(entity).insert((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_mat.clone()),
            RigidBody::Kinematic,
            Collider::capsule(0.35, 1.0),
            cb_shared::components::Health::new(100.0),
            cb_shared::components::PlayerCombatant,
            cb_shared::components::ImmortalPlayer,
        ));

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                RemotePlayerHead,
                Transform::from_xyz(0.0, 0.75, 0.0),
                Visibility::default(),
            )).with_children(|head_parent| {
                head_parent.spawn((
                    Mesh3d(head_mesh),
                    MeshMaterial3d(body_mat),
                    Transform::default(),
                ));
                head_parent.spawn((
                    Mesh3d(visor_mesh),
                    MeshMaterial3d(visor_mat),
                    Transform::from_xyz(0.0, 0.02, -0.16),
                ));
                head_parent.spawn((
                    RemotePlayerGun,
                    Mesh3d(gun_mesh),
                    MeshMaterial3d(gun_mat),
                    Transform::from_xyz(0.30, -0.15, -0.32),
                    cb_shared::components::WeaponConfig {
                        name: "RemotePistol",
                        fire_mode: cb_shared::components::FireMode::SemiAuto,
                        damage: 20.0,
                        penetration: 0.2,
                        range: 60.0,
                        projectile_speed: Some(200.0),
                    },
                ));
            });
        });
    }
}

pub fn update_remote_player_pitch(
    q_remote: Query<(&RemotePlayer, &Children)>,
    mut q_heads: Query<&mut Transform, With<RemotePlayerHead>>,
) {
    for (remote, children) in q_remote.iter() {
        for child in children.iter() {
            if let Ok(mut head_tf) = q_heads.get_mut(child) {
                head_tf.rotation = Quat::from_rotation_x(remote.pitch);
            }
        }
    }
}

pub fn update_remote_player_gun_visibility(
    q_remote: Query<(&cb_shared::components::Health, &Children), With<RemotePlayer>>,
    q_heads: Query<&Children, With<RemotePlayerHead>>,
    mut q_guns: Query<&mut Visibility, With<RemotePlayerGun>>,
) {
    for (health, children) in q_remote.iter() {
        let is_dead = health.current <= 0.0;
        for child in children.iter() {
            if let Ok(head_children) = q_heads.get(child) {
                for head_child in head_children.iter() {
                    if let Ok(mut vis) = q_guns.get_mut(head_child) {
                        *vis = if is_dead { Visibility::Hidden } else { Visibility::Inherited };
                    }
                }
            }
        }
    }
}

pub fn player_input(
    input_opt: Option<Res<cb_input::PlayerInput>>,
    mut query: Query<(
        &mut cb_movement::fsm::CharacterState, 
        &Transform, 
        &mut cb_shared::components::WeaponInventory, 
        Option<&IsDead>,
        &mut cb_shared::components::WeaponConfig,
        &mut cb_shared::components::FireRate,
        &mut cb_shared::components::Magazine,
        &mut cb_shared::components::Spread,
        &mut cb_shared::components::RecoilPattern,
    ), With<Player>>,
) {
    let Some(input) = input_opt else { return };
    let Ok((
        mut state, transform, mut inventory, is_dead,
        mut w_config, mut w_fire, mut w_mag, mut w_spread, mut w_recoil
    )) = query.single_mut() else { return };

    if is_dead.is_some() {
        state.desired_direction = Vec3::ZERO;
        state.wishes_jump = false;
        state.wishes_sprint = false;
        state.wishes_tac_sprint = false;
        state.wishes_crouch = false;
        return;
    }

    // Move relative to facing direction
    let forward = transform.local_z() * -input.move_dir.y;
    let right = transform.local_x() * input.move_dir.x;
    let mut direction = forward + right;
    direction.y = 0.0;

    state.desired_direction = direction;
    state.wishes_jump   = input.jump;
    state.wishes_sprint = input.sprint;
    state.wishes_tac_sprint = input.tac_sprint;
    state.wishes_crouch = input.crouch;

    let mut desired_slot = inventory.active_slot;
    if let Some(pending) = inventory.pending_slot.take() {
        desired_slot = pending;
    } else if input.swap_prev || input.swap_next {
        desired_slot = if input.swap_next {
            if inventory.active_slot == 1 { 2 } else { 1 }
        } else {
            if inventory.active_slot == 2 { 1 } else { 2 }
        };
    }
    
    if inventory.active_slot != desired_slot {
            let can_swap = if desired_slot == 1 { inventory.primary.is_some() } else { inventory.secondary.is_some() };
            if can_swap {
                    // Save current to active slot
                    let active_bundle = cb_shared::components::WeaponBundle {
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

                    // Apply desired slot
                    inventory.active_slot = desired_slot;
                    let target_bundle = if desired_slot == 1 { inventory.primary.as_ref().unwrap() } else { inventory.secondary.as_ref().unwrap() };
                    
                    *w_config = target_bundle.config.clone();
                    *w_fire = target_bundle.fire.clone();
                    *w_mag = target_bundle.mag.clone();
                    *w_spread = target_bundle.spread.clone();
                    *w_recoil = target_bundle.recoil.clone();
                    
                    info!("Swapped to weapon slot {}", desired_slot);
            }
        }

    // Weapon firing: only allowed when walking or standing still (grounded, not sprinting or airborne)
    let can_shoot = matches!(
        state.current,
        cb_movement::fsm::MovementState::Idle
            | cb_movement::fsm::MovementState::Walk
            | cb_movement::fsm::MovementState::Crouch
            | cb_movement::fsm::MovementState::Prone
    ) && !state.wishes_sprint
        && !state.wishes_tac_sprint
        && state.is_grounded;

    if can_shoot {
        w_fire.trigger_held = input.fire_held;
        if input.fire_just {
            w_fire.trigger_just = true;
        }
    } else {
        w_fire.trigger_held = false;
        w_fire.trigger_just = false;
    }
    
    if input.reload {
        w_mag.start_reload();
    }
}


pub fn camera_look(
    mut mouse_events: MessageReader<bevy::input::mouse::MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    cursor_options: Query<&bevy::window::CursorOptions, With<Window>>,
    mut q_player: Query<(&mut Transform, Option<&mut avian3d::prelude::Rotation>), (With<Player>, Without<PlayerCamera>)>,
    mut q_camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    if mouse_buttons.pressed(MouseButton::Right) { return; }

    // Only rotate when cursor is grabbed
    let Ok(cursor) = cursor_options.single() else { return };
    if cursor.grab_mode == bevy::window::CursorGrabMode::None { return; }

    let Ok((mut player_tf, player_rot_opt)) = q_player.single_mut() else { return };
    let Ok(mut cam_tf)    = q_camera.single_mut()  else { return };

    let mut total_delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        total_delta += ev.delta;
    }
    if total_delta == Vec2::ZERO { return; }

    let sensitivity = 0.0015_f32;

    // Yaw   " rotate player entity around Y
    player_tf.rotate_y(-total_delta.x * sensitivity);
    if let Some(mut player_rot) = player_rot_opt {
        player_rot.0 = player_tf.rotation;
    }

    // Pitch   " rotate camera child around X, clamped
    let current_pitch = cam_tf.rotation.to_euler(EulerRot::YXZ).1;
    let new_pitch = (current_pitch - total_delta.y * sensitivity)
        .clamp(-1.48, 1.48); // ~85  deg
    cam_tf.rotation = Quat::from_axis_angle(Vec3::X, new_pitch);
}


pub fn update_weapon_visuals(
    q_inventory: Query<&cb_shared::components::WeaponInventory, With<Player>>,
    mut q_fp_weapon: Query<&mut Transform, With<FirstPersonWeapon>>,
    q_remote: Query<(&RemotePlayer, &Children)>,
    q_heads: Query<&Children, With<RemotePlayerHead>>,
    mut q_remote_guns: Query<&mut Transform, (With<RemotePlayerGun>, Without<FirstPersonWeapon>)>,
) {
    if let Ok(inventory) = q_inventory.single() {
        if let Ok(mut fp_tf) = q_fp_weapon.single_mut() {
            if inventory.active_slot == 2 {
                // Rifle: longer barrel, extended silhouette
                fp_tf.scale = Vec3::new(1.2, 1.25, 2.4);
                fp_tf.translation = Vec3::new(0.0, -0.21, -0.45);
                fp_tf.rotation = Quat::from_rotation_y(0.1); // slight yaw for visual cue
            } else {
                // Pistol: compact handgun
                fp_tf.scale = Vec3::ONE;
                fp_tf.translation = Vec3::new(0.0, -0.22, -0.42);
            }
        }
    }

    for (remote, children) in q_remote.iter() {
        for child in children.iter() {
            if let Ok(head_children) = q_heads.get(child) {
                for head_child in head_children.iter() {
                    if let Ok(mut gun_tf) = q_remote_guns.get_mut(head_child) {
                        if remote.active_weapon == 2 {
                            // Rifle for remote players
                            gun_tf.scale = Vec3::new(1.3, 1.3, 2.5);
                            gun_tf.translation = Vec3::new(0.30, -0.15, -0.48);
                        } else {
                            // Pistol for remote players
                            gun_tf.scale = Vec3::ONE;
                            gun_tf.translation = Vec3::new(0.30, -0.15, -0.32);
                        }
                    }
                }
            }
        }
    }
}

