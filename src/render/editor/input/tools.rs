use crate::core::track::Track;
use crate::render::editor::state::{EditorState, EditorTool};
use crate::render::editor::ui::save_snapshot_helper;
use macroquad::prelude::*;

pub fn handle_tool_interaction(
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
            handle_select_move(
                state,
                track,
                track_texture,
                wall_texture,
                world_mouse,
                over_ui,
            );
        }
        EditorTool::NodePlace => {
            handle_node_place(
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
            handle_node_delete(state, track, track_texture, wall_texture, over_ui);
        }
        EditorTool::PenDraw => {
            handle_pen_draw(
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

fn handle_select_move(
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

fn handle_node_place(
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

fn handle_node_delete(
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

fn handle_pen_draw(
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
