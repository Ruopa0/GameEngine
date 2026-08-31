use std::net::{Ipv4Addr, SocketAddr};
use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::server::*;
use serde::de::DeserializeSeed;

pub struct ServerNetPlugin;

impl Plugin for ServerNetPlugin {
    fn build(&self, app: &mut App) {
        // Lightyear server plugins
        app.add_plugins(ServerPlugins::default());
        
        // Enforce headless 120Hz tick rate
        app.add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_secs_f64(1.0 / 120.0),
        ));
        
        app.add_systems(Startup, start_server);
        
        app.add_systems(Update, (
            relay_play_mode_enter,
            relay_play_mode_exit,
            replicate_players_to_clients,
            handle_editor_actions,
            
            periodic_snapshot_save,
            handle_client_disconnect,
        ));
    }
}

fn start_server(mut commands: Commands) {
    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5000);

    let netcode_config = NetcodeConfig::default()
        .with_protocol_id(0)
        .with_key([0; 32]);

    let server_entity = commands
        .spawn((
            NetcodeServer::new(netcode_config),
            ServerUdpIo::default(),
            LocalAddr(server_addr),
        ))
        .id();

    commands.trigger(Start { entity: server_entity });
}

pub fn replicate_players_to_clients(
    mut commands: Commands, 
    query: Query<Entity, Added<cb_engine::player::Player>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(lightyear::prelude::Replicate::to_clients(lightyear::prelude::NetworkTarget::All));
    }
}

pub fn relay_play_mode_enter(
    _commands: Commands,
    mut receivers: Query<&mut MessageReceiver<crate::protocol::EnterPlayModeMessage>>,
    mut senders: Query<(Entity, &lightyear::prelude::Client, &mut MessageSender<crate::protocol::EnterPlayModeMessage>)>,
) {
    let mut any_received = false;
    for mut receiver in receivers.iter_mut() {
        if receiver.receive().next().is_some() {
            info!("Server: Received EnterPlayModeMessage from a client!");
            any_received = true;
        }
    }
    if any_received {
        info!("Server: Broadcasting EnterPlayModeMessage and spawning Players for {} clients", senders.iter().count());
        for (entity, _client, mut sender) in senders.iter_mut() {
            let _client_id = entity.to_bits();
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EnterPlayModeMessage);
        }
    }
}

pub fn relay_play_mode_exit(
    mut receivers: Query<&mut MessageReceiver<crate::protocol::ExitPlayModeMessage>>,
    mut senders: Query<(Entity, &mut MessageSender<crate::protocol::ExitPlayModeMessage>)>,
    mut load_events: MessageWriter<cb_engine::editor::serialization::LoadSceneEvent>,
) {
    let mut any_received = false;
    for mut receiver in receivers.iter_mut() {
        if receiver.receive().next().is_some() {
            any_received = true;
        }
    }
    if any_received {
        load_events.write(cb_engine::editor::serialization::LoadSceneEvent("level.ron".to_string()));
        
        for (_entity, mut sender) in senders.iter_mut() {
            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::ExitPlayModeMessage);
        }
    }
}

pub fn handle_editor_actions(
    mut commands: Commands,
    mut receivers: Query<(Entity, &mut MessageReceiver<crate::protocol::EditorAction>)>,
    mut senders: Query<(Entity, &mut MessageSender<crate::protocol::EditorAction>)>,
    mut query_objects: Query<(Entity, &cb_engine::editor::serialization::NetworkId, &mut Transform, Option<&cb_engine::editor::serialization::SceneObject>, Option<&cb_engine::editor::serialization::EditorLock>)>,
) {
    for (receiver_entity, mut receiver) in receivers.iter_mut() {
        for action in receiver.receive() {
            match action {
                crate::protocol::EditorAction::SpawnObject { id, object_type, asset_path, transform } => {
                    info!("Server: Received SpawnObject id={} type={}", id, object_type);
                    let _entity = commands.spawn((
                        cb_engine::editor::serialization::SceneObject { object_type: object_type.clone(), asset_path: asset_path.clone() },
                        cb_engine::editor::serialization::NetworkId(id),
                        transform,
                        GlobalTransform::default(),
                    ));
                    
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::SpawnObject { id, object_type: object_type.clone(), asset_path: asset_path.clone(), transform });
                    }
                },
                crate::protocol::EditorAction::MoveObject { id, transform, sender_user_id } => {
                    let mut found = false;
                    for (_entity, net_id, mut tf, _scene_obj, _lock) in query_objects.iter_mut() {
                        if net_id.0 == id {
                            *tf = transform;
                            found = true;
                            break;
                        }
                    }
                    if !found { warn!("Server: Got move for unknown id {}", id); }
                    
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::MoveObject { id, transform, sender_user_id });
                    }
                },
                crate::protocol::EditorAction::DespawnObject { id } => {
                    for (entity, net_id, _, _, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).despawn();
                            break;
                        }
                    }
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::DespawnObject { id });
                    }
                },
                crate::protocol::EditorAction::LockObject { id, user_id } => {
                    for (entity, net_id, _, _, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).insert(cb_engine::editor::serialization::EditorLock { user_id });
                            break;
                        }
                    }
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::LockObject { id, user_id });
                    }
                },
                crate::protocol::EditorAction::UnlockObject { id, user_id } => {
                    for (entity, net_id, _, _, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).remove::<cb_engine::editor::serialization::EditorLock>();
                            break;
                        }
                    }
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UnlockObject { id, user_id });
                    }
                },
                crate::protocol::EditorAction::AddComponent { id, type_path } => {
                    let type_path_clone = type_path.clone();
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
                            let _ = cb_engine::editor::serialization::add_default_component(world, e, &type_path_clone);
                        }
                    });
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::AddComponent { id, type_path: type_path.clone() });
                    }
                },
                crate::protocol::EditorAction::RemoveComponent { id, type_path } => {
                    let type_path_clone = type_path.clone();
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
                            let _ = cb_engine::editor::serialization::remove_component_by_name(world, e, &type_path_clone);
                        }
                    });
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::RemoveComponent { id, type_path: type_path.clone() });
                    }
                },
                crate::protocol::EditorAction::UpdateComponent { id, type_path, ron_data } => {
                    let type_path_clone = type_path.clone();
                    let ron_data_clone = ron_data.clone();
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
                            let _ = cb_engine::editor::serialization::apply_component_ron(world, e, &type_path_clone, &ron_data_clone);
                        }
                    });
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UpdateComponent { id, type_path: type_path.clone(), ron_data: ron_data.clone() });
                    }
                },
                crate::protocol::EditorAction::RenameObject { id, name } => {
                    for (entity, net_id, _, _, _) in query_objects.iter() {
                        if net_id.0 == id {
                            commands.entity(entity).insert(Name::new(name.clone()));
                            break;
                        }
                    }
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::RenameObject { id, name: name.clone() });
                    }
                },
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
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::ReparentObject { id, parent_id });
                    }
                },
                crate::protocol::EditorAction::UpdateEditorCamera { user_id, transform } => {
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UpdateEditorCamera { user_id, transform });
                    }
                },
                crate::protocol::EditorAction::UpdatePlayerTransform { user_id, transform, pitch, active_weapon } => {
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UpdatePlayerTransform { user_id, transform, pitch, active_weapon });
                    }
                },
                crate::protocol::EditorAction::PlayerHit { victim_user_id, attacker_user_id, damage, hit_point, hit_normal } => {
                    info!("Server: Relaying PlayerHit (victim={}, attacker={}, damage={})", victim_user_id, attacker_user_id, damage);
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::PlayerHit {
                            victim_user_id,
                            attacker_user_id,
                            damage,
                            hit_point,
                            hit_normal,
                        });
                    }
                },
                crate::protocol::EditorAction::DamageNetworkObject { id, damage, hit_point, hit_normal } => {
                    info!("Server: Relaying DamageNetworkObject (id={}, damage={})", id, damage);
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::DamageNetworkObject {
                            id,
                            damage,
                            hit_point,
                            hit_normal,
                        });
                    }
                },
                crate::protocol::EditorAction::PlayerFired { user_id, origin, direction, projectile_speed } => {
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::PlayerFired {
                            user_id,
                            origin,
                            direction,
                            projectile_speed,
                        });
                    }
                },
                crate::protocol::EditorAction::PlayerRespawned { user_id } => {
                    for (_sender_entity, mut sender) in senders.iter_mut() {
                        sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::PlayerRespawned {
                            user_id,
                        });
                    }
                },
                crate::protocol::EditorAction::RequestFullSync => {
                    commands.queue(move |world: &mut World| {
                        let type_registry_arc = world.resource::<AppTypeRegistry>().0.clone();
                        let type_registry = type_registry_arc.read();
                        
                        let mut sync_objects = Vec::new();
                        let mut query = world.query::<(Entity, &cb_engine::editor::serialization::NetworkId, &Transform, Option<&cb_engine::editor::serialization::SceneObject>, Option<&cb_engine::editor::serialization::EditorLock>, Option<&Name>)>();
                        
                        for (entity, net_id, transform, scene_obj, lock, name) in query.iter(world) {
                            if let Some(scene_obj) = scene_obj {
                                let mut components = Vec::new();
                                if let Ok(entity_ref) = world.get_entity(entity) {
                                    for registration in type_registry.iter() {
                                        if let Some(reflect_comp) = registration.data::<bevy::ecs::reflect::ReflectComponent>()
                                            && let Some(reflected) = reflect_comp.reflect(entity_ref)
                                                && let Some(_reflect_ser) = registration.data::<bevy::reflect::ReflectSerialize>() {
                                                    let serializer = bevy::reflect::serde::ReflectSerializer::new(reflected, &type_registry);
                                                    if let Ok(ron_str) = ron::to_string(&serializer) {
                                                        components.push((registration.type_info().type_path().to_string(), ron_str));
                                                    }
                                                }
                                    }
                                }

                                sync_objects.push(crate::protocol::SyncedObject {
                                    id: net_id.0,
                                    name: name.map(|n| n.as_str().to_string()),
                                    object_type: scene_obj.object_type.clone(),
                                    asset_path: scene_obj.asset_path.clone(),
                                    transform: *transform,
                                    lock_user_id: lock.map(|l| l.user_id),
                                    components,
                                });
                            }
                        }

                        let sync_count = sync_objects.len();
                        if let Some(mut sender) = world.get_mut::<MessageSender<crate::protocol::EditorAction>>(receiver_entity) {
                            if sync_count > 0 {
                                info!("Server: Sending FullSceneSync ({} objects with all components) to client {}", sync_count, receiver_entity.to_bits());
                                sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::FullSceneSync {
                                    objects: sync_objects,
                                });
                            } else if std::path::Path::new("level.ron").exists()
                                && let Ok(scene_ron) = std::fs::read_to_string("level.ron") {
                                    info!("Server: sync_objects was empty, reading level.ron from disk ({} bytes) and serving LoadScene to client", scene_ron.len());
                                    sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::LoadScene {
                                        path: "level.ron".to_string(),
                                        scene_ron,
                                    });
                            }
                        }
                    });
                },
                crate::protocol::EditorAction::SaveScene { path } => {
                    info!("Server: Received SaveScene for '{}'", path);
                    commands.queue(move |world: &mut World| {
                        world.write_message(cb_engine::editor::serialization::SaveSceneEvent(path));
                    });
                },
                crate::protocol::EditorAction::LoadScene { path, scene_ron } => {
                    info!("Server: Received LoadScene for '{}' ({} bytes). Syncing to server world and all clients...", path, scene_ron.len());
                    let _ = std::fs::write(&path, &scene_ron);
                    let scene_ron_clone = scene_ron.clone();
                    commands.queue(move |world: &mut World| {
                        let mut q = world.query_filtered::<Entity, (Or<(With<cb_engine::editor::serialization::SceneObject>, With<cb_engine::editor::serialization::NetworkId>)>, Without<cb_engine::player::Player>, Without<cb_engine::player::PlayerCamera>)>();
                        let to_despawn: Vec<Entity> = q.iter(world).collect();
                        for e in to_despawn {
                            if let Ok(entity_mut) = world.get_entity_mut(e) {
                                entity_mut.despawn();
                            }
                        }
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
                    for (sender_entity, mut sender) in senders.iter_mut() {
                        if sender_entity != receiver_entity {
                            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::LoadScene {
                                path: path.clone(),
                                scene_ron: scene_ron.clone(),
                            });
                        }
                    }
                },
                crate::protocol::EditorAction::ClearScene => {
                    info!("Server: Received ClearScene. Clearing server world and relaying to clients...");
                    commands.queue(move |world: &mut World| {
                        let mut q = world.query_filtered::<Entity, (Or<(With<cb_engine::editor::serialization::SceneObject>, With<cb_engine::editor::serialization::NetworkId>)>, Without<cb_engine::player::Player>, Without<cb_engine::player::PlayerCamera>)>();
                        let to_despawn: Vec<Entity> = q.iter(world).collect();
                        for e in to_despawn {
                            if let Ok(entity_mut) = world.get_entity_mut(e) {
                                entity_mut.despawn();
                            }
                        }
                    });
                    for (sender_entity, mut sender) in senders.iter_mut() {
                        if sender_entity != receiver_entity {
                            sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::ClearScene);
                        }
                    }
                },
                crate::protocol::EditorAction::FullSceneSync { .. } => {}, // Client only
            }
        }
    }
}

pub fn periodic_snapshot_save(
    time: Res<Time>,
    mut last_save: Local<f32>,
    mut save_events: MessageWriter<cb_engine::editor::serialization::SaveSceneEvent>,
) {
    let current = time.elapsed_secs();
    if current - *last_save > 10.0 { // Save every 10 seconds
        save_events.write(cb_engine::editor::serialization::SaveSceneEvent("level.ron".to_string()));
        *last_save = current;
    }
}

pub fn handle_client_disconnect(
    mut commands: Commands,
    disconnect_events: Query<Entity, Added<lightyear::prelude::Disconnected>>,
    query_objects: Query<(Entity, &cb_engine::editor::serialization::NetworkId, &cb_engine::editor::serialization::EditorLock)>,
    mut senders: Query<&mut MessageSender<crate::protocol::EditorAction>>,
) {
    for client_entity in disconnect_events.iter() {
        let disconnected_client_id = client_entity.to_bits();
        info!("Server: Client {} disconnected. Cleaning up locks...", disconnected_client_id);
        
        for (entity, net_id, lock) in query_objects.iter() {
            if lock.user_id == disconnected_client_id {
                commands.entity(entity).remove::<cb_engine::editor::serialization::EditorLock>();
                
                // Broadcast unlock to remaining clients
                for mut sender in senders.iter_mut() {
                    sender.send::<crate::protocol::PlayModeChannel>(crate::protocol::EditorAction::UnlockObject {
                        id: net_id.0,
                        user_id: disconnected_client_id,
                    });
                }
            }
        }
    }
}
