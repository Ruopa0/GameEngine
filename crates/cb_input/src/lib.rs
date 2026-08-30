#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::empty_line_after_doc_comments, clippy::if_same_then_else)]
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>();
        app.insert_resource(InputEnabled(true));
        app.add_systems(PreUpdate, gather_input);
    }
}

/// Abstracted input state, decoupled from physical keys.
/// Easily serialized for netcode later.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct PlayerInput {
    pub move_dir: Vec2,   // X: right, Y: forward
    pub look_dir: Vec2,   // Mouse delta
    pub jump: bool,
    pub sprint: bool,
    pub crouch: bool,
    pub crouch_just_pressed: bool, // for double-tap detection
    pub fire_held: bool,
    pub fire_just: bool,
    pub reload: bool,
    pub interact: bool,
    pub prone: bool,
    pub prone_just_pressed: bool,
    pub tac_sprint: bool,
    pub interact_just: bool,
    pub swap_prev: bool,
    pub swap_next: bool,
}

#[derive(Resource)]
pub struct InputEnabled(pub bool);

fn gather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut input: ResMut<PlayerInput>,
    enabled: Res<InputEnabled>,
    time: Res<Time>,
    mut last_sprint_time: Local<f32>,
    mut is_tac_sprinting: Local<bool>,
    cursor_options: Query<&bevy::window::CursorOptions, With<Window>>,
) {
    let is_cursor_grabbed = cursor_options.iter().next().is_some_and(|c| c.grab_mode != bevy::window::CursorGrabMode::None);

    if !enabled.0 || !is_cursor_grabbed {
        *input = PlayerInput::default();
        return;
    }

    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    input.move_dir = dir.normalize_or_zero();
    input.jump = keyboard.just_pressed(KeyCode::Space);
    input.sprint = keyboard.pressed(KeyCode::ShiftLeft);

    let now = time.elapsed_secs();
    if keyboard.just_pressed(KeyCode::ShiftLeft) {
        if now - *last_sprint_time < 0.3 {
            *is_tac_sprinting = true;
        }
        *last_sprint_time = now;
    }

    if !input.sprint {
        *is_tac_sprinting = false;
    }

    input.tac_sprint = *is_tac_sprinting;

    input.crouch = keyboard.pressed(KeyCode::KeyC);
    input.crouch_just_pressed = keyboard.just_pressed(KeyCode::KeyC);
    input.prone = keyboard.pressed(KeyCode::ControlLeft);
    input.prone_just_pressed = keyboard.just_pressed(KeyCode::ControlLeft);
    
    input.fire_held = mouse_buttons.pressed(MouseButton::Left);
    input.fire_just = mouse_buttons.just_pressed(MouseButton::Left);
    input.reload = keyboard.just_pressed(KeyCode::KeyR);
    input.interact = keyboard.just_pressed(KeyCode::KeyF);
    input.interact_just = keyboard.just_pressed(KeyCode::KeyF);
    input.swap_prev = keyboard.just_pressed(KeyCode::Digit1);
    input.swap_next = keyboard.just_pressed(KeyCode::Digit2);
}


