use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct Zipline {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Component)]
pub struct RidingZipline {
    pub start: Vec3,
    pub end: Vec3,
    pub progress: f32,
    pub speed: f32,
}

pub struct ZiplinePlugin;

impl Plugin for ZiplinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Zipline>()
           .add_systems(Update, (handle_zipline_interact, update_riding_ziplines, spawn_zipline_visuals));
    }
}

/// Spawns the physical cable and mounting anchors for any Zipline entity without visuals
fn spawn_zipline_visuals(
    mut commands: Commands,
    q_ziplines: Query<(Entity, &Zipline), Without<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, zipline) in q_ziplines.iter() {
        let delta = zipline.end - zipline.start;
        let length = delta.length();
        if length < 0.1 { continue; }

        let mid_point = zipline.start + delta * 0.5;
        let rotation = Quat::from_rotation_arc(Vec3::Y, delta.normalize_or_zero());

        let cable_mesh = meshes.add(Cylinder::new(0.025, length));
        let cable_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.75, 0.1), // High-vis industrial yellow
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        });

        // Insert visual cable onto the zipline entity
        commands.entity(entity).insert((
            Mesh3d(cable_mesh),
            MeshMaterial3d(cable_mat),
            Transform::from_translation(mid_point).with_rotation(rotation),
        ));

        // Spawn visual anchor posts at start and top
        let post_mesh = meshes.add(Cylinder::new(0.08, 1.8));
        let post_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.22, 0.25),
            metallic: 0.9,
            perceptual_roughness: 0.3,
            ..default()
        });

        commands.spawn((
            Name::new("ZiplineBaseAnchor"),
            Mesh3d(post_mesh.clone()),
            MeshMaterial3d(post_mat.clone()),
            Transform::from_translation(zipline.start),
        ));

        commands.spawn((
            Name::new("ZiplineTopAnchor"),
            Mesh3d(post_mesh),
            MeshMaterial3d(post_mat),
            Transform::from_translation(zipline.end),
        ));
    }
}

/// Player presses F to ride a nearby zipline
fn handle_zipline_interact(
    mut commands: Commands,
    input_opt: Option<Res<cb_input::PlayerInput>>,
    q_player: Query<(Entity, &Transform), (With<crate::player::Player>, Without<RidingZipline>)>,
    q_ziplines: Query<&Zipline>,
) {
    let Some(input) = input_opt else { return };
    if !input.interact_just {
        return;
    }

    let Ok((player_entity, player_tf)) = q_player.single() else { return };

    let interact_dist = 3.5;
    for zipline in q_ziplines.iter() {
        if player_tf.translation.distance(zipline.start) <= interact_dist {
            info!("Player attached to zipline! Ascending to rooftop...");
            commands.entity(player_entity).insert(RidingZipline {
                start: zipline.start,
                end: zipline.end,
                progress: 0.0,
                speed: 18.0, // fast tactical ascent
            });
            break;
        }
    }
}

/// Updates players currently riding a zipline, handles 'C' cancellation, and rooftop landing
fn update_riding_ziplines(
    mut commands: Commands,
    time: Res<Time>,
    input_opt: Option<Res<cb_input::PlayerInput>>,
    mut q_riders: Query<(
        Entity,
        &mut Transform,
        &mut RidingZipline,
        Option<&mut avian3d::prelude::LinearVelocity>,
    ), With<crate::player::Player>>,
) {
    let dt = time.delta_secs();
    let cancel_pressed = input_opt.as_ref().map(|i| i.crouch_just_pressed).unwrap_or(false);

    for (entity, mut tf, mut ride, vel_opt) in q_riders.iter_mut() {
        // C key cancels the zipline ride at any time
        if cancel_pressed {
            info!("Player cancelled zipline ride with 'C' key!");
            if let Some(mut vel) = vel_opt {
                vel.0 = Vec3::new(0.0, 2.0, 0.0);
            }
            commands.entity(entity).remove::<RidingZipline>();
            continue;
        }

        let total_dist = ride.start.distance(ride.end).max(0.1);
        ride.progress += (ride.speed * dt) / total_dist;

        if ride.progress >= 1.0 {
            // Reached top of building: place player safely onto the rooftop
            info!("Player reached the top of the building!");
            let step_onto_roof = ride.end + Vec3::new(0.0, 1.0, 0.0);
            tf.translation = step_onto_roof;
            if let Some(mut vel) = vel_opt {
                vel.0 = Vec3::ZERO;
            }
            commands.entity(entity).remove::<RidingZipline>();
        } else {
            // Smoothly ascend along the cable
            let current_pos = ride.start.lerp(ride.end, ride.progress);
            tf.translation = current_pos;
            if let Some(mut vel) = vel_opt {
                vel.0 = (ride.end - ride.start).normalize_or_zero() * ride.speed;
            }
        }
    }
}
