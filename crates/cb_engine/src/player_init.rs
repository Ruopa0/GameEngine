
use bevy::prelude::*;
use crate::player::Player;
use cb_movement::parkour::ParkourSense;
use bevy_tnua::prelude::TnuaConfig;

pub fn auto_add_player_components(
    mut commands: Commands,
    query: Query<Entity, (With<Player>, Without<ParkourSense>)>,
    mut configs: ResMut<Assets<cb_movement::kcc::CharacterSchemeConfig>>,
) {
    for entity in query.iter() {
        let handle = configs.add(cb_movement::kcc::CharacterSchemeConfig {
            basis: bevy_tnua::builtins::TnuaBuiltinWalkConfig::default(),
            jump: bevy_tnua::builtins::TnuaBuiltinJumpConfig::default(),
        });
        commands.entity(entity).insert((
            ParkourSense::default(),
            avian3d::prelude::SleepingDisabled,
            bevy_tnua::prelude::TnuaController::<cb_movement::kcc::CharacterScheme>::default(),
            TnuaConfig::<cb_movement::kcc::CharacterScheme>(handle),
        ));
    }
}
