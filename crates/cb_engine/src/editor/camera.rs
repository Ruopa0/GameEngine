use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::camera::{RenderTarget, ImageRenderTarget};
use bevy_egui::EguiContexts;

use super::EngineState;

// ---------------------------------------------------------------------------------
// EDITOR CAMERA LOGIC
// The editor has two distinct views: the "Viewport" (scene editor view) and the "Game View".
// Instead of drawing these directly to the OS window, we render them into off-screen
// textures (Images) and then give those textures to Egui to draw inside the dock panels!
//
// Key Responsibilities:
// 1. Initializing the 3D RenderTargets (textures) for both views.
// 2. Ensuring the cameras' aspect ratios dynamically resize when the user drags the Egui panels.
// 3. Handling free-cam WASD/Mouse movement when editing the scene.
// 4. Handling the "F" key to focus the camera on a selected object.
// ---------------------------------------------------------------------------------

/// Plugin for the editor camera and viewport rendering.
pub struct EditorCameraPlugin;

impl Plugin for EditorCameraPlugin {
    fn build(&self, app: &mut App) {
        // Startup: create the off-screen textures
        app.add_systems(Startup, setup_viewport_textures)
           // Update: handle movement and resizing every frame
           .add_systems(Update, (
               spawn_or_enable_editor_camera,
               force_camera_render_targets,
               editor_camera_movement.run_if(in_state(EngineState::Edit)),
               focus_on_selected.run_if(in_state(EngineState::Edit)),
               register_egui_textures,
               resize_viewport_textures,
           ));
    }
}

#[derive(Resource)]
pub struct ViewportTextures {
    pub editor_view: Handle<Image>,
    pub game_view: Handle<Image>,
    pub editor_egui_id: Option<bevy_egui::egui::TextureId>,
    pub game_egui_id: Option<bevy_egui::egui::TextureId>,
}

#[derive(Component)]
pub struct EditorCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub focus: Vec3,
    pub radius: f32,
}

fn setup_viewport_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d { width: 1280, height: 720, depth_or_array_layers: 1 };
    
    let mut editor_image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("editor_viewport"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    editor_image.resize(size);
    let editor_handle = images.add(editor_image);

    let mut game_image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("game_viewport"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    game_image.resize(size);
    let game_handle = images.add(game_image);

    commands.insert_resource(ViewportTextures {
        editor_view: editor_handle,
        game_view: game_handle,
        editor_egui_id: None,
        game_egui_id: None,
    });
}

fn register_egui_textures(
    mut contexts: EguiContexts,
    mut textures: ResMut<ViewportTextures>,
) {
    if textures.editor_egui_id.is_none() {
        textures.editor_egui_id = Some(contexts.add_image(bevy_egui::EguiTextureHandle::Strong(textures.editor_view.clone())));
    }
    if textures.game_egui_id.is_none() {
        textures.game_egui_id = Some(contexts.add_image(bevy_egui::EguiTextureHandle::Strong(textures.game_view.clone())));
    }
}

fn force_camera_render_targets(
    textures: Res<ViewportTextures>,
    mut q_editor_cam: Query<&mut RenderTarget, With<EditorCamera>>,
    mut q_other_cams: Query<&mut RenderTarget, (With<Camera>, Without<EditorCamera>, Without<crate::editor::ui::UiCamera>)>,
) {
    for mut target in q_editor_cam.iter_mut() {
        if let RenderTarget::Image(_) = &*target {
            // Already rendering to image
        } else {
            *target = RenderTarget::Image(ImageRenderTarget {
                handle: textures.editor_view.clone(),
                scale_factor: 1.0,
            });
        }
    }
    for mut target in q_other_cams.iter_mut() {
        // Assume all non-editor cameras are game cameras
        if let RenderTarget::Image(_) = &*target {
            // Already rendering to image
        } else {
            *target = RenderTarget::Image(ImageRenderTarget {
                handle: textures.game_view.clone(),
                scale_factor: 1.0,
            });
        }
    }
}

fn spawn_or_enable_editor_camera(
    mut commands: Commands,
    q_editor_cam: Query<Entity, With<EditorCamera>>,
) {
    if q_editor_cam.is_empty() {
        // Start looking at the default cube (0, 0, 0)
        let focus = Vec3::ZERO;
        let radius = 10.0;
        let pitch = -0.5;
        let yaw = 0.5;
        let rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        let translation = focus + rotation * Vec3::Z * radius;
        
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform {
                translation,
                rotation,
                ..default()
            },
            GlobalTransform::default(),
            EditorCamera {
                pitch,
                yaw,
                focus,
                radius,
            },
        ));
    }
}

fn editor_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut q_cam: Query<(&mut Transform, &mut EditorCamera)>,
) {
    let Ok((mut transform, mut state)) = q_cam.single_mut() else { return };

    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    let is_orbit = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let left_pressed = mouse_buttons.pressed(MouseButton::Left);
    let right_pressed = mouse_buttons.pressed(MouseButton::Right);
    let middle_pressed = mouse_buttons.pressed(MouseButton::Middle);

    let sensitivity = 0.002;

    if is_orbit && left_pressed {
        // Orbit around focus
        state.yaw -= mouse_delta.x * sensitivity;
        state.pitch -= mouse_delta.y * sensitivity;
        state.pitch = state.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
        
        let rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
        transform.rotation = rotation;
        transform.translation = state.focus + rotation * Vec3::Z * state.radius;
    } else if right_pressed {
        // Look around (FPS style)
        state.yaw -= mouse_delta.x * sensitivity;
        state.pitch -= mouse_delta.y * sensitivity;
        state.pitch = state.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

        let rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
        transform.rotation = rotation;
    } else if middle_pressed {
        // Pan
        let pan_speed = 0.01;
        let right = *transform.right();
        let up = *transform.up();
        let translation_delta = -right * mouse_delta.x * pan_speed + up * mouse_delta.y * pan_speed;
        transform.translation += translation_delta;
        state.focus += translation_delta;
    }

    // Move
    let mut velocity = Vec3::ZERO;
    let forward = transform.forward();
    let right = transform.right();
    let up = Vec3::Y;

    let can_move = (is_orbit && left_pressed) || right_pressed || middle_pressed;

    if can_move {
        if keys.pressed(KeyCode::KeyW) { velocity += *forward; }
        if keys.pressed(KeyCode::KeyS) { velocity -= *forward; }
        if keys.pressed(KeyCode::KeyD) { velocity += *right; }
        if keys.pressed(KeyCode::KeyA) { velocity -= *right; }
        if keys.pressed(KeyCode::KeyE) { velocity += up; }
        if keys.pressed(KeyCode::KeyQ) { velocity -= up; }
    }

    let speed = if keys.pressed(KeyCode::ShiftLeft) { 20.0 } else { 5.0 };
    
    if velocity.length_squared() > 0.0 {
        velocity = velocity.normalize() * speed;
        transform.translation += velocity * time.delta_secs();
        
        // Update focus point when moving
        let rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
        state.focus = transform.translation - rotation * Vec3::Z * state.radius;
    } else if right_pressed && mouse_delta.length_squared() > 0.0 {
        // Update focus point when looking around
        let rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
        state.focus = transform.translation - rotation * Vec3::Z * state.radius;
    }
}

fn focus_on_selected(
    keys: Res<ButtonInput<KeyCode>>,
    q_selected: Query<&GlobalTransform, With<crate::editor::picking::Selected>>,
    mut q_cam: Query<(&mut Transform, &mut EditorCamera)>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    let Ok(selected_transform) = q_selected.single() else { return };
    let Ok((mut transform, mut state)) = q_cam.single_mut() else { return };

    let target_pos = selected_transform.translation();
    
    // Offset the camera 10 units away along its current backward vector
    let offset = transform.back() * 10.0;
    transform.translation = target_pos + offset;
    
    // Make camera look at target
    transform.look_at(target_pos, Vec3::Y);
    
    // Update EditorCamera internal pitch and yaw to match new rotation
    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    state.yaw = yaw;
    state.pitch = pitch;
    
    state.focus = target_pos;
    state.radius = 10.0;
}


fn resize_viewport_textures(
    viewport_state: Res<super::ui::EditorViewportState>,
    textures: Res<ViewportTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    if viewport_state.viewport_size.x > 0.0 && viewport_state.viewport_size.y > 0.0 {
        if let Some(image) = images.get_mut(&textures.editor_view) {
            let width = viewport_state.viewport_size.x as u32;
            let height = viewport_state.viewport_size.y as u32;
            let current_size = image.texture_descriptor.size;
            if current_size.width != width || current_size.height != height {
                image.resize(Extent3d { width, height, depth_or_array_layers: 1 });
            }
        }
    }
    
    if viewport_state.game_view_size.x > 0.0 && viewport_state.game_view_size.y > 0.0 {
        if let Some(image) = images.get_mut(&textures.game_view) {
            let width = viewport_state.game_view_size.x as u32;
            let height = viewport_state.game_view_size.y as u32;
            let current_size = image.texture_descriptor.size;
            if current_size.width != width || current_size.height != height {
                image.resize(Extent3d { width, height, depth_or_array_layers: 1 });
            }
        }
    }
}

