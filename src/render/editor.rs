pub mod input;
pub mod preview;
pub mod state;
pub mod ui;

pub use preview::{
    draw_checkpoint_gates, draw_grid_targets, draw_nodes, draw_track_spline_preview,
};
pub use state::{EditorHistory, EditorSnapshot, EditorState, EditorTool};

use crate::core::track::Track;
use macroquad::prelude::*;

pub struct TrackEditor {
    pub state: EditorState,
}

impl TrackEditor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
        }
    }

    pub fn is_mouse_over_ui(&self, mouse_pos: Vec2) -> bool {
        ui::is_mouse_over_ui(&self.state, mouse_pos)
    }

    pub fn save_snapshot(&mut self, track: &Track) {
        ui::save_snapshot_helper(&mut self.state, track);
    }

    pub fn undo(
        &mut self,
        track: &mut Track,
        track_tex: Option<&Texture2D>,
        wall_tex: Option<&Texture2D>,
    ) {
        ui::undo_helper(&mut self.state, track, track_tex, wall_tex);
    }

    pub fn redo(
        &mut self,
        track: &mut Track,
        track_tex: Option<&Texture2D>,
        wall_tex: Option<&Texture2D>,
    ) {
        ui::redo_helper(&mut self.state, track, track_tex, wall_tex);
    }

    pub fn update_and_draw_ui(
        &mut self,
        track: &mut Track,
        camera_zoom: &mut f32,
        track_tex: Option<&Texture2D>,
        wall_tex: Option<&Texture2D>,
    ) -> bool {
        ui::update_and_draw_ui(&mut self.state, track, camera_zoom, track_tex, wall_tex)
    }

    pub fn handle_input(
        &mut self,
        track: &mut Track,
        track_tex: Option<&Texture2D>,
        wall_tex: Option<&Texture2D>,
        world_mouse: Vec2,
        screen_mouse: Vec2,
        camera_target: &mut Vec2,
        camera_zoom: &mut f32,
        dt: f32,
    ) {
        input::handle_editor_input(
            &mut self.state,
            track,
            track_tex,
            wall_tex,
            world_mouse,
            screen_mouse,
            camera_target,
            camera_zoom,
            dt,
        );
    }

    pub fn draw_snap_previews(&self, track: &Track, world_mouse: Vec2, camera_zoom: f32) {
        preview::draw_grid_targets(track, world_mouse, self.state.snap_distance);
        preview::draw_checkpoint_gates(track);
        preview::draw_track_spline_preview(track);
        preview::draw_nodes(
            track,
            self.state.selected_node_index,
            self.state.hovered_node_index,
            camera_zoom,
        );
    }

    pub fn draw_help_overlay(&self) {
        ui::draw_help_overlay();
    }
}

// Delegate field accesses for backwards compatibility
impl std::ops::Deref for TrackEditor {
    type Target = EditorState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for TrackEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
