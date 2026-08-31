use bevy::prelude::*;

#[cfg(feature = "weapons")]
use crate::editor::serialization::EditorColor;
#[cfg(feature = "weapons")]
use cb_weapons::ballistics::HitVfxEvent;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "weapons")]
        {
            app.add_message::<HitVfxEvent>()
                .add_systems(Update, (spawn_hit_particles, update_particles));
        }
        #[cfg(not(feature = "weapons"))]
        {
            app.add_systems(Update, update_particles);
        }
    }
}

/// Settings for particle system limits
pub struct ParticleSettings {
    pub max_particles: u32,
    pub chest_particle_count: u32,
}

impl Default for ParticleSettings {
    fn default() -> Self {
        Self { max_particles: 200, chest_particle_count: 3 }
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub start_lifetime: f32,
}

#[cfg(feature = "weapons")]
fn spawn_hit_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: MessageReader<HitVfxEvent>,
    q_colors: Query<&EditorColor>,
) {
    let mut rng = fastrand::Rng::new();

    for ev in events.read() {
        // Try to get the color of the hit object
        let hit_color = q_colors
            .get(ev.hit_entity)
            .map(|c| c.0)
            .unwrap_or(Color::srgb(0.8, 0.8, 0.8));

        let material = materials.add(StandardMaterial {
            base_color: hit_color,
            emissive: (hit_color.to_linear() * 1.5),
            // Remove unlit as it might not look right, but wait, particles with unlit is good for sparks.
            unlit: true,
            ..default()
        });

        let mesh = meshes.add(Cuboid::new(0.035, 0.035, 0.035));

        // Spawn 4-6 particles (clean and tasteful)
        let num_particles = rng.u32(4..7);
        for _ in 0..num_particles {
            // Direction heavily biased along normal
            let random_dir = Vec3::new(
                rng.f32() * 2.0 - 1.0,
                rng.f32() * 2.0 - 1.0,
                rng.f32() * 2.0 - 1.0,
            )
            .normalize_or_zero();

            let dir = (ev.normal * 2.0 + random_dir).normalize_or_zero();
            let speed = rng.f32() * 3.5 + 1.5;

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(ev.point),
                Particle {
                    velocity: dir * speed,
                    lifetime: rng.f32() * 0.4 + 0.2, // 0.2 - 0.6 seconds
                    start_lifetime: 0.6,
                },
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += particle.velocity * dt;
        particle.velocity.y -= 9.81 * dt; // gravity

        let scale = (particle.lifetime / particle.start_lifetime).max(0.0);
        transform.scale = Vec3::splat(scale);
    }
}
