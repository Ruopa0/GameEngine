use std::net::{Ipv4Addr, SocketAddr};
use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use serde::de::DeserializeSeed;

// ---------------------------------------------------------------------------------
// CLIENT NETWORKING LOGIC
// This file handles the networking for the Editor and Player instances (the clients).
// It connects the client to the local headless server.
//
// Key Responsibilities:
// 1. Connecting to the Lightyear server on port 5000.
// 2. Transmitting editor actions (like drag-and-drop spawns or moving an object)
//    over the network to the server so all other clients see the changes.
// 3. Ensuring that when the user clicks "Play", the server is notified so it can
//    broadcast that state to other clients.
// ---------------------------------------------------------------------------------

/// The main Bevy plugin that sets up client networking.
pub struct ClientNetPlugin;

impl Plugin for ClientNetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins::default());
        
        // Setup systems that run precisely when we change EngineState (Play vs Edit)
        app.add_systems(Update, connect_client);
        app.add_systems(OnEnter(cb_engine::editor::EngineState::Play), sync_play_mode_enter);
        app.add_systems(OnEnter(cb_engine::editor::EngineState::Edit), (sync_play_mode_exit, cleanup_remote_players_on_exit));
        
        // Continuous networking updates (sending our local actions to the server)
        app.add_systems(Update, (
            auto_reconnect,
            handle_play_mode_from_server, 
            handle_remote_editor_actions,
            request_initial_sync,
            replicate_local_player, 
            send_editor_actions, 
            broadcast_local_editor_camera,
            broadcast_local_player_transform,
            send_save_scene,
            send_load_scene,
            send_clear_scene,
        ));
    }
}

pub fn auto_reconnect(
    mut commands: Commands,
    q_disconnected: Query<Entity, Added<lightyear::prelude::Disconnected>>,
    mut connect_events: bevy::prelude::MessageWriter<cb_engine::editor::serialization::ConnectToServerEvent>,
) {
    for client_entity in q_disconnected.iter() {
        info!("Client connection dropped! Auto-reconnecting in next frame...");
        commands.entity(client_entity).despawn();
        connect_events.write(cb_engine::editor::serialization::ConnectToServerEvent);
    }
}

pub fn request_initial_sync(
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>, Added<MessageSender<crate::protocol::EditorAction>>>,
) {
    for mut sender in senders.iter_mut() {
        info!("Client: Requesting full initial scene sync from server...");
        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::RequestFullSync);
    }
}

fn connect_client(
    mut commands: Commands,
    mut events: bevy::prelude::MessageReader<cb_engine::editor::serialization::ConnectToServerEvent>,
    q_local_objects: Query<Entity, (With<cb_engine::editor::serialization::SceneObject>, Without<lightyear::prelude::Replicated>)>,
    q_existing_clients: Query<Entity, With<NetcodeClient>>,
) {
    for _ in events.read() {
        if !q_existing_clients.is_empty() {
            info!("A client connection already exists. Cleaning up previous connection...");
            for client_e in q_existing_clients.iter() {
                commands.entity(client_e).despawn();
            }
        }

        info!("Connecting to server and syncing server scene data...");
        for entity in q_local_objects.iter() {
            commands.entity(entity).despawn();
        }
        let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 5000);
        let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);

        let client_id = rand::random::<u64>();

        let auth = Authentication::Manual {
            server_addr,
            protocol_id: 0,
            private_key: [0; 32],
            client_id,
        };

        let netcode_config = NetcodeConfig::default();

        match NetcodeClient::new(auth, netcode_config) {
            Ok(netcode_client) => {
                let client_entity = commands
                    .spawn((
                        netcode_client,
                        UdpIo::default(),
                        LocalAddr(client_addr),
                    ))
                    .id();

                commands.trigger(Connect { entity: client_entity });
            }
            Err(e) => {
                error!("Failed to create NetcodeClient: {:?}", e);
            }
        }
    }
}

pub fn sync_play_mode_enter(
    mut senders_enter: Query<&mut MessageSender<crate::protocol::EnterPlayModeMessage>>,
) {
    info!("sync_play_mode_enter: checking senders. Found: {}", senders_enter.iter().count());
    for mut sender in senders_enter.iter_mut() {
        info!("sync_play_mode_enter: sending EnterPlayModeMessage!");
        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EnterPlayModeMessage);
    }
}

pub fn sync_play_mode_exit(
    mut senders_exit: Query<&mut MessageSender<crate::protocol::ExitPlayModeMessage>>,
) {
    for mut sender in senders_exit.iter_mut() {
        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::ExitPlayModeMessage);
    }
}

pub fn handle_play_mode_from_server(
    mut receivers_enter: Query<&mut MessageReceiver<crate::protocol::EnterPlayModeMessage>>,
    mut receivers_exit: Query<&mut MessageReceiver<crate::protocol::ExitPlayModeMessage>>,
    mut next_state: ResMut<NextState<cb_engine::editor::EngineState>>,
    mut dialogs: ResMut<cb_engine::editor::ui::EditorUiDialogs>,
    current_state: Res<State<cb_engine::editor::EngineState>>,
) {
    let is_connected = !receivers_enter.is_empty();
    if !is_connected { return; }
    for mut receiver in receivers_enter.iter_mut() {
        if receiver.receive().next().is_some()
            && *current_state.get() != cb_engine::editor::EngineState::Play {
                dialogs.show_join_play = true;
            }
    }
    for mut receiver in receivers_exit.iter_mut() {
        if receiver.receive().next().is_some()
            && *current_state.get() != cb_engine::editor::EngineState::Edit {
                next_state.set(cb_engine::editor::EngineState::Edit);
            }
    }
}

pub fn replicate_local_player(
    mut commands: Commands, 
    query: Query<Entity, (Added<cb_engine::player::Player>, Without<lightyear::prelude::Replicated>)>,
    senders: Query<&MessageSender<crate::protocol::EditorAction>>,
) {
    if !senders.is_empty() {
        for entity in query.iter() {
            commands.entity(entity).insert(lightyear::prelude::Replicate::to_server());
        }
    }
}


pub fn send_editor_actions(
    mut commands: Commands,
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
    mut action_requests: bevy::prelude::MessageReader<cb_engine::editor::serialization::EditorActionRequest>,
) {
    let is_connected = !senders.is_empty();
    
    for request in action_requests.read() {
        if is_connected {
            for mut sender in senders.iter_mut() {
                let msg = match request {
                    cb_engine::editor::serialization::EditorActionRequest::MoveObject { id, transform, sender_user_id } => {
                        crate::protocol::EditorAction::MoveObject { id: *id, transform: *transform, sender_user_id: *sender_user_id }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::SpawnObject { id, object_type, asset_path, transform } => {
                        crate::protocol::EditorAction::SpawnObject { id: *id, object_type: object_type.clone(), asset_path: asset_path.clone(), transform: *transform }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::DespawnObject { id } => {
                        crate::protocol::EditorAction::DespawnObject { id: *id }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::AddComponent { id, type_path } => {
                        crate::protocol::EditorAction::AddComponent { id: *id, type_path: type_path.clone() }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::RemoveComponent { id, type_path } => {
                        crate::protocol::EditorAction::RemoveComponent { id: *id, type_path: type_path.clone() }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::UpdateComponent { id, type_path, ron_data } => {
                        crate::protocol::EditorAction::UpdateComponent { id: *id, type_path: type_path.clone(), ron_data: ron_data.clone() }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::RenameObject { id, name } => {
                        crate::protocol::EditorAction::RenameObject { id: *id, name: name.clone() }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::ReparentObject { id, parent_id } => {
                        crate::protocol::EditorAction::ReparentObject { id: *id, parent_id: *parent_id }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::LockObject { id, user_id } => {
                        crate::protocol::EditorAction::LockObject { id: *id, user_id: *user_id }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::UnlockObject { id, user_id } => {
                        crate::protocol::EditorAction::UnlockObject { id: *id, user_id: *user_id }
                    }
                    cb_engine::editor::serialization::EditorActionRequest::UpdateEditorCamera { user_id, transform } => {
                        crate::protocol::EditorAction::UpdateEditorCamera { user_id: *user_id, transform: *transform }
                    }
                };
                sender.send::<crate::protocol::PlayModeChannel>(msg);
            }
        }
        
        // Client Prediction: apply locally regardless of connection status!
        if let cb_engine::editor::serialization::EditorActionRequest::SpawnObject { id, object_type, asset_path, transform } = request {
            commands.spawn((
                cb_engine::editor::serialization::SceneObject { object_type: object_type.clone(), asset_path: asset_path.clone() },
                cb_engine::editor::serialization::NetworkId(*id),
                *transform,
                GlobalTransform::default(),
            ));
        }
    }
}

pub fn handle_remote_editor_actions(
    mut commands: Commands,
    mut receivers: Query<&mut MessageReceiver<crate::protocol::EditorAction>>,
    mut query_objects: Query<(Entity, &cb_engine::editor::serialization::NetworkId, Option<&mut Transform>)>,
    session: Res<cb_engine::editor::serialization::LocalEditorSession>,
) {
    for mut receiver in receivers.iter_mut() {
        while let Some(message) = receiver.receive().next() {
            match message {
                crate::protocol::EditorAction::RequestFullSync => {}
                crate::protocol::EditorAction::FullSceneSync { objects } => {
                    info!("Client: Received FullSceneSync with {} objects", objects.len());
                    for (entity, _, _) in query_objects.iter() {
                        commands.entity(entity).despawn();
                    }
                    for obj in objects {
                        let mut entity_cmds = commands.spawn((
                            cb_engine::editor::serialization::SceneObject {
                                object_type: obj.object_type,
                                asset_path: obj.asset_path,
                            },
                            cb_engine::editor::serialization::NetworkId(obj.id),
                            obj.transform,
                            GlobalTransform::default(),
                        ));
                        if let Some(name) = obj.name {
                            entity_cmds.insert(Name::new(name));
                        }
                        if let Some(lock_user_id) = obj.lock_user_id {
                            entity_cmds.insert(cb_engine::editor::serialization::EditorLock { user_id: lock_user_id });
                        }
                        let entity_id = entity_cmds.id();
                        for (type_path, ron_data) in obj.components {
                            commands.queue(move |world: &mut World| {
                                let _ = cb_engine::editor::serialization::apply_component_ron(world, entity_id, &type_path, &ron_data);
                            });
                        }
                    }
                }
                crate::protocol::EditorAction::MoveObject { id, transform, sender_user_id } => {
                    if sender_user_id == session.client_id {
                        continue;
                    }
                    for (_, net_id, mut obj_transform_opt) in query_objects.iter_mut() {
                        if net_id.0 == id
                            && let Some(ref mut obj_transform) = obj_transform_opt {
                                **obj_transform = transform;
                            }
                    }
                }
                crate::protocol::EditorAction::SpawnObject { id, object_type, asset_path, transform } => {
                    let mut exists = false;
                    for (_, net_id, _) in query_objects.iter() {
                        if net_id.0 == id {
                            exists = true;
                            break;
                        }
                    }
                    if !exists {
                        commands.spawn((
                            cb_engine::editor::serialization::SceneObject { object_type, asset_path },
                            cb_engine::editor::serialization::NetworkId(id),
                            transform,
                            GlobalTransform::default(),
                        ));
                    }
                }
                crate::protocol::EditorAction::DespawnObject { id } => {
                    for (entity, net_id, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).despawn();
                        }
                    }
                }
                crate::protocol::EditorAction::AddComponent { id, type_path } => {
                    commands.queue(move |world: &mut World| {
                        let mut target_entity = None;
                        let mut q = world.query::<(Entity, &cb_engine::editor::serialization::NetworkId)>();
                        for (e, nid) in q.iter(world) {
                            if nid.0 == id {
                                target_entity = Some(e);
                                break;
                            }
                        }
                        if let Some(e) = target_entity {
                            let _ = cb_engine::editor::serialization::add_default_component(world, e, &type_path);
                        }
                    });
                }
                crate::protocol::EditorAction::RemoveComponent { id, type_path } => {
                    commands.queue(move |world: &mut World| {
                        let mut target_entity = None;
                        let mut q = world.query::<(Entity, &cb_engine::editor::serialization::NetworkId)>();
                        for (e, nid) in q.iter(world) {
                            if nid.0 == id {
                                target_entity = Some(e);
                                break;
                            }
                        }
                        if let Some(e) = target_entity {
                            let _ = cb_engine::editor::serialization::remove_component_by_name(world, e, &type_path);
                        }
                    });
                }
                crate::protocol::EditorAction::UpdateComponent { id, type_path, ron_data } => {
                    commands.queue(move |world: &mut World| {
                        let mut target_entity = None;
                        let mut q = world.query::<(Entity, &cb_engine::editor::serialization::NetworkId)>();
                        for (e, nid) in q.iter(world) {
                            if nid.0 == id {
                                target_entity = Some(e);
                                break;
                            }
                        }
                        if let Some(e) = target_entity {
                            let _ = cb_engine::editor::serialization::apply_component_ron(world, e, &type_path, &ron_data);
                        }
                    });
                }
                crate::protocol::EditorAction::RenameObject { id, name } => {
                    for (entity, net_id, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).insert(Name::new(name.clone()));
                        }
                    }
                }
                crate::protocol::EditorAction::ReparentObject { id, parent_id } => {
                    commands.queue(move |world: &mut World| {
                        let mut child_entity = None;
                        let mut parent_entity = None;
                        let mut q = world.query::<(Entity, &cb_engine::editor::serialization::NetworkId)>();
                        for (e, nid) in q.iter(world) {
                            if nid.0 == id {
                                child_entity = Some(e);
                            }
                            if let Some(pid) = parent_id
                                && nid.0 == pid {
                                    parent_entity = Some(e);
                                }
                        }
                        if let Some(child) = child_entity {
                            if let Some(parent) = parent_entity {
                                world.entity_mut(child).set_parent_in_place(parent);
                            } else {
                                world.entity_mut(child).remove_parent_in_place();
                            }
                        }
                    });
                }
                crate::protocol::EditorAction::LockObject { id, user_id } => {
                    for (entity, net_id, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).insert(cb_engine::editor::serialization::EditorLock { user_id });
                        }
                    }
                }
                crate::protocol::EditorAction::UnlockObject { id, user_id: _ } => {
                    for (entity, net_id, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).remove::<cb_engine::editor::serialization::EditorLock>();
                        }
                    }
                }
                crate::protocol::EditorAction::LoadScene { path, scene_ron } => {
                    info!("Client: Received LoadScene for '{}' ({} bytes)", path, scene_ron.len());
                    for (entity, _, _) in query_objects.iter() {
                        commands.entity(entity).despawn();
                    }
                    commands.insert_resource(cb_engine::editor::serialization::ActiveSceneState {
                        current_path: Some(path.clone()),
                        is_dirty: false,
                    });
                    let scene_ron_clone = scene_ron.clone();
                    commands.queue(move |world: &mut World| {
                        let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
                        let type_registry = type_registry_arc.read();
                        if let Ok(mut deserializer) = ron::de::Deserializer::from_str(&scene_ron_clone) {
                            let scene_deserializer = bevy::scene::serde::SceneDeserializer {
                                type_registry: &type_registry,
                            };
                            if let Ok(scene) = scene_deserializer.deserialize(&mut deserializer) {
                                let mut dynamic_scenes = world.resource_mut::<Assets<DynamicScene>>();
                                let handle = dynamic_scenes.add(scene);
                                let mut scene_spawner = world.resource_mut::<SceneSpawner>();
                                scene_spawner.spawn_dynamic(handle);
                            }
                        }
                    });
                }
                crate::protocol::EditorAction::ClearScene => {
                    info!("Client: Received ClearScene from server");
                    for (entity, _, _) in query_objects.iter() {
                        commands.entity(entity).despawn();
                    }
                    commands.insert_resource(cb_engine::editor::serialization::ActiveSceneState {
                        current_path: None,
                        is_dirty: false,
                    });
                }
                crate::protocol::EditorAction::UpdateEditorCamera { user_id, transform } => {
                    if user_id != session.client_id {
                        commands.queue(move |world: &mut World| {
                            let mut target = None;
                            let mut q = world.query::<(Entity, &cb_engine::editor::serialization::RemoteEditorCamera)>();
                            for (e, cam) in q.iter(world) {
                                if cam.user_id == user_id {
                                    target = Some(e);
                                    break;
                                }
                            }
                            if let Some(e) = target {
                                if let Some(mut tf) = world.get_mut::<Transform>(e) {
                                    *tf = transform;
                                }
                            } else {
                                world.spawn((
                                    cb_engine::editor::serialization::RemoteEditorCamera { user_id },
                                    transform,
                                    GlobalTransform::default(),
                                ));
                            }
                        });
                    }
                }
                crate::protocol::EditorAction::UpdatePlayerTransform { user_id, transform, pitch }
                    if user_id != session.client_id => {
                        commands.queue(move |world: &mut World| {
                            let mut target = None;
                            let mut q = world.query::<(Entity, &mut cb_engine::player::RemotePlayer)>();
                            for (e, mut rp) in q.iter_mut(world) {
                                if rp.user_id == user_id {
                                    rp.pitch = pitch;
                                    target = Some(e);
                                    break;
                                }
                            }
                            if let Some(e) = target {
                                if let Some(mut tf) = world.get_mut::<Transform>(e) {
                                    *tf = transform;
                                }
                            } else {
                                world.spawn((
                                    cb_engine::player::RemotePlayer { user_id, pitch },
                                    transform,
                                    GlobalTransform::default(),
                                ));
                            }
                        });
                    }
                _ => {}
            }
        }
    }
}

pub fn broadcast_local_editor_camera(
    session: Res<cb_engine::editor::serialization::LocalEditorSession>,
    q_camera: Query<&Transform, (With<cb_engine::editor::camera::EditorCamera>, Changed<Transform>)>,
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
) {
    if let Ok(transform) = q_camera.single() {
        for mut sender in senders.iter_mut() {
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UpdateEditorCamera {
                user_id: session.client_id,
                transform: *transform,
            });
        }
    }
}

pub fn broadcast_local_player_transform(
    session: Res<cb_engine::editor::serialization::LocalEditorSession>,
    q_player: Query<&Transform, With<cb_engine::player::Player>>,
    q_camera: Query<&Transform, With<cb_engine::player::PlayerCamera>>,
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
) {
    if let Ok(player_tf) = q_player.single() {
        let pitch = if let Ok(cam_tf) = q_camera.single() {
            cam_tf.rotation.to_euler(EulerRot::YXZ).1
        } else {
            0.0
        };
        for mut sender in senders.iter_mut() {
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UpdatePlayerTransform {
                user_id: session.client_id,
                transform: *player_tf,
                pitch,
            });
        }
    }
}

pub fn cleanup_remote_players_on_exit(
    mut commands: Commands,
    q_remote: Query<Entity, With<cb_engine::player::RemotePlayer>>,
) {
    for e in q_remote.iter() {
        commands.entity(e).despawn();
    }
}

pub fn send_save_scene(
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
    mut save_events: bevy::prelude::MessageReader<cb_engine::editor::serialization::SaveSceneEvent>,
) {
    for mut sender in senders.iter_mut() {
        for ev in save_events.read() {
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::SaveScene {
                path: ev.0.clone(),
            });
        }
    }
}

pub fn send_load_scene(
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
    mut load_events: bevy::prelude::MessageReader<cb_engine::editor::serialization::LoadSceneEvent>,
) {
    for mut sender in senders.iter_mut() {
        for ev in load_events.read() {
            if let Ok(data) = std::fs::read_to_string(&ev.0) {
                sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::LoadScene {
                    path: ev.0.clone(),
                    scene_ron: data,
                });
            }
        }
    }
}

pub fn send_clear_scene(
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
    mut clear_events: bevy::prelude::MessageReader<cb_engine::editor::serialization::ClearSceneEvent>,
) {
    for mut sender in senders.iter_mut() {
        for _ in clear_events.read() {
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::ClearScene);
        }
    }
}
