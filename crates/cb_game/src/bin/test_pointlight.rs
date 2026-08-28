use bevy::prelude::*;
use bevy::reflect::{ReflectSerialize, ReflectDeserialize};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    
    let registry = app.world().resource::<AppTypeRegistry>().read();
    let registration = registry.get(std::any::TypeId::of::<PointLight>()).unwrap();
    println!("PointLight ReflectSerialize: {}", registration.data::<ReflectSerialize>().is_some());
    println!("PointLight ReflectDeserialize: {}", registration.data::<ReflectDeserialize>().is_some());
}
