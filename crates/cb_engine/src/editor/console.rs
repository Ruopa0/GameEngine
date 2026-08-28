use bevy::prelude::*;

pub struct EditorConsolePlugin;

impl Plugin for EditorConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConsoleState>();
    }
}

#[derive(Resource)]
pub struct ConsoleState {
    pub logs: Vec<String>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            logs: vec!["Code Blue Engine initialized...".to_string()],
        }
    }
}

impl ConsoleState {
    pub fn push_log(&mut self, log: impl Into<String>) {
        self.logs.push(log.into());
    }
}
