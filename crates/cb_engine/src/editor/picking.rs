use bevy::prelude::*;

use super::camera::EditorCamera;
use super::EngineState;

pub struct EditorPickingPlugin;

impl Plugin for EditorPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_picking.run_if(in_state(EngineState::Edit)).in_set(super::EditorSet::Picking));
    }
}

#[derive(Component)]
pub struct Selected;

fn ray_sphere_intersect(ray_origin: Vec3, ray_dir: Vec3, sphere_center: Vec3, sphere_radius: f32) -> Option<f32> {
    let v = ray_origin - sphere_center;
    let b = 2.0 * ray_dir.dot(v);
    let c = v.length_squared() - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * c;
    
    if discriminant < 0.0 {
        return None;
    }
    
    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) / 2.0;
    let t2 = (-b + sqrt_d) / 2.0;
    
    if t1 >= 0.0 {
        Some(t1)
    } else if t2 >= 0.0 {
        Some(t2)
    } else {
        None
    }
}

fn handle_picking(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    q_objects: Query<(Entity, &GlobalTransform, Option<&super::serialization::NetworkId>), With<super::serialization::SceneObject>>,
    q_selected: Query<(Entity, Option<&super::serialization::NetworkId>), With<Selected>>,
    active_gizmo: Res<crate::editor::gizmos::ActiveGizmo>,
    viewport_state: Res<super::ui::EditorViewportState>,
    session: Res<super::serialization::LocalEditorSession>,
    mut action_requests: MessageWriter<super::serialization::EditorActionRequest>,
) {
    if active_gizmo.hovered_axis.is_some() || active_gizmo.dragging_axis.is_some() {
        return;
    }

    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if !viewport_state.is_hovered {
        return;
    }

    let Ok((camera, camera_transform)) = q_camera.single() else { return };

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

    // Deselect everything & unlock previous
    for (entity, net_id_opt) in q_selected.iter() {
        commands.entity(entity).remove::<Selected>();
        if let Some(nid) = net_id_opt {
            action_requests.write(super::serialization::EditorActionRequest::UnlockObject {
                id: nid.0,
                user_id: session.client_id,
            });
        }
    }

    let mut best_hit = None;
    let mut min_t = f32::MAX;

    for (entity, transform, net_id_opt) in q_objects.iter() {
        let center = transform.translation();
        // A generic radius for picking meshes and icons
        let radius = 1.0; 
        
        if let Some(t) = ray_sphere_intersect(ray_origin, ray_dir, center, radius) {
            if t < min_t {
                min_t = t;
                best_hit = Some((entity, net_id_opt.copied()));
            }
        }
    }

    if let Some((entity, net_id_opt)) = best_hit {
        commands.entity(entity).insert(Selected);
        if let Some(nid) = net_id_opt {
            action_requests.write(super::serialization::EditorActionRequest::LockObject {
                id: nid.0,
                user_id: session.client_id,
            });
        }
    }
}
