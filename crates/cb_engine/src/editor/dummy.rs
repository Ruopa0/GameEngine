use bevy::prelude::*;

pub fn dummy_save_scene(world: &World, mut events: MessageReader<crate::editor::serialization::SaveSceneEvent>) {
    let mut builder = bevy::scene::DynamicSceneBuilder::from_world(world);
}
