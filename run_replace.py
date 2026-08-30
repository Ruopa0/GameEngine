import sys

with open('crates/cb_engine/src/editor/ui.rs', 'r', encoding='utf-8') as f:
    content = f.read()

search = '''                                        let mut current_roughness = 0.5;
                                        let mut current_metallic = 0.0;
                                        
                                        if let Some(ed_col) = self.world.get::<super::serialization::EditorColor>(entity) {
                                            let srgba = ed_col.0.to_srgba();
                                            current_color = [srgba.red, srgba.green, srgba.blue];
                                        } else if let Some(materials) = self.world.get_resource::<Assets<StandardMaterial>>() {
                                            if let Some(mat) = materials.get(&mat_handle) {
                                                let srgba = mat.base_color.to_srgba();
                                                current_color = [srgba.red, srgba.green, srgba.blue];
                                                current_roughness = mat.perceptual_roughness;
                                                current_metallic = mat.metallic;
                                            }
                                        }
                                        
                                        egui::Frame::group(ui.style()).show(ui, |ui| {
                                            ui.label(egui::RichText::new("🎨 Material & Color").strong());
                                            ui.separator();
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Base Color:");
                                                if ui.color_edit_button_rgb(&mut current_color).changed() {
                                                    color_changed = true;
                                                }
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Roughness: ");
                                                mat_changed |= ui.add(egui::Slider::new(&mut current_roughness, 0.0..=1.0)).changed();
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Metallic:  ");
                                                mat_changed |= ui.add(egui::Slider::new(&mut current_metallic, 0.0..=1.0)).changed();
                                            });
                                        });
                                        
                                        if color_changed {
                                            let new_col = super::serialization::EditorColor(Color::srgb(current_color[0], current_color[1], current_color[2]));
                                            self.world.entity_mut(entity).insert(new_col);
                                            // Optional: only sync on release
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_col);
                                            }
                                        }
                                        
                                        if mat_changed {
                                            if let Some(mut materials) = self.world.get_resource_mut::<Assets<StandardMaterial>>() {
                                                if let Some(mat) = materials.get_mut(&mat_handle) {
                                                    mat.perceptual_roughness = current_roughness;
                                                    mat.metallic = current_metallic;
                                                }
                                            }
                                        }'''

replace = '''                                        let mut current_roughness = 0.5;
                                        let mut current_metallic = 0.0;
                                        
                                        if let Some(ed_col) = self.world.get::<super::serialization::EditorColor>(entity) {
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
                                        }
                                        
                                        egui::Frame::group(ui.style()).show(ui, |ui| {
                                            ui.label(egui::RichText::new("🎨 Material & Color").strong());
                                            ui.separator();
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Base Color:");
                                                if ui.color_edit_button_rgb(&mut current_color).changed() {
                                                    color_changed = true;
                                                }
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Roughness: ");
                                                mat_changed |= ui.add(egui::Slider::new(&mut current_roughness, 0.0..=1.0)).changed();
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Metallic:  ");
                                                mat_changed |= ui.add(egui::Slider::new(&mut current_metallic, 0.0..=1.0)).changed();
                                            });
                                        });
                                        
                                        if color_changed {
                                            let new_col = super::serialization::EditorColor(Color::srgb(current_color[0], current_color[1], current_color[2]));
                                            self.world.entity_mut(entity).insert(new_col);
                                            // Optional: only sync on release
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_col);
                                            }
                                        }
                                        
                                        if mat_changed {
                                            let new_mat = super::serialization::EditorMaterial {
                                                roughness: current_roughness,
                                                metallic: current_metallic,
                                            };
                                            self.world.entity_mut(entity).insert(new_mat);
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_mat);
                                            }
                                        }'''
search = search.replace('\r\n', '\n')
content_normalized = content.replace('\r\n', '\n')

if search in content_normalized:
    print('Found search string')
    content = content_normalized.replace(search, replace)
    with open('crates/cb_engine/src/editor/ui.rs', 'w', encoding='utf-8') as f:
        f.write(content)
else:
    print('Search string not found')
