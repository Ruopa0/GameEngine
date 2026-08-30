import sys
import re

with open('crates/cb_engine/src/editor/ui.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Regex to replace the block
pattern = re.compile(
    r'(if let Some\(ed_col\) = self\.world\.get::<super::serialization::EditorColor>\(entity\) \{.*?current_metallic = mat\.metallic;\s*\}\s*\})',
    re.DOTALL
)

replacement = '''if let Some(ed_col) = self.world.get::<super::serialization::EditorColor>(entity) {
                                            let srgba = ed_col.0.to_srgba();
                                            current_color = [srgba.red, srgba.green, srgba.blue];
                                        } else if let Some(materials) = self.world.get_resource::<Assets<StandardMaterial>>() {
                                            if let Some(mat) = materials.get(&mat_handle) {
                                                let srgba = mat.base_color.to_srgba();
                                                current_color = [srgba.red, srgba.green, srgba.blue];
                                            }
                                        }

                                        if let Some(ed_mat) = self.world.get::<super::serialization::EditorMaterial>(entity) {
                                            current_roughness = ed_mat.roughness;
                                            current_metallic = ed_mat.metallic;
                                        } else if let Some(materials) = self.world.get_resource::<Assets<StandardMaterial>>() {
                                            if let Some(mat) = materials.get(&mat_handle) {
                                                current_roughness = mat.perceptual_roughness;
                                                current_metallic = mat.metallic;
                                            }
                                        }'''

content = pattern.sub(replacement, content)

pattern2 = re.compile(
    r'(if mat_changed \{\s*if let Some\(mut materials\) = self\.world\.get_resource_mut::<Assets<StandardMaterial>>\(\) \{\s*if let Some\(mat\) = materials\.get_mut\(&mat_handle\) \{\s*mat\.perceptual_roughness = current_roughness;\s*mat\.metallic = current_metallic;\s*\}\s*\}\s*\})',
    re.DOTALL
)

replacement2 = '''if mat_changed {
                                            let new_mat = super::serialization::EditorMaterial {
                                                roughness: current_roughness,
                                                metallic: current_metallic,
                                            };
                                            self.world.entity_mut(entity).insert(new_mat);
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_mat);
                                            }
                                        }'''

content = pattern2.sub(replacement2, content)

with open('crates/cb_engine/src/editor/ui.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
