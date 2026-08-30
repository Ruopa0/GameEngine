use bevy::prelude::*;
use lightyear::prelude::*;
use cb_movement::fsm::CharacterState;
use serde::{Deserialize, Serialize};

// --- Protocol Configuration ---------------------------------------------------

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerInputs {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub crouch: bool,
    pub pitch: f32,
    pub yaw: f32,
}

// In Lightyear 0.17, to use native inputs we register the input type as a Leafwing or Native input.
// We'll use Native input for simplicity.

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EnterPlayModeMessage;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExitPlayModeMessage;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SyncedObject {
    pub id: u64,
    pub name: Option<String>,
    pub object_type: String,
    pub asset_path: Option<String>,
    pub transform: Transform,
    pub lock_user_id: Option<u64>,
    pub components: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum EditorAction {
    RequestFullSync,
    FullSceneSync { objects: Vec<SyncedObject> },
    MoveObject { id: u64, transform: Transform, sender_user_id: u64 },
    SpawnObject { id: u64, object_type: String, asset_path: Option<String>, transform: Transform },
    DespawnObject { id: u64 },
    SaveScene { path: String },
    LoadScene { path: String, scene_ron: String },
    ClearScene,
    AddComponent { id: u64, type_path: String },
    RemoveComponent { id: u64, type_path: String },
    UpdateComponent { id: u64, type_path: String, ron_data: String },
    RenameObject { id: u64, name: String },
    ReparentObject { id: u64, parent_id: Option<u64> },
    LockObject { id: u64, user_id: u64 },
    UnlockObject { id: u64, user_id: u64 },
    UpdateEditorCamera { user_id: u64, transform: Transform },
    UpdatePlayerTransform { user_id: u64, transform: Transform, pitch: f32 },
    PlayerHit { victim_user_id: u64, attacker_user_id: u64, damage: f32, hit_point: Vec3, hit_normal: Vec3 },
    DamageNetworkObject { id: u64, damage: f32, hit_point: Vec3, hit_normal: Vec3 },
    PlayerFired { user_id: u64, origin: Vec3, direction: Vec3 },
    PlayerRespawned { user_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct PlayModeChannel;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register Components
        app.register_component::<CharacterState>();

        app.add_channel::<PlayModeChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        }).add_direction(NetworkDirection::Bidirectional);
        
        app.register_message::<EnterPlayModeMessage>().add_direction(NetworkDirection::Bidirectional);
        app.register_message::<ExitPlayModeMessage>().add_direction(NetworkDirection::Bidirectional);
        app.register_message::<EditorAction>().add_direction(NetworkDirection::Bidirectional);
        
        app.register_component::<cb_engine::player::Player>();
    }
}

