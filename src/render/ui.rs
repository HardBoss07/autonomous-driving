use crate::core::car::{CarConfig, CarState};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

pub fn draw_editor_toggle_button() -> bool {
    let mut toggle = false;
    widgets::Window::new(hash!(), vec2(15.0, 110.0), vec2(240.0, 45.0))
        .label("Track Menu")
        .ui(&mut root_ui(), |ui| {
            if ui.button(None, "Open Track Builder Mode") {
                toggle = true;
            }
        });
    toggle
}

pub fn draw_telemetry(car: &CarState, is_drifting: bool) {
    draw_rectangle(15.0, 15.0, 240.0, 85.0, Color::new(0.0, 0.0, 0.0, 0.75));
    draw_text(&format!("FPS: {}", get_fps()), 25.0, 38.0, 24.0, GREEN);
    draw_text(
        &format!("Speed: {:.1} px/s", car.speed()),
        25.0,
        62.0,
        18.0,
        WHITE,
    );
    draw_text(
        &format!("Grip: {:.0}%", car.current_grip * 100.0),
        25.0,
        84.0,
        18.0,
        if is_drifting { ORANGE } else { SKYBLUE },
    );
}

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
        ui.slider(hash!(), "Grip Drift", 0.01..0.45, &mut config.grip_drift);
        ui.slider(
            hash!(),
            "Recovery",
            0.5..10.0,
            &mut config.drift_recovery_rate,
        );
    });
}
