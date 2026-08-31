use crate::core::car::CarState;
use macroquad::prelude::*;
use std::f32::consts::PI;

pub fn draw_grid(grid_size: f32, screen_width: f32, screen_height: f32) {
    let line_color = Color::new(0.12, 0.12, 0.15, 1.0);

    for x in (0..(screen_width as i32)).step_by(grid_size as usize) {
        draw_line(x as f32, 0.0, x as f32, screen_height, 1.0, line_color);
    }
    for y in (0..(screen_height as i32)).step_by(grid_size as usize) {
        draw_line(0.0, y as f32, screen_width, y as f32, 1.0, line_color);
    }
}

pub fn draw_car(car: &CarState, car_texture: Option<&Texture2D>, handbrake: bool) {
    let render_rotation = car.heading + PI / 2.0;

    if let Some(texture) = car_texture {
        draw_texture_ex(
            texture,
            car.pos_x - 16.0,
            car.pos_y - 32.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32.0, 64.0)),
                rotation: render_rotation,
                pivot: Some(vec2(car.pos_x, car.pos_y)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle_ex(
            car.pos_x - 15.0,
            car.pos_y - 30.0,
            30.0,
            60.0,
            DrawRectangleParams {
                color: if handbrake { RED } else { BLUE },
                rotation: render_rotation,
                offset: vec2(0.5, 0.5),
            },
        );
    }
}

pub fn draw_drift_indicator(car: &CarState, is_drifting: bool) {
    if is_drifting {
        draw_circle_lines(car.pos_x, car.pos_y, 40.0, 2.0, RED);
    }
}
