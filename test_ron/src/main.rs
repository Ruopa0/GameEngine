use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Reflect, Clone, Debug, Copy, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EditorPointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub shadows_enabled: bool,
}

fn main() {
    let mut app = App::new();
    app.register_type::<EditorPointLight>();
    
    let mut world = app.world_mut();
    let e = world.spawn(EditorPointLight {
        color: Color::WHITE,
        intensity: 800.0,
        range: 20.0,
        radius: 0.0,
        shadows_enabled: false,
    }).id();
    
    let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry_arc.read();
    
    let entity_ref = world.entity(e);
    let registration = type_registry.get(std::any::TypeId::of::<EditorPointLight>()).unwrap();
    
    let reflect_comp = registration.data::<bevy::ecs::reflect::ReflectComponent>().unwrap();
    let reflected = reflect_comp.reflect(entity_ref).unwrap();
    
    let serializer = bevy::reflect::serde::TypedReflectSerializer::new(reflected, &type_registry);
    match ron::to_string(&serializer) {
        Ok(s) => println!("Serialized: {}", s),
        Err(e) => println!("Error: {:?}", e),
    }
}

