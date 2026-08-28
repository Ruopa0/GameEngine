pub mod protocol;
pub mod client;
pub mod server;

use bevy::prelude::*;

pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(protocol::ProtocolPlugin);
    }
}

