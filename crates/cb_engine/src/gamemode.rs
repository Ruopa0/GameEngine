use bevy::prelude::*;

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum MatchStatus {
    #[default]
    InProgress,
    Victory,
    Defeat,
}

#[derive(Resource, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct MatchState {
    pub status: MatchStatus,
    pub score: u32,
    pub target_score: u32,
    pub targets_remaining: u32,
    pub total_targets: u32,
    pub kills: u32,
    pub elapsed_seconds: f32,
    pub win_reason: String,
    pub lose_reason: String,
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            status: MatchStatus::InProgress,
            score: 0,
            target_score: 5,
            targets_remaining: 0,
            total_targets: 0,
            kills: 0,
            elapsed_seconds: 0.0,
            win_reason: "All mission objectives completed!".to_string(),
            lose_reason: "You were eliminated!".to_string(),
        }
    }
}

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct TargetDummy;

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct GoalZone;

#[derive(Message, Debug, Clone)]
pub struct TargetDestroyedEvent {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct ResetMatchEvent;

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MatchState>()
           .register_type::<MatchState>()
           .register_type::<TargetDummy>()
           .register_type::<GoalZone>()
           .add_message::<TargetDestroyedEvent>()
           .add_message::<ResetMatchEvent>()
           .add_systems(Update, (
               update_match_timer,
               check_player_death_and_void,
               check_target_count,
               handle_target_destroyed,
               check_goal_zone_collision,
               handle_reset_match,
           ).run_if(in_state(crate::editor::EngineState::Play)));
    }
}

fn update_match_timer(
    time: Res<Time>,
    mut match_state: ResMut<MatchState>,
) {
    if match_state.status == MatchStatus::InProgress {
        match_state.elapsed_seconds += time.delta_secs();
    }
}

fn check_player_death_and_void(
    q_player: Query<(&Transform, Option<&cb_weapons::components::Health>), With<crate::player::Player>>,
    mut match_state: ResMut<MatchState>,
) {
    if match_state.status != MatchStatus::InProgress {
        return;
    }
    for (tf, health_opt) in q_player.iter() {
        // Void fall check
        if tf.translation.y < -25.0 {
            match_state.status = MatchStatus::Defeat;
            match_state.lose_reason = "You fell into the void!".to_string();
            return;
        }
        // Health death check
        if let Some(health) = health_opt {
            if health.current <= 0.0 {
                match_state.status = MatchStatus::Defeat;
                match_state.lose_reason = "You were eliminated in combat!".to_string();
                return;
            }
        }
    }
}

fn check_target_count(
    q_targets: Query<Entity, With<TargetDummy>>,
    mut match_state: ResMut<MatchState>,
) {
    let count = q_targets.iter().count() as u32;
    match_state.targets_remaining = count;
    if count > match_state.total_targets {
        match_state.total_targets = count;
    }
    if match_state.total_targets > 0 && count == 0 && match_state.status == MatchStatus::InProgress {
        match_state.status = MatchStatus::Victory;
        match_state.win_reason = "All target dummies eliminated!".to_string();
    }
}

fn check_goal_zone_collision(
    q_player: Query<&Transform, With<crate::player::Player>>,
    q_goals: Query<&Transform, With<GoalZone>>,
    mut match_state: ResMut<MatchState>,
) {
    if match_state.status != MatchStatus::InProgress {
        return;
    }
    for player_tf in q_player.iter() {
        for goal_tf in q_goals.iter() {
            let dist = player_tf.translation.distance(goal_tf.translation);
            if dist < 2.0 {
                match_state.status = MatchStatus::Victory;
                match_state.win_reason = "Objective extraction zone reached!".to_string();
                return;
            }
        }
    }
}

fn handle_target_destroyed(
    mut events: MessageReader<TargetDestroyedEvent>,
    mut killed_events: MessageReader<cb_weapons::health::EntityKilledEvent>,
    mut match_state: ResMut<MatchState>,
) {
    for _ in events.read() {
        match_state.score += 100;
        match_state.kills += 1;
    }
    for _ in killed_events.read() {
        match_state.score += 100;
        match_state.kills += 1;
    }
}

fn handle_reset_match(
    mut events: MessageReader<ResetMatchEvent>,
    mut match_state: ResMut<MatchState>,
    mut q_player: Query<(&mut Transform, Option<&mut cb_weapons::components::Health>), With<crate::player::Player>>,
    q_spawn_points: Query<&Transform, (With<crate::editor::serialization::SceneObject>, Without<crate::player::Player>)>,
) {
    for _ in events.read() {
        match_state.status = MatchStatus::InProgress;
        match_state.elapsed_seconds = 0.0;
        match_state.score = 0;
        match_state.kills = 0;
        
        let spawn_pos = q_spawn_points.iter().next().map(|t| t.translation + Vec3::Y * 1.5).unwrap_or(Vec3::new(0.0, 2.0, 0.0));
        
        for (mut tf, health_opt) in q_player.iter_mut() {
            tf.translation = spawn_pos;
            tf.rotation = Quat::IDENTITY;
            if let Some(mut health) = health_opt {
                health.current = health.max;
            }
        }
    }
}
