import sys
import re

with open('crates/cb_engine/src/editor/serialization.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Restore EditorMaterial
material_component = r'''
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct EditorMaterial {
    pub roughness: f32,
    pub metallic: f32,
}

pub fn apply_editor_material(
    q_mat: Query<(&EditorMaterial, &MeshMaterial3d<StandardMaterial>), Changed<EditorMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (ed_mat, handle) in q_mat.iter() {
        if let Some(mat) = materials.get_mut(handle.id()) {
            mat.roughness = ed_mat.roughness;
            mat.metallic = ed_mat.metallic;
        }
    }
}
'''
if "pub struct EditorMaterial" not in content:
    content += material_component

# Add to SerializationPlugin
if "register_type::<EditorMaterial>()" not in content:
    content = content.replace("register_type::<EditorColor>()", "register_type::<EditorColor>().register_type::<EditorMaterial>()")
    content = content.replace("apply_editor_color,", "apply_editor_color, apply_editor_material,")

# Add to restore_missing_visuals ("cube" match)
if "EditorMaterial" not in content.split('"cube" =>')[1].split("}")[0]:
    cube_replacement = r'''commands.entity(entity).insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    avian3d::prelude::RigidBody::Static,
                    avian3d::prelude::Collider::cuboid(0.5, 0.5, 0.5),
                    EditorMaterial { roughness: 0.5, metallic: 0.0 },
                ));'''
    content = re.sub(r'commands\.entity\(entity\)\.insert\(\(\s*Mesh3d\(mesh\),\s*MeshMaterial3d\(material\),\s*avian3d::prelude::RigidBody::Static,\s*avian3d::prelude::Collider::cuboid\(0\.5, 0\.5, 0\.5\),\s*\)\);', cube_replacement, content)

with open('crates/cb_engine/src/editor/serialization.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
