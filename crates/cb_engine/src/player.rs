use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use cb_movement::fsm::CharacterState;

// ─── Marker components ────────────────────────────────────────────────────────

use serde::{Serialize, Deserialize};

#[derive(Component, Reflect, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerCamera;

pub fn spawn_player(commands: &mut Commands, transform: Transform) -> Entity {
    commands
        .spawn((
            Player,
            transform,
            // Physics
            RigidBody::Dynamic,
            Collider::capsule(0.35, 1.0),   // radius 0.35, half-height 1.0
            LockedAxes::ROTATION_LOCKED,
            GravityScale(2.0),
            // Tnua
            TnuaController::<cb_movement::kcc::CharacterScheme>::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(0.34, 0.0)),
            // Movement FSM
            CharacterState::default(),
            // Health & Combat
            cb_weapons::components::Health::new(100.0),
            cb_weapons::health::ImmortalPlayer,
            cb_weapons::components::PlayerCombatant,
        ))
        .with_children(|parent| {
            // FPS camera at eye level
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.75, 0.0),
                IsDefaultUiCamera,
                cb_weapons::components::WeaponBundle {
                    config: cb_weapons::components::WeaponConfig {
                        name: "Pistol",
                        fire_mode: cb_weapons::components::FireMode::SemiAuto,
                        damage: 25.0,
                        penetration: 0.2,
                        range: 50.0,
                        projectile_speed: Some(150.0),
                    },
                    fire: cb_weapons::components::FireRate::new(400.0),
                    mag: cb_weapons::components::Magazine::new(8, 24, 1.5),
                    spread: cb_weapons::components::Spread::default(),
                    recoil: cb_weapons::components::RecoilPattern::default(),
                },
            ));
        })
        .id()
}

#[derive(Component, Reflect, Default, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[reflect(Component)]
pub struct RemotePlayer {
    pub user_id: u64,
    pub pitch: f32,
}

#[derive(Component)]
pub struct RemotePlayerHead;

#[derive(Component)]
pub struct RemotePlayerGun;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
           .register_type::<PlayerCamera>()
           .register_type::<RemotePlayer>()
           .add_systems(Update, (
               setup_player_mesh,
               setup_remote_player_mesh,
               update_remote_player_pitch,
               player_input,
               camera_look,
           ));
    }
}

pub fn setup_player_mesh(
    mut commands: Commands,
    q_new_cameras: Query<Entity, (With<PlayerCamera>, Added<PlayerCamera>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for cam_entity in q_new_cameras.iter() {
        cb_weapons::viewmodel::spawn_first_person_weapon(
            &mut commands,
            cam_entity,
            &mut meshes,
            &mut materials,
        );
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
            cb_weapons::components::Health::new(100.0),
            cb_weapons::components::PlayerCombatant,
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

pub fn player_input(
    input: Res<cb_input::PlayerInput>,
    mut query: Query<(&mut cb_movement::fsm::CharacterState, &Transform), With<Player>>,
    mut q_weapon: Query<(&mut cb_weapons::components::FireRate, &mut cb_weapons::components::Magazine)>,
) {
    let Ok((mut state, transform)) = query.single_mut() else { return };

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

    if let Ok((mut fire_rate, mut mag)) = q_weapon.single_mut() {
        fire_rate.trigger_held = input.fire_held;
        fire_rate.trigger_just = input.fire_just;
        
        if input.reload {
            mag.start_reload();
        }
    }
}


pub fn camera_look(
    mut mouse_events: MessageReader<bevy::input::mouse::MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    cursor_options: Query<&bevy::window::CursorOptions, With<Window>>,
    mut q_player: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut q_camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    if mouse_buttons.pressed(MouseButton::Right) { return; }

    // Only rotate when cursor is grabbed
    let Ok(cursor) = cursor_options.single() else { return };
    if cursor.grab_mode == bevy::window::CursorGrabMode::None { return; }

    let Ok(mut player_tf) = q_player.single_mut() else { return };
    let Ok(mut cam_tf)    = q_camera.single_mut()  else { return };

    let mut total_delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        total_delta += ev.delta;
    }
    if total_delta == Vec2::ZERO { return; }

    let sensitivity = 0.0015_f32;

    // Yaw — rotate player entity around Y
    player_tf.rotate_y(-total_delta.x * sensitivity);

    // Pitch — rotate camera child around X, clamped
    let current_pitch = cam_tf.rotation.to_euler(EulerRot::YXZ).1;
    let new_pitch = (current_pitch - total_delta.y * sensitivity)
        .clamp(-1.48, 1.48); // ~85°
    cam_tf.rotation = Quat::from_axis_angle(Vec3::X, new_pitch);
}

