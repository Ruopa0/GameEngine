use bevy::prelude::*;

pub mod schedule;
pub mod editor;
pub mod player;
pub mod gamemode;
pub mod scripting;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(schedule::SchedulePlugin)
           .add_plugins(scripting::ScriptingPlugin)
           .add_plugins(gamemode::GameModePlugin);
    }
}
