use bevy::prelude::*;
use super::EngineState;
use super::picking::Selected;
use super::EditorSet;

use bevy::input::mouse::MouseMotion;
use bevy::math::Isometry3d;
use super::camera::EditorCamera;

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

#[derive(Resource, Default)]
pub struct ActiveGizmo {
    pub hovered_axis: Option<(Entity, Vec3, f32)>, // entity, axis, t_axis
    pub dragging_axis: Option<(Entity, Vec3, f32)>, // entity, axis, drag_start_t
    pub drag_start_transform: Option<(Entity, Transform)>, // entity, transform before drag began
}

pub struct EditorGizmosPlugin;

impl Plugin for EditorGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GizmoMode>()
           .init_resource::<ActiveGizmo>()
           .add_systems(Update, (
               draw_gizmos.run_if(in_state(EngineState::Edit)).in_set(EditorSet::GizmoUpdate),
               draw_remote_presence.run_if(in_state(EngineState::Edit)).in_set(EditorSet::GizmoUpdate),
               update_gizmo_state.run_if(in_state(EngineState::Edit)).in_set(EditorSet::GizmoUpdate),
               apply_gizmo_drag.run_if(in_state(EngineState::Edit)).in_set(EditorSet::GizmoUpdate),
               handle_hotkeys.run_if(in_state(EngineState::Edit)).in_set(EditorSet::GizmoUpdate),
           ));
    }
}

// Very simple 3D translation gizmo: 
// It draws axes. If you hold X, Y, or Z while dragging the mouse, it translates the object.
fn draw_gizmos(
    mut gizmos: Gizmos,
    gizmo_mode: Res<GizmoMode>,
    active_gizmo: Res<ActiveGizmo>,
    session: Res<super::serialization::LocalEditorSession>,
    q_selected: Query<(Entity, &GlobalTransform, Option<&super::serialization::EditorLock>), With<Selected>>,
) {
    for (entity, global_transform, lock_opt) in q_selected.iter() {
        let origin = global_transform.translation();
        
        if let Some(lock) = lock_opt {
            if lock.user_id != session.client_id {
                let col = super::user_color::get_user_color_bevy(lock.user_id);
                let tf = global_transform.compute_transform();
                // Draw locked wireframe cube matching entity rotation and tightly wrapping with 0.1 padding
                gizmos.cube(
                    Transform {
                        translation: tf.translation,
                        rotation: tf.rotation,
                        scale: tf.scale + Vec3::splat(0.1),
                    },
                    col,
                );
                continue; // Do not draw draggable handles for locked object
            }
        }
        
        let is_active = |ax: Vec3| -> bool {
            if let Some((e, a, _)) = active_gizmo.dragging_axis {
                e == entity && a == ax
            } else if let Some((e, a, _)) = active_gizmo.hovered_axis {
                e == entity && a == ax
            } else {
                false
            }
        };

        let color = |ax: Vec3, default: Color| -> Color {
            if is_active(ax) { Color::srgb(1.0, 1.0, 0.0) } else { default }
        };

        match *gizmo_mode {
            GizmoMode::Translate => {
                gizmos.arrow(origin, origin + Vec3::X * 2.0, color(Vec3::X, Color::srgb(1.0, 0.0, 0.0)));
                gizmos.arrow(origin, origin + Vec3::Y * 2.0, color(Vec3::Y, Color::srgb(0.0, 1.0, 0.0)));
                gizmos.arrow(origin, origin + Vec3::Z * 2.0, color(Vec3::Z, Color::srgb(0.0, 0.0, 1.0)));
            }
            GizmoMode::Rotate => {
                gizmos.circle(Isometry3d::new(origin, Quat::from_rotation_arc(Vec3::Z, Vec3::X)), 2.0, color(Vec3::X, Color::srgb(1.0, 0.0, 0.0)));
                gizmos.circle(Isometry3d::new(origin, Quat::from_rotation_arc(Vec3::Z, Vec3::Y)), 2.0, color(Vec3::Y, Color::srgb(0.0, 1.0, 0.0)));
                gizmos.circle(Isometry3d::from_translation(origin), 2.0, color(Vec3::Z, Color::srgb(0.0, 0.0, 1.0)));
            }
            GizmoMode::Scale => {
                gizmos.line(origin, origin + Vec3::X * 2.0, color(Vec3::X, Color::srgb(1.0, 0.0, 0.0)));
                gizmos.line(origin, origin + Vec3::Y * 2.0, color(Vec3::Y, Color::srgb(0.0, 1.0, 0.0)));
                gizmos.line(origin, origin + Vec3::Z * 2.0, color(Vec3::Z, Color::srgb(0.0, 0.0, 1.0)));
                gizmos.sphere(Isometry3d::from_translation(origin + Vec3::X * 2.0), 0.2, color(Vec3::X, Color::srgb(1.0, 0.0, 0.0)));
                gizmos.sphere(Isometry3d::from_translation(origin + Vec3::Y * 2.0), 0.2, color(Vec3::Y, Color::srgb(0.0, 1.0, 0.0)));
                gizmos.sphere(Isometry3d::from_translation(origin + Vec3::Z * 2.0), 0.2, color(Vec3::Z, Color::srgb(0.0, 0.0, 1.0)));
            }
        }
    }
}

fn draw_remote_presence(
    mut gizmos: Gizmos,
    session: Res<super::serialization::LocalEditorSession>,
    q_remote_cameras: Query<(&GlobalTransform, &super::serialization::RemoteEditorCamera)>,
    q_locked_objects: Query<(&GlobalTransform, &super::serialization::EditorLock), Without<Selected>>,
) {
    // 1. Draw remote teammate camera frustums and view cones
    for (gt, cam) in q_remote_cameras.iter() {
        if cam.user_id == session.client_id { continue; }
        let col = super::user_color::get_user_color_bevy(cam.user_id);
        let pos = gt.translation();
        let forward = gt.forward();
        let up = gt.up();
        let right = gt.right();
        
        // Draw camera origin sphere
        gizmos.sphere(Isometry3d::from_translation(pos), 0.3, col);
        
        // Draw viewing frustum wireframe
        let tip = pos + forward * 2.0;
        let p1 = tip + (up * 0.6) + (right * 0.9);
        let p2 = tip + (up * 0.6) - (right * 0.9);
        let p3 = tip - (up * 0.6) - (right * 0.9);
        let p4 = tip - (up * 0.6) + (right * 0.9);

        gizmos.line(pos, p1, col);
        gizmos.line(pos, p2, col);
        gizmos.line(pos, p3, col);
        gizmos.line(pos, p4, col);
        gizmos.line(p1, p2, col);
        gizmos.line(p2, p3, col);
        gizmos.line(p3, p4, col);
        gizmos.line(p4, p1, col);
        
        // Direction pointer ray
        gizmos.arrow(pos, pos + forward * 3.0, col);
    }

    // 2. Draw colored bounding boxes around objects locked by other teammates
    for (gt, lock) in q_locked_objects.iter() {
        if lock.user_id == session.client_id { continue; }
        let col = super::user_color::get_user_color_bevy(lock.user_id);
        let tf = gt.compute_transform();
        gizmos.cube(
            Transform {
                translation: tf.translation,
                rotation: tf.rotation,
                scale: tf.scale + Vec3::splat(0.1),
            },
            col,
        );
    }
}

fn closest_points_line_line(p1: Vec3, d1: Vec3, p2: Vec3, d2: Vec3) -> (f32, f32) {
    let dp = p1 - p2;
    let a = d1.length_squared();
    let b = d1.dot(d2);
    let c = d2.length_squared();
    let d = d1.dot(dp);
    let e = d2.dot(dp);
    
    let denom = a * c - b * b;
    if denom.abs() < 1e-6 {
        return (0.0, d / b);
    }
    
    let t1 = (b * e - c * d) / denom;
    let t2 = (a * e - b * d) / denom;
    (t1, t2)
}

fn update_gizmo_state(
    viewport_state: Res<super::ui::EditorViewportState>,
    q_camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    q_selected: Query<(Entity, &Transform, &GlobalTransform, Option<&super::serialization::NetworkId>), With<Selected>>,
    mut active_gizmo: ResMut<ActiveGizmo>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut history: ResMut<super::history::HistoryState>,
) {
    let Ok((camera, camera_transform)) = q_camera.single() else { return };
    
    // Check if mouse was just released -> commit Move command to history
    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some((start_entity, start_transform)) = active_gizmo.drag_start_transform.take() {
            if let Ok((_, current_transform, _, maybe_net_id)) = q_selected.get(start_entity) {
                if *current_transform != start_transform {
                    if let Some(net_id) = maybe_net_id {
                        history.record_action(super::history::EditorCommand::Move {
                            id: net_id.0,
                            from: start_transform,
                            to: *current_transform,
                        });
                    }
                }
            }
        }
        active_gizmo.dragging_axis = None;
    }

    if !mouse_buttons.pressed(MouseButton::Left) {
        active_gizmo.dragging_axis = None;
    }

    if active_gizmo.dragging_axis.is_none() {
        let mut best = None;
        if viewport_state.is_hovered {
            let ndc = Vec2::new(
                viewport_state.normalized_mouse_pos.x * 2.0 - 1.0,
                (1.0 - viewport_state.normalized_mouse_pos.y) * 2.0 - 1.0,
            );
            let projection = camera.clip_from_view(); 
            let ndc_to_world = camera_transform.to_matrix() * projection.inverse();
            let near = ndc_to_world.project_point3(ndc.extend(1.0));
            let far = ndc_to_world.project_point3(ndc.extend(0.0));
            let ray_dir = (far - near).normalize();
            let ray_origin = camera_transform.translation();

            let mut min_dist = 0.3; // Hitbox radius
            
            for (entity, _, global_transform, _) in q_selected.iter() {
                let origin = global_transform.translation();
                let axes = [Vec3::X, Vec3::Y, Vec3::Z];
                for axis in axes {
                    let (t_ray, t_axis) = closest_points_line_line(ray_origin, ray_dir, origin, axis);
                    if (0.0..=2.0).contains(&t_axis) && t_ray > 0.0 {
                        let point_on_ray = ray_origin + ray_dir * t_ray;
                        let point_on_axis = origin + axis * t_axis;
                        let dist = point_on_ray.distance(point_on_axis);
                        if dist < min_dist {
                            min_dist = dist;
                            best = Some((entity, axis, t_axis));
                        }
                    }
                }
            }
        }
        active_gizmo.hovered_axis = best;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        active_gizmo.dragging_axis = active_gizmo.hovered_axis;
        if let Some((e, _, _)) = active_gizmo.dragging_axis {
            if let Ok((_, transform, _, _)) = q_selected.get(e) {
                active_gizmo.drag_start_transform = Some((e, *transform));
            }
        }
    }
}

fn apply_gizmo_drag(
    mut q_selected: Query<(Entity, &mut Transform, &GlobalTransform, Option<&super::serialization::NetworkId>, Option<&super::serialization::EditorLock>), With<Selected>>,
    active_gizmo: Res<ActiveGizmo>,
    gizmo_mode: Res<GizmoMode>,
    session: Res<super::serialization::LocalEditorSession>,
    viewport_state: Res<super::ui::EditorViewportState>,
    q_camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    mut action_requests: MessageWriter<super::serialization::EditorActionRequest>,
) {
    let Ok((camera, camera_transform)) = q_camera.single() else { return };
    
    // Support keys for fallback
    let mut current_axis = active_gizmo.dragging_axis;
    if keys.pressed(KeyCode::KeyX) {
        if let Some((e, _, _, _, _)) = q_selected.iter().next() {
            current_axis = Some((e, Vec3::X, 0.0)); // Fake t_axis
        }
    } else if keys.pressed(KeyCode::KeyY) {
        if let Some((e, _, _, _, _)) = q_selected.iter().next() {
            current_axis = Some((e, Vec3::Y, 0.0));
        }
    } else if keys.pressed(KeyCode::KeyZ) {
        if let Some((e, _, _, _, _)) = q_selected.iter().next() {
            current_axis = Some((e, Vec3::Z, 0.0));
        }
    }

    let Some((drag_entity, axis, drag_start_t)) = current_axis else { return };

    let mut total_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        total_delta += event.delta;
    }

    match *gizmo_mode {
        GizmoMode::Translate => {
            if active_gizmo.dragging_axis.is_some() && !keys.pressed(KeyCode::KeyX) && !keys.pressed(KeyCode::KeyY) && !keys.pressed(KeyCode::KeyZ) {
                // Precise 3D raycast
                let ndc = Vec2::new(
                    viewport_state.normalized_mouse_pos.x * 2.0 - 1.0,
                    (1.0 - viewport_state.normalized_mouse_pos.y) * 2.0 - 1.0,
                );
                let projection = camera.clip_from_view(); 
                let ndc_to_world = camera_transform.to_matrix() * projection.inverse();
                let near = ndc_to_world.project_point3(ndc.extend(1.0));
                let far = ndc_to_world.project_point3(ndc.extend(0.0));
                let ray_dir = (far - near).normalize();
                let ray_origin = camera_transform.translation();

                let origin = if let Ok((_, _, gt, _, _)) = q_selected.get(drag_entity) {
                    gt.translation()
                } else {
                    return;
                };

                let (_, t_axis) = closest_points_line_line(ray_origin, ray_dir, origin, axis);
                let delta = t_axis - drag_start_t;

                if delta.abs() > 0.0001 {
                    for (_, mut transform, _, net_id, lock_opt) in q_selected.iter_mut() {
                        if let Some(lock) = lock_opt {
                            if lock.user_id != session.client_id {
                                continue;
                            }
                        }
                        transform.translation += axis * delta;
                        if let Some(nid) = net_id {
                            action_requests.write(super::serialization::EditorActionRequest::MoveObject { 
                                id: nid.0, 
                                transform: *transform,
                                sender_user_id: session.client_id,
                            });
                        }
                    }
                }
            } else {
                // Fallback to screen movement for key presses
                if total_delta.length_squared() == 0.0 { return; }
                let cam_right = camera_transform.right();
                let cam_up = camera_transform.up();
                let axis_screen_x = axis.dot(*cam_right);
                let axis_screen_y = axis.dot(*cam_up); 
                let movement = total_delta.x * axis_screen_x - total_delta.y * axis_screen_y;
                for (_, mut transform, _, net_id, lock_opt) in q_selected.iter_mut() {
                    if let Some(lock) = lock_opt {
                        if lock.user_id != session.client_id {
                            continue;
                        }
                    }
                    let sensitivity = 0.02;
                    transform.translation += axis * movement * sensitivity;
                    if let Some(nid) = net_id {
                        action_requests.write(super::serialization::EditorActionRequest::MoveObject { 
                            id: nid.0, 
                            transform: *transform,
                            sender_user_id: session.client_id,
                        });
                    }
                }
            }
        }
        GizmoMode::Rotate => {
            if total_delta.length_squared() == 0.0 { return; }
            let cam_right = camera_transform.right();
            let cam_up = camera_transform.up();
            let axis_screen_x = axis.dot(*cam_right);
            let axis_screen_y = axis.dot(*cam_up); 
            let movement = total_delta.x * axis_screen_x - total_delta.y * axis_screen_y;
            for (_, mut transform, _, net_id, lock_opt) in q_selected.iter_mut() {
                if let Some(lock) = lock_opt {
                    if lock.user_id != session.client_id {
                        continue;
                    }
                }
                let sensitivity = 0.01;
                if axis == Vec3::X { transform.rotate_x(movement * sensitivity); }
                if axis == Vec3::Y { transform.rotate_y(movement * sensitivity); }
                if axis == Vec3::Z { transform.rotate_z(movement * sensitivity); }
                transform.rotation = transform.rotation.normalize();
                if let Some(nid) = net_id {
                    action_requests.write(super::serialization::EditorActionRequest::MoveObject { 
                        id: nid.0, 
                        transform: *transform,
                        sender_user_id: session.client_id,
                    });
                }
            }
        }
        GizmoMode::Scale => {
            if total_delta.length_squared() == 0.0 { return; }
            let cam_right = camera_transform.right();
            let cam_up = camera_transform.up();
            let axis_screen_x = axis.dot(*cam_right);
            let axis_screen_y = axis.dot(*cam_up); 
            let movement = total_delta.x * axis_screen_x - total_delta.y * axis_screen_y;
            for (_, mut transform, _, net_id, lock_opt) in q_selected.iter_mut() {
                if let Some(lock) = lock_opt {
                    if lock.user_id != session.client_id {
                        continue;
                    }
                }
                let sensitivity = 0.02;
                transform.scale += axis * movement * sensitivity;
                if let Some(nid) = net_id {
                    action_requests.write(super::serialization::EditorActionRequest::MoveObject { 
                        id: nid.0, 
                        transform: *transform,
                        sender_user_id: session.client_id,
                    });
                }
            }
        }
    }
}

fn handle_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut gizmo_mode: ResMut<GizmoMode>,
) {
    if keys.just_pressed(KeyCode::KeyW) {
        *gizmo_mode = GizmoMode::Translate;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        *gizmo_mode = GizmoMode::Rotate;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        *gizmo_mode = GizmoMode::Scale;
    }
}
