use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: None,
        exit_condition: bevy::window::ExitCondition::DontExit,
        ..default()
    }));
    println!("PointLight test binary");
}
