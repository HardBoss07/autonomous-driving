use crate::core::geometry::BoundingBox;
use crate::core::track::{Track, TrackSegment};
use macroquad::prelude::*;

pub fn draw_outer_walls(
    track: &Track,
    wall_texture: Option<&Texture2D>,
    view_bounds: Option<BoundingBox>,
) {
    if wall_texture.is_some() && !track.wall_meshes.is_empty() {
        draw_wall_meshes(track, view_bounds);
    } else if wall_texture.is_none() {
        draw_procedural_walls(track, view_bounds);
    }
}

fn draw_wall_meshes(track: &Track, view_bounds: Option<BoundingBox>) {
    for (index, mesh) in track.wall_meshes.iter().enumerate() {
        if let Some(ref view) = view_bounds {
            if let Some(bounding_box) = track.wall_mesh_bounding_boxes.get(index) {
                if !view.intersects(bounding_box) {
                    continue;
                }
            }
        }
        draw_mesh(mesh);
    }
}

fn draw_procedural_walls(track: &Track, view_bounds: Option<BoundingBox>) {
    let wall_core = Color::new(0.85, 0.15, 0.15, 1.0);
    let wall_dark = Color::new(0.12, 0.12, 0.15, 1.0);

    let grid_segments = track.grid_segments();
    let wall_chains: [&[TrackSegment]; 2] = [grid_segments, &track.segments];

    for chain in wall_chains {
        if chain.len() < 2 {
            continue;
        }
        for index in 0..chain.len() - 1 {
            let segment_first = &chain[index];
            let segment_second = &chain[index + 1];

            if let Some(ref view) = view_bounds {
                let segment_bbox = compute_segment_bbox(segment_first, segment_second);
                if !view.intersects(&segment_bbox) {
                    continue;
                }
            }

            draw_wall_segment(segment_first, segment_second, wall_core, wall_dark);
        }
    }
}

fn compute_segment_bbox(
    segment_first: &TrackSegment,
    segment_second: &TrackSegment,
) -> BoundingBox {
    let min_position = segment_first
        .left_bound
        .min(segment_second.left_bound)
        .min(segment_first.right_bound)
        .min(segment_second.right_bound);
    let max_position = segment_first
        .left_bound
        .max(segment_second.left_bound)
        .max(segment_first.right_bound)
        .max(segment_second.right_bound);
    BoundingBox::new(
        min_position - vec2(10.0, 10.0),
        max_position + vec2(10.0, 10.0),
    )
}

fn draw_wall_segment(
    segment_first: &TrackSegment,
    segment_second: &TrackSegment,
    wall_core: Color,
    wall_dark: Color,
) {
    draw_line(
        segment_first.left_bound.x,
        segment_first.left_bound.y,
        segment_second.left_bound.x,
        segment_second.left_bound.y,
        4.0,
        wall_core,
    );
    let left_outer_first = segment_first.left_bound - segment_first.normal * 3.0;
    let left_outer_second = segment_second.left_bound - segment_second.normal * 3.0;
    draw_line(
        left_outer_first.x,
        left_outer_first.y,
        left_outer_second.x,
        left_outer_second.y,
        2.0,
        wall_dark,
    );

    draw_line(
        segment_first.right_bound.x,
        segment_first.right_bound.y,
        segment_second.right_bound.x,
        segment_second.right_bound.y,
        4.0,
        wall_core,
    );
    let right_outer_first = segment_first.right_bound + segment_first.normal * 3.0;
    let right_outer_second = segment_second.right_bound + segment_second.normal * 3.0;
    draw_line(
        right_outer_first.x,
        right_outer_first.y,
        right_outer_second.x,
        right_outer_second.y,
        2.0,
        wall_dark,
    );
}
