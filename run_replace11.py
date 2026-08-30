import sys
import re

with open('crates/cb_engine/src/editor/serialization.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace City Generator
city_pattern = re.compile(r'fn handle_generate_city\(.*?EditorColor\(Color::srgb\(0\.3, 0\.3, 0\.32\)\),\s*\)\);\s*}\s*}', re.DOTALL)
city_replacement = r'''fn handle_generate_city(
    mut events: MessageReader<GenerateCityEvent>,
    mut commands: Commands,
    mut active_state: ResMut<ActiveSceneState>,
    q_existing: Query<Entity, With<SceneObject>>,
) {
    let mut generated = false;
    for _ in events.read() {
        generated = true;
    }

    if generated {
        for entity in q_existing.iter() {
            commands.entity(entity).despawn();
        }

        let mut rng = fastrand::Rng::new();

        // Spawn a large grey concrete floor
        commands.spawn((
            Name::new("CityFloor"),
            SceneObject {
                object_type: "cube".to_string(),
                asset_path: None,
            },
            NetworkId(rand::random::<u64>()),
            Transform::from_xyz(0.0, -0.5, 0.0).with_scale(Vec3::new(300.0, 1.0, 300.0)),
            EditorColor(Color::srgb(0.3, 0.3, 0.32)),
        ));

        // City Layout Parameters
        let grid_size = 8;
        let block_width = 30.0;
        let road_width = 10.0;
        let offset = (block_width + road_width);
        let start_pos = -(grid_size as f32) * offset / 2.0;

        let mut building_index = 0;
        let mut chest_index = 0;

        for x in 0..grid_size {
            for z in 0..grid_size {
                let center_x = start_pos + (x as f32) * offset;
                let center_z = start_pos + (z as f32) * offset;

                if rng.f32() < 0.15 {
                    continue; 
                }

                let subdivisions = rng.u32(1..=4);
                
                for _ in 0..subdivisions {
                    let bx = center_x + (rng.f32() * 10.0 - 5.0);
                    let bz = center_z + (rng.f32() * 10.0 - 5.0);
                    
                    let b_width = rng.f32() * 10.0 + 8.0;
                    let b_depth = rng.f32() * 10.0 + 8.0;
                    let b_height = rng.f32() * 40.0 + 15.0;

                    let color = Color::srgb(
                        0.2 + rng.f32() * 0.5,
                        0.2 + rng.f32() * 0.5,
                        0.2 + rng.f32() * 0.5,
                    );

                    commands.spawn((
                        Name::new(format!("Building_{}", building_index)),
                        SceneObject {
                            object_type: "cube".to_string(),
                            asset_path: None,
                        },
                        NetworkId(rand::random::<u64>()),
                        Transform::from_xyz(bx, b_height / 2.0, bz).with_scale(Vec3::new(b_width, b_height, b_depth)),
                        EditorColor(color),
                    ));
                    building_index += 1;

                    if rng.f32() < 0.15 {
                        commands.spawn((
                            Name::new(format!("WeaponChest_{}", chest_index)),
                            SceneObject {
                                object_type: "weapon_chest".to_string(),
                                asset_path: None,
                            },
                            NetworkId(rand::random::<u64>()),
                            Transform::from_xyz(bx, b_height + 0.5, bz),
                        ));
                        chest_index += 1;
                    }
                }
            }
        }
    }
}'''
content = city_pattern.sub(city_replacement, content)

# Add weapon_chest to visualizer
light_pattern = re.compile(r'("light" => \{[\s\S]*?\}\s*),', re.DOTALL)
light_replacement = r'''\1
            "weapon_chest" => {
                let mesh = meshes.add(Cuboid::new(1.5, 1.0, 1.0));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.9, 0.7, 0.1),
                    emissive: LinearRgba::new(0.9, 0.7, 0.1, 1.0).into(),
                    ..default()
                });
                commands.entity(entity).insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    avian3d::prelude::RigidBody::Static,
                    avian3d::prelude::Collider::cuboid(1.5, 1.0, 1.0),
                    cb_weapons::components::Health::new(50.0),
                    crate::gamemode_chest::WeaponChest,
                ));
            },'''
content = light_pattern.sub(light_replacement, content)

with open('crates/cb_engine/src/editor/serialization.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
