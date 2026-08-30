// CodeBlue -- Gray-box test harness
// Wires together: Physics | Tnua KCC | Movement FSM | FPS camera + input

use bevy::{prelude::*, window::CursorGrabMode};
use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;


use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "CodeBlue")]
struct Cli {
    #[arg(long)]
    server: bool,

    #[arg(long)]
    client: bool,
}

fn trigger_system(
    mut events: bevy::prelude::MessageWriter<cb_engine::editor::serialization::GenerateCityEvent>,
    input: Res<ButtonInput<KeyCode>>
) {
    if input.just_pressed(KeyCode::KeyG) {
        events.write(cb_engine::editor::serialization::GenerateCityEvent);
        println!("Triggered GenerateCityEvent!");
    }
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    if cli.server {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            close_when_requested: false,
            ..default()
        }))
        .add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<cb_movement::kcc::CharacterScheme>::new(Update),
            TnuaAvian3dPlugin::new(Update),
                cb_engine::EnginePlugin,
                cb_movement::MovementPlugin,
                cb_weapons::WeaponsPlugin,
                cb_netcode::server::ServerNetPlugin,
                cb_netcode::NetcodePlugin,
            ))
            .add_systems(Startup, setup_level);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CodeBlue -- Editor".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::<cb_movement::kcc::CharacterScheme>::new(Update),
            TnuaAvian3dPlugin::new(Update),
            cb_engine::EnginePlugin,
            cb_input::InputPlugin,
            cb_movement::MovementPlugin,
            cb_weapons::WeaponsPlugin,
            cb_engine::editor::serialization::EditorSerializationPlugin,
        ));

        if cli.client {
            app.add_plugins((
                cb_netcode::client::ClientNetPlugin,
                cb_netcode::NetcodePlugin,
            ));
            // In client mode, level and player spawning will be handled via network replication,
            // but for now we spawn level statically.
            app.add_systems(Startup, setup_level);
        } else {
            // Standalone mode
            app.add_systems(Startup, setup_level);
        }

        app.add_plugins(cb_engine::player::PlayerPlugin); 
        app.add_plugins(bevy_egui::EguiPlugin::default());
        app.add_plugins(cb_engine::console::ConsolePlugin);
        app.add_systems(Update, (toggle_cursor_grab, trigger_system));
    }

    app.run();
}

// --- Startup ------------------------------------------------------------------

fn setup_level(
    mut load_events: MessageWriter<cb_engine::editor::serialization::LoadSceneEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    load_events.write(cb_engine::editor::serialization::LoadSceneEvent("level.ron".to_string()));

    // Fallback floor if scene is empty
    if !std::path::Path::new("level.ron").exists() {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.25, 0.25, 0.28))),
            Transform::from_xyz(0.0, -0.051, 0.0),
            RigidBody::Static,
            Collider::cuboid(100.0, 0.1, 100.0),
        ));
    }

    // --- Test obstacles (jump/vault targets) ---
    let box_mat = materials.add(Color::srgb(0.45, 0.45, 0.5));

    // Low box -- speed vault candidate (~0.6 m tall)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 0.6, 2.0))),
        MeshMaterial3d(box_mat.clone()),
        Transform::from_xyz(6.0, 0.3, -4.0),
        RigidBody::Static,
        Collider::cuboid(3.0, 0.6, 2.0),
    ));

    // Mid box -- mantle candidate (~1.2 m tall)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 1.2, 2.0))),
        MeshMaterial3d(box_mat.clone()),
        Transform::from_xyz(-6.0, 0.6, -4.0),
        RigidBody::Static,
        Collider::cuboid(3.0, 1.2, 2.0),
    ));

    // Tall Wall -- WallRun candidate
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 4.0, 8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(-6.0, 2.0, -10.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 4.0, 8.0),
    ));

    // Elevated platform with ramp approach
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 0.3, 8.0))),
        MeshMaterial3d(box_mat.clone()),
        Transform::from_xyz(0.0, 2.0, -18.0),
        RigidBody::Static,
        Collider::cuboid(8.0, 0.3, 8.0),
    ));

    // Ramp to elevated platform
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 0.1, 6.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.35))),
        Transform::from_xyz(0.0, 1.0, -13.0)
            .with_rotation(Quat::from_rotation_x(-0.32)), // ~18 deg slope
        RigidBody::Static,
        Collider::cuboid(3.0, 0.1, 6.0),
    ));

    // --- Target Dummies ---
    for i in 0..3 {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, 1.8, 0.6))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_xyz(3.0 + i as f32 * 2.0, 0.9, -10.0),
            RigidBody::Static,
            Collider::cuboid(0.6, 1.8, 0.6),
            cb_weapons::components::Health::new(100.0),
        ));
    }

    // --- Directional light ---
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 15_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
        )),
    ));

    // --- Player entity ---
    cb_engine::player::spawn_player(&mut commands, Transform::from_xyz(0.0, 2.0, 5.0));

    // --- HUD debug hint ---
    commands.spawn((
        Text::new("WASD -- Move  |  Space -- Jump  |  Shift -- Sprint  |  Esc -- Release cursor"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}


// --- Cursor Grab --------------------------------------------------------------

fn toggle_cursor_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<Window>>,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else { return };
    
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    if mouse.just_pressed(MouseButton::Left) && cursor.grab_mode == CursorGrabMode::None {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}





