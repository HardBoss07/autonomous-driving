use crate::core::track::Track;
use crate::render::editor::state::{EditorSnapshot, EditorState, EditorTool};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

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

    widgets::Window::new(
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

fn draw_tool_buttons(state: &mut EditorState, ui: &mut macroquad::ui::Ui) {
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

fn draw_track_properties(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut macroquad::ui::Ui,
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

fn draw_checkpoint_controls(
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut macroquad::ui::Ui,
) {
    ui.label(None, "--- LAP CHECKPOINTS ---");
    let mut spacing = track.checkpoint_spacing;
    ui.slider(hash!(), "Gate Spacing", 100.0..1200.0, &mut spacing);
    if (spacing - track.checkpoint_spacing).abs() > 1.0 {
        track.checkpoint_spacing = spacing;
        track.rebuild_mesh(5, track_texture, wall_texture);
    }
}

fn draw_viewport_and_history(
    state: &mut EditorState,
    track: &mut Track,
    camera_zoom: &mut f32,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    ui: &mut macroquad::ui::Ui,
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

pub fn draw_help_overlay() {
    let position_x = screen_width() - 310.0;
    let position_y = 15.0;
    let width = 295.0;
    let height = 250.0;

    draw_rectangle(
        position_x,
        position_y,
        width,
        height,
        Color::new(0.02, 0.02, 0.05, 0.85),
    );
    draw_rectangle_lines(
        position_x,
        position_y,
        width,
        height,
        2.0,
        Color::new(0.3, 0.5, 0.9, 0.8),
    );

    let font_size = 15.0;
    let line_height = 20.0;
    let mut current_y = position_y + 25.0;

    draw_text(
        "EDITOR CONTROLS HUD",
        position_x + 12.0,
        current_y,
        16.0,
        GOLD,
    );
    current_y += line_height + 5.0;

    let keybinds = [
        ("WASD / Arrows", "Pan Camera"),
        ("Mouse Scroll", "Zoom to Cursor"),
        ("Left Ctrl + Scroll", "Rotate Start Grid 5°"),
        ("Middle Drag", "Pan Canvas"),
        ("Key [ F ]", "Focus Track Center"),
        ("Left Click", "Draw / Place / Drag"),
        ("Right Click", "Quick Delete Node"),
        ("Ctrl + Z / Y", "Undo / Redo"),
    ];

    for (key, action) in keybinds {
        draw_text(key, position_x + 12.0, current_y, font_size, WHITE);
        draw_text(action, position_x + 145.0, current_y, font_size, LIGHTGRAY);
        current_y += line_height;
    }
}

pub fn create_track_snapshot(track: &Track) -> EditorSnapshot {
    EditorSnapshot {
        raw_points: track.raw_points.clone(),
        grid_position: track.starting_grid.position,
        grid_rotation: track.starting_grid.rotation,
        is_closed: track.is_closed,
    }
}

pub fn save_snapshot_helper(state: &mut EditorState, track: &Track) {
    let snapshot = create_track_snapshot(track);
    state.history.push_snapshot(snapshot);
}

pub fn undo_helper(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
) {
    let current = create_track_snapshot(track);
    if let Some(snapshot) = state.history.pop_undo(current) {
        apply_track_snapshot(track, snapshot, track_texture, wall_texture);
    }
}

pub fn redo_helper(
    state: &mut EditorState,
    track: &mut Track,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
) {
    let current = create_track_snapshot(track);
    if let Some(snapshot) = state.history.pop_redo(current) {
        apply_track_snapshot(track, snapshot, track_texture, wall_texture);
    }
}

pub fn apply_track_snapshot(
    track: &mut Track,
    snapshot: EditorSnapshot,
    track_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
) {
    track.raw_points = snapshot.raw_points;
    track.starting_grid.position = snapshot.grid_position;
    track.starting_grid.rotation = snapshot.grid_rotation;
    track.is_closed = snapshot.is_closed;
    track.rebuild_mesh(5, track_texture, wall_texture);
}
