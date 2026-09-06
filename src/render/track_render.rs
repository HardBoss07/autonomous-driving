pub mod mesh;

use crate::core::geometry::BoundingBox;
use crate::core::track::Track;
use macroquad::prelude::*;

pub use mesh::{draw_outer_walls, draw_procedural_fallback_mesh, draw_quad, draw_starting_grid};

pub fn draw_track(
    track: &Track,
    grid_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    view_bounds: Option<BoundingBox>,
) {
    if !track.meshes.is_empty() {
        if track.meshes[0].texture.is_some() {
            draw_track_meshes(track, view_bounds);
        } else {
            draw_procedural_fallback_mesh(track);
        }
    }

    draw_outer_walls(track, wall_texture, view_bounds);
    draw_starting_grid(&track.starting_grid, grid_texture);
}

fn draw_track_meshes(track: &Track, view_bounds: Option<BoundingBox>) {
    for (index, mesh) in track.meshes.iter().enumerate() {
        if let Some(ref view) = view_bounds {
            if let Some(bounding_box) = track.mesh_bounding_boxes.get(index) {
                if !view.intersects(bounding_box) {
                    continue;
                }
            }
        }
        draw_mesh(mesh);
    }
}
