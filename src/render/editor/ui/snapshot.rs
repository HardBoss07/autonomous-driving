use crate::core::track::Track;
use crate::render::editor::state::{EditorSnapshot, EditorState};
use macroquad::prelude::Texture2D;

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
