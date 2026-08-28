use bevy::prelude::*;
use super::serialization::{EditorActionRequest, NetworkId, SceneObject};

// ---------------------------------------------------------------------------------
// EDITOR UNDO / REDO HISTORY SYSTEM (COMMAND PATTERN)
// ---------------------------------------------------------------------------------
// This module provides a complete Undo/Redo stack for the Code Blue level editor.
// It tracks user actions (moving, spawning, deleting, and reparenting entities)
// so that actions can be safely undone (Ctrl+Z) or reapplied (Ctrl+Y / Ctrl+Shift+Z).
//
// Key Concepts:
// 1. `EditorCommand`: An enum describing an atomic action and how to invert it.
// 2. `HistoryState`: A Resource storing the `undo_stack` and `redo_stack`.
// 3. `UndoEvent` / `RedoEvent`: Messages dispatched to trigger undo/redo.
// 4. Network Synchronization: Undoing or redoing an action also emits an
//    `EditorActionRequest` so connected editors / server stay in sync!
// ---------------------------------------------------------------------------------

/// Represents a single reversible editor action.
#[derive(Clone, Debug, PartialEq)]
pub enum EditorCommand {
    /// Object was translated, rotated, or scaled.
    Move {
        id: u64,
        from: Transform,
        to: Transform,
    },
    /// A new object was spawned into the scene.
    Spawn {
        id: u64,
        object_type: String,
        asset_path: Option<String>,
        transform: Transform,
        name: Option<String>,
    },
    /// An existing object was removed/despawned from the scene.
    Despawn {
        id: u64,
        object_type: String,
        asset_path: Option<String>,
        transform: Transform,
        name: Option<String>,
    },
    /// An entity was reparented in the Hierarchy.
    Reparent {
        child_id: u64,
        old_parent_id: Option<u64>,
        new_parent_id: Option<u64>,
    },
}

/// Central resource storing the undo and redo command history.
#[derive(Resource, Debug)]
pub struct HistoryState {
    pub undo_stack: Vec<EditorCommand>,
    pub redo_stack: Vec<EditorCommand>,
    pub max_depth: usize,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100, // Maximum number of undo steps saved
        }
    }
}

impl HistoryState {
    /// Push a new user command onto the undo stack and clear the redo history.
    pub fn record_action(&mut self, cmd: EditorCommand) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
        // Performing any new action invalidates future redo branch
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Message triggered to perform an Undo.
#[derive(Message, Default)]
pub struct UndoEvent;

/// Message triggered to perform a Redo.
#[derive(Message, Default)]
pub struct RedoEvent;

/// Plugin managing history registration and undo/redo systems.
pub struct EditorHistoryPlugin;

impl Plugin for EditorHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoryState>()
            .add_message::<UndoEvent>()
            .add_message::<RedoEvent>()
            .add_systems(
                Update,
                (
                    handle_history_shortcuts.run_if(in_state(super::EngineState::Edit)),
                    handle_undo_events.run_if(in_state(super::EngineState::Edit)),
                    handle_redo_events.run_if(in_state(super::EngineState::Edit)),
                ),
            );
    }
}

/// Listens for keyboard shortcuts:
/// - Ctrl + Z: Undo
/// - Ctrl + Y or Ctrl + Shift + Z: Redo
fn handle_history_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut undo_writer: MessageWriter<UndoEvent>,
    mut redo_writer: MessageWriter<RedoEvent>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        if shift {
            // Ctrl + Shift + Z -> Redo
            redo_writer.write(RedoEvent);
        } else {
            // Ctrl + Z -> Undo
            undo_writer.write(UndoEvent);
        }
    } else if ctrl && keys.just_pressed(KeyCode::KeyY) {
        // Ctrl + Y -> Redo
        redo_writer.write(RedoEvent);
    }
}

/// Executes Undo operations by inverting the topmost command on the undo stack.
fn handle_undo_events(
    mut commands: Commands,
    mut undo_reader: MessageReader<UndoEvent>,
    mut history: ResMut<HistoryState>,
    session: Res<super::serialization::LocalEditorSession>,
    mut action_requests: MessageWriter<EditorActionRequest>,
    mut q_objects: Query<(Entity, &NetworkId, &mut Transform, Option<&Name>), With<SceneObject>>,
) {
    for _ in undo_reader.read() {
        if let Some(cmd) = history.undo_stack.pop() {
            match &cmd {
                EditorCommand::Move { id, from, to: _ } => {
                    // Revert transform to 'from'
                    for (_, net_id, mut transform, _) in q_objects.iter_mut() {
                        if net_id.0 == *id {
                            *transform = *from;
                            // Synchronize reverted transform to server / other clients
                            action_requests.write(EditorActionRequest::MoveObject {
                                id: *id,
                                transform: *from,
                                sender_user_id: session.client_id,
                            });
                        }
                    }
                }
                EditorCommand::Spawn { id, object_type: _, asset_path: _, transform: _, name: _ } => {
                    // Undo spawn -> Despawn the spawned entity
                    for (entity, net_id, _, _) in q_objects.iter() {
                        if net_id.0 == *id {
                            commands.entity(entity).despawn();
                            action_requests.write(EditorActionRequest::DespawnObject { id: *id });
                        }
                    }
                }
                EditorCommand::Despawn { id, object_type, asset_path, transform, name } => {
                    // Undo despawn -> Respawn the entity with same NetworkId and Transform
                    let mut entity_cmds = commands.spawn((
                        SceneObject {
                            object_type: object_type.clone(),
                            asset_path: asset_path.clone(),
                        },
                        NetworkId(*id),
                        *transform,
                        GlobalTransform::default(),
                    ));
                    if let Some(n) = name {
                        entity_cmds.insert(Name::new(n.clone()));
                    }
                    action_requests.write(EditorActionRequest::SpawnObject {
                        id: *id,
                        object_type: object_type.clone(),
                        asset_path: asset_path.clone(),
                        transform: *transform,
                    });
                }
                EditorCommand::Reparent { child_id, old_parent_id, new_parent_id: _ } => {
                    // Revert child parent to old_parent_id
                    let mut child_entity = None;
                    let mut old_parent_entity = None;

                    for (entity, net_id, _, _) in q_objects.iter() {
                        if net_id.0 == *child_id {
                            child_entity = Some(entity);
                        }
                        if let Some(pid) = old_parent_id {
                            if net_id.0 == *pid {
                                old_parent_entity = Some(entity);
                            }
                        }
                    }

                    if let Some(child) = child_entity {
                        if let Some(parent) = old_parent_entity {
                            commands.entity(child).set_parent_in_place(parent);
                        } else {
                            commands.entity(child).remove_parent_in_place();
                        }
                    }
                }
            }
            // Move inverted command onto redo stack
            history.redo_stack.push(cmd);
        }
    }
}

/// Executes Redo operations by re-applying the topmost command on the redo stack.
fn handle_redo_events(
    mut commands: Commands,
    mut redo_reader: MessageReader<RedoEvent>,
    mut history: ResMut<HistoryState>,
    session: Res<super::serialization::LocalEditorSession>,
    mut action_requests: MessageWriter<EditorActionRequest>,
    mut q_objects: Query<(Entity, &NetworkId, &mut Transform, Option<&Name>), With<SceneObject>>,
) {
    for _ in redo_reader.read() {
        if let Some(cmd) = history.redo_stack.pop() {
            match &cmd {
                EditorCommand::Move { id, from: _, to } => {
                    // Re-apply transform to 'to'
                    for (_, net_id, mut transform, _) in q_objects.iter_mut() {
                        if net_id.0 == *id {
                            *transform = *to;
                            // Synchronize reapplied transform to server / other clients
                            action_requests.write(EditorActionRequest::MoveObject {
                                id: *id,
                                transform: *to,
                                sender_user_id: session.client_id,
                            });
                        }
                    }
                }
                EditorCommand::Spawn { id, object_type, asset_path, transform, name } => {
                    // Redo spawn -> Respawn the entity
                    let mut entity_cmds = commands.spawn((
                        SceneObject {
                            object_type: object_type.clone(),
                            asset_path: asset_path.clone(),
                        },
                        NetworkId(*id),
                        *transform,
                        GlobalTransform::default(),
                    ));
                    if let Some(n) = name {
                        entity_cmds.insert(Name::new(n.clone()));
                    }
                    action_requests.write(EditorActionRequest::SpawnObject {
                        id: *id,
                        object_type: object_type.clone(),
                        asset_path: asset_path.clone(),
                        transform: *transform,
                    });
                }
                EditorCommand::Despawn { id, object_type: _, asset_path: _, transform: _, name: _ } => {
                    // Redo despawn -> Remove the entity
                    for (entity, net_id, _, _) in q_objects.iter() {
                        if net_id.0 == *id {
                            commands.entity(entity).despawn();
                            action_requests.write(EditorActionRequest::DespawnObject { id: *id });
                        }
                    }
                }
                EditorCommand::Reparent { child_id, old_parent_id: _, new_parent_id } => {
                    // Re-apply child parent to new_parent_id
                    let mut child_entity = None;
                    let mut new_parent_entity = None;

                    for (entity, net_id, _, _) in q_objects.iter() {
                        if net_id.0 == *child_id {
                            child_entity = Some(entity);
                        }
                        if let Some(pid) = new_parent_id {
                            if net_id.0 == *pid {
                                new_parent_entity = Some(entity);
                            }
                        }
                    }

                    if let Some(child) = child_entity {
                        if let Some(parent) = new_parent_entity {
                            commands.entity(child).set_parent_in_place(parent);
                        } else {
                            commands.entity(child).remove_parent_in_place();
                        }
                    }
                }
            }
            // Move reapplied command back onto undo stack
            history.undo_stack.push(cmd);
        }
    }
}
