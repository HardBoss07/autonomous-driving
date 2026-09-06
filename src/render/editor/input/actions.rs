use crate::core::track::Track;
use crate::render::editor::state::EditorState;
use crate::render::editor::ui::{
    apply_track_snapshot, create_track_snapshot, save_snapshot_helper,
};
use macroquad::prelude::*;

pub fn handle_undo_redo_shortcuts(
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

pub fn update_hovered_node(
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

pub fn handle_right_click_delete(
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
