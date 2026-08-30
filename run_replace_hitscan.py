import sys
import re

with open('crates/cb_weapons/src/ballistics.rs', 'r', encoding='utf-8') as f:
    content = f.read()

pattern = re.compile(r'pub fn process_hitscan\(.*?\)\s*\{\s*for shot in shots\.read\(\) \{\s*// Find WeaponConfig:.*?\s*\}\s*\} else \{', re.DOTALL)

replacement = '''pub fn process_hitscan(
    mut commands: Commands,
    configs:    Query<&WeaponConfig>,
    mut shots:  MessageReader<ShotFiredEvent>,
    q_combatants: Query<(), With<crate::components::PlayerCombatant>>,
    mut damage: MessageWriter<DamageEvent>,
    mut vfx: MessageWriter<HitVfxEvent>,
    spatial:    SpatialQuery,
    time:       Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for shot in shots.read() {
        let config = match configs.get(shot.weapon) {
            Ok(c)  => c,
            Err(_) => continue,
        };

        // Deterministic spread seed: time + entity bits
        let seed = (time.elapsed().as_millis() as u64).wrapping_add(shot.shooter.to_bits());
        let dir = apply_spread(shot.direction, shot.spread_rad, seed);

        if let Some(speed) = config.projectile_speed {
            // Projectile weapon — spawn a physics entity that flies
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.08)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(5.0, 3.0, 0.0),
                        emissive: LinearRgba::new(5.0, 3.0, 0.0, 1.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(shot.origin),
                    ..default()
                },
                crate::components::Projectile {
                    velocity: dir * speed,
                    damage: config.damage,
                    penetration: config.penetration,
                    lifespan: config.range / speed,
                    owner: shot.shooter,
                },
            ));
        } else {'''

content = pattern.sub(replacement, content)

with open('crates/cb_weapons/src/ballistics.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
