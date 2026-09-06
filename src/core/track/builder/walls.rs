pub mod geometry;

use crate::core::geometry::BoundingBox;
use crate::core::track::segment::TrackSegment;
use geometry::{append_left_wall, append_right_wall};
use macroquad::prelude::*;

pub fn generate_wall_meshes(
    cached_grid_segments: &[TrackSegment],
    segments: &[TrackSegment],
    wall_texture: Option<&Texture2D>,
) -> (Vec<Mesh>, Vec<BoundingBox>) {
    let mut wall_meshes = Vec::new();
    let mut wall_bounding_boxes = Vec::new();

    let texture = match wall_texture {
        Some(tex) => tex,
        None => return (wall_meshes, wall_bounding_boxes),
    };

    let tile_length = 20.0;
    let wall_width = 8.0;
    let chunk_size = 150;
    let wall_chains: [&[TrackSegment]; 2] = [cached_grid_segments, segments];

    for chain in wall_chains {
        if chain.len() < 2 {
            continue;
        }

        let total_segments = chain.len() - 1;
        let mut chunk_index = 0;

        while chunk_index < total_segments {
            let end_index = (chunk_index + chunk_size).min(total_segments);
            if let Some((mesh, bbox)) = build_wall_chunk(
                chain,
                chunk_index,
                end_index,
                tile_length,
                wall_width,
                texture,
            ) {
                wall_meshes.push(mesh);
                wall_bounding_boxes.push(bbox);
            }
            chunk_index = end_index;
        }
    }

    (wall_meshes, wall_bounding_boxes)
}

fn build_wall_chunk(
    chain: &[TrackSegment],
    start_index: usize,
    end_index: usize,
    tile_length: f32,
    wall_width: f32,
    texture: &Texture2D,
) -> Option<(Mesh, BoundingBox)> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut min_bound = vec2(f32::MAX, f32::MAX);
    let mut max_bound = vec2(f32::MIN, f32::MIN);

    for index in start_index..end_index {
        let segment_first = &chain[index];
        let segment_second = &chain[index + 1];

        let distance_first = segment_first.distance_along_track;
        let distance_second = segment_second.distance_along_track;

        let texture_u1 = (distance_first % tile_length) / tile_length;
        let mut texture_u2 = (distance_second % tile_length) / tile_length;
        if texture_u2 < texture_u1 {
            texture_u2 += 1.0;
        }

        append_left_wall(
            &mut vertices,
            &mut indices,
            &mut min_bound,
            &mut max_bound,
            segment_first,
            segment_second,
            texture_u1,
            texture_u2,
            wall_width,
        );

        append_right_wall(
            &mut vertices,
            &mut indices,
            &mut min_bound,
            &mut max_bound,
            segment_first,
            segment_second,
            texture_u1,
            texture_u2,
            wall_width,
        );
    }

    if vertices.is_empty() {
        None
    } else {
        Some((
            Mesh {
                vertices,
                indices,
                texture: Some(texture.clone()),
            },
            BoundingBox::new(min_bound, max_bound),
        ))
    }
}
