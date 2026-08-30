#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::winit::WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                    ..default()
                }),
        )
        .add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<cb_movement::kcc::CharacterScheme>::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            cb_engine::EnginePlugin,
            cb_engine::editor::serialization::EditorSerializationPlugin,
            cb_movement::MovementPlugin,
            cb_weapons::WeaponsPlugin,
            cb_netcode::NetcodePlugin,
            cb_netcode::server::ServerNetPlugin,
        ))
        .add_systems(Startup, setup_server_level)
        .run();
}

fn setup_server_level(
    mut load_events: MessageWriter<cb_engine::editor::serialization::LoadSceneEvent>,
    mut commands: Commands,
) {
    load_events.write(cb_engine::editor::serialization::LoadSceneEvent("level.ron".to_string()));
    
    // --- Gray-box floor (Server-side physics only, no meshes/materials) ---
    commands.spawn((
        Transform::from_xyz(0.0, -0.051, 0.0),
        RigidBody::Static,
        Collider::cuboid(100.0, 0.1, 100.0),
    ));

    // The other blocks will be loaded from scene.ron now.
}


