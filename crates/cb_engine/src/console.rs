use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use cb_shared::components::ImmortalPlayer;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FramepacePlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .init_resource::<ConsoleState>()
        .add_message::<ConsoleCommandEvent>()
        .add_systems(Update, (
            toggle_console,
            draw_console,
            handle_commands,
            draw_fps,
        ));
    }
}

#[derive(Resource)]
pub struct ConsoleState {
    pub is_open: bool,
    pub input: String,
    pub history: Vec<String>,
    pub show_fps: bool,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            is_open: false,
            input: String::new(),
            history: vec!["Type 'help' for a list of commands.".to_string()],
            show_fps: false,
        }
    }
}

#[derive(Message)]
pub struct ConsoleCommandEvent(pub String);

fn toggle_console(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ConsoleState>,
) {
    if keys.just_pressed(KeyCode::Backquote) {
        state.is_open = !state.is_open;
    }
}

fn draw_console(
    mut contexts: EguiContexts,
    mut state: ResMut<ConsoleState>,
    mut ev_cmd: MessageWriter<ConsoleCommandEvent>,
) {
    if !state.is_open {
        return;
    }

    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Developer Console")
            .default_size([600.0, 300.0])
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 40.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &state.history {
                            if line.starts_with('>') {
                                ui.label(egui::RichText::new(line).strong().color(egui::Color32::from_rgb(100, 220, 255)));
                            } else {
                                ui.label(line);
                            }
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(">").strong().color(egui::Color32::from_rgb(100, 200, 255)));
                    let text_edit = egui::TextEdit::singleline(&mut state.input)
                        .hint_text("Type command (e.g. 'help', 'fps 1', 'maxfps 144')...")
                        .desired_width(ui.available_width() - 65.0);
                    let response = ui.add(text_edit);
                    let send_clicked = ui.button("Send").clicked();
                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

                    if send_clicked || enter_pressed {
                        let cmd = state.input.clone();
                        if !cmd.trim().is_empty() {
                            state.history.push(format!("> {}", cmd));
                            ev_cmd.write(ConsoleCommandEvent(cmd));
                        }
                        state.input.clear();
                        response.request_focus();
                    }
                });
            });
    }
}

fn draw_fps(
    mut contexts: EguiContexts,
    state: Res<ConsoleState>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if !state.show_fps {
        return;
    }

    if let Ok(ctx) = contexts.ctx_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                egui::Area::new(egui::Id::new("fps_overlay"))
                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                    .show(ctx, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1} FPS", value))
                                .color(egui::Color32::GREEN)
                                .strong()
                                .size(20.0),
                        );
                    });
            }
        }
    }
}

fn handle_commands(
    mut events: MessageReader<ConsoleCommandEvent>,
    mut state: ResMut<ConsoleState>,
    mut framepace: ResMut<FramepaceSettings>,
    mut q_player: Query<(Entity, &mut cb_shared::components::Health, Option<&ImmortalPlayer>, &Transform), With<crate::player::Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in events.read() {
        let cmd = ev.0.trim();
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "help" => {
                state.history.push("Commands:".to_string());
                state.history.push("  help - Show this message".to_string());
                state.history.push("  clear - Clear console history".to_string());
                state.history.push("  show_fps <0|1> - Toggle FPS overlay".to_string());
                state.history.push("  fps_max <limit> - Set max FPS (e.g., 60, 144, 0 for unlimited)".to_string());
                state.history.push("  show_physics <0|1> - (Not implemented - use Editor visualizer)".to_string());
                state.history.push("  god_mode <0|1> - Toggle player immortality".to_string());
                state.history.push("  heal <amount> - Heal the player".to_string());
                state.history.push("  spawn_bot - Spawn a target dummy".to_string());
                state.history.push("  noclip <0|1> - Toggle player collisions".to_string());
            }
            "clear" => {
                state.history.clear();
                state.history.push("Console cleared.".to_string());
            }
            "show_fps" => {
                if parts.len() > 1 {
                    let val = parts[1] == "1" || parts[1] == "true";
                    state.show_fps = val;
                    state.history.push(format!("FPS overlay set to {}", val));
                }
            }
            "fps_max" => {
                if parts.len() > 1 {
                    if let Ok(limit) = parts[1].parse::<f64>() {
                        if limit > 0.0 {
                            framepace.limiter = Limiter::from_framerate(limit);
                            state.history.push(format!("FPS limit set to {}", limit));
                        } else {
                            framepace.limiter = Limiter::Off;
                            state.history.push("FPS limit disabled".to_string());
                        }
                    }
                }
            }
            "god_mode" => {
                if parts.len() > 1 {
                    let val = parts[1] == "1" || parts[1] == "true";
                    if let Some((entity, _, _, _)) = q_player.iter_mut().next() {
                        if val {
                            commands.entity(entity).insert(ImmortalPlayer);
                            state.history.push("God mode ENABLED".to_string());
                        } else {
                            commands.entity(entity).remove::<ImmortalPlayer>();
                            state.history.push("God mode DISABLED".to_string());
                        }
                    } else {
                        state.history.push("Player not found".to_string());
                    }
                }
            }
            "heal" => {
                if parts.len() > 1 {
                    if let Ok(amt) = parts[1].parse::<f32>() {
                        if let Some((_, mut health, _, _)) = q_player.iter_mut().next() {
                            health.current = (health.current + amt).min(health.max);
                            state.history.push(format!("Healed player for {}. HP is now {}/{}", amt, health.current, health.max));
                        }
                    }
                }
            }
            "noclip" => {
                if parts.len() > 1 {
                    let val = parts[1] == "1" || parts[1] == "true";
                    if let Some((entity, _, _, _)) = q_player.iter_mut().next() {
                        if val {
                            commands.entity(entity).insert(avian3d::prelude::CollisionLayers::new(0b0, 0b0));
                            state.history.push("Noclip ENABLED (Collisions removed)".to_string());
                        } else {
                            commands.entity(entity).remove::<avian3d::prelude::CollisionLayers>();
                            state.history.push("Noclip DISABLED".to_string());
                        }
                    }
                }
            }
            "spawn_bot" => {
                if let Some((_, _, _, transform)) = q_player.iter_mut().next() {
                    let forward = transform.forward();
                    let pos = transform.translation + forward * 5.0; // 5 meters in front
                    
                    let mesh = meshes.add(Cuboid::new(0.6, 1.8, 0.6));
                    let material = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.2, 0.2),
                        ..default()
                    });
                    
                    commands.spawn((
                        Name::new("Bot"),
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(pos),
                        avian3d::prelude::RigidBody::Static,
                        avian3d::prelude::Collider::cuboid(0.6, 1.8, 0.6),
                        cb_shared::components::Health::new(100.0),
                        crate::gamemode::TargetDummy,
                    ));
                    state.history.push(format!("Spawned Target Dummy at {:?}", pos));
                }
            }
            _ => {
                state.history.push(format!("Unknown command: {}", parts[0]));
            }
        }
    }
}
