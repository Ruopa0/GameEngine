use bevy::prelude::*;
use std::fs;
use bevy::scene::serde::SceneSerializer;
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

// ---------------------------------------------------------------------------------
// SCENE SERIALIZATION LOGIC
// This file handles saving the 3D scene out to a file (like `level.ron`) and loading
// it back in. It heavily relies on Bevy's Reflection (`bevy::reflect`) system.
//
// Key Responsibilities:
// 1. Registering all components we want to be visible in the Inspector and saveable.
// 2. The `handle_save` system which converts the scene into text and writes to disk.
// 3. The `handle_load` system which reads the text, despawns the old scene, and spawns the new one.
// 4. Defining the `EditorActionRequest` event for networked multi-user editing.
// ---------------------------------------------------------------------------------

/// Plugin for handling scene saving, loading, and type registration.
pub struct EditorSerializationPlugin;

impl Plugin for EditorSerializationPlugin {
    fn build(&self, app: &mut App) {
        // Register events that can be sent over the network
        app.add_message::<SaveSceneEvent>()
           .add_message::<LoadSceneEvent>()
           .add_message::<ClearSceneEvent>()
           .add_message::<ConnectToServerEvent>()
           .add_message::<EditorActionRequest>()
           
           // Register custom components so they can be saved/loaded and shown in the Inspector
           .init_resource::<LocalEditorSession>()
           .init_resource::<ActiveSceneState>()
           .register_type::<SceneObject>()
           .register_type::<NetworkId>()
           .register_type::<EditorLock>()
           .register_type::<RemoteEditorCamera>()
           // Register standard components
           .register_type::<Name>()
           .register_type::<Transform>()
           
           // Register Avian3D physics components so they show up in the "Add Component" menu
           // and can be tweaked dynamically in the Inspector panel.
           .register_type::<avian3d::prelude::RigidBody>()
           .register_type::<avian3d::prelude::Friction>()
           .register_type::<avian3d::prelude::Restitution>()
           .register_type::<avian3d::prelude::GravityScale>()
           .register_type::<avian3d::prelude::Mass>()
           .register_type::<avian3d::prelude::LinearVelocity>()
           .register_type::<avian3d::prelude::AngularVelocity>()
           .register_type::<avian3d::prelude::LinearDamping>()
           .register_type::<avian3d::prelude::AngularDamping>()
           .register_type::<avian3d::prelude::LockedAxes>()
           .register_type::<PointLight>()
           .register_type::<crate::scripting::ScriptComponent>()
           .register_type::<crate::gamemode::TargetDummy>()
           .register_type::<crate::gamemode::GoalZone>()
           .register_type::<TestComponent>()
           .register_type::<cb_weapons::components::Health>()
           .register_type::<cb_weapons::health::ImmortalPlayer>()
           .register_type::<EditorColor>()
           .register_type::<EditorPointLight>()
           .register_type::<avian3d::prelude::RigidBody>()
           .register_type::<avian3d::prelude::Friction>()
           .register_type::<avian3d::prelude::Restitution>()
           .register_type::<avian3d::prelude::GravityScale>()
           .register_type::<avian3d::prelude::Mass>()
           .add_message::<GenerateCityEvent>()
           .add_systems(Update, (
                handle_save, 
                handle_load, 
                handle_clear_scene, 
                restore_missing_visuals, 
                apply_editor_color, 
                apply_editor_pointlight,
                handle_generate_city
            ));
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ActiveSceneState {
    pub current_path: Option<String>,
    pub is_dirty: bool,
}

impl Default for ActiveSceneState {
    fn default() -> Self {
        Self {
            current_path: Some("level.ron".to_string()),
            is_dirty: false,
        }
    }
}

impl ActiveSceneState {
    pub fn display_name(&self) -> String {
        match &self.current_path {
            Some(path) => {
                let p = std::path::Path::new(path);
                p.file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone())
            }
            None => "Untitled.ron".to_string(),
        }
    }
}

#[derive(Message, Debug, Clone)]
pub struct SaveSceneEvent(pub String);

#[derive(Message, Debug, Clone)]
pub struct LoadSceneEvent(pub String);

#[derive(Message, Debug, Clone)]
pub struct ClearSceneEvent;

#[derive(Message, Debug, Clone)]
pub struct GenerateCityEvent;

#[derive(Message, Debug)]
pub struct ConnectToServerEvent;

#[derive(Resource, Debug, Clone, Copy)]
pub struct LocalEditorSession {
    pub client_id: u64,
}

impl Default for LocalEditorSession {
    fn default() -> Self {
        Self {
            client_id: rand::random::<u64>(),
        }
    }
}

#[derive(Component, Reflect, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EditorLock {
    pub user_id: u64,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq)]
#[reflect(Component)]
pub struct RemoteEditorCamera {
    pub user_id: u64,
}

#[derive(Message, Debug, Clone)]
pub enum EditorActionRequest {
    MoveObject { id: u64, transform: Transform, sender_user_id: u64 },
    SpawnObject { id: u64, object_type: String, asset_path: Option<String>, transform: Transform },
    DespawnObject { id: u64 },
    AddComponent { id: u64, type_path: String },
    RemoveComponent { id: u64, type_path: String },
    UpdateComponent { id: u64, type_path: String, ron_data: String },
    RenameObject { id: u64, name: String },
    ReparentObject { id: u64, parent_id: Option<u64> },
    LockObject { id: u64, user_id: u64 },
    UnlockObject { id: u64, user_id: u64 },
    UpdateEditorCamera { user_id: u64, transform: Transform },
}

pub fn apply_component_ron(
    world: &mut World,
    entity: Entity,
    type_path: &str,
    ron_data: &str,
) -> Result<(), String> {
    let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry_arc.read();
    
    let registration = type_registry
        .iter()
        .find(|r| r.type_info().type_path() == type_path || r.type_info().type_path_table().short_path() == type_path)
        .ok_or_else(|| format!("Type {} not registered in TypeRegistry", type_path))?;
        
    let reflect_deser = registration
        .data::<bevy::reflect::ReflectDeserialize>()
        .ok_or_else(|| format!("Type {} has no ReflectDeserialize", type_path))?;
        
    let mut deserializer = ron::de::Deserializer::from_str(ron_data)
        .map_err(|e| format!("Failed to parse RON: {}", e))?;
        
    let reflect_value = reflect_deser
        .deserialize(&mut deserializer)
        .map_err(|e| format!("Failed to deserialize component {}: {}", type_path, e))?;
        
    let reflect_comp = registration
        .data::<bevy::ecs::reflect::ReflectComponent>()
        .ok_or_else(|| format!("Type {} has no ReflectComponent", type_path))?;
        
    reflect_comp.insert(&mut world.entity_mut(entity), reflect_value.as_ref(), &type_registry);
    Ok(())
}

pub fn add_default_component(
    world: &mut World,
    entity: Entity,
    type_path: &str,
) -> Result<(), String> {
    let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry_arc.read();
    
    let registration = type_registry
        .iter()
        .find(|r| r.type_info().type_path() == type_path || r.type_info().type_path_table().short_path() == type_path)
        .ok_or_else(|| format!("Type {} not registered in TypeRegistry", type_path))?;
        
    let reflect_default = registration
        .data::<bevy::reflect::std_traits::ReflectDefault>()
        .ok_or_else(|| format!("Type {} has no ReflectDefault", type_path))?;
        
    let default_val = reflect_default.default();
    
    let reflect_comp = registration
        .data::<bevy::ecs::reflect::ReflectComponent>()
        .ok_or_else(|| format!("Type {} has no ReflectComponent", type_path))?;
        
    reflect_comp.insert(&mut world.entity_mut(entity), default_val.as_ref(), &type_registry);
    Ok(())
}

pub fn remove_component_by_name(
    world: &mut World,
    entity: Entity,
    type_path: &str,
) -> Result<(), String> {
    let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry_arc.read();
    
    let registration = type_registry
        .iter()
        .find(|r| r.type_info().type_path() == type_path || r.type_info().type_path_table().short_path() == type_path)
        .ok_or_else(|| format!("Type {} not registered in TypeRegistry", type_path))?;
        
    let reflect_comp = registration
        .data::<bevy::ecs::reflect::ReflectComponent>()
        .ok_or_else(|| format!("Type {} has no ReflectComponent", type_path))?;
        
    reflect_comp.remove(&mut world.entity_mut(entity));
    Ok(())
}

#[derive(Component, Reflect, Default, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SceneObject {
    pub object_type: String, // "cube", "light", etc.
    pub asset_path: Option<String>,
}

#[derive(Component, Reflect, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[reflect(Component, Serialize, Deserialize)]
pub struct NetworkId(pub u64);


fn handle_save(
    world: &World,
    mut save_events: MessageReader<SaveSceneEvent>,
    q_objects: Query<Entity, With<SceneObject>>,
) {
    let mut path_to_save = None;
    for ev in save_events.read() {
        path_to_save = Some(ev.0.clone());
    }
    
    if let Some(path) = path_to_save {
        let type_registry = world.resource::<AppTypeRegistry>().read();
        let mut filter = bevy::scene::SceneFilter::deny_all();
        for registration in type_registry.iter() {
            if registration.data::<bevy::reflect::ReflectSerialize>().is_some() {
                filter = filter.allow_by_id(registration.type_id());
            }
        }
        
        let mut builder = bevy::scene::DynamicSceneBuilder::from_world(world);
        builder = builder.with_component_filter(filter);
        
        let scene = builder.extract_entities(q_objects.iter()).build();
        let serializer = SceneSerializer::new(&scene, &type_registry);
        
        match ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default()) {
            Ok(serialized) => {
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    if !parent.as_os_str().is_empty() {
                        let _ = fs::create_dir_all(parent);
                    }
                }
                if let Err(e) = fs::write(&path, serialized) {
                    error!("Failed to write scene file: {}", e);
                } else {
                    info!("Scene successfully saved to {}", path);
                }
            }
            Err(e) => error!("Failed to serialize scene: {}", e),
        }
    }
}

fn handle_load(
    mut events: MessageReader<LoadSceneEvent>,
    mut commands: Commands,
    q_existing: Query<Entity, With<SceneObject>>,
    mut dynamic_scenes: ResMut<Assets<DynamicScene>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    type_registry: Res<AppTypeRegistry>,
) {
    for ev in events.read() {
        let data = match fs::read_to_string(&ev.0) {
            Ok(d) => d,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    info!("Scene file {} not found (expected on first boot). Starting empty.", ev.0);
                } else if ev.0 != "editor_backup.ron" {
                    error!("Failed to read scene file: {} (Error: {})", ev.0, e);
                }
                continue;
            }
        };
        
        let mut deserializer = match ron::de::Deserializer::from_str(&data) {
            Ok(d) => d,
            Err(e) => { error!("Failed to parse scene RON: {}", e); continue; }
        };
        
        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry.read(),
        };
        
        match scene_deserializer.deserialize(&mut deserializer) {
            Ok(scene) => {
                // Clear existing
                for entity in q_existing.iter() {
                    commands.entity(entity).despawn();
                }
                
                let handle = dynamic_scenes.add(scene);
                scene_spawner.spawn_dynamic(handle);
                
                commands.insert_resource(ActiveSceneState {
                    current_path: Some(ev.0.clone()),
                    is_dirty: false,
                });
                
                info!("Scene successfully loaded from {}", ev.0);
            }
            Err(e) => {
                error!("Failed to deserialize scene: {}", e);
            }
        }
    }
}

fn handle_clear_scene(
    mut events: MessageReader<ClearSceneEvent>,
    mut commands: Commands,
    q_existing: Query<Entity, With<SceneObject>>,
) {
    for _ in events.read() {
        info!("Clearing current scene entities...");
        for entity in q_existing.iter() {
            commands.entity(entity).despawn();
        }
        commands.insert_resource(ActiveSceneState {
            current_path: None,
            is_dirty: false,
        });
    }
}

pub fn restore_missing_visuals(
    mut commands: Commands,
    q_objects: Query<(Entity, &SceneObject, Option<&EditorColor>), (Without<Mesh3d>, Without<EditorPointLight>, Without<SceneRoot>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, obj, color_opt) in q_objects.iter() {
        match obj.object_type.as_str() {
            "cube" => {
                let mesh = meshes.add(Cuboid::default());
                let material = materials.add(StandardMaterial::default());
                let mut e_cmds = commands.entity(entity);
                e_cmds.insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    avian3d::prelude::RigidBody::Static,
                    avian3d::prelude::Collider::cuboid(1.0, 1.0, 1.0),
                ));
                if color_opt.is_none() {
                    e_cmds.insert(EditorColor(Color::WHITE));
                }
            }
            "target_dummy" => {
                let mesh = meshes.add(Cuboid::new(0.6, 1.8, 0.6));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.2, 0.2),
                    ..default()
                });
                commands.entity(entity).insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    avian3d::prelude::RigidBody::Static,
                    avian3d::prelude::Collider::cuboid(0.6, 1.8, 0.6),
                    cb_weapons::components::Health::new(100.0),
                    crate::gamemode::TargetDummy,
                    EditorColor(Color::srgb(0.8, 0.2, 0.2)),
                ));
            }
            "light" => {
                commands.entity(entity).insert(EditorPointLight {
                    intensity: 1500.0,
                    shadows_enabled: true,
                    ..default()
                });
            }
            "spawn_point" => {
                let mesh = meshes.add(Cylinder::new(0.5, 0.1));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.8, 0.2),
                    ..default()
                });
                commands.entity(entity).insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                ));
            }
            "goal_zone" => {
                let mesh = meshes.add(Cylinder::new(1.5, 0.1));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.9, 0.7, 0.1),
                    emissive: LinearRgba::new(0.5, 0.4, 0.0, 1.0),
                    ..default()
                });
                commands.entity(entity).insert((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    avian3d::prelude::Collider::cylinder(1.5, 1.0),
                    avian3d::prelude::Sensor,
                    crate::gamemode::GoalZone,
                ));
            }
            "gltf" => {
                if let Some(ref path) = obj.asset_path {
                    let scene = asset_server.load(format!("{}#Scene0", path));
                    commands.entity(entity).insert(SceneRoot(scene));
                }
            }
            _ => {}
        }
    }
}



#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct TestComponent {
    pub speed: f32,
    pub name: String,
}

#[derive(Component, Reflect, Clone, Debug, Copy, serde::Serialize, serde::Deserialize)]
#[reflect(Component, Default, Serialize, Deserialize)]
pub struct EditorColor(pub Color);

impl Default for EditorColor {
    fn default() -> Self {
        Self(Color::WHITE)
    }
}


#[derive(Component, Reflect, Clone, Debug, Copy, serde::Serialize, serde::Deserialize)]
#[reflect(Component, Default, Serialize, Deserialize)]
pub struct EditorPointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub shadows_enabled: bool,
}

impl Default for EditorPointLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 800.0,
            range: 20.0,
            radius: 0.0,
            shadows_enabled: false,
        }
    }
}

pub fn apply_editor_pointlight(
    mut commands: Commands,
    mut q: Query<(Entity, &EditorPointLight, Option<&mut PointLight>), Changed<EditorPointLight>>,
) {
    for (entity, ed_light, light_opt) in q.iter_mut() {
        if let Some(mut light) = light_opt {
            light.color = ed_light.color;
            light.intensity = ed_light.intensity;
            light.range = ed_light.range;
            light.radius = ed_light.radius;
            light.shadows_enabled = ed_light.shadows_enabled;
        } else {
            let mut light = PointLight::default();
            light.color = ed_light.color;
            light.intensity = ed_light.intensity;
            light.range = ed_light.range;
            light.radius = ed_light.radius;
            light.shadows_enabled = ed_light.shadows_enabled;
            commands.entity(entity).insert(light);
        }
    }
}

pub fn apply_editor_color(
    q_colors: Query<(&EditorColor, &MeshMaterial3d<StandardMaterial>), Changed<EditorColor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (ed_color, mat_handle) in q_colors.iter() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = ed_color.0;
        }
    }
}

fn handle_generate_city(
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
        
        // Spawn 64-128 random colored buildings
        for i in 0..100 {
            let x = (rng.f32() * 200.0) - 100.0;
            let z = (rng.f32() * 200.0) - 100.0;
            let width = rng.f32() * 8.0 + 4.0;
            let depth = rng.f32() * 8.0 + 4.0;
            let height = rng.f32() * 30.0 + 10.0;
            
            let color = Color::srgb(rng.f32() * 0.8, rng.f32() * 0.8, rng.f32() * 0.8);
            
            commands.spawn((
                Name::new(format!("Building_{}", i)),
                SceneObject {
                    object_type: "cube".to_string(),
                    asset_path: None,
                },
                NetworkId(rand::random::<u64>()),
                Transform::from_xyz(x, height / 2.0, z)
                    .with_scale(Vec3::new(width, height, depth)),
                EditorColor(color),
            ));
        }

        // Spawn a large grey concrete floor
        commands.spawn((
            Name::new("CityFloor"),
            SceneObject {
                object_type: "cube".to_string(),
                asset_path: None,
            },
            NetworkId(rand::random::<u64>()),
            Transform::from_xyz(0.0, -0.5, 0.0)
                .with_scale(Vec3::new(250.0, 1.0, 250.0)),
            EditorColor(Color::srgb(0.3, 0.3, 0.32)),
        ));

        active_state.current_path = Some("assets/scenes/exampleMap.ron".to_string());
        active_state.is_dirty = true;
    }
}

