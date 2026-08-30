use bevy::prelude::*;
use bevy_egui::{egui, EguiPrimaryContextPass};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::{DockArea, DockState, NodeIndex};
use bevy_inspector_egui::bevy_inspector;

use super::console::ConsoleState;
use bevy::ecs::relationship::Relationship;

// ---------------------------------------------------------------------------------
// EDITOR USER INTERFACE (EGUI)
// This file handles all the visual UI panels for the Code Blue Editor.
// It leverages `bevy_egui` and `egui_dock` to create a robust, dockable window layout
// similar to Unity or Unreal Engine.
//
// Key Responsibilities:
// 1. Defining the default panel layout (Hierarchy, Inspector, Viewport, Assets, etc.).
// 2. Rendering the interactive Asset Browser (polling `.gltf` files from disk).
// 3. Providing drag-and-drop mechanics from the UI into the 3D Viewport.
// 4. Overriding the default Transform components with a cleaner, compact UI.
// ---------------------------------------------------------------------------------

/// The main Bevy plugin that sets up the Editor UI.
pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        // We use a "DockState" tree to define the initial layout of the panels.
        let mut tree = DockState::new(vec!["Viewport".to_string(), "Game View".to_string()]);
        
        let surface = tree.main_surface_mut();
        // Split the screen to put the Inspector on the right (25% width)
        let [vp, _inspector] = surface.split_right(NodeIndex::root(), 0.75, vec!["Inspector".to_string()]);
        // Split the remaining left side to put the Hierarchy on the left (20% width)
        let [vp, _hierarchy] = surface.split_left(vp, 0.2, vec!["Hierarchy".to_string()]);
        // Split the bottom to put the Console/Assets at the bottom (30% height)
        let [_vp, _console] = surface.split_below(vp, 0.7, vec!["Console".to_string(), "Assets".to_string()]);

        app.insert_resource(EditorUiState { tree, gltf_input: String::new() })
           .init_resource::<EditorViewportState>()
           .init_resource::<EditorUiDialogs>()
           .init_resource::<AssetBrowserState>()
           .init_resource::<InspectorState>()
           .add_systems(Startup, setup_ui_camera)
           .add_systems(OnEnter(super::EngineState::Play), focus_game_view_on_play)
           .add_systems(OnEnter(super::EngineState::Edit), focus_viewport_on_edit)
           // Render the UI in a specific pass before other things update
           .add_systems(EguiPrimaryContextPass, render_editor_ui);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub enum InspectorMode {
    Simple,
    Developer,
}

#[derive(Resource)]
pub struct InspectorState {
    pub mode: InspectorMode,
    pub component_search: String,
    pub show_add_component_modal: bool,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            mode: InspectorMode::Simple,
            component_search: String::new(),
            show_add_component_modal: false,
        }
    }
}

pub struct ComponentMeta {
    pub type_path: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
}

pub const COMPONENT_CATALOG: &[ComponentMeta] = &[
    // Physics (Avian 3D)
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::RigidBody",
        name: "Physical Body",
        icon: "[RigidBody]",
        category: "Physics & Motion",
        description: "Enables realistic physics, gravity, and collisions (Dynamic, Static, or Kinematic).",
        keywords: &["physics", "rigid", "body", "dynamic", "static", "collision", "gravity"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::GravityScale",
        name: "Gravity Multiplier",
        icon: "    ",
        category: "Physics & Motion",
        description: "Controls how strongly gravity pulls this object down (or floats up).",
        keywords: &["gravity", "fall", "float", "weight", "pull"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::material::Restitution",
        name: "Bounciness (Restitution)",
        icon: "    ",
        category: "Physics & Motion",
        description: "Controls elasticity and how high the object bounces when hitting surfaces.",
        keywords: &["bounce", "elastic", "restitution", "rebound", "jump"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::material::Friction",
        name: "Surface Friction",
        icon: "[Friction]",
        category: "Physics & Motion",
        description: "Controls grip and sliding resistance when contacting other objects.",
        keywords: &["friction", "slide", "grip", "rough", "ice", "skate"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::Mass",
        name: "Mass & Weight",
        icon: "      ",
        category: "Physics & Motion",
        description: "Sets the physical mass and resistance to acceleration.",
        keywords: &["mass", "weight", "heavy", "light", "kg"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::LinearVelocity",
        name: "Movement Velocity",
        icon: "[Velocity]",
        category: "Physics & Motion",
        description: "Initial movement speed and direction vector in 3D space.",
        keywords: &["velocity", "speed", "movement", "direction", "impulse"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::AngularVelocity",
        name: "Spin Velocity",
        icon: "[Angular]",
        category: "Physics & Motion",
        description: "Rotational spin speed around X, Y, and Z axes.",
        keywords: &["spin", "rotation", "angular", "turn", "roll"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::LinearDamping",
        name: "Air Drag (Movement)",
        icon: "[Damping]",
        category: "Physics & Motion",
        description: "Slows down translational movement over time (air resistance).",
        keywords: &["drag", "damping", "air", "friction", "slow"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::AngularDamping",
        name: "Spin Drag (Rotation)",
        icon: "[Spin]",
        category: "Physics & Motion",
        description: "Slows down rotational spinning over time.",
        keywords: &["drag", "spin", "damping", "rotation"],
    },
    ComponentMeta {
        type_path: "avian3d::dynamics::rigid_body::LockedAxes",
        name: "Constrain Movement",
        icon: "[Lock]",
        category: "Physics & Motion",
        description: "Locks rotation or position along specific axes (e.g. 2D platformers).",
        keywords: &["lock", "constrain", "freeze", "axis", "axes", "2d"],
    },
    // Visuals & Lights
    ComponentMeta {
        type_path: "cb_engine::editor::serialization::EditorPointLight",
        name: "Point Light Source",
        icon: "[Light]",
        category: "Visuals & Lights",
        description: "Emits light in all directions with custom color, brightness, range, and shadows.",
        keywords: &["light", "lamp", "glow", "bulb", "illumination", "shadow", "bright"],
    },
    // Scripting & Logic
    ComponentMeta {
        type_path: "cb_engine::scripting::ScriptComponent",
        name: "Behavior Script",
        icon: "[Scene]",
        category: "[Scene] Scripting & Logic",
        description: "Attaches a Rhai script to execute custom game logic and behaviors.",
        keywords: &["script", "code", "rhai", "behavior", "logic", "event", "program"],
    },
    // General
    ComponentMeta {
        type_path: "bevy_core::name::Name",
        name: "Entity Name",
        icon: "       ",
        category: "        General & Identity",
        description: "Human-readable label shown in the Scene Hierarchy.",
        keywords: &["name", "label", "tag", "title", "identity"],
    },
    ComponentMeta {
        type_path: "bevy_transform::components::transform::Transform",
        name: "Transform (Position & Size)",
        icon: "    ",
        category: "        General & Identity",
        description: "3D world position, rotation angles, and scale multiplier.",
        keywords: &["transform", "position", "rotation", "scale", "location", "place"],
    },
    ComponentMeta {
        type_path: "cb_engine::editor::serialization::TestComponent",
        name: "Test Component",
        icon: "[Script]",
        category: "        General & Identity",
        description: "Custom test component for experimentation.",
        keywords: &["test", "debug", "dummy"],
    },
    ComponentMeta {
        type_path: "cb_engine::editor::serialization::EditorColor",
        name: "Material Color",
        icon: "[Color]",
        category: "Visuals & Lights",
        description: "Sets the solid base color of the 3D model material.",
        keywords: &["color", "material", "paint", "texture"],
    },
    ComponentMeta {
        type_path: "cb_engine::editor::serialization::EditorMaterial",
        name: "Material Properties",
        icon: "[Mat]",
        category: "Visuals & Lights",
        description: "Adjust the roughness (matte/glossy) and metallic surface values.",
        keywords: &["material", "roughness", "metallic", "gloss", "shine"],
    },
    ComponentMeta {
        type_path: "cb_weapons::components::Health",
        name: "Health Points",
        icon: "[HP]",
        category: "Gameplay & Combat",
        description: "Gives this entity a life pool, allowing it to take damage.",
        keywords: &["health", "hp", "life", "damage", "destructible"],
    },
    ComponentMeta {
        type_path: "crate::gamemode::TargetDummy",
        name: "Target Dummy Logic",
        icon: "[Dummy]",
        category: "Gameplay & Combat",
        description: "Registers this object as a target dummy for hit-markers and points.",
        keywords: &["target", "dummy", "aim", "shoot"],
    },
    ComponentMeta {
        type_path: "crate::gamemode::GoalZone",
        name: "Goal Zone (Win Area)",
        icon: "[Goal]",
        category: "Gameplay & Combat",
        description: "A trigger volume that players can enter to score points or win.",
        keywords: &["goal", "zone", "win", "area", "trigger"],
    },
    ComponentMeta {
        type_path: "cb_weapons::health::ImmortalPlayer",
        name: "God Mode",
        icon: "[God]",
        category: "Gameplay & Combat",
        description: "Prevents this entity from ever dying or taking damage.",
        keywords: &["god", "immortal", "invincible"],
    },
];

pub fn open_in_vscode(path: &str) {
    #[cfg(target_os = "windows")]
    {
        if std::process::Command::new("cmd")
            .args(["/C", "code", path])
            .spawn()
            .is_err()
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "", path])
                .spawn();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if std::process::Command::new("code")
            .arg(path)
            .spawn()
            .is_err()
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    }
}

#[derive(Resource)]
pub struct AssetBrowserState {
    pub current_path: std::path::PathBuf,
    pub files: Vec<std::path::PathBuf>,
    pub last_refresh: f64,
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self {
            current_path: std::path::PathBuf::from("assets"),
            files: Vec::new(),
            last_refresh: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct EditorViewportState {
    pub is_hovered: bool,
    pub normalized_mouse_pos: Vec2,
    pub viewport_size: Vec2,
    pub game_view_size: Vec2,
}

#[derive(Resource)]
pub struct EditorUiDialogs {
    pub show_save_before_play: bool,
    pub show_join_play: bool,
    pub show_help_window: bool,
    pub show_about_dialog: bool,
    pub show_save_as_dialog: bool,
    pub show_open_dialog: bool,
    pub show_new_scene_dialog: bool,
    pub modal_file_input: String,
    pub filter_scene_types_only: bool,
}

impl Default for EditorUiDialogs {
    fn default() -> Self {
        Self {
            show_save_before_play: false,
            show_join_play: false,
            show_help_window: false,
            show_about_dialog: false,
            show_save_as_dialog: false,
            show_open_dialog: false,
            show_new_scene_dialog: false,
            modal_file_input: "level.ron".to_string(),
            filter_scene_types_only: true,
        }
    }
}

fn open_file_dialog() -> Option<String> {
    let dialog = rfd::FileDialog::new()
        .set_directory("assets")
        .add_filter("Code Blue Scene (*.ron, *.scn.ron, *.scn)", &["ron", "scn.ron", "scn"])
        .add_filter("All Files (*.*)", &["*"]);
    
    if let Some(path) = dialog.pick_file() {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = path.strip_prefix(&current_dir) {
                return Some(relative.to_string_lossy().replace("\\", "/"));
            }
        }
        return Some(path.to_string_lossy().replace("\\", "/"));
    }
    None
}

fn save_file_dialog(default_name: &str) -> Option<String> {
    let dialog = rfd::FileDialog::new()
        .set_directory("assets")
        .set_file_name(default_name)
        .add_filter("Code Blue Scene (*.ron)", &["ron"])
        .add_filter("All Files (*.*)", &["*"]);
    
    if let Some(path) = dialog.save_file() {
        let mut path_str = path.to_string_lossy().replace("\\", "/");
        if !path_str.ends_with(".ron") && !path_str.ends_with(".scn") {
            path_str.push_str(".ron");
        }
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = path.strip_prefix(&current_dir) {
                return Some(relative.to_string_lossy().replace("\\", "/"));
            }
        }
        return Some(path_str);
    }
    None
}

#[derive(Resource)]
pub struct EditorUiState {
    pub tree: DockState<String>,
    pub gltf_input: String,
}

#[derive(Component)]
pub struct UiCamera;

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            ..default()
        },
        UiCamera,
    ));
}

fn sync_component_to_network<T: Component + Reflect>(
    world: &mut World,
    entity: Entity,
    component: T,
) {
    world.entity_mut(entity).insert(component);
    let net_id_opt = world.get::<super::serialization::NetworkId>(entity).copied();
    if let Some(net_id) = net_id_opt {
        let registry_arc = world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = registry_arc.read();
        let mut update_to_send = None;
        if let Ok(entity_ref) = world.get_entity(entity) {
            for registration in type_registry.iter() {
                if registration.type_id() == std::any::TypeId::of::<T>() {
                    if let Some(reflect_comp) = registration.data::<bevy::ecs::reflect::ReflectComponent>() {
                        if let Some(reflected) = reflect_comp.reflect(entity_ref) {
                            if let Some(_reflect_ser) = registration.data::<bevy::reflect::ReflectSerialize>() {
                                let serializer = bevy::reflect::serde::TypedReflectSerializer::new(reflected, &type_registry);
                                if let Ok(ron_str) = ron::to_string(&serializer) {
                                    update_to_send = Some((registration.type_info().type_path().to_string(), ron_str));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(type_registry);
        if let Some((type_path, ron_data)) = update_to_send {
            world.write_message(super::serialization::EditorActionRequest::UpdateComponent {
                id: net_id.0,
                type_path,
                ron_data,
            });
        }
    }
}

fn sync_all_entity_components(
    world: &mut World,
    entity: Entity,
) {
    let net_id_opt = world.get::<super::serialization::NetworkId>(entity).copied();
    if let Some(net_id) = net_id_opt {
        let registry_arc = world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = registry_arc.read();
        let mut updates = Vec::new();
        if let Ok(entity_ref) = world.get_entity(entity) {
            for registration in type_registry.iter() {
                if let Some(reflect_comp) = registration.data::<bevy::ecs::reflect::ReflectComponent>() {
                    if let Some(reflected) = reflect_comp.reflect(entity_ref) {
                        if let Some(_reflect_ser) = registration.data::<bevy::reflect::ReflectSerialize>() {
                            let serializer = bevy::reflect::serde::ReflectSerializer::new(reflected, &type_registry);
                            if let Ok(ron_str) = ron::to_string(&serializer) {
                                updates.push((registration.type_info().type_path().to_string(), ron_str));
                            }
                        }
                    }
                }
            }
        }
        drop(type_registry);
        for (type_path, ron_data) in updates {
            world.write_message(super::serialization::EditorActionRequest::UpdateComponent {
                id: net_id.0,
                type_path,
                ron_data,
            });
        }
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    gltf_input: &'a mut String,
    inspector_state: &'a mut InspectorState,
    _dialogs: &'a mut EditorUiDialogs,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.clone().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Hierarchy" => {
                let mut new_selection = None;
                let mut reparent_cmds = Vec::new();
                
                let mut flat_nodes = Vec::new();
                {
                    let session_client_id = self.world.resource::<super::serialization::LocalEditorSession>().client_id;
                    let mut query = self.world.query_filtered::<(Entity, Option<&Name>, Option<&super::serialization::SceneObject>, Option<&super::serialization::NetworkId>, Option<&super::serialization::EditorLock>, Option<&ChildOf>, Has<crate::editor::picking::Selected>), With<super::serialization::SceneObject>>();
                    for (e, name, obj, net_id, lock_opt, parent, is_selected) in query.iter(self.world) {
                        let type_name = obj.map(|o| o.object_type.as_str()).unwrap_or("Object");
                        let net_tag = if let Some(nid) = net_id {
                            format!(" [     Net #{}]", nid.0 % 10000)
                        } else {
                            String::new()
                        };
                        let lock_tag = if let Some(lock) = lock_opt {
                            if lock.user_id == session_client_id {
                                " [Locked: You]".to_string()
                            } else {
                                format!(" [Locked: #{:04}]", lock.user_id % 10000)
                            }
                        } else {
                            String::new()
                        };
                        let label = if let Some(n) = name {
                            format!("{} - {}{}{}", type_name, n.as_str(), net_tag, lock_tag)
                        } else {
                            format!("{} ({:?}){}{}", type_name, e, net_tag, lock_tag)
                        };
                        flat_nodes.push((e, parent.map(|p| p.get()), label, is_selected));
                    }
                }

                // Drop zone for root (remove parent)
                let root_rect = ui.available_rect_before_wrap();
                let root_response = ui.interact(root_rect, ui.id().with("root_drop"), egui::Sense::hover());
                
                if let Some(dragged_entity) = root_response.dnd_release_payload::<Entity>() {
                    reparent_cmds.push((*dragged_entity, None));
                }

                #[allow(clippy::too_many_arguments)]
                fn draw_node(
                    ui: &mut egui::Ui,
                    entity: Entity,
                    label: &str,
                    is_selected: bool,
                    flat_nodes: &[(Entity, Option<Entity>, String, bool)],
                    new_selection: &mut Option<Entity>,
                    reparent_cmds: &mut Vec<(Entity, Option<Entity>)>,
                    script_cmds: &mut Vec<(Entity, String)>
                ) {
                    let children: Vec<_> = flat_nodes.iter().filter(|(_, p, _, _)| *p == Some(entity)).collect();
                    
                    let id = ui.make_persistent_id(entity);
                    let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(id).unwrap_or(true));
                    
                    ui.horizontal(|ui| {
                        if children.is_empty() {
                            ui.add_space(15.0);
                        } else {
                            if ui.button(if is_open { "v" } else { ">" }).clicked() {
                                is_open = !is_open;
                            }
                        }
                        
                        let label_ui = ui.selectable_label(is_selected, label);
                        
                        // Drag source
                        label_ui.dnd_set_drag_payload(entity);
                        

                        // Drop target for Entity Reparenting
                        if label_ui.dnd_hover_payload::<Entity>().is_some() || label_ui.dnd_hover_payload::<String>().is_some() {
                            ui.painter().rect(label_ui.rect, 0.0, egui::Color32::TRANSPARENT, (1.0, egui::Color32::YELLOW), egui::StrokeKind::Middle);
                        }
                        
                        if let Some(dragged) = label_ui.dnd_release_payload::<Entity>() {

                            if *dragged != entity {
                                reparent_cmds.push((*dragged, Some(entity)));
                            }
                        }
                        
                        // Drop target for Scripts
                        if let Some(script_path) = label_ui.dnd_release_payload::<String>() {
                            if script_path.ends_with(".rhai") {
                                script_cmds.push((entity, (*script_path).clone()));
                            }
                        }
                        
                        if label_ui.clicked() {
                            *new_selection = Some(entity);
                        }
                    });
                    
                    ui.data_mut(|d| d.insert_temp(id, is_open));
                    
                    if is_open && !children.is_empty() {
                        ui.indent(id, |ui| {
                            for (child, _, child_label, child_selected) in children {
                                draw_node(ui, *child, child_label, *child_selected, flat_nodes, new_selection, reparent_cmds, script_cmds);
                            }
                        });
                    }
                }

                // Draw roots
                let mut script_cmds = Vec::new();
                if flat_nodes.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.heading("Empty Scene");
                        ui.label("Your scene is completely empty.");
                        ui.label("Use the Prefabs or Assets panel below to spawn objects.");
                    });
                } else {
                    for (entity, parent, label, is_selected) in &flat_nodes {
                        if parent.is_none() {
                            draw_node(ui, *entity, label, *is_selected, &flat_nodes, &mut new_selection, &mut reparent_cmds, &mut script_cmds);
                        }
                    }
                }

                // Apply selections and update selection lock
                if let Some(selected_entity) = new_selection {
                    let session_client_id = self.world.resource::<super::serialization::LocalEditorSession>().client_id;
                    let mut selected_query = self.world.query_filtered::<(Entity, Option<&super::serialization::NetworkId>), With<crate::editor::picking::Selected>>();
                    let currently_selected: Vec<(Entity, Option<u64>)> = selected_query.iter(self.world).map(|(e, nid)| (e, nid.map(|n| n.0))).collect();
                    
                    for (e, prev_nid) in currently_selected {
                        self.world.entity_mut(e).remove::<crate::editor::picking::Selected>();
                        if let Some(id) = prev_nid {
                            self.world.write_message(super::serialization::EditorActionRequest::UnlockObject {
                                id,
                                user_id: session_client_id,
                            });
                        }
                    }
                    self.world.entity_mut(selected_entity).insert(crate::editor::picking::Selected);
                    if let Some(nid) = self.world.get::<super::serialization::NetworkId>(selected_entity).copied() {
                        self.world.write_message(super::serialization::EditorActionRequest::LockObject {
                            id: nid.0,
                            user_id: session_client_id,
                        });
                    }
                }

                // Apply reparents
                for (child, parent_opt) in reparent_cmds {
                    let child_net_id = self.world.get::<super::serialization::NetworkId>(child).map(|n| n.0);
                    let parent_net_id = parent_opt.and_then(|p| self.world.get::<super::serialization::NetworkId>(p).map(|n| n.0));
                    if let Some(parent) = parent_opt {
                        if child != parent {
                            self.world.entity_mut(child).set_parent_in_place(parent);
                        }
                    } else {
                        self.world.entity_mut(child).remove_parent_in_place();
                    }
                    if let Some(cid) = child_net_id {
                        self.world.write_message(super::serialization::EditorActionRequest::ReparentObject {
                            id: cid,
                            parent_id: parent_net_id,
                        });
                    }
                }
                for (entity, script_path) in script_cmds {
                    let script_comp = crate::scripting::ScriptComponent {
                        path: script_path,
                    };
                    sync_component_to_network(self.world, entity, script_comp);
                }
            }
            "Inspector" => {
                let mut selected_entity = None;
                {
                    let mut q = self.world.query_filtered::<Entity, With<crate::editor::picking::Selected>>();
                    if let Ok(entity) = q.single(self.world) {
                        selected_entity = Some(entity);
                    }
                }

                if let Some(entity) = selected_entity {
                    let session_client_id = self.world.resource::<super::serialization::LocalEditorSession>().client_id;
                    let is_locked_by_other = if let Some(lock) = self.world.get::<super::serialization::EditorLock>(entity) {
                        lock.user_id != session_client_id
                    } else {
                        false
                    };

                    if is_locked_by_other {
                        let lock_user_id = self.world.get::<super::serialization::EditorLock>(entity).map(|l| l.user_id).unwrap_or(0);
                        let lock_col = super::user_color::get_user_color_egui(lock_user_id);
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.colored_label(lock_col, format!("[Locked] User #{:04}", lock_user_id % 10000));
                            ui.label("Editing this object is currently disabled to prevent conflicts.");
                        });
                    } else {
                        // Mode Switch Header
                        let mut current_mode = self.inspector_state.mode.clone();
                        ui.horizontal(|ui| {
                            if ui.selectable_label(current_mode == InspectorMode::Simple, "Simple Mode").clicked() {
                                self.inspector_state.mode = InspectorMode::Simple;
                                current_mode = InspectorMode::Simple;
                            }
                            if ui.selectable_label(current_mode == InspectorMode::Developer, "        Developer Mode").clicked() {
                                self.inspector_state.mode = InspectorMode::Developer;
                                current_mode = InspectorMode::Developer;
                            }
                        });
                        ui.separator();

                        match current_mode {
                            InspectorMode::Developer => {
                                bevy_inspector::ui_for_entity(self.world, entity, ui);
                                
                                let released = ui.input(|i| i.pointer.any_released() || i.key_pressed(egui::Key::Enter));

                                if released {
                                    sync_all_entity_components(self.world, entity);
                                }
                            }
                            InspectorMode::Simple => {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    // 1. Identity & Name Card
                                    let mut name_changed = false;
                                    let mut current_name = self.world.get::<Name>(entity).map(|n| n.as_str().to_string()).unwrap_or_default();
                                    let net_id_opt = self.world.get::<super::serialization::NetworkId>(entity).copied();
                                    egui::Frame::group(ui.style()).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("        Entity Name:").strong());
                                            if ui.text_edit_singleline(&mut current_name).changed() {
                                                name_changed = true;
                                            }
                                        });
                                        if let Some(net_id) = net_id_opt {
                                            ui.label(egui::RichText::new(format!("Network ID: #{:06}", net_id.0 % 1000000)).color(egui::Color32::GRAY).size(11.0));
                                        }
                                    });
                                    if name_changed {
                                        self.world.entity_mut(entity).insert(Name::new(current_name.clone()));
                                        if let Some(net_id) = net_id_opt {
                                            self.world.write_message(super::serialization::EditorActionRequest::RenameObject {
                                                id: net_id.0,
                                                name: current_name,
                                            });
                                        }
                                    }
                                    ui.add_space(6.0);

                                    // 2. Transform Card
                                    let mut tf_changed = false;
                                    let mut current_tf = self.world.get::<Transform>(entity).copied().unwrap_or_default();
                                    egui::Frame::group(ui.style()).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("     Transform (Position & Size)").strong());
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("Reset").on_hover_text("Reset position, rotation, and scale").clicked() {
                                                    current_tf = Transform::default();
                                                    tf_changed = true;
                                                }
                                            });
                                        });
                                        ui.separator();
                                        
                                        // Position
                                        ui.horizontal(|ui| {
                                            ui.label("Position: ");
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.translation.x).prefix("X: ").speed(0.1)).changed();
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.translation.y).prefix("Y: ").speed(0.1)).changed();
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.translation.z).prefix("Z: ").speed(0.1)).changed();
                                        });

                                        // Rotation (Euler angles in degrees)
                                        let (roll, pitch, yaw) = current_tf.rotation.to_euler(EulerRot::XYZ);
                                        let mut deg_roll = roll.to_degrees();
                                        let mut deg_pitch = pitch.to_degrees();
                                        let mut deg_yaw = yaw.to_degrees();
                                        let mut rot_changed = false;
                                        ui.horizontal(|ui| {
                                            ui.label("Rotation: ");
                                            rot_changed |= ui.add(egui::DragValue::new(&mut deg_roll).prefix("X: ").suffix(" deg").speed(1.0)).changed();
                                            rot_changed |= ui.add(egui::DragValue::new(&mut deg_pitch).prefix("Y: ").suffix(" deg").speed(1.0)).changed();
                                            rot_changed |= ui.add(egui::DragValue::new(&mut deg_yaw).prefix("Z: ").suffix(" deg").speed(1.0)).changed();
                                        });
                                        if rot_changed {
                                            current_tf.rotation = Quat::from_euler(EulerRot::XYZ, deg_roll.to_radians(), deg_pitch.to_radians(), deg_yaw.to_radians());
                                            tf_changed = true;
                                        }

                                        // Scale
                                        ui.horizontal(|ui| {
                                            ui.label("Scale:     ");
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.scale.x).prefix("X: ").speed(0.05)).changed();
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.scale.y).prefix("Y: ").speed(0.05)).changed();
                                            tf_changed |= ui.add(egui::DragValue::new(&mut current_tf.scale.z).prefix("Z: ").speed(0.05)).changed();
                                        });
                                    });
                                    if tf_changed {
                                        if let Some(mut tf) = self.world.get_mut::<Transform>(entity) {
                                            *tf = current_tf;
                                        }
                                        if let Some(net_id) = net_id_opt {
                                            let session_client_id = self.world.resource::<super::serialization::LocalEditorSession>().client_id;
                                            self.world.write_message(super::serialization::EditorActionRequest::MoveObject {
                                                id: net_id.0,
                                                transform: current_tf,
                                                sender_user_id: session_client_id,
                                            });
                                        }
                                    }
                                    ui.add_space(6.0);

                                    // 3. Physics (Avian 3D) Card
                                    let mut remove_physics = false;
                                    let has_physics = self.world.get::<avian3d::prelude::RigidBody>(entity).is_some();
                                    if has_physics {
                                        egui::Frame::group(ui.style()).show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Physics Body (Avian3D)").strong());
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("        Remove").clicked() {
                                                        remove_physics = true;
                                                    }
                                                });
                                            });
                                            ui.separator();

                                            // Body Type
                                            let mut current_rb = self.world.get::<avian3d::prelude::RigidBody>(entity).copied().unwrap_or(avian3d::prelude::RigidBody::Dynamic);
                                            ui.horizontal(|ui| {
                                                ui.label("Body Type:");
                                                let mut rb_changed = false;
                                                rb_changed |= ui.selectable_value(&mut current_rb, avian3d::prelude::RigidBody::Dynamic, "Dynamic").changed();
                                                rb_changed |= ui.selectable_value(&mut current_rb, avian3d::prelude::RigidBody::Static, "Static").changed();
                                                rb_changed |= ui.selectable_value(&mut current_rb, avian3d::prelude::RigidBody::Kinematic, "Kinematic").changed();
                                                if rb_changed {
                                                    sync_component_to_network(self.world, entity, current_rb);
                                                }
                                            });

                                            // Bounciness (Restitution)
                                            let mut restitution = self.world.get::<avian3d::prelude::Restitution>(entity).map(|r| r.coefficient).unwrap_or(0.0);
                                            ui.horizontal(|ui| {
                                                ui.label("     Bounciness:");
                                                if ui.add(egui::Slider::new(&mut restitution, 0.0..=1.0).text("Rebound")).changed() {
                                                    sync_component_to_network(self.world, entity, avian3d::prelude::Restitution::new(restitution));
                                                }
                                            });

                                            // Surface Friction
                                            let mut friction = self.world.get::<avian3d::prelude::Friction>(entity).map(|f| f.static_coefficient).unwrap_or(1.0);
                                            ui.horizontal(|ui| {
                                                ui.label("Surface Friction:");
                                                if ui.add(egui::Slider::new(&mut friction, 0.0..=5.0).text("Grip")).changed() {
                                                    sync_component_to_network(self.world, entity, avian3d::prelude::Friction::new(friction));
                                                }
                                            });

                                            // Gravity Multiplier
                                            let mut gravity = self.world.get::<avian3d::prelude::GravityScale>(entity).map(|g| g.0).unwrap_or(1.0);
                                            ui.horizontal(|ui| {
                                                ui.label("     Gravity Scale:");
                                                if ui.add(egui::Slider::new(&mut gravity, 0.0..=5.0).text("Multiplier")).changed() {
                                                    sync_component_to_network(self.world, entity, avian3d::prelude::GravityScale(gravity));
                                                }
                                            });

                                            // Mass
                                            let mut mass = self.world.get::<avian3d::prelude::Mass>(entity).map(|m| m.0).unwrap_or(1.0);
                                            ui.horizontal(|ui| {
                                                ui.label("       Mass (kg):");
                                                if ui.add(egui::DragValue::new(&mut mass).speed(0.1).range(0.01..=10000.0)).changed() {
                                                    sync_component_to_network(self.world, entity, avian3d::prelude::Mass(mass));
                                                }
                                            });
                                        });
                                        ui.add_space(6.0);
                                    }
                                    if remove_physics {
                                        self.world.entity_mut(entity).remove::<avian3d::prelude::RigidBody>();
                                        self.world.entity_mut(entity).remove::<avian3d::prelude::Restitution>();
                                        self.world.entity_mut(entity).remove::<avian3d::prelude::Friction>();
                                        self.world.entity_mut(entity).remove::<avian3d::prelude::GravityScale>();
                                        self.world.entity_mut(entity).remove::<avian3d::prelude::Mass>();
                                        if let Some(net_id) = net_id_opt {
                                            self.world.write_message(super::serialization::EditorActionRequest::RemoveComponent {
                                                id: net_id.0,
                                                type_path: "avian3d::dynamics::rigid_body::RigidBody".to_string(),
                                            });
                                        }
                                    }

                                    // 4. Point Light Card
                                    let mut remove_light = false;
                                    let has_light = self.world.get::<super::serialization::EditorPointLight>(entity).is_some();
                                    if has_light {
                                        let mut current_light = self.world.get::<super::serialization::EditorPointLight>(entity).cloned().unwrap_or_default();
                                        let mut light_changed = false;
                                        egui::Frame::group(ui.style()).show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("[Light] Point Light Source").strong());
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("        Remove").clicked() {
                                                        remove_light = true;
                                                    }
                                                });
                                            });
                                            ui.separator();

                                            // Color
                                            let srgba = current_light.color.to_srgba();
                                            let mut color_arr = [srgba.red, srgba.green, srgba.blue];
                                            ui.horizontal(|ui| {
                                                ui.label("Light Color:");
                                                if ui.color_edit_button_rgb(&mut color_arr).changed() {
                                                    current_light.color = Color::srgb(color_arr[0], color_arr[1], color_arr[2]);
                                                    light_changed = true;
                                                }
                                            });

                                            // Brightness / Intensity
                                            ui.horizontal(|ui| {
                                                ui.label("Brightness:");
                                                light_changed |= ui.add(egui::Slider::new(&mut current_light.intensity, 0.0..=500_000.0).logarithmic(true).suffix(" lm")).changed();
                                            });

                                            // Range
                                            ui.horizontal(|ui| {
                                                ui.label("Light Radius:");
                                                light_changed |= ui.add(egui::Slider::new(&mut current_light.range, 0.5..=100.0).suffix(" m")).changed();
                                            });

                                            // Shadows
                                            ui.horizontal(|ui| {
                                                light_changed |= ui.checkbox(&mut current_light.shadows_enabled, "Cast Shadows").changed();
                                            });
                                        });
                                        if light_changed {
                                            sync_component_to_network(self.world, entity, current_light);
                                        }
                                        ui.add_space(6.0);
                                    }
                                    if remove_light {
                                        self.world.entity_mut(entity).remove::<super::serialization::EditorPointLight>();
                                        self.world.entity_mut(entity).remove::<PointLight>();
                                        if let Some(net_id) = net_id_opt {
                                            self.world.write_message(super::serialization::EditorActionRequest::RemoveComponent {
                                                id: net_id.0,
                                                type_path: "cb_engine::editor::serialization::EditorPointLight".to_string(),
                                            });
                                        }
                                    }

                                    // 5. Material & Color Card
                                    let mat_handle_opt = self.world.get::<MeshMaterial3d<StandardMaterial>>(entity).map(|m| m.0.clone());
                                    if let Some(mat_handle) = mat_handle_opt {
                                        let mut mat_changed = false;
                                        let mut color_changed = false;
                                        let mut current_color = [0.8, 0.8, 0.8];
                                        let mut current_roughness = 0.5;
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
                                            ui.label(egui::RichText::new("Material & Color").strong());
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
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_col);
                                            }
                                        }
                                        
                                        if mat_changed {
                                            let new_mat = super::serialization::EditorMaterial {
                                                roughness: current_roughness,
                                                metallic: current_metallic,
                                            };
                                            self.world.entity_mut(entity).insert(new_mat.clone());
                                            if ui.input(|i| i.pointer.any_released()) {
                                                sync_component_to_network(self.world, entity, new_mat);
                                            }
                                        }
                                        ui.add_space(6.0);
                                    }

                                    // 6. Script Card
                                    let mut remove_script = false;
                                    let has_script = self.world.get::<crate::scripting::ScriptComponent>(entity).is_some();
                                    if has_script {
                                        let mut current_script = self.world.get::<crate::scripting::ScriptComponent>(entity).cloned().unwrap_or_default();
                                        let mut script_changed = false;
                                        egui::Frame::group(ui.style()).show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("[Scene] Behavior Script (Rhai)").strong());
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button("        Remove").clicked() {
                                                        remove_script = true;
                                                    }
                                                });
                                            });
                                            ui.separator();

                                            ui.horizontal(|ui| {
                                                ui.label("Script File:");
                                                script_changed |= ui.text_edit_singleline(&mut current_script.path).changed();
                                            });

                                            ui.add_space(4.0);
                                            if ui.button("     Open & Edit in VS Code").on_hover_text("Opens this script in VS Code with live hot-reloading!").clicked() {
                                                open_in_vscode(&current_script.path);
                                            }
                                            ui.label(egui::RichText::new("Tip: Saving changes in VS Code hot-reloads script logic in real time.").color(egui::Color32::from_rgb(140, 190, 255)).size(11.0));
                                        });
                                        if script_changed {
                                            sync_component_to_network(self.world, entity, current_script);
                                        }
                                        ui.add_space(6.0);
                                    }
                                    if remove_script {
                                        self.world.entity_mut(entity).remove::<crate::scripting::ScriptComponent>();
                                        if let Some(net_id) = net_id_opt {
                                            self.world.write_message(super::serialization::EditorActionRequest::RemoveComponent {
                                                id: net_id.0,
                                                type_path: "cb_engine::scripting::ScriptComponent".to_string(),
                                            });
                                        }
                                    }

                                    // 6. Prominent Add Component Button
                                    ui.add_space(10.0);
                                    ui.vertical_centered(|ui| {
                                        if ui.add_sized([ui.available_width() * 0.95, 34.0], egui::Button::new(egui::RichText::new("Add Component...").strong().size(14.0))).clicked() {
                                            self.inspector_state.show_add_component_modal = true;
                                        }
                                    });
                                });
                                
                                let released = ui.input(|i| i.pointer.any_released() || i.key_pressed(egui::Key::Enter));
                                if released {
                                    sync_all_entity_components(self.world, entity);
                                }
                            }
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.heading("No entity selected");
                        ui.label("Select an entity in the Hierarchy to view and edit its components.");
                    });
                }
            }
            "Viewport" => {
                if let Some(textures) = self.world.get_resource::<super::camera::ViewportTextures>() {
                    if let Some(tex_id) = textures.editor_egui_id {
                        let size = ui.available_size();
                        let response = ui.image(egui::load::SizedTexture::new(tex_id, size));
                        
                        let mut state = self.world.resource_mut::<EditorViewportState>();
                        state.viewport_size = Vec2::new(size.x, size.y);
                        
                        // Camera Controls Overlay
                        let overlay_text = "RMB: Look | WASD: Move | MMB: Pan | Shift: Sprint | F: Focus";
                        ui.painter().text(
                            response.rect.left_bottom() + egui::vec2(10.0, -10.0),
                            egui::Align2::LEFT_BOTTOM,
                            overlay_text,
                            egui::FontId::proportional(12.0),
                            egui::Color32::from_white_alpha(150),
                        );
                        
                        state.is_hovered = response.hovered();
                        if response.hovered() {
                            if let Some(pos) = response.hover_pos() {
                                let rect = response.rect;
                                state.normalized_mouse_pos = Vec2::new(
                                    (pos.x - rect.min.x) / rect.width(),
                                    (pos.y - rect.min.y) / rect.height(),
                                );
                            }
                        }
                        
                        if let Some(payload) = response.dnd_release_payload::<String>() {
                            let dropped_gltf = (*payload).clone();
                            let pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(response.rect.center());
                            let rect = response.rect;
                            let normalized = Vec2::new(
                                (pos.x - rect.min.x) / rect.width(),
                                (pos.y - rect.min.y) / rect.height(),
                            );
                            
                            let ndc = Vec2::new(
                                normalized.x * 2.0 - 1.0,
                                (1.0 - normalized.y) * 2.0 - 1.0,
                            );
                            
                            let mut camera_query = self.world.query_filtered::<(&Camera, &GlobalTransform), With<super::camera::EditorCamera>>();
                            
                            let mut cam_info = None;
                            if let Some((c, t)) = camera_query.iter(self.world).next() {
                                cam_info = Some((c.clone(), *t));
                            }
                            
                            if let Some((camera, camera_transform)) = cam_info {
                                let projection = camera.clip_from_view(); 
                                let ndc_to_world = camera_transform.to_matrix() * projection.inverse();
                                let near = ndc_to_world.project_point3(ndc.extend(1.0));
                                let far = ndc_to_world.project_point3(ndc.extend(0.0));
                                let ray_dir = (far - near).normalize();
                                let ray_origin = camera_transform.translation();
                                
                                // Intersect with Y=0 plane
                                // ray_origin.y + t * ray_dir.y = 0
                                let t = -ray_origin.y / ray_dir.y;
                                let hit_point = if t >= 0.0 {
                                    ray_origin + ray_dir * t
                                } else {
                                    ray_origin + ray_dir * 10.0 // Fallback if pointing upwards
                                };
                                
                                // Note: we no longer load the scene immediately here.
                                // Instead, we emit an action request, and the local action handler
                                // or the server will spawn it and the restore_visuals system will load the scene.
                                self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                                    id: rand::random::<u64>(),
                                    object_type: "gltf".to_string(), 
                                    asset_path: Some(dropped_gltf.clone()), 
                                    transform: Transform::from_translation(hit_point) 
                                });
                            }
                        }
                    }
                } else {
                    ui.label("Editor Viewport (Texture coming soon)");
                }
            }
            "Game View" => {
                if let Some(textures) = self.world.get_resource::<super::camera::ViewportTextures>() {
                    if let Some(tex_id) = textures.game_egui_id {
                        let size = ui.available_size();
                        let img_response = ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, size)).sense(egui::Sense::click_and_drag()));
                        
                        let mut state = self.world.resource_mut::<EditorViewportState>();
                        state.game_view_size = Vec2::new(size.x, size.y);

                        // If user clicks on the Game View while in Play mode, re-lock cursor for gameplay!
                        let current_state = *self.world.resource::<State<super::EngineState>>().get();
                        if current_state == super::EngineState::Play {
                            if img_response.clicked() || img_response.drag_started() {
                                let mut q_cursor = self.world.query_filtered::<&mut bevy::window::CursorOptions, With<Window>>();
                                if let Some(mut cursor) = q_cursor.iter_mut(self.world).next() {
                                    cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
                                    cursor.visible = false;
                                }
                            }

                            let rect = img_response.rect;
                            let painter = ui.painter_at(rect);

                            // 1. Crosshair in center
                            let center = rect.center();
                            let crosshair_col = egui::Color32::from_white_alpha(220);
                            painter.line_segment([center - egui::vec2(8.0, 0.0), center - egui::vec2(2.0, 0.0)], egui::Stroke::new(2.0, crosshair_col));
                            painter.line_segment([center + egui::vec2(2.0, 0.0), center + egui::vec2(8.0, 0.0)], egui::Stroke::new(2.0, crosshair_col));
                            painter.line_segment([center - egui::vec2(0.0, 8.0), center - egui::vec2(0.0, 2.0)], egui::Stroke::new(2.0, crosshair_col));
                            painter.line_segment([center + egui::vec2(0.0, 2.0), center + egui::vec2(0.0, 8.0)], egui::Stroke::new(2.0, crosshair_col));

                            // 2. Health Bar (Bottom-Left)
                            let mut player_hp = 100.0;
                            let mut max_hp = 100.0;
                            let mut player_query = self.world.query_filtered::<&cb_weapons::components::Health, With<crate::player::Player>>();
                            if let Some(hp) = player_query.iter(self.world).next() {
                                player_hp = hp.current.max(0.0);
                                max_hp = hp.max;
                            }
                            let hp_pct = (player_hp / max_hp).clamp(0.0, 1.0);
                            let hp_rect = egui::Rect::from_min_size(rect.left_bottom() + egui::vec2(16.0, -42.0), egui::vec2(180.0, 24.0));
                            painter.rect_filled(hp_rect, 4.0, egui::Color32::from_black_alpha(180));
                            let fill_rect = egui::Rect::from_min_size(hp_rect.min + egui::vec2(2.0, 2.0), egui::vec2((hp_rect.width() - 4.0) * hp_pct, hp_rect.height() - 4.0));
                            let hp_col = if hp_pct > 0.5 { egui::Color32::from_rgb(40, 210, 80) } else if hp_pct > 0.25 { egui::Color32::from_rgb(230, 160, 30) } else { egui::Color32::from_rgb(230, 40, 40) };
                            painter.rect_filled(fill_rect, 3.0, hp_col);
                            painter.text(hp_rect.center(), egui::Align2::CENTER_CENTER, format!("       {:.0} / {:.0} HP", player_hp, max_hp), egui::FontId::proportional(12.5), egui::Color32::WHITE);

                            // 3. Ammo Counter (Bottom-Right)
                            let mut mag_current = 0;
                            let mut mag_reserve = 0;
                            let mut is_reloading = false;
                            let mut weapon_query = self.world.query::<&cb_weapons::components::Magazine>();
                            if let Some(mag) = weapon_query.iter(self.world).next() {
                                mag_current = mag.current;
                                mag_reserve = mag.reserve;
                                is_reloading = mag.is_reloading;
                            }
                            let ammo_text = if is_reloading { "Reloading...".to_string() } else { format!("Ammo: {} / {}", mag_current, mag_reserve) };
                            let ammo_rect = egui::Rect::from_min_size(rect.right_bottom() + egui::vec2(-140.0, -42.0), egui::vec2(124.0, 24.0));
                            painter.rect_filled(ammo_rect, 4.0, egui::Color32::from_black_alpha(180));
                            painter.text(ammo_rect.center(), egui::Align2::CENTER_CENTER, ammo_text, egui::FontId::proportional(13.0), egui::Color32::WHITE);

                            // 4. Objective & Timer HUD (Top-Center)
                            let match_state = self.world.resource::<crate::gamemode::MatchState>().clone();
                            let mins = (match_state.elapsed_seconds / 60.0) as u32;
                            let secs = (match_state.elapsed_seconds % 60.0) as u32;
                            let obj_rect = egui::Rect::from_min_size(egui::pos2(rect.center().x - 130.0, rect.top() + 14.0), egui::vec2(260.0, 28.0));
                            painter.rect_filled(obj_rect, 4.0, egui::Color32::from_black_alpha(180));
                            painter.text(obj_rect.center(), egui::Align2::CENTER_CENTER, format!("     Targets: {}  |         {:02}:{:02}", match_state.targets_remaining, mins, secs), egui::FontId::proportional(13.0), egui::Color32::from_rgb(220, 235, 255));

                            // 5. Win / Lose Overlay Modals
                            match match_state.status {
                                crate::gamemode::MatchStatus::Victory => {
                                    let modal_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(340.0, 200.0));
                                    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(140));
                                    painter.rect(modal_rect, 8.0, egui::Color32::from_rgb(25, 35, 25), egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 220, 100)), egui::StrokeKind::Inside);
                                    
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(14.0);
                                            ui.heading(egui::RichText::new("     VICTORY!").color(egui::Color32::from_rgb(255, 215, 0)).size(24.0).strong());
                                            ui.add_space(4.0);
                                            ui.label(egui::RichText::new(&match_state.win_reason).color(egui::Color32::from_rgb(200, 255, 200)).size(13.0));
                                            ui.add_space(6.0);
                                            ui.label(egui::RichText::new(format!("Score: {}  *  Time: {:02}:{:02}", match_state.score, mins, secs)).color(egui::Color32::WHITE).size(13.0));
                                            ui.add_space(16.0);
                                            if ui.add_sized([160.0, 32.0], egui::Button::new("Restart Match")).clicked() {
                                                self.world.write_message(crate::gamemode::ResetMatchEvent);
                                            }
                                        });
                                    });
                                }
                                crate::gamemode::MatchStatus::Defeat => {
                                    let modal_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(340.0, 200.0));
                                    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(150));
                                    painter.rect(modal_rect, 8.0, egui::Color32::from_rgb(40, 20, 20), egui::Stroke::new(2.0, egui::Color32::from_rgb(230, 60, 60)), egui::StrokeKind::Inside);
                                    
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(14.0);
                                            ui.heading(egui::RichText::new("DEFEAT").color(egui::Color32::from_rgb(255, 70, 70)).size(24.0).strong());
                                            ui.add_space(4.0);
                                            ui.label(egui::RichText::new(&match_state.lose_reason).color(egui::Color32::from_rgb(255, 180, 180)).size(13.0));
                                            ui.add_space(6.0);
                                            ui.label(egui::RichText::new(format!("Kills: {}  *  Time: {:02}:{:02}", match_state.kills, mins, secs)).color(egui::Color32::WHITE).size(13.0));
                                            ui.add_space(16.0);
                                            if ui.add_sized([160.0, 32.0], egui::Button::new("Respawn")).clicked() {
                                                self.world.write_message(crate::gamemode::ResetMatchEvent);
                                            }
                                        });
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    ui.label("Game View (Texture coming soon)");
                }
            }
            "Console" => {
                let logs = self.world.resource::<ConsoleState>().logs.clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for log in logs {
                        ui.label(log);
                    }
                });
            }
            "Assets" => {
                ui.heading("Prefabs");
                ui.horizontal_wrapped(|ui| {
                    if ui.button("+ Empty Node").on_hover_text("Spawn an empty Transform node").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "empty".to_string(), 
                            asset_path: None, 
                            transform: Transform::default() 
                        });
                    }
                    if ui.button("+ Target Dummy").on_hover_text("Spawn a physical target cube with health").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "target_dummy".to_string(), 
                            asset_path: None, 
                            transform: Transform::from_xyz(0.0, 1.0, -4.0) 
                        });
                    }
                    if ui.button("[Light] Point Light").on_hover_text("Spawn a point light source").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "light".to_string(), 
                            asset_path: None, 
                            transform: Transform::default() 
                        });
                    }
                    if ui.button("[Spawn] Spawn Point").on_hover_text("Spawn a multiplayer player spawn point").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "spawn_point".to_string(), 
                            asset_path: None, 
                            transform: Transform::default() 
                        });
                    }
                    if ui.button("     Goal Zone").on_hover_text("Spawn an extraction/win goal zone").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "goal_zone".to_string(), 
                            asset_path: None, 
                            transform: Transform::from_xyz(0.0, 0.05, -12.0) 
                        });
                    }
                    if ui.button("[Crate] Weapon Crate").on_hover_text("Spawn an interactable weapon crate").clicked() {
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject { 
                            id: rand::random::<u64>(),
                            object_type: "weapon_crate".to_string(), 
                            asset_path: None, 
                            transform: Transform::default() 
                        });
                    }
                });
                
                ui.separator();
                ui.heading("Asset Browser");
                
                let time_elapsed = self.world.resource::<Time>().elapsed_secs_f64();
                let mut browser = self.world.resource_mut::<AssetBrowserState>();
                
                if time_elapsed - browser.last_refresh > 2.0 {
                    browser.last_refresh = time_elapsed;
                    if let Ok(entries) = std::fs::read_dir(&browser.current_path) {
                        browser.files.clear();
                        for entry in entries.filter_map(|e| e.ok()) {
                            browser.files.push(entry.path());
                        }
                        // Sort: directories first, then files
                        browser.files.sort_by(|a, b| {
                            match (a.is_dir(), b.is_dir()) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.file_name().cmp(&b.file_name()),
                            }
                        });
                    }
                }
                
                ui.horizontal(|ui| {
                    if ui.button("<- Back").clicked() {
                        if let Some(parent) = browser.current_path.parent().map(|p| p.to_path_buf()) {
                            if parent.starts_with("assets") || parent.to_string_lossy() == "assets" {
                                browser.current_path = parent;
                                browser.last_refresh = 0.0;
                            }
                        }
                    }
                    ui.label(browser.current_path.to_string_lossy().to_string());
                });
                
                ui.separator();
                
                egui::ScrollArea::vertical().id_salt("asset_browser_scroll").max_height(200.0).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut clicked_dir = None;
                        for file in &browser.files {
                            let name = file.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if file.is_dir() {
                                if ui.button(format!("     {}", name)).clicked() {
                                    clicked_dir = Some(file.clone());
                                }
                            } else {
                                let icon = if name.ends_with(".gltf") || name.ends_with(".glb") { "[Model]" }
                                           else if name.ends_with(".png") || name.ends_with(".jpg") { "       " }
                                           else if name.ends_with(".ron") || name.ends_with(".rhai") { "[Scene]" }
                                           else { "[File]" };
                                
                                let path = file.strip_prefix("assets").unwrap_or(file).to_string_lossy().replace("\\", "/");
                                let _item_id = ui.id().with(&path);
                                let mut response = ui.button(format!("{} {}", icon, name));
                                
                                if name.ends_with(".gltf") || name.ends_with(".glb") {
                                    response = response.on_hover_text("Drag and drop into the Viewport to spawn this 3D model");
                                } else if name.ends_with(".rhai") {
                                    response = response.on_hover_text("Click to edit in VS Code, or drag & drop onto an object to attach");
                                    if response.clicked() {
                                        open_in_vscode(&file.to_string_lossy());
                                    }
                                }
                                
                                if response.hovered() {
                                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                                }
                                
                                if response.drag_started() {
                                    egui::DragAndDrop::set_payload(ui.ctx(), path);
                                }
                            }
                        }
                        if let Some(d) = clicked_dir {
                            browser.current_path = d;
                            browser.last_refresh = 0.0;
                        }
                    });
                });
                ui.separator();
                ui.heading("Import GLTF");
                let mut gltf_path = self.gltf_input.clone();
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut gltf_path);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory("assets")
                            .add_filter("GLTF/GLB Models", &["gltf", "glb"])
                            .pick_file() {
                            
                            if let Ok(current_dir) = std::env::current_dir() {
                                let assets_dir = current_dir.join("assets");
                                if let Ok(relative) = path.strip_prefix(&assets_dir) {
                                    gltf_path = relative.to_string_lossy().replace("\\", "/");
                                } else {
                                    // If outside assets, try to copy it in or just use absolute (which might fail in Bevy)
                                    gltf_path = path.to_string_lossy().replace("\\", "/");
                                }
                            }
                        }
                    }
                    if ui.button("Import").clicked() && !gltf_path.is_empty() {
                        let id = rand::random::<u64>();
                        self.world.write_message(super::serialization::EditorActionRequest::SpawnObject {
                            id,
                            object_type: "gltf".to_string(),
                            asset_path: Some(gltf_path.clone()),
                            transform: Transform::default(),
                        });
                    }
                });
                *self.gltf_input = gltf_path;
            }
            _ => {
                ui.label("Unknown Tab");
            }
        }
    }
}

fn render_editor_ui(world: &mut World) {
    let mut ui_state = world.remove_resource::<EditorUiState>().unwrap();
    
    // Check if we should toggle console via ~
    let keys = world.resource::<ButtonInput<KeyCode>>();
    if keys.just_pressed(KeyCode::Backquote) {
        // Find console tab and focus it
        if let Some((surface, node, _tab)) = ui_state.tree.find_tab(&"Console".to_string()) {
            ui_state.tree.set_active_tab((surface, node, _tab));
        }
    }

    // Check if user pressed Delete or Backspace to despawn selected object
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        let mut selected = None;
        {
            let mut q = world.query_filtered::<(Entity, Option<&super::serialization::NetworkId>, &Transform, &super::serialization::SceneObject, Option<&Name>), With<crate::editor::picking::Selected>>();
            if let Some((e, net_id, transform, obj, name)) = q.iter(world).next() {
                let id = net_id.map(|n| n.0).unwrap_or_else(rand::random::<u64>);
                selected = Some((e, id, *transform, obj.clone(), name.map(|n| n.as_str().to_string())));
            }
        }
        if let Some((entity, id, transform, obj, name)) = selected {
            world.despawn(entity);
            world.resource_mut::<super::history::HistoryState>().record_action(super::history::EditorCommand::Despawn {
                id,
                object_type: obj.object_type.clone(),
                asset_path: obj.asset_path.clone(),
                transform,
                name,
            });
            world.write_message(super::serialization::EditorActionRequest::DespawnObject { id });
        }
    }

    // Clone contexts to avoid borrowing World while egui is running
    let ctx = {
        let mut q = world.query::<&mut bevy_egui::EguiContext>();
        if let Some(mut context) = q.iter_mut(world).next() {
            context.get_mut().clone()
        } else {
            warn!("render_editor_ui: EguiContext not found on any window");
            world.insert_resource(ui_state);
            return;
        }
    };

    let mut dialogs = world.remove_resource::<EditorUiDialogs>().unwrap();
    let mut inspector_state = world.remove_resource::<InspectorState>().unwrap_or_default();

    // Keyboard shortcuts
    let (key_n, key_o, key_s, key_shift_s) = {
        let keys = world.resource::<ButtonInput<KeyCode>>();
        let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        (
            ctrl && keys.just_pressed(KeyCode::KeyN),
            ctrl && !shift && keys.just_pressed(KeyCode::KeyO),
            ctrl && !shift && keys.just_pressed(KeyCode::KeyS),
            ctrl && shift && keys.just_pressed(KeyCode::KeyS),
        )
    };

    if key_n {
        dialogs.show_new_scene_dialog = true;
    }
    if key_o {
        if let Some(path) = open_file_dialog() {
            world.write_message(super::serialization::LoadSceneEvent(path));
        } else {
            dialogs.show_open_dialog = true;
        }
    }
    if key_s {
        let active_path = world.resource::<super::serialization::ActiveSceneState>().current_path.clone();
        if let Some(path) = active_path {
            world.write_message(super::serialization::SaveSceneEvent(path));
        } else {
            if let Some(path) = save_file_dialog("level.ron") {
                world.write_message(super::serialization::SaveSceneEvent(path.clone()));
                world.resource_mut::<super::serialization::ActiveSceneState>().current_path = Some(path);
            } else {
                dialogs.show_save_as_dialog = true;
            }
        }
    }
    if key_shift_s {
        let current_name = world.resource::<super::serialization::ActiveSceneState>().display_name();
        if let Some(path) = save_file_dialog(&current_name) {
            world.write_message(super::serialization::SaveSceneEvent(path.clone()));
            world.resource_mut::<super::serialization::ActiveSceneState>().current_path = Some(path);
        } else {
            dialogs.show_save_as_dialog = true;
        }
    }

    if dialogs.show_new_scene_dialog {
        egui::Window::new("[File] Create New Scene")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.label("Clear the current scene and start a fresh untitled scene?");
                ui.label(egui::RichText::new("       Any unsaved changes in the current scene will be discarded.").color(egui::Color32::from_rgb(255, 180, 50)));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("[File] Clear & New Scene").clicked() {
                        world.write_message(super::serialization::ClearSceneEvent);
                        dialogs.show_new_scene_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        dialogs.show_new_scene_dialog = false;
                    }
                });
            });
    }

    if dialogs.show_save_as_dialog {
        egui::Window::new("Save Scene As")
            .collapsible(false)
            .resizable(false)
            .default_size([400.0, 160.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("File Path / Name:");
                    ui.text_edit_singleline(&mut dialogs.modal_file_input);
                });
                ui.horizontal(|ui| {
                    ui.label("File Type Filter:");
                    ui.checkbox(&mut dialogs.filter_scene_types_only, "Scene Files (*.ron, *.scn.ron)");
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Browse OS Dialog...").clicked() {
                        if let Some(path) = save_file_dialog(&dialogs.modal_file_input) {
                            dialogs.modal_file_input = path;
                        }
                    }
                    if ui.button("Save").clicked() {
                        let mut path = dialogs.modal_file_input.trim().to_string();
                        if !path.is_empty() {
                            if !path.ends_with(".ron") && !path.ends_with(".scn") {
                                path.push_str(".ron");
                            }
                            world.write_message(super::serialization::SaveSceneEvent(path.clone()));
                            world.resource_mut::<super::serialization::ActiveSceneState>().current_path = Some(path);
                            dialogs.show_save_as_dialog = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        dialogs.show_save_as_dialog = false;
                    }
                });
            });
    }

    if dialogs.show_open_dialog {
        egui::Window::new("Open Scene")
            .collapsible(false)
            .resizable(true)
            .default_size([460.0, 320.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("File Path:");
                    ui.text_edit_singleline(&mut dialogs.modal_file_input);
                    if ui.button("Browse OS...").clicked() {
                        if let Some(path) = open_file_dialog() {
                            dialogs.modal_file_input = path;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Filter by Scene Types:");
                    ui.checkbox(&mut dialogs.filter_scene_types_only, "Scene Files only (*.ron, *.scn.ron, *.scn)");
                });
                ui.separator();
                ui.label("Available Scene Files (Root & Assets):");
                egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                    let mut picked_file = None;
                    if let Ok(entries) = std::fs::read_dir(".") {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_scene = name.ends_with(".ron") || name.ends_with(".scn.ron") || name.ends_with(".scn");
                            if (!dialogs.filter_scene_types_only || is_scene)
                                && entry.path().is_file() {
                                    let icon = if is_scene { "[Scene]" } else { "[File]" };
                                    if ui.button(format!("{} {}", icon, name)).clicked() {
                                        picked_file = Some(name);
                                    }
                                }
                        }
                    }
                    if let Ok(entries) = std::fs::read_dir("assets") {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.path().to_string_lossy().replace("\\", "/");
                            let is_scene = name.ends_with(".ron") || name.ends_with(".scn.ron") || name.ends_with(".scn");
                            if (!dialogs.filter_scene_types_only || is_scene)
                                && entry.path().is_file() {
                                    let icon = if is_scene { "[Scene]" } else { "[File]" };
                                    if ui.button(format!("{} {}", icon, name)).clicked() {
                                        picked_file = Some(name);
                                    }
                                }
                        }
                    }
                    if let Some(p) = picked_file {
                        dialogs.modal_file_input = p;
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Open Selected Scene").clicked() {
                        let path = dialogs.modal_file_input.trim().to_string();
                        if !path.is_empty() {
                            world.write_message(super::serialization::LoadSceneEvent(path));
                            dialogs.show_open_dialog = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        dialogs.show_open_dialog = false;
                    }
                });
            });
    }

    if dialogs.show_save_before_play {
        egui::Window::new("Save before playing?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.label("Would you like to save the current scene before entering Play Mode?");
                ui.horizontal(|ui| {
                    if ui.button("Save & Play").clicked() {
                        let active_path = world.resource::<super::serialization::ActiveSceneState>().current_path.clone().unwrap_or_else(|| "level.ron".to_string());
                        world.write_message(super::serialization::SaveSceneEvent(active_path));
                        world.resource_mut::<NextState<super::EngineState>>().set(super::EngineState::Play);
                        if let Some(mut ie) = world.get_resource_mut::<cb_input::InputEnabled>() {
                            ie.0 = true;
                        }
                        dialogs.show_save_before_play = false;
                    }
                    if ui.button("Cancel").clicked() {
                        dialogs.show_save_before_play = false;
                    }
                });
            });
    }

    if dialogs.show_join_play {
        egui::Window::new("Play Mode Request")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.label("Another user has started Play Mode. Would you like to join?");
                ui.horizontal(|ui| {
                    if ui.button("Save Scene & Join").clicked() {
                        let active_path = world.resource::<super::serialization::ActiveSceneState>().current_path.clone().unwrap_or_else(|| "level.ron".to_string());
                        world.write_message(super::serialization::SaveSceneEvent(active_path));
                        world.resource_mut::<NextState<super::EngineState>>().set(super::EngineState::Play);
                        if let Some(mut ie) = world.get_resource_mut::<cb_input::InputEnabled>() {
                            ie.0 = true;
                        }
                        dialogs.show_join_play = false;
                    }
                    if ui.button("Decline").clicked() {
                        dialogs.show_join_play = false;
                    }
                });
            });
    }

    if dialogs.show_help_window {
        let mut open = dialogs.show_help_window;
        egui::Window::new("Code Blue Editor - Quick Start Guide")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Camera Navigation (Viewport)");
                    ui.label("* Hold Right Mouse Button (RMB) to look around.");
                    ui.label("* Use W, A, S, D, Q, E to fly while holding RMB.");
                    ui.label("* Hold Shift for faster movement.");
                    ui.label("* Hold Middle Mouse Button (MMB) to pan.");
                    ui.label("* Press F to focus the camera on the selected entity.");
                    ui.separator();
                    
                    ui.heading("        Scene Building & File Management");
                    ui.label("* File -> Save Scene (Ctrl+S) / Save Scene As (Ctrl+Shift+S).");
                    ui.label("* File -> Open Scene (Ctrl+O) with .ron scene type filtering.");
                    ui.label("* File -> New Scene (Ctrl+N) to start fresh.");
                    ui.label("* Use the Assets panel (bottom right) to browse for .gltf/.glb models.");
                    ui.label("* Drag and drop a model from the Asset Browser into the Viewport to spawn it.");
                    ui.label("* Select an entity in the Hierarchy to view its properties in the Inspector.");
                    ui.label("* Use the Move (W), Rotate (E), and Scale (R) tools to adjust it.");
                    ui.label("* Press Delete or Backspace to remove the selected entity.");
                    ui.separator();

                    ui.heading("Undo & Redo");
                    ui.label("* Press Ctrl + Z to undo the last move, spawn, or delete action.");
                    ui.label("* Press Ctrl + Y or Ctrl + Shift + Z to redo.");
                    ui.separator();
                    
                    ui.heading("     Multiplayer & Play Mode");
                    ui.label("* When one editor clicks Play, all connected editors will be invited to join the session.");
                    ui.label("* Changes made in the Editor automatically sync across all connected users via the headless server.");
                });
            });
        dialogs.show_help_window = open;
    }

    if dialogs.show_about_dialog {
        let mut open = dialogs.show_about_dialog;
        let mut close_clicked = false;
        egui::Window::new("       About Code Blue")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_size([460.0, 360.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.heading(egui::RichText::new("Code Blue Engine").size(20.0).strong().color(egui::Color32::from_rgb(100, 180, 255)));
                    ui.label(egui::RichText::new("Version 0.1.0 * Built in Rust").size(12.0).color(egui::Color32::GRAY));
                    ui.add_space(8.0);
                });

                // Philosophy Quote Frame
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 30, 45, 200))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("    If they can't make their own game engine, they are not programmers. It's like a chef who can only make frozen pizza.    ")
                            .italics()
                            .color(egui::Color32::from_rgb(255, 215, 120))
                            .size(12.5));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("-- Markus \"Notch\" Persson").size(11.0).color(egui::Color32::from_rgb(180, 200, 220)));
                        });
                    });

                ui.add_space(6.0);
                ui.label(egui::RichText::new("Code Blue was built from first principles to explore the bleeding edge of modern game engine architecture -- crafted in pure Rust and powered by Google's Antigravity.")
                    .color(egui::Color32::from_rgb(210, 220, 230))
                    .size(12.0));

                ui.add_space(6.0);
                ui.separator();
                ui.label(egui::RichText::new("       Technical Architecture:").strong());
                ui.label("* Data-Driven ECS: Bevy 0.16");
                ui.label("* Physics: Avian3D (Parallel Rigid Body Simulation)");
                ui.label("* Multiplayer Netcode: Lightyear 120Hz Client-Server Replication");
                ui.label("* Scripting: Rhai AST Engine with Live Disk Hot-Reloading");
                ui.label("* Visuals: First-Person Viewmodel with Dynamic Aim Lag & Sway");
                ui.separator();

                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Created by Ruan Prinsloo * Powered by Google Antigravity").size(11.0).color(egui::Color32::from_rgb(140, 180, 240)).strong());
                    ui.add_space(4.0);
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });
        dialogs.show_about_dialog = open && !close_clicked;
    }

    if inspector_state.show_add_component_modal {
        let mut open = inspector_state.show_add_component_modal;
        let mut component_to_add = None;
        egui::Window::new("Add Component")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([460.0, 480.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut inspector_state.component_search);
                    if ui.button("X").on_hover_text("Clear search").clicked() {
                        inspector_state.component_search.clear();
                    }
                });
                ui.separator();

                let search_lower = inspector_state.component_search.to_lowercase();
                
                egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                    let mut current_category = "";
                    for meta in COMPONENT_CATALOG {
                        let matches_search = search_lower.is_empty()
                            || meta.name.to_lowercase().contains(&search_lower)
                            || meta.category.to_lowercase().contains(&search_lower)
                            || meta.description.to_lowercase().contains(&search_lower)
                            || meta.keywords.iter().any(|k| k.to_lowercase().contains(&search_lower));

                        if !matches_search {
                            continue;
                        }

                        if meta.category != current_category {
                            current_category = meta.category;
                            ui.add_space(8.0);
                            ui.heading(current_category);
                            ui.separator();
                        }

                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{} {}", meta.icon, meta.name)).strong().size(14.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("+ Add").clicked() {
                                        component_to_add = Some(meta.type_path);
                                    }
                                });
                            });
                            ui.label(egui::RichText::new(meta.description).color(egui::Color32::from_rgb(180, 190, 200)).size(11.5));
                        });
                        ui.add_space(2.0);
                    }
                });
            });

        if let Some(type_path) = component_to_add {
            let mut selected_entity = None;
            {
                let mut q = world.query_filtered::<Entity, With<crate::editor::picking::Selected>>();
                if let Ok(entity) = q.single(world) {
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                let _ = super::serialization::add_default_component(world, entity, type_path);
                if let Some(net_id) = world.get::<super::serialization::NetworkId>(entity).copied() {
                    world.write_message(super::serialization::EditorActionRequest::AddComponent {
                        id: net_id.0,
                        type_path: type_path.to_string(),
                    });
                }
            }
            open = false;
        }
        inspector_state.show_add_component_modal = open;
    }

    #[allow(deprecated)]
    #[allow(deprecated)]
    egui::TopBottomPanel::top("editor_menu_bar").show(&ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("[File] New Scene\tCtrl+N").clicked() {
                    dialogs.show_new_scene_dialog = true;
                    ui.close_menu();
                }
                if ui.button("Open Scene...\tCtrl+O").clicked() {
                    if let Some(path) = open_file_dialog() {
                        world.write_message(super::serialization::LoadSceneEvent(path));
                    } else {
                        dialogs.show_open_dialog = true;
                    }
                    ui.close_menu();
                }
                if ui.button("Open Scene (In-Editor)...").clicked() {
                    dialogs.show_open_dialog = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save Scene\tCtrl+S").clicked() {
                    let active_path = world.resource::<super::serialization::ActiveSceneState>().current_path.clone();
                    if let Some(path) = active_path {
                        world.write_message(super::serialization::SaveSceneEvent(path));
                    } else {
                        if let Some(path) = save_file_dialog("level.ron") {
                            world.write_message(super::serialization::SaveSceneEvent(path.clone()));
                            world.resource_mut::<super::serialization::ActiveSceneState>().current_path = Some(path);
                        } else {
                            dialogs.show_save_as_dialog = true;
                        }
                    }
                    ui.close_menu();
                }
                if ui.button("Save Scene As...\tCtrl+Shift+S").clicked() {
                    let current_name = world.resource::<super::serialization::ActiveSceneState>().display_name();
                    if let Some(path) = save_file_dialog(&current_name) {
                        world.write_message(super::serialization::SaveSceneEvent(path.clone()));
                        world.resource_mut::<super::serialization::ActiveSceneState>().current_path = Some(path);
                    } else {
                        dialogs.show_save_as_dialog = true;
                    }
                    ui.close_menu();
                }
                if ui.button("     Save Scene As (In-Editor)...").clicked() {
                    dialogs.modal_file_input = world.resource::<super::serialization::ActiveSceneState>().display_name();
                    dialogs.show_save_as_dialog = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("Multi-Connect", |ui| {
                if ui.button("     Connect / Reconnect to Server").clicked() {
                    world.write_message(super::serialization::ConnectToServerEvent);
                    ui.close_menu();
                }
            });
            ui.menu_button("Edit", |ui| {
                let can_undo = world.resource::<super::history::HistoryState>().can_undo();
                let can_redo = world.resource::<super::history::HistoryState>().can_redo();

                if ui.add_enabled(can_undo, egui::Button::new("Undo\tCtrl+Z")).clicked() {
                    world.write_message(super::history::UndoEvent);
                    ui.close_menu();
                }
                if ui.add_enabled(can_redo, egui::Button::new("Redo\tCtrl+Y")).clicked() {
                    world.write_message(super::history::RedoEvent);
                    ui.close_menu();
                }
                if ui.button("Clear History").clicked() {
                    let mut history = world.resource_mut::<super::history::HistoryState>();
                    history.undo_stack.clear();
                    history.redo_stack.clear();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("+ Empty Node").clicked() {
                    let net_id = rand::random::<u64>();
                    world.resource_mut::<super::history::HistoryState>().record_action(super::history::EditorCommand::Spawn {
                        id: net_id,
                        object_type: "empty".to_string(),
                        asset_path: None,
                        transform: Transform::default(),
                        name: None,
                    });
                    world.write_message(super::serialization::EditorActionRequest::SpawnObject {
                        id: net_id,
                        object_type: "empty".to_string(),
                        asset_path: None,
                        transform: Transform::default(),
                    });
                }
                if ui.button("[Model] Spawn Cube").clicked() {
                    let net_id = rand::random::<u64>();
                    world.resource_mut::<super::history::HistoryState>().record_action(super::history::EditorCommand::Spawn {
                        id: net_id,
                        object_type: "cube".to_string(),
                        asset_path: None,
                        transform: Transform::default(),
                        name: None,
                    });
                    world.write_message(super::serialization::EditorActionRequest::SpawnObject {
                        id: net_id,
                        object_type: "cube".to_string(),
                        asset_path: None,
                        transform: Transform::default(),
                    });
                }
                ui.separator();
                if ui.button("     Generate Example City Map").clicked() {
                    world.write_message(super::serialization::GenerateCityEvent);
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Window", |ui| {
                ui.label(egui::RichText::new("Panels & Views").weak().size(11.0));
                ui.separator();
                for &(tab_id, desc) in ALL_TABS {
                    let is_open = is_tab_open(&ui_state.tree, tab_id);
                    let label = if is_open {
                        format!("* {}  ({})", tab_id, desc)
                    } else {
                        format!("  {}  ({})", tab_id, desc)
                    };
                    if ui.selectable_label(is_open, label).on_hover_text(format!("Open or focus the {} panel", tab_id)).clicked() {
                        open_or_focus_tab(&mut ui_state.tree, tab_id);
                        ui.close_menu();
                    }
                }
                ui.separator();
                if ui.button("Open All Windows").on_hover_text("Open all 6 editor panels into the dock").clicked() {
                    for &(tab_id, _) in ALL_TABS {
                        if !is_tab_open(&ui_state.tree, tab_id) {
                            open_or_focus_tab(&mut ui_state.tree, tab_id);
                        }
                    }
                    ui.close_menu();
                }
                if ui.button("Reset Layout to Default").on_hover_text("Restore the default 4-way split layout").clicked() {
                    reset_dock_layout(&mut ui_state.tree);
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Documentation & Quick Start").clicked() {
                    dialogs.show_help_window = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("       About Code Blue...").clicked() {
                    dialogs.show_about_dialog = true;
                    ui.close_menu();
                }
            });

            ui.separator();
            
            // Gizmo Modes
            ui.horizontal(|ui| {
            let current_gizmo_mode = *world.resource::<super::gizmos::GizmoMode>();
            if ui.selectable_label(current_gizmo_mode == super::gizmos::GizmoMode::Translate, "Move (W)")
                .on_hover_text("Move selected object (Hotkey: W)").clicked() {
                *world.resource_mut::<super::gizmos::GizmoMode>() = super::gizmos::GizmoMode::Translate;
            }
            if ui.selectable_label(current_gizmo_mode == super::gizmos::GizmoMode::Rotate, "Rotate (E)")
                .on_hover_text("Rotate selected object (Hotkey: E)").clicked() {
                *world.resource_mut::<super::gizmos::GizmoMode>() = super::gizmos::GizmoMode::Rotate;
            }
            if ui.selectable_label(current_gizmo_mode == super::gizmos::GizmoMode::Scale, "Scale (R)")
                .on_hover_text("Scale selected object (Hotkey: R)").clicked() {
                *world.resource_mut::<super::gizmos::GizmoMode>() = super::gizmos::GizmoMode::Scale;
            }
        });
            
            ui.separator();
            
            // Play / Stop
            let current_state = *world.resource::<State<super::EngineState>>().get();
            if current_state == super::EngineState::Edit {
                if ui.button("Play").on_hover_text("Enter Play Mode and test your game").clicked() {
                    dialogs.show_save_before_play = true;
                }
            } else {
                if ui.button("    Stop").on_hover_text("Exit Play Mode and return to Editor").clicked() {
                    world.resource_mut::<NextState<super::EngineState>>().set(super::EngineState::Edit);
                }
            }

            ui.separator();
            
            // Active Scene Display Badge
            let active_scene_name = world.resource::<super::serialization::ActiveSceneState>().display_name();
            ui.label(egui::RichText::new(format!("[File] Scene: {}", active_scene_name)).strong().color(egui::Color32::from_rgb(180, 210, 255)));

            // Live Connection Status & User ID Badge
            let session_client_id = world.resource::<super::serialization::LocalEditorSession>().client_id;
            let mut q_net = world.query_filtered::<Entity, With<super::serialization::NetworkId>>();
            let has_net_objects = q_net.iter(world).next().is_some();
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let user_col = super::user_color::get_user_color_egui(session_client_id);
                if has_net_objects {
                    ui.label(egui::RichText::new(format!("    Online | User #{:04}", session_client_id % 10000)).color(user_col).strong());
                } else {
                    ui.label(egui::RichText::new(format!("    Standalone | User #{:04}", session_client_id % 10000)).color(user_col));
                }
            });
        });
    });

    egui::CentralPanel::default().show(&ctx, |ui| {
        let mut tab_viewer = TabViewer {
            world,
            gltf_input: &mut ui_state.gltf_input,
            inspector_state: &mut inspector_state,
            _dialogs: &mut dialogs,
        };
        DockArea::new(&mut ui_state.tree)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut tab_viewer);
    });

    // In Play Mode, pressing Escape releases cursor for editor navigation
    if let Some(state) = world.get_resource::<State<super::EngineState>>() {
        if *state.get() == super::EngineState::Play {
            let keys = world.resource::<ButtonInput<KeyCode>>();
            if keys.just_pressed(KeyCode::Escape) {
                let mut q_cursor = world.query_filtered::<&mut bevy::window::CursorOptions, With<Window>>();
                if let Some(mut cursor) = q_cursor.iter_mut(world).next() {
                    cursor.grab_mode = bevy::window::CursorGrabMode::None;
                    cursor.visible = true;
                }
            }
        }
    }

    world.insert_resource(ui_state);
    world.insert_resource(dialogs);
    world.insert_resource(inspector_state);
}

pub const ALL_TABS: &[(&str, &str)] = &[
    ("Viewport", "3D Scene Viewport"),
    ("Game View", "Play / Game Camera"),
    ("Hierarchy", "Scene Outliner & Hierarchy"),
    ("Inspector", "Component Properties & Physics"),
    ("Console", "Developer Console & Logs"),
    ("Assets", "Asset Browser & Prefabs"),
];

pub fn is_tab_open(tree: &DockState<String>, name: &str) -> bool {
    tree.find_tab(&name.to_string()).is_some()
}

pub fn open_or_focus_tab(tree: &mut DockState<String>, name: &str) {
    if let Some((surface, node, tab)) = tree.find_tab(&name.to_string()) {
        tree.set_active_tab((surface, node, tab));
    } else {
        tree.push_to_first_leaf(name.to_string());
        if let Some((surface, node, tab)) = tree.find_tab(&name.to_string()) {
            tree.set_active_tab((surface, node, tab));
        }
    }
}

pub fn reset_dock_layout(tree: &mut DockState<String>) {
    let mut new_tree = DockState::new(vec!["Viewport".to_string(), "Game View".to_string()]);
    let surface = new_tree.main_surface_mut();
    let [vp, _inspector] = surface.split_right(NodeIndex::root(), 0.75, vec!["Inspector".to_string()]);
    let [vp, _hierarchy] = surface.split_left(vp, 0.2, vec!["Hierarchy".to_string()]);
    let [_vp, _console] = surface.split_below(vp, 0.7, vec!["Console".to_string(), "Assets".to_string()]);
    *tree = new_tree;
}

pub fn focus_game_view_on_play(
    mut ui_state: ResMut<EditorUiState>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<Window>>,
    input_enabled: Option<ResMut<cb_input::InputEnabled>>,
) {
    if let Some((surface, node, tab)) = ui_state.tree.find_tab(&"Game View".to_string()) {
        ui_state.tree.set_active_tab((surface, node, tab));
    }
    if let Ok(mut cursor) = cursor_options.single_mut() {
        cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        cursor.visible = false;
    }
    if let Some(mut ie) = input_enabled {
        ie.0 = true;
    }
}

pub fn focus_viewport_on_edit(
    mut ui_state: ResMut<EditorUiState>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<Window>>,
    input_enabled: Option<ResMut<cb_input::InputEnabled>>,
) {
    if let Some((surface, node, tab)) = ui_state.tree.find_tab(&"Viewport".to_string()) {
        ui_state.tree.set_active_tab((surface, node, tab));
    }
    if let Ok(mut cursor) = cursor_options.single_mut() {
        cursor.grab_mode = bevy::window::CursorGrabMode::None;
        cursor.visible = true;
    }
    if let Some(mut ie) = input_enabled {
        ie.0 = false;
    }
}

pub fn transform_ui(
    value: &mut dyn std::any::Any,
    ui: &mut egui::Ui,
    options: &dyn std::any::Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let mut changed = false;
    if let Some(transform) = value.downcast_mut::<Transform>() {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Location");
                changed |= env.ui_for_reflect_with_options(&mut transform.translation, ui, id.with("loc"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Rotation");
                changed |= env.ui_for_reflect_with_options(&mut transform.rotation, ui, id.with("rot"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Scale");
                changed |= env.ui_for_reflect_with_options(&mut transform.scale, ui, id.with("scl"), options);
            });
        });
    }
    changed
}

pub fn transform_ui_readonly(
    value: &dyn std::any::Any,
    ui: &mut egui::Ui,
    options: &dyn std::any::Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) {
    if let Some(transform) = value.downcast_ref::<Transform>() {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Location");
                env.ui_for_reflect_readonly_with_options(&transform.translation, ui, id.with("loc"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Rotation");
                env.ui_for_reflect_readonly_with_options(&transform.rotation, ui, id.with("rot"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Scale");
                env.ui_for_reflect_readonly_with_options(&transform.scale, ui, id.with("scl"), options);
            });
        });
    }
}



pub fn transform_ui_many(
    _ui: &mut egui::Ui,
    _options: &dyn std::any::Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn bevy::reflect::PartialReflect],
    _projector: &dyn bevy_inspector_egui::reflect_inspector::ProjectorReflect,
) -> bool {
    false
}

pub fn global_transform_ui(
    value: &mut dyn std::any::Any,
    ui: &mut egui::Ui,
    options: &dyn std::any::Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    if let Some(global_transform) = value.downcast_mut::<GlobalTransform>() {
        let transform = global_transform.compute_transform();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Global Location");
                env.ui_for_reflect_readonly_with_options(&transform.translation, ui, id.with("g_loc"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Global Rotation");
                env.ui_for_reflect_readonly_with_options(&transform.rotation, ui, id.with("g_rot"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Global Scale");
                env.ui_for_reflect_readonly_with_options(&transform.scale, ui, id.with("g_scl"), options);
            });
        });
    }
    false
}

pub fn global_transform_ui_readonly(
    value: &dyn std::any::Any,
    ui: &mut egui::Ui,
    options: &dyn std::any::Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) {
    if let Some(global_transform) = value.downcast_ref::<GlobalTransform>() {
        let transform = global_transform.compute_transform();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Global Location");
                env.ui_for_reflect_readonly_with_options(&transform.translation, ui, id.with("g_loc"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Global Rotation");
                env.ui_for_reflect_readonly_with_options(&transform.rotation, ui, id.with("g_rot"), options);
            });
            ui.horizontal(|ui| {
                ui.label("Global Scale");
                env.ui_for_reflect_readonly_with_options(&transform.scale, ui, id.with("g_scl"), options);
            });
        });
    }
}

pub fn global_transform_ui_many(
    _ui: &mut egui::Ui,
    _options: &dyn std::any::Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn bevy::reflect::PartialReflect],
    _projector: &dyn bevy_inspector_egui::reflect_inspector::ProjectorReflect,
) -> bool {
    false
}






