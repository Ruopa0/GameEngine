use bevy::prelude::*;
use avian3d::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Code Blue - Editor".to_string(),
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }));

    // Add Physics
    app.add_plugins(PhysicsPlugins::default());

    // Add Egui
    app.add_plugins(bevy_egui::EguiPlugin::default());

    // Add Engine Core (Schedule)
    app.add_plugins(cb_engine::EnginePlugin);
    app.add_plugins(cb_engine::console::ConsolePlugin);
    
    // Add game logic plugins
    app.add_plugins((
        bevy_tnua::prelude::TnuaControllerPlugin::<cb_movement::kcc::CharacterScheme>::new(FixedUpdate),
        bevy_tnua_avian3d::TnuaAvian3dPlugin::new(FixedUpdate),
        cb_input::InputPlugin,
        cb_movement::MovementPlugin,
        cb_weapons::WeaponsPlugin,
        cb_engine::player::PlayerPlugin,
        cb_netcode::NetcodePlugin,
        cb_netcode::client::ClientNetPlugin,
    ));

    // Add Editor explicitly (since it was removed from EnginePlugin)
    app.add_plugins(cb_engine::editor::EditorPlugin);

    // Boot straight into Editor mode
    app.add_systems(Startup, setup_editor_state);


    app.run();
}

fn setup_editor_state(
    mut next_state: ResMut<NextState<cb_engine::editor::EngineState>>,
    mut load_events: MessageWriter<cb_engine::editor::serialization::LoadSceneEvent>,
) {
    next_state.set(cb_engine::editor::EngineState::Edit);
    load_events.write(cb_engine::editor::serialization::LoadSceneEvent("level.ron".to_string()));
}


