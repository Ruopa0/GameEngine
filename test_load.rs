use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
       .add_plugins(avian3d::prelude::PhysicsPlugins::default())
       .add_plugins(cb_engine::EnginePlugin)
       .add_plugins(cb_engine::editor::serialization::EditorSerializationPlugin);

    let type_registry = app.world().resource::<AppTypeRegistry>().clone();
    let reg = type_registry.read();

    for path in &["level.ron", "target/debug/exampleMap.ron"] {
        if let Ok(data) = std::fs::read_to_string(path) {
            let mut deserializer = ron::de::Deserializer::from_str(&data).unwrap();
            let scene_deserializer = SceneDeserializer { type_registry: &reg };
            match scene_deserializer.deserialize(&mut deserializer) {
                Ok(scene) => println!("{}: OK ({} entities)", path, scene.entities.len()),
                Err(e) => println!("{}: ERROR: {}", path, e),
            }
        } else {
            println!("{}: file not found", path);
        }
    }
}
