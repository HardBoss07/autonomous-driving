use crate::core::geometry::BoundingBox;
use crate::core::track::segment::TrackSegment;
use macroquad::prelude::*;

pub fn generate_gpu_mesh(
    segments: &[TrackSegment],
    track_texture: Option<&Texture2D>,
) -> (Vec<Mesh>, Vec<BoundingBox>) {
    let mut meshes = Vec::new();
    let mut bounding_boxes = Vec::new();

    if segments.len() < 2 {
        return (meshes, bounding_boxes);
    }

    let tile_length = 120.0;
    let chunk_size = 200;
    let total_segments = segments.len() - 1;
    let mut chunk_index = 0;

    while chunk_index < total_segments {
        let end_index = (chunk_index + chunk_size).min(total_segments);
        let (mesh, bbox) =
            build_mesh_chunk(segments, chunk_index, end_index, tile_length, track_texture);
        meshes.push(mesh);
        bounding_boxes.push(bbox);
        chunk_index = end_index;
    }

    (meshes, bounding_boxes)
}

fn build_mesh_chunk(
    segments: &[TrackSegment],
    start_index: usize,
    end_index: usize,
    tile_length: f32,
    track_texture: Option<&Texture2D>,
) -> (Mesh, BoundingBox) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut min_bound = vec2(f32::MAX, f32::MAX);
    let mut max_bound = vec2(f32::MIN, f32::MIN);

    for index in start_index..end_index {
        let segment_first = &segments[index];
        let segment_second = &segments[index + 1];

        let distance_first = segment_first.distance_along_track;
        let distance_second = segment_second.distance_along_track;

        let texture_v1 = (distance_first % tile_length) / tile_length;
        let mut texture_v2 = (distance_second % tile_length) / tile_length;
        if texture_v2 < texture_v1 {
            texture_v2 += 1.0;
        }

        min_bound = min_bound
            .min(segment_first.left_bound)
            .min(segment_first.right_bound)
            .min(segment_second.left_bound)
            .min(segment_second.right_bound);
        max_bound = max_bound
            .max(segment_first.left_bound)
            .max(segment_first.right_bound)
            .max(segment_second.left_bound)
            .max(segment_second.right_bound);

        let base_index = vertices.len() as u16;
        append_segment_quad(
            &mut vertices,
            segment_first,
            segment_second,
            texture_v1,
            texture_v2,
        );
        append_quad_indices(&mut indices, base_index);
    }

    let mesh = Mesh {
        vertices,
        indices,
        texture: track_texture.cloned(),
    };
    (mesh, BoundingBox::new(min_bound, max_bound))
}

fn append_segment_quad(
    vertices: &mut Vec<Vertex>,
    segment_first: &TrackSegment,
    segment_second: &TrackSegment,
    texture_v1: f32,
    texture_v2: f32,
) {
    vertices.push(Vertex {
        position: vec3(segment_first.left_bound.x, segment_first.left_bound.y, 0.0),
        uv: vec2(0.0, texture_v1),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(
            segment_first.right_bound.x,
            segment_first.right_bound.y,
            0.0,
        ),
        uv: vec2(1.0, texture_v1),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(
            segment_second.left_bound.x,
            segment_second.left_bound.y,
            0.0,
        ),
        uv: vec2(0.0, texture_v2),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(
            segment_second.right_bound.x,
            segment_second.right_bound.y,
            0.0,
        ),
        uv: vec2(1.0, texture_v2),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
}

fn append_quad_indices(indices: &mut Vec<u16>, base_index: u16) {
    indices.push(base_index);
    indices.push(base_index + 1);
    indices.push(base_index + 2);

    indices.push(base_index + 1);
    indices.push(base_index + 3);
    indices.push(base_index + 2);
}
