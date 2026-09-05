use crate::core::geometry::BoundingBox;
use crate::core::track::Track;
use macroquad::prelude::*;

pub fn draw_procedural_fallback_mesh(track: &Track) {
    if track.segments.len() < 2 {
        return;
    }

    for index in 0..track.segments.len() - 1 {
        let segment_first = &track.segments[index];
        let segment_second = &track.segments[index + 1];

        draw_quad(
            segment_first.left_bound,
            segment_first.right_bound,
            segment_second.right_bound,
            segment_second.left_bound,
            Color::new(0.2, 0.2, 0.22, 1.0),
        );

        let curb_width = 16.0;
        let stripe_index = (segment_first.distance_along_track / 32.0) as i32;
        let curb_color = if stripe_index % 2 == 0 {
            Color::new(0.85, 0.15, 0.15, 1.0)
        } else {
            WHITE
        };

        let left_outer_first = segment_first.left_bound - segment_first.normal * curb_width;
        let left_outer_second = segment_second.left_bound - segment_second.normal * curb_width;
        draw_quad(
            segment_first.left_bound,
            left_outer_first,
            left_outer_second,
            segment_second.left_bound,
            curb_color,
        );

        let right_outer_first = segment_first.right_bound + segment_first.normal * curb_width;
        let right_outer_second = segment_second.right_bound + segment_second.normal * curb_width;
        draw_quad(
            segment_first.right_bound,
            right_outer_first,
            right_outer_second,
            segment_second.right_bound,
            curb_color,
        );
    }
}

pub fn draw_starting_grid(track: &Track, grid_texture: Option<&Texture2D>) {
    let grid = &track.starting_grid;
    let right = grid.right_vector();
    let half_width = grid.width * 0.5;

    let front_center = grid.start_point();
    let front_left = front_center - right * half_width;
    let front_right = front_center + right * half_width;

    let back_center = grid.end_point();
    let back_left = back_center - right * half_width;
    let back_right = back_center + right * half_width;

    if let Some(texture) = grid_texture {
        draw_textured_grid(front_left, front_right, back_left, back_right, texture);
    } else {
        draw_untextured_grid(
            grid,
            front_center,
            front_left,
            front_right,
            back_left,
            back_right,
            right,
            half_width,
        );
    }
}

fn draw_textured_grid(
    front_left: Vec2,
    front_right: Vec2,
    back_left: Vec2,
    back_right: Vec2,
    texture: &Texture2D,
) {
    let vertex_fl = Vertex {
        position: vec3(front_left.x, front_left.y, 0.0),
        uv: vec2(0.0, 0.0),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    };
    let vertex_fr = Vertex {
        position: vec3(front_right.x, front_right.y, 0.0),
        uv: vec2(1.0, 0.0),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    };
    let vertex_br = Vertex {
        position: vec3(back_right.x, back_right.y, 0.0),
        uv: vec2(1.0, 1.0),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    };
    let vertex_bl = Vertex {
        position: vec3(back_left.x, back_left.y, 0.0),
        uv: vec2(0.0, 1.0),
        color: WHITE.into(),
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    };

    let grid_mesh = Mesh {
        vertices: vec![vertex_fl, vertex_fr, vertex_br, vertex_bl],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    };

    draw_mesh(&grid_mesh);
}

fn draw_untextured_grid(
    grid: &crate::core::track::StartingGrid,
    front_center: Vec2,
    front_left: Vec2,
    front_right: Vec2,
    back_left: Vec2,
    back_right: Vec2,
    right: Vec2,
    half_width: f32,
) {
    draw_quad(
        back_left,
        back_right,
        front_right,
        front_left,
        Color::new(0.18, 0.18, 0.2, 1.0),
    );

    let checkers = 10;
    let checker_width = grid.width / checkers as f32;
    let checker_height = 16.0;

    for index in 0..checkers {
        let offset = -half_width + (index as f32 * checker_width);
        let center_position = front_center + right * (offset + checker_width * 0.5);
        let color = if index % 2 == 0 { WHITE } else { BLACK };

        draw_rectangle_ex(
            center_position.x - checker_width * 0.5,
            center_position.y - checker_height * 0.5,
            checker_width,
            checker_height,
            DrawRectangleParams {
                color,
                rotation: grid.rotation,
                offset: vec2(0.5, 0.5),
            },
        );
    }
}

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
    let wall_chains: [&[crate::core::track::TrackSegment]; 2] = [grid_segments, &track.segments];

    for chain in wall_chains {
        if chain.len() < 2 {
            continue;
        }
        for index in 0..chain.len() - 1 {
            let segment_first = &chain[index];
            let segment_second = &chain[index + 1];

            if let Some(ref view) = view_bounds {
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
                let segment_bounding_box = BoundingBox::new(
                    min_position - vec2(10.0, 10.0),
                    max_position + vec2(10.0, 10.0),
                );
                if !view.intersects(&segment_bounding_box) {
                    continue;
                }
            }

            draw_wall_segment(segment_first, segment_second, wall_core, wall_dark);
        }
    }
}

fn draw_wall_segment(
    segment_first: &crate::core::track::TrackSegment,
    segment_second: &crate::core::track::TrackSegment,
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

pub fn draw_quad(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2, color: Color) {
    draw_triangle(v1, v2, v3, color);
    draw_triangle(v1, v3, v4, color);
}
