pub mod overlay;
pub mod snapshot;
pub mod widgets;

pub use overlay::draw_help_overlay;
pub use snapshot::{
    apply_track_snapshot, create_track_snapshot, redo_helper, save_snapshot_helper, undo_helper,
};
pub use widgets::{
    draw_checkpoint_controls, draw_tool_buttons, draw_track_properties, draw_viewport_and_history,
};

use crate::core::track::Track;
use crate::render::editor::state::EditorState;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets as mq_widgets};

pub fn is_mouse_over_ui(state: &EditorState, mouse_position: Vec2) -> bool {
    state.ui_rect.contains(mouse_position) || root_ui().is_mouse_over(mouse_position)
}

pub fn update_and_draw_ui(
    state: &mut EditorState,
    track: &mut Track,
    camera_zoom: &mut f32,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
) -> bool {
    let mut exit_editor = false;

    mq_widgets::Window::new(
        hash!(),
        vec2(state.ui_rect.x, state.ui_rect.y),
        vec2(state.ui_rect.w, state.ui_rect.h),
    )
    .label("Track Builder Tools")
    .ui(&mut root_ui(), |ui| {
        draw_tool_buttons(state, ui);
        ui.separator();

        draw_track_properties(state, track, track_texture, wall_texture, ui);
        ui.separator();

        draw_checkpoint_controls(track, track_texture, wall_texture, ui);
        ui.separator();

        draw_viewport_and_history(state, track, camera_zoom, track_texture, wall_texture, ui);
        ui.separator();

        if ui.button(None, "Save Track & Drive!") {
            exit_editor = true;
            state.is_drawing = false;
        }
    });

    exit_editor
}
