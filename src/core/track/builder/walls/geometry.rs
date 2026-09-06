use crate::core::track::segment::TrackSegment;
use macroquad::prelude::*;

pub struct WallQuadParams<'a> {
    pub inner_first: Vec2,
    pub inner_second: Vec2,
    pub normal_first: Vec2,
    pub normal_second: Vec2,
    pub texture_u1: f32,
    pub texture_u2: f32,
    pub wall_width: f32,
    pub vertices: &'a mut Vec<Vertex>,
    pub indices: &'a mut Vec<u16>,
    pub min_bound: &'a mut Vec2,
    pub max_bound: &'a mut Vec2,
}

pub fn append_wall_strip(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    min_bound: &mut Vec2,
    max_bound: &mut Vec2,
    inner_first: Vec2,
    inner_second: Vec2,
    outer_first: Vec2,
    outer_second: Vec2,
    texture_u1: f32,
    texture_u2: f32,
) {
    *min_bound = (*min_bound)
        .min(inner_first)
        .min(outer_first)
        .min(inner_second)
        .min(outer_second);
    *max_bound = (*max_bound)
        .max(inner_first)
        .max(outer_first)
        .max(inner_second)
        .max(outer_second);

    let base_index = vertices.len() as u16;
    vertices.push(Vertex {
        position: vec3(outer_first.x, outer_first.y, 0.0),
        uv: vec2(0.0, texture_u1),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(inner_first.x, inner_first.y, 0.0),
        uv: vec2(1.0, texture_u1),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(outer_second.x, outer_second.y, 0.0),
        uv: vec2(0.0, texture_u2),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });
    vertices.push(Vertex {
        position: vec3(inner_second.x, inner_second.y, 0.0),
        uv: vec2(1.0, texture_u2),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    });

    indices.push(base_index);
    indices.push(base_index + 1);
    indices.push(base_index + 2);
    indices.push(base_index + 1);
    indices.push(base_index + 3);
    indices.push(base_index + 2);
}

pub fn append_left_wall(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    min_bound: &mut Vec2,
    max_bound: &mut Vec2,
    segment_first: &TrackSegment,
    segment_second: &TrackSegment,
    texture_u1: f32,
    texture_u2: f32,
    wall_width: f32,
) {
    let inner_first = segment_first.left_bound;
    let outer_first = segment_first.left_bound - segment_first.normal * wall_width;
    let inner_second = segment_second.left_bound;
    let outer_second = segment_second.left_bound - segment_second.normal * wall_width;

    append_wall_strip(
        vertices,
        indices,
        min_bound,
        max_bound,
        inner_first,
        inner_second,
        outer_first,
        outer_second,
        texture_u1,
        texture_u2,
    );
}

pub fn append_right_wall(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    min_bound: &mut Vec2,
    max_bound: &mut Vec2,
    segment_first: &TrackSegment,
    segment_second: &TrackSegment,
    texture_u1: f32,
    texture_u2: f32,
    wall_width: f32,
) {
    let inner_first = segment_first.right_bound;
    let outer_first = segment_first.right_bound + segment_first.normal * wall_width;
    let inner_second = segment_second.right_bound;
    let outer_second = segment_second.right_bound + segment_second.normal * wall_width;

    append_wall_strip(
        vertices,
        indices,
        min_bound,
        max_bound,
        inner_first,
        inner_second,
        outer_first,
        outer_second,
        texture_u1,
        texture_u2,
    );
}
