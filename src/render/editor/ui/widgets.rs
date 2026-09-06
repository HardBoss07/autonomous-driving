use crate::core::track::Track;
use crate::render::editor::state::{EditorState, EditorTool};
use crate::render::editor::ui::snapshot::{redo_helper, save_snapshot_helper, undo_helper};
use macroquad::prelude::Texture2D;
use macroquad::ui::{Ui, hash};

pub fn draw_tool_buttons(state: &mut EditorState, ui: &mut Ui) {
    ui.label(None, "--- EDITING TOOLS ---");
    if ui.button(
        None,
        if state.active_tool == EditorTool::PenDraw {
            "> Freehand Pen Draw"
        } else {
            "Tool: Freehand Pen"
        },
    ) {
        state.active_tool = EditorTool::PenDraw;
        state.is_drawing = false;
    }
    if ui.button(
        None,
        if state.active_tool == EditorTool::NodePlace {
            "> Click Node Place"
        } else {
            "Tool: Click Node"
        },
    ) {
        state.active_tool = EditorTool::NodePlace;
        state.is_drawing = false;
    }
    if ui.button(
        None,
        if state.active_tool == EditorTool::SelectMove {
            "> Select / Move"
        } else {
            "Tool: Select / Move"
        },
    ) {
        state.active_tool = EditorTool::SelectMove;
        state.is_drawing = false;
    }
    if ui.button(
        None,
        if state.active_tool == EditorTool::NodeDelete {
            "> Delete Node"
        } else {
            "Tool: Delete Node"
        },
    ) {
        state.active_tool = EditorTool::NodeDelete;
        state.is_drawing = false;
    }
}

pub fn draw_track_properties(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut Ui,
) {
    ui.label(None, "--- TRACK PROPERTIES ---");
    let loop_text = if track.is_closed {
        "Loop State: Closed (Toggle Open)"
    } else {
        "Loop State: Open (Toggle Close)"
    };
    if ui.button(None, loop_text) {
        save_snapshot_helper(state, track);
        track.is_closed = !track.is_closed;
        track.rebuild_mesh(5, track_texture, wall_texture);
    }

    let mut tolerance = track.simplify_tolerance;
    ui.slider(hash!(), "Curve Smoothness", 0.0..12.0, &mut tolerance);
    if (tolerance - track.simplify_tolerance).abs() > 0.01 {
        track.simplify_tolerance = tolerance;
        track.simplify_raw_points();
        track.rebuild_mesh(5, track_texture, wall_texture);
    }

    if ui.button(None, "Clear Track Curve") {
        save_snapshot_helper(state, track);
        track.clear();
        state.is_drawing = false;
    }
}

pub fn draw_checkpoint_controls(
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut Ui,
) {
    ui.label(None, "--- LAP CHECKPOINTS ---");
    let mut spacing = track.checkpoint_spacing;
    ui.slider(hash!(), "Gate Spacing", 100.0..1200.0, &mut spacing);
    if (spacing - track.checkpoint_spacing).abs() > 1.0 {
        track.checkpoint_spacing = spacing;
        track.rebuild_mesh(5, track_texture, wall_texture);
    }
}

pub fn draw_viewport_and_history(
    state: &mut EditorState,
    track: &mut Track,
    camera_zoom: &mut f32,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut Ui,
) {
    ui.label(None, "--- VIEWPORT & HISTORY ---");
    ui.slider(hash!(), "Zoom Level", 0.05..2.5, camera_zoom);

    if ui.button(None, "Undo Action (Ctrl+Z)") {
        undo_helper(state, track, track_texture, wall_texture);
    }
    if ui.button(None, "Redo Action (Ctrl+Y)") {
        redo_helper(state, track, track_texture, wall_texture);
    }

    let help_btn_text = if state.show_help_overlay {
        "Hide Controls HUD"
    } else {
        "Show Controls HUD"
    };
    if ui.button(None, help_btn_text) {
        state.show_help_overlay = !state.show_help_overlay;
    }
}
