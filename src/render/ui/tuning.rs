use crate::core::car::CarConfig;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

pub fn draw_tuning(config: &mut CarConfig) {
    widgets::Window::new(
        hash!(),
        vec2(screen_width() - 340.0, 15.0),
        vec2(320.0, 370.0),
    )
    .label("Tuning Controls (WASD + Space)")
    .ui(&mut root_ui(), |ui| {
        ui.slider(hash!(), "Max Speed", 200.0..2000.0, &mut config.max_speed);
        ui.slider(
            hash!(),
            "Engine Force",
            500.0..5000.0,
            &mut config.engine_force,
        );
        ui.slider(
            hash!(),
            "Brake Force",
            500.0..5000.0,
            &mut config.brake_force,
        );
        ui.slider(hash!(), "Turn Rate", 1.0..10.0, &mut config.turn_rate);

        let mut drag_ui = config.drag_coeff * 10000.0;
        ui.slider(hash!(), "Drag (x10k)", 1.0..50.0, &mut drag_ui);
        config.drag_coeff = drag_ui / 10000.0;

        ui.slider(hash!(), "Grip Normal", 0.50..0.99, &mut config.grip_normal);
        ui.slider(hash!(), "Grip Drift", 0.001..0.15, &mut config.grip_drift);
        ui.slider(
            hash!(),
            "Recovery",
            0.5..10.0,
            &mut config.drift_recovery_rate,
        );
    });
}
