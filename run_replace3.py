import sys

with open('crates/cb_weapons/src/ballistics.rs', 'r', encoding='utf-8') as f:
    content = f.read()

search = '''                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.08)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(5.0, 3.0, 0.0),
                        emissive: LinearRgba::new(5.0, 3.0, 0.0, 1.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(shot.origin),
                    ..default()
                },'''
replace = '''                Mesh3d(meshes.add(Sphere::new(0.08))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(5.0, 3.0, 0.0),
                    emissive: LinearRgba::new(5.0, 3.0, 0.0, 1.0),
                    ..default()
                })),
                Transform::from_translation(shot.origin),'''

content = content.replace(search, replace)

with open('crates/cb_weapons/src/ballistics.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
