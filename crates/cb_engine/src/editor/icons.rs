use super::camera::EditorCamera;
use super::serialization::SceneObject;
use super::EditorSet;
use super::EngineState;
use bevy::prelude::*;

pub struct EditorIconsPlugin;

impl Plugin for EditorIconsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_light_icons, billboard_icons)
                .run_if(in_state(EngineState::Edit))
                .in_set(EditorSet::GizmoUpdate),
        );
    }
}

#[derive(Component)]
pub struct EditorIcon;

#[derive(Component)]
pub struct HasEditorIcon;

fn spawn_light_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_lights: Query<Entity, (With<PointLight>, With<SceneObject>, Without<HasEditorIcon>)>,
) {
    for entity in q_lights.iter() {
        let icon_handle = asset_server.load("icons/light_icon.png");

        let mesh = meshes.add(Rectangle::new(1.0, 1.0));
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(icon_handle),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        let icon = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
                EditorIcon,
            ))
            .id();

        commands.entity(entity).insert(HasEditorIcon); // Mark parent so we don't spawn again
        commands.entity(entity).add_child(icon);
    }
}

fn billboard_icons(
    q_camera: Query<&GlobalTransform, With<EditorCamera>>,
    mut q_icons: Query<(&GlobalTransform, &mut Transform), With<EditorIcon>>,
) {
    let Ok(camera_transform) = q_camera.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();

    for (global_transform, mut transform) in q_icons.iter_mut() {
        let icon_pos = global_transform.translation();
        let direction = (camera_pos - icon_pos).normalize_or_zero();

        if direction != Vec3::ZERO {
            // Need to look at the camera in global space.
            // Sprite is a child, so we just set its local rotation to counteract the parent's and face the camera.
            // Wait, Sprite renders based on GlobalTransform.
            // Actually Sprite in Bevy 3D needs to be rotated to face camera.
            transform.look_to(direction, Vec3::Y);
        }
    }
}
