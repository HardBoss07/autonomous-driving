pub mod actions;
pub mod camera;
pub mod tools;

pub use actions::{handle_right_click_delete, handle_undo_redo_shortcuts, update_hovered_node};
pub use camera::{
    handle_camera_pan_keys, handle_focus_key, handle_middle_mouse_pan, handle_mouse_wheel,
};
pub use tools::handle_tool_interaction;

use crate::core::track::Track;
use crate::render::editor::state::EditorState;
use crate::render::editor::ui::is_mouse_over_ui;
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
