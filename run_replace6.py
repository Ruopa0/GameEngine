import sys
import re

with open('crates/cb_engine/src/editor/serialization.rs', 'r', encoding='utf-8') as f:
    content = f.read()

pattern = re.compile(r'("light" => \{.*?\n\s*\})', re.DOTALL)

replacement = r'''\1
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
            }'''

content = pattern.sub(replacement, content)

with open('crates/cb_engine/src/editor/serialization.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
