/// Hitscan ballistics -- reads ShotFiredEvent, raycasts server-side, applies damage.
/// In multiplayer: server calls this with lag-compensated world snapshot positions.

use bevy::prelude::*;

use avian3d::prelude::*;
use crate::systems::ShotFiredEvent;
use crate::components::WeaponConfig;

/// Damage event -- consumed by the game rules / health system
#[derive(Message, Clone, Debug)]
pub struct DamageEvent {
    pub target:  Entity,
    pub amount:  f32,
    pub point:   Vec3,
    pub normal:  Vec3,
}

#[derive(Message, Clone)]
pub struct HitVfxEvent {
    pub point: Vec3,
    pub normal: Vec3,
    pub hit_entity: Entity,
}

pub fn process_hitscan(
    mut commands: Commands,
    configs:    Query<&WeaponConfig>,
    mut shots:  MessageReader<ShotFiredEvent>,
    q_health:   Query<&crate::components::Health>,
    q_child_of: Query<&ChildOf>,
    mut damage: MessageWriter<DamageEvent>,
    mut vfx: MessageWriter<HitVfxEvent>,
    spatial:    SpatialQuery,
    time:       Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use bevy::ecs::relationship::Relationship;
    for shot in shots.read() {
        let config = match configs.get(shot.weapon) {
            Ok(c)  => c,
            Err(_) => continue,
        };

        // Deterministic spread seed: time + entity bits
        let seed = (time.elapsed().as_millis() as u64).wrapping_add(shot.shooter.to_bits());
        let dir = apply_spread(shot.direction, shot.spread_rad, seed);

        if let Some(speed) = config.projectile_speed {
            // Projectile weapon -- spawn a physics entity that flies
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.08))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(5.0, 3.0, 0.0),
                    emissive: LinearRgba::new(5.0, 3.0, 0.0, 1.0),
                    ..default()
                })),
                Transform::from_translation(shot.origin),
                crate::components::Projectile {
                    velocity: dir * speed,
                    damage: config.damage,
                    penetration: config.penetration,
                    lifespan: config.range / speed,
                    owner: shot.shooter,
                    is_local: shot.is_local,
                },
            ));
        } else {
            // Hitscan weapon -- immediate raycast
            if let Some(hit) = spatial.cast_ray(
                shot.origin,
                Dir3::new(dir).unwrap_or(Dir3::NEG_Z),
                config.range,
                true,
                &SpatialQueryFilter::from_excluded_entities([shot.shooter]),
            ) {
                let hit_point = shot.origin + dir * hit.distance;

                // Wall penetration: check if we should continue through thin geometry
                let final_damage = config.damage * penetration_factor(hit.distance, config.penetration);

                vfx.write(HitVfxEvent {
                    point: hit_point,
                    normal: hit.normal,
                    hit_entity: hit.entity,
                });

                let mut target_entity = hit.entity;
                if !q_health.contains(target_entity) {
                    if let Ok(child_of) = q_child_of.get(target_entity) {
                        if q_health.contains(child_of.get()) {
                            target_entity = child_of.get();
                        }
                    }
                }

                if shot.is_local && target_entity != shot.shooter && q_health.contains(target_entity) {
                    damage.write(DamageEvent {
                        target: target_entity,
                        amount: final_damage,
                        point:  hit_point,
                        normal: hit.normal,
                    });
                }
            }
        }
    }
}

pub fn process_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut crate::components::Projectile)>,
    q_health: Query<&crate::components::Health>,
    q_child_of: Query<&ChildOf>,
    mut damage: MessageWriter<DamageEvent>,
    mut vfx: MessageWriter<HitVfxEvent>,
    spatial: SpatialQuery,
    time: Res<Time>,
) {
    use bevy::ecs::relationship::Relationship;
    let dt = time.delta_secs();
    for (entity, mut transform, mut proj) in query.iter_mut() {
        proj.lifespan -= dt;
        if proj.lifespan <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let origin = transform.translation;
        let dir = proj.velocity * dt;
        let distance = dir.length();

        if let Some(hit) = spatial.cast_ray(
            origin,
            Dir3::new(dir).unwrap_or(Dir3::NEG_Z),
            distance,
            true,
            &SpatialQueryFilter::from_excluded_entities([proj.owner, entity]),
        ) {
            let hit_point = origin + dir.normalize_or_zero() * hit.distance;
            
            vfx.write(HitVfxEvent {
                point: hit_point,
                normal: hit.normal,
                hit_entity: hit.entity,
            });

            let mut target_entity = hit.entity;
            if !q_health.contains(target_entity) {
                if let Ok(child_of) = q_child_of.get(target_entity) {
                    if q_health.contains(child_of.get()) {
                        target_entity = child_of.get();
                    }
                }
            }

            if proj.is_local && target_entity != proj.owner && q_health.contains(target_entity) {
                damage.write(DamageEvent {
                    target: target_entity,
                    amount: proj.damage,
                    point:  hit_point,
                    normal: hit.normal,
                });
            }

            commands.entity(entity).despawn();
        } else {
            transform.translation += dir;
            // Basic gravity arc for projectiles
            proj.velocity.y -= 9.81 * dt; 
        }
    }
}

fn apply_spread(dir: Vec3, spread_rad: f32, seed: u64) -> Vec3 {
    use std::f32::consts::TAU;
    let t1 = hash_f32(seed);
    let t2 = hash_f32(seed.wrapping_add(1));
    
    let theta = t1 * TAU;
    let phi   = t2 * spread_rad;
    let offset = Vec3::new(phi * theta.cos(), phi * theta.sin(), 0.0);
    (dir + offset).normalize()
}

fn penetration_factor(distance: f32, factor: f32) -> f32 {
    (1.0 - factor * (distance / 5.0).min(1.0)).max(0.0)
}

/// Very simple hash function for deterministic spread 
fn hash_f32(mut state: u64) -> f32 {
    state ^= state >> 30;
    state = state.wrapping_mul(0xbf58476d1ce4e5b9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94d049bb133111eb);
    state ^= state >> 31;
    // Map to 0.0 .. 1.0
    (state as u32 as f32) / (u32::MAX as f32)
}




