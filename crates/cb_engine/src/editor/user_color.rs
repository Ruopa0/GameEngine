use bevy::prelude::*;
use bevy_egui::egui;

const PALETTE: &[(u8, u8, u8)] = &[
    (60, 220, 120), // Emerald Green
    (40, 160, 255), // Sky Blue
    (255, 150, 20), // Amber Orange
    (180, 80, 255), // Electric Purple
    (255, 60, 120), // Hot Pink / Crimson
    (255, 215, 30), // Sunshine Gold
    (20, 225, 210), // Mint Cyan
    (255, 110, 50), // Coral Red
];

pub fn get_user_color_rgb(user_id: u64) -> (u8, u8, u8) {
    let idx = (user_id as usize) % PALETTE.len();
    PALETTE[idx]
}

pub fn get_user_color_bevy(user_id: u64) -> Color {
    let (r, g, b) = get_user_color_rgb(user_id);
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub fn get_user_color_egui(user_id: u64) -> egui::Color32 {
    let (r, g, b) = get_user_color_rgb(user_id);
    egui::Color32::from_rgb(r, g, b)
}
