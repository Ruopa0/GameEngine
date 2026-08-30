use bevy::prelude::*;

// ---------------------------------------------------------------------------------
// EDITOR MODULE ENTRY POINT
// This file defines the core Editor plugin for Code Blue. It acts as the central
// hub for all editor-related functionality, including the user interface (UI),
// picking (selecting objects with the mouse), gizmos (visual helpers like transform handles),
// and serialization (saving/loading scenes).
// ---------------------------------------------------------------------------------

pub mod camera;
pub mod console;
pub mod gizmos;
pub mod history;
pub mod icons;
pub mod picking;
pub mod serialization;
pub mod ui;
pub mod user_color;

/// The central State machine for the engine.
/// Bevy States allow us to run completely different logic depending on whether we
/// are currently editing a level, or actively playing it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EngineState {
    /// Active gameplay mode. Physics, AI, and player input are processed.
    Play,
    /// Level design mode. The game is paused, and the editor UI is active.
    #[default]
    Edit,
}

/// SystemSets allow us to group systems together and control the exact order they run.
/// For example, we want to update the Gizmos before we calculate what the user is picking.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorSet {
    GizmoUpdate,
    Picking,
}

/// The main plugin that initializes the entire editor environment.
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // 1. Initialize State and Resources
        app.init_state::<EngineState>()
            .init_resource::<PlayModeSnapshot>()
            // When we transition between Edit and Play modes, these systems will run ONCE
            .add_systems(OnEnter(EngineState::Play), on_enter_play_mode)
            .add_systems(OnEnter(EngineState::Edit), on_exit_play_mode)
            // 2. Configure System Ordering
            .configure_sets(Update, EditorSet::GizmoUpdate.before(EditorSet::Picking))
            // 3. Add Sub-Plugins
            .add_plugins((
                bevy_inspector_egui::DefaultInspectorConfigPlugin,
                camera::EditorCameraPlugin,
                ui::EditorUiPlugin,
                picking::EditorPickingPlugin,
                gizmos::EditorGizmosPlugin,
                serialization::EditorSerializationPlugin,
                console::EditorConsolePlugin,
                icons::EditorIconsPlugin,
                history::EditorHistoryPlugin,
            ))
            // 4. Run setup logic on Startup (when the engine first boots)
            .add_systems(
                Startup,
                |input_enabled: Option<ResMut<cb_input::InputEnabled>>,
                 app_type_registry: Res<AppTypeRegistry>| {
                    // We use bevy_inspector_egui to auto-generate UI for our components.
                    // However, Bevy's default Transform UI takes up a lot of space.
                    // Here, we override the default UI rendering for Transforms with our own custom layout.
                    let mut registry = app_type_registry.write();
                    if let Some(registration) =
                        registry.get_mut(std::any::TypeId::of::<Transform>())
                    {
                        registration.insert(
                            bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl::new(
                                ui::transform_ui,
                                ui::transform_ui_readonly,
                                ui::transform_ui_many,
                            ),
                        );
                    }

                    // Do the same override for GlobalTransform.
                    if let Some(registration) =
                        registry.get_mut(std::any::TypeId::of::<GlobalTransform>())
                    {
                        registration.insert(
                            bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl::new(
                                ui::global_transform_ui,
                                ui::global_transform_ui_readonly,
                                ui::global_transform_ui_many,
                            ),
                        );
                    }

                    // Disable player input when starting the editor (since we default to Edit mode)
                    if let Some(mut ie) = input_enabled {
                        ie.0 = false;
                    }
                },
            );
    }
}

/// A resource to hold an in-memory RON string of the scene right before Play mode started.
/// This allows us to instantly restore the scene state when returning to Edit mode.
#[derive(Resource, Default)]
pub struct PlayModeSnapshot(pub Option<String>);

/// A marker component used to identify entities that existed BEFORE Play mode started.
/// Anything without this component was spawned during gameplay (e.g., bullets) and should be deleted on Stop.
#[derive(Component)]
pub struct KeepOnStop;

fn on_enter_play_mode(
    world: &World,
    mut commands: Commands,
    q_objects: Query<(Entity, &Transform, &serialization::SceneObject, Option<&Name>)>,
    q_all: Query<Entity>,
) {
    // 1. Snapshot the world
    let type_registry = world.resource::<AppTypeRegistry>().read();
    let mut filter = bevy::scene::SceneFilter::deny_all();
    for registration in type_registry.iter() {
        if registration
            .data::<bevy::reflect::ReflectSerialize>()
            .is_some()
        {
            filter = filter.allow_by_id(registration.type_id());
        }
    }

    let mut builder = bevy::scene::DynamicSceneBuilder::from_world(world);
    builder = builder.with_component_filter(filter);

    let entities: Vec<Entity> = q_objects.iter().map(|(e, _, _, _)| e).collect();
    let scene = builder.extract_entities(entities.into_iter()).build();
    let serializer = bevy::scene::serde::SceneSerializer::new(&scene, &type_registry);

    let snapshot = match ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
    {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to serialize scene snapshot: {:?}", e);
            String::new()
        }
    };

    commands.insert_resource(PlayModeSnapshot(Some(snapshot)));

    for e in q_all.iter() {
        commands.entity(e).insert(KeepOnStop);
    }

    // 2. Spawn player
    let mut spawn_pos = Transform::from_xyz(0.0, 2.0, 5.0);
    let mut found_any = false;
    for (_, transform, obj, name_opt) in q_objects.iter() {
        if obj.object_type == "spawn_point" {
            if let Some(name) = name_opt {
                if name.as_str().to_lowercase().contains("start") {
                    spawn_pos = *transform;
                    break;
                }
            }
            if !found_any {
                spawn_pos = *transform;
                found_any = true;
            }
        }
    }
    crate::player::spawn_player(&mut commands, spawn_pos);
}

fn on_exit_play_mode(
    mut commands: Commands,
    q_temporary: Query<
        Entity,
        (
            Without<KeepOnStop>,
            Without<serialization::SceneObject>,
            Without<Window>,
            Without<Camera>,
        ),
    >,
    q_objects: Query<Entity, With<serialization::SceneObject>>,
    q_kept: Query<Entity, With<KeepOnStop>>,
    mut snapshot: ResMut<PlayModeSnapshot>,
    mut dynamic_scenes: ResMut<Assets<DynamicScene>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    type_registry: Res<AppTypeRegistry>,
) {
    // Only perform play mode cleanup if we actually entered play mode and took a snapshot!
    // This prevents despawning the Window / UI Cameras when booting into Edit mode on startup.
    let Some(ron_str) = snapshot.0.take() else {
        return;
    };

    // Despawn temporary entities spawned during play mode
    for entity in q_temporary.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn current scene objects to load clean snapshot
    for entity in q_objects.iter() {
        commands.entity(entity).despawn();
    }

    // Clean up KeepOnStop markers
    for entity in q_kept.iter() {
        commands.entity(entity).remove::<KeepOnStop>();
    }

    if ron_str.is_empty() {
        return;
    }

    let mut deserializer = match ron::de::Deserializer::from_str(&ron_str) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to parse memory snapshot: {}", e);
            return;
        }
    };

    let scene_deserializer = bevy::scene::serde::SceneDeserializer {
        type_registry: &type_registry.read(),
    };

    use serde::de::DeserializeSeed;
    match scene_deserializer.deserialize(&mut deserializer) {
        Ok(scene) => {
            let handle = dynamic_scenes.add(scene);
            scene_spawner.spawn_dynamic(handle);
            info!("Successfully restored scene from memory snapshot.");
        }
        Err(e) => {
            error!("Failed to deserialize memory snapshot: {}", e);
        }
    }
}
