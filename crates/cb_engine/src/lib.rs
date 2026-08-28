#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
use bevy::prelude::*;

pub mod schedule;
pub mod editor;
pub mod player;
pub mod gamemode;
pub mod scripting;
pub mod vfx;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(schedule::SchedulePlugin)
           .add_plugins(scripting::ScriptingPlugin)
           .add_plugins(gamemode::GameModePlugin)
           .add_plugins(vfx::VfxPlugin);
    }
}



