use crate::core::car::CarState;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

fn format_time(seconds: f32) -> String {
    let mins = (seconds / 60.0) as u32;
    let secs = seconds % 60.0;
    if mins > 0 {
        format!("{}:{:05.2}", mins, secs).to_string()
    } else {
        format!("{:.2}s", secs).to_string()
    }
}

pub fn draw_editor_toggle_button() -> bool {
    let mut toggle = false;
    widgets::Window::new(hash!(), vec2(15.0, 260.0), vec2(240.0, 45.0))
        .label("Track Menu")
        .ui(&mut root_ui(), |ui| {
            if ui.button(None, "Open Track Builder Mode") {
                toggle = true;
            }
        });
    toggle
}

pub fn draw_telemetry(car: &CarState, is_drifting: bool) {
    let current_time = get_time();

    draw_rectangle(15.0, 15.0, 260.0, 245.0, Color::new(0.0, 0.0, 0.0, 0.8));

    draw_text(&format!("FPS: {}", get_fps()), 25.0, 35.0, 20.0, GREEN);
    draw_text(
        &format!("Speed: {:.1} px/s", car.speed()),
        25.0,
        55.0,
        16.0,
        WHITE,
    );
    draw_text(
        &format!("Grip: {:.0}%", car.current_grip * 100.0),
        25.0,
        73.0,
        16.0,
        if is_drifting { ORANGE } else { SKYBLUE },
    );

    draw_line(25.0, 82.0, 260.0, 82.0, 1.0, GRAY);

    let cur_lap = if car.timing.current_lap_start_time > 0.0 {
        (current_time - car.timing.current_lap_start_time) as f32
    } else {
        0.0
    };

    draw_text(
        &format!("Lap: {}", car.timing.completed_laps + 1),
        25.0,
        100.0,
        18.0,
        YELLOW,
    );
    draw_text(
        &format!("Current: {}", format_time(cur_lap)),
        25.0,
        120.0,
        16.0,
        WHITE,
    );

    let best_lap_str = car
        .timing
        .best_lap_time
        .map_or("--:--".to_string(), format_time);
    let last_lap_str = car
        .timing
        .last_lap_time
        .map_or("--:--".to_string(), format_time);

    draw_text(&format!("Best: {}", best_lap_str), 25.0, 138.0, 16.0, GOLD);
    draw_text(
        &format!("Last: {}", last_lap_str),
        25.0,
        156.0,
        16.0,
        LIGHTGRAY,
    );

    draw_line(25.0, 165.0, 260.0, 165.0, 1.0, GRAY);

    for s in 0..3 {
        let y_offset = 182.0 + (s as f32) * 18.0;
        let active_indicator =
            if car.timing.current_sector == s && car.timing.current_lap_start_time > 0.0 {
                "*"
            } else {
                ""
            };
        let sec_str = car.timing.current_sector_times[s].map_or("--:--".to_string(), format_time);
        let best_sec_str = car.timing.best_sector_times[s].map_or("--:--".to_string(), format_time);

        draw_text(
            &format!(
                "S{}: {}{} (Best: {})",
                s + 1,
                sec_str,
                active_indicator,
                best_sec_str
            ),
            25.0,
            y_offset,
            15.0,
            if car.timing.current_sector == s {
                GREEN
            } else {
                WHITE
            },
        );
    }

    draw_text(
        "[R] Reset Position | [C] Gates Debug",
        25.0,
        248.0,
        13.0,
        LIGHTGRAY,
    );
}
