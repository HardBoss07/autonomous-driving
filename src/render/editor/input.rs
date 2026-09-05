use crate::core::track::Track;
use crate::render::editor::state::{EditorState, EditorTool};
use crate::render::editor::ui::{
    apply_track_snapshot, create_track_snapshot, is_mouse_over_ui, save_snapshot_helper,
};
use macroquad::prelude::*;

pub fn handle_editor_input(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    world_mouse: Vec2,
    screen_mouse: Vec2,
    camera_target: &mut Vec2,
    camera_zoom: &mut f32,
    delta_time: f32,
) {
    let over_ui = is_mouse_over_ui(state, screen_mouse);

    handle_camera_pan_keys(camera_target, *camera_zoom, delta_time);
    handle_middle_mouse_pan(state, screen_mouse, camera_target, *camera_zoom);
    handle_mouse_wheel(
        state,
        track,
        track_texture,
        wall_texture,
        screen_mouse,
        camera_target,
        camera_zoom,
        over_ui,
    );
    handle_focus_key(track, camera_target, over_ui);

    if handle_undo_redo_shortcuts(state, track, track_texture, wall_texture) {
        return;
    }

    update_hovered_node(state, track, world_mouse, *camera_zoom);

    if handle_right_click_delete(state, track, track_texture, wall_texture, over_ui) {
        return;
    }

    handle_tool_interaction(
        state,
        track,
        track_texture,
        wall_texture,
        world_mouse,
        over_ui,
    );
}

fn handle_camera_pan_keys(camera_target: &mut Vec2, camera_zoom: f32, delta_time: f32) {
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

    if pan_direction.length() > 0.0 {
        let speed = 750.0 / camera_zoom;
        *camera_target += pan_direction.normalize() * speed * delta_time;
    }
}

fn handle_middle_mouse_pan(
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

fn handle_mouse_wheel(
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

    if left_ctrl && wheel_y != 0.0 {
        if !state.is_rotating_grid {
            save_snapshot_helper(state, track);
            state.is_rotating_grid = true;
        }
        let step_rad = 5.0_f32.to_radians();
        track.starting_grid.rotation += wheel_y.signum() * step_rad;
        track.rebuild_mesh(3, track_texture, wall_texture);
    } else {
        state.is_rotating_grid = false;

        if wheel_y != 0.0 && !over_ui {
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
    }
}

fn handle_focus_key(track: &Track, camera_target: &mut Vec2, over_ui: bool) {
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

fn handle_undo_redo_shortcuts(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
) -> bool {
    let ctrl_down = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
    let shift_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    let z_or_y_pressed = is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Y);

    if ctrl_down && z_or_y_pressed {
        let current = create_track_snapshot(track);
        if shift_down {
            if let Some(snapshot) = state.history.pop_redo(current) {
                apply_track_snapshot(track, snapshot, track_texture, wall_texture);
            }
        } else if let Some(snapshot) = state.history.pop_undo(current) {
            apply_track_snapshot(track, snapshot, track_texture, wall_texture);
        }
        return true;
    }
    false
}

fn update_hovered_node(
    state: &mut EditorState,
    track: &Track,
    world_mouse: Vec2,
    camera_zoom: f32,
) {
    state.hovered_node_index = None;
    let hit_radius = 25.0 / camera_zoom.max(0.2);
    let mut min_dist = hit_radius;
    for (index, point) in track.raw_points.iter().enumerate() {
        let distance = world_mouse.distance(*point);
        if distance < min_dist {
            min_dist = distance;
            state.hovered_node_index = Some(index);
        }
    }
}

fn handle_right_click_delete(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    over_ui: bool,
) -> bool {
    if !over_ui && is_mouse_button_pressed(MouseButton::Right) {
        if let Some(index) = state.hovered_node_index {
            save_snapshot_helper(state, track);
            track.remove_raw_point(index);
            state.hovered_node_index = None;
            state.selected_node_index = None;
            track.rebuild_mesh(5, track_texture, wall_texture);
            return true;
        }
    }
    false
}

fn handle_tool_interaction(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    world_mouse: Vec2,
    over_ui: bool,
) {
    let exit_target = track.starting_grid.exit_target();
    let entry_target = track.starting_grid.entry_target();

    match state.active_tool {
        EditorTool::SelectMove => {
            handle_select_move_tool(
                state,
                track,
                track_texture,
                wall_texture,
                world_mouse,
                over_ui,
            );
        }
        EditorTool::NodePlace => {
            handle_node_place_tool(
                state,
                track,
                track_texture,
                wall_texture,
                world_mouse,
                over_ui,
                exit_target,
                entry_target,
            );
        }
        EditorTool::NodeDelete => {
            handle_node_delete_tool(state, track, track_texture, wall_texture, over_ui);
        }
        EditorTool::PenDraw => {
            handle_pen_draw_tool(
                state,
                track,
                track_texture,
                wall_texture,
                world_mouse,
                over_ui,
                exit_target,
                entry_target,
            );
        }
    }
}

fn handle_select_move_tool(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    world_mouse: Vec2,
    over_ui: bool,
) {
    let grid_position = track.starting_grid.position;

    if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
        if world_mouse.distance(grid_position) < 70.0 {
            save_snapshot_helper(state, track);
            state.is_dragging_start = true;
        } else if let Some(index) = state.hovered_node_index {
            save_snapshot_helper(state, track);
            state.selected_node_index = Some(index);
            state.is_dragging_node = true;
        } else {
            state.selected_node_index = None;
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        if state.is_dragging_start || state.is_dragging_node {
            track.rebuild_mesh(5, track_texture, wall_texture);
        }
        state.is_dragging_start = false;
        state.is_dragging_node = false;
    }

    if state.is_dragging_start {
        track.starting_grid.position = world_mouse;
        track.rebuild_mesh(3, track_texture, wall_texture);
    }

    if state.is_dragging_node {
        if let Some(index) = state.selected_node_index {
            track.move_raw_point(index, world_mouse);
            track.rebuild_mesh(3, track_texture, wall_texture);
        }
    }
}

fn handle_node_place_tool(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    world_mouse: Vec2,
    over_ui: bool,
    exit_target: Vec2,
    entry_target: Vec2,
) {
    if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
        save_snapshot_helper(state, track);

        if track.raw_points.is_empty() {
            track.add_raw_point(exit_target);
        }

        if world_mouse.distance(entry_target) < state.snap_distance && track.raw_points.len() > 2 {
            track.add_raw_point(entry_target);
            track.is_closed = true;
        } else {
            track.add_raw_point(world_mouse);
        }

        track.rebuild_mesh(5, track_texture, wall_texture);
    }
}

fn handle_node_delete_tool(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    over_ui: bool,
) {
    if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
        if let Some(index) = state.hovered_node_index {
            save_snapshot_helper(state, track);
            track.remove_raw_point(index);
            state.hovered_node_index = None;
            track.rebuild_mesh(5, track_texture, wall_texture);
        }
    }
}

fn handle_pen_draw_tool(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    world_mouse: Vec2,
    over_ui: bool,
    exit_target: Vec2,
    entry_target: Vec2,
) {
    if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
        save_snapshot_helper(state, track);
        state.is_drawing = true;

        if track.raw_points.is_empty() {
            track.add_raw_point(exit_target);
        }
    }

    if state.is_drawing && is_mouse_button_down(MouseButton::Left) {
        let target = if world_mouse.distance(entry_target) < state.snap_distance {
            entry_target
        } else {
            world_mouse
        };

        track.add_raw_point(target);
        track.rebuild_mesh(2, track_texture, wall_texture);
    }

    if is_mouse_button_released(MouseButton::Left) && state.is_drawing {
        state.is_drawing = false;

        if let Some(&last) = track.raw_points.last() {
            if last.distance(entry_target) < state.snap_distance {
                if let Some(last_mut) = track.raw_points.last_mut() {
                    *last_mut = entry_target;
                }
                track.is_closed = true;
            }
        }
        track.simplify_raw_points();
        track.rebuild_mesh(5, track_texture, wall_texture);
    }
}
