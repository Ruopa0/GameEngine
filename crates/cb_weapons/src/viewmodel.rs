use bevy::prelude::*;

/// Component attached to the first-person weapon model.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FirstPersonWeapon {
    /// Default offset relative to the first-person camera (bottom-center of view).
    pub rest_pos: Vec3,
    /// Current smoothed offset from camera.
    pub current_pos: Vec3,
    /// Target sway rotation accumulated from mouse movement.
    pub sway_rot: Quat,
    /// Current rotational lag quaternion.
    pub current_rot: Quat,
    /// Movement bobbing phase.
    pub bob_timer: f32,
    /// Recoil kick back offset.
    pub recoil_pos: Vec3,
    /// Recoil pitch kick.
    pub recoil_rot: Quat,
}

impl Default for FirstPersonWeapon {
    fn default() -> Self {
        Self {
            // Bottom-center of the screen, extending forward
            rest_pos: Vec3::new(0.0, -0.22, -0.42),
            current_pos: Vec3::new(0.0, -0.22, -0.42),
            sway_rot: Quat::IDENTITY,
            current_rot: Quat::IDENTITY,
            bob_timer: 0.0,
            recoil_pos: Vec3::ZERO,
            recoil_rot: Quat::IDENTITY,
        }
    }
}

/// Spawns a stylized, multi-part first-person firearm attached directly to the FPS camera.
pub fn spawn_first_person_weapon(
    commands: &mut Commands,
    camera_entity: Entity,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    // Dark gunmetal body material
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.15),
        metallic: 0.85,
        perceptual_roughness: 0.25,
        ..default()
    });

    // Accent material (subtle tactical cyan/gold glow highlight)
    let accent_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.65, 0.85),
        emissive: LinearRgba::new(0.02, 0.25, 0.35, 1.0),
        metallic: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Dark titanium barrel material
    let barrel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.22, 0.25),
        metallic: 0.95,
        perceptual_roughness: 0.15,
        ..default()
    });

    let main_receiver_mesh = meshes.add(Cuboid::new(0.065, 0.085, 0.26));
    let barrel_mesh = meshes.add(Cuboid::new(0.032, 0.032, 0.22));
    let muzzle_mesh = meshes.add(Cuboid::new(0.038, 0.038, 0.045));
    let top_rail_mesh = meshes.add(Cuboid::new(0.022, 0.018, 0.24));
    let front_sight_mesh = meshes.add(Cuboid::new(0.008, 0.022, 0.012));
    let rear_sight_mesh = meshes.add(Cuboid::new(0.018, 0.018, 0.012));
    let lower_grip_mesh = meshes.add(Cuboid::new(0.045, 0.10, 0.065));

    let weapon_root = commands
        .spawn((
            FirstPersonWeapon::default(),
            Transform::from_xyz(0.0, -0.22, -0.42),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Main receiver
            parent.spawn((
                Mesh3d(main_receiver_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Barrel extending forward
            parent.spawn((
                Mesh3d(barrel_mesh),
                MeshMaterial3d(barrel_mat.clone()),
                Transform::from_xyz(0.0, 0.01, -0.20),
            ));

            // Muzzle brake tip
            parent.spawn((
                Mesh3d(muzzle_mesh),
                MeshMaterial3d(barrel_mat.clone()),
                Transform::from_xyz(0.0, 0.01, -0.32),
            ));

            // Top Picatinny rail
            parent.spawn((
                Mesh3d(top_rail_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.052, -0.05),
            ));

            // Front sight post (with glowing accent)
            parent.spawn((
                Mesh3d(front_sight_mesh),
                MeshMaterial3d(accent_mat.clone()),
                Transform::from_xyz(0.0, 0.068, -0.28),
            ));

            // Rear sight notch
            parent.spawn((
                Mesh3d(rear_sight_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.068, 0.06),
            ));

            // Lower grip (angled slightly back)
            parent.spawn((
                Mesh3d(lower_grip_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, -0.07, 0.05).with_rotation(Quat::from_rotation_x(0.22)),
            ));
        })
        .id();

    commands.entity(camera_entity).add_child(weapon_root);
    weapon_root
}

/// Updates mouse sway, movement bob, and recoil impulse on the viewmodel.
pub fn update_viewmodel_sway(
    time: Res<Time>,
    mut mouse_events: MessageReader<bevy::input::mouse::MouseMotion>,
    input: Option<Res<cb_input::PlayerInput>>,
    mut query: Query<(&mut FirstPersonWeapon, &mut Transform)>,
) {
    let Some(input) = input else { return; };
    let dt = time.delta_secs();
    let mut mouse_delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        mouse_delta += ev.delta;
    }

    let is_sprinting = input.sprint;
    let is_tac_sprinting = input.tac_sprint;
    let is_crouching = input.crouch;
    let is_moving = input.move_dir.length_squared() > 0.01;

    for (mut weapon, mut transform) in query.iter_mut() {
        // 1. Mouse Sway & Rotational Inertia
        let sway_amount_x = (-mouse_delta.x * 0.0018).clamp(-0.15, 0.15);
        let sway_amount_y = (-mouse_delta.y * 0.0018).clamp(-0.15, 0.15);
        let sway_roll = (mouse_delta.x * 0.0012).clamp(-0.12, 0.12);

        let target_sway_rot = Quat::from_euler(EulerRot::YXZ, sway_amount_x, sway_amount_y, sway_roll);
        // Smoothly interpolate sway towards target, then spring back to neutral
        weapon.sway_rot = weapon.sway_rot.slerp(target_sway_rot, (dt * 15.0).min(1.0));
        weapon.sway_rot = weapon.sway_rot.slerp(Quat::IDENTITY, (dt * 8.0).min(1.0));

        // 2. Walking Bobbing
        let speed_factor = if is_tac_sprinting { 2.0 } else if is_sprinting { 1.5 } else if is_crouching { 0.5 } else { 1.0 };
        if is_moving {
            weapon.bob_timer += dt * 10.0 * speed_factor;
        } else {
            // Idle gentle breathing
            weapon.bob_timer += dt * 1.8;
        }

        let bob_h = weapon.bob_timer.sin() * (if is_moving { 0.008 } else { 0.002 });
        let bob_v = (weapon.bob_timer * 2.0).cos().abs() * (if is_moving { 0.006 } else { 0.0015 });

        // 3. Recoil recovery
        weapon.recoil_pos = weapon.recoil_pos.lerp(Vec3::ZERO, (dt * 16.0).min(1.0));
        weapon.recoil_rot = weapon.recoil_rot.slerp(Quat::IDENTITY, (dt * 20.0).min(1.0));

        let mut pose_offset = Vec3::ZERO;
        let mut pose_rot = Quat::IDENTITY;

        if is_tac_sprinting && is_moving {
            pose_offset = Vec3::new(0.1, 0.02, -0.1);
            pose_rot = Quat::from_euler(EulerRot::YXZ, -0.3, 0.8, -0.4);
        } else if is_sprinting && is_moving {
            pose_offset = Vec3::new(0.05, -0.05, 0.05);
            pose_rot = Quat::from_rotation_z(0.1) * Quat::from_rotation_x(0.1);
        }

        // 4. Combine into final local transform
        let target_pos = weapon.rest_pos + Vec3::new(bob_h, -bob_v, 0.0) + weapon.recoil_pos + pose_offset;
        weapon.current_pos = weapon.current_pos.lerp(target_pos, (dt * 12.0).min(1.0));

        // Smooth rotational lag
        let target_rot = weapon.sway_rot * weapon.recoil_rot * pose_rot;
        weapon.current_rot = weapon.current_rot.slerp(target_rot, (dt * 12.0).min(1.0));

        transform.translation = weapon.current_pos;
        transform.rotation = weapon.current_rot;
    }
}

/// Applies recoil kick to the first-person weapon whenever a shot is fired.
pub fn viewmodel_recoil_kick(
    mut events: MessageReader<crate::systems::ShotFiredEvent>,
    mut query: Query<&mut FirstPersonWeapon>,
) {
    for _ in events.read() {
        for mut weapon in query.iter_mut() {
            // Kick backwards along Z
            weapon.recoil_pos += Vec3::new(0.0, 0.015, 0.05);
            // Pitch barrel upwards
            weapon.recoil_rot *= Quat::from_rotation_x(0.085);
        }
    }
}

pub struct ViewModelPlugin;

impl Plugin for ViewModelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FirstPersonWeapon>()
           .add_systems(Update, (update_viewmodel_sway, viewmodel_recoil_kick));
    }
}
