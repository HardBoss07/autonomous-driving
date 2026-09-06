use crate::core::geometry::FloatExt;
use crate::core::track::Track;
use crate::render::editor::state::EditorState;
use crate::render::editor::ui::save_snapshot_helper;
use macroquad::prelude::*;

pub fn handle_camera_pan_keys(camera_target: &mut Vec2, camera_zoom: f32, delta_time: f32) {
    let mut pan_direction = vec2(0.0, 0.0);
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        pan_direction.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        pan_direction.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        pan_direction.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        pan_direction.x += 1.0;
    }

    if pan_direction.length_squared() > 0.0 {
        let speed = 750.0 / camera_zoom;
        *camera_target += pan_direction.normalize() * speed * delta_time;
    }
}

pub fn handle_middle_mouse_pan(
    state: &mut EditorState,
    screen_mouse: Vec2,
    camera_target: &mut Vec2,
    camera_zoom: f32,
) {
    if is_mouse_button_pressed(MouseButton::Middle) {
        state.last_mouse_world_pan = Some(screen_mouse);
    }
    if is_mouse_button_down(MouseButton::Middle) {
        if let Some(last_pos) = state.last_mouse_world_pan {
            let delta = (screen_mouse - last_pos) / camera_zoom;
            *camera_target -= delta;
            state.last_mouse_world_pan = Some(screen_mouse);
        }
    } else {
        state.last_mouse_world_pan = None;
    }
}

pub fn handle_mouse_wheel(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    screen_mouse: Vec2,
    camera_target: &mut Vec2,
    camera_zoom: &mut f32,
    over_ui: bool,
) {
    let wheel_y = mouse_wheel().1;
    let left_ctrl = is_key_down(KeyCode::LeftControl);

    if left_ctrl && !wheel_y.is_near_zero() {
        if !state.is_rotating_grid {
            save_snapshot_helper(state, track);
            state.is_rotating_grid = true;
        }
        let step_rad = 5.0_f32.to_radians();
        track.starting_grid.rotation += wheel_y.signum() * step_rad;
        track.rebuild_mesh(3, track_texture, wall_texture);
    } else {
        state.is_rotating_grid = false;

        if !wheel_y.is_near_zero() && !over_ui {
            zoom_to_mouse(wheel_y, camera_zoom, camera_target, screen_mouse);
        }
    }
}

fn zoom_to_mouse(
    wheel_y: f32,
    camera_zoom: &mut f32,
    camera_target: &mut Vec2,
    screen_mouse: Vec2,
) {
    let zoom_factor = if wheel_y > 0.0 { 1.15 } else { 0.85 };
    let old_zoom = *camera_zoom;
    let new_zoom = (old_zoom * zoom_factor).clamp(0.05, 3.0);

    if (new_zoom - old_zoom).abs() > 0.0001 {
        let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
        let mouse_offset = screen_mouse - screen_center;
        let world_before = *camera_target + mouse_offset / old_zoom;

        *camera_zoom = new_zoom;
        *camera_target = world_before - mouse_offset / new_zoom;
    }
}

pub fn handle_focus_key(track: &Track, camera_target: &mut Vec2, over_ui: bool) {
    if is_key_pressed(KeyCode::F) && !over_ui {
        if !track.raw_points.is_empty() {
            let mut min_point = track.raw_points[0];
            let mut max_point = track.raw_points[0];
            for point in &track.raw_points {
                min_point = min_point.min(*point);
                max_point = max_point.max(*point);
            }
            *camera_target = (min_point + max_point) * 0.5;
        } else {
            *camera_target = track.starting_grid.position;
        }
    }
}
