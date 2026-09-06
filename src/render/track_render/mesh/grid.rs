use crate::core::track::StartingGrid;
use crate::render::track_render::mesh::primitives::draw_quad;
use macroquad::prelude::*;

pub fn draw_starting_grid(grid: &StartingGrid, grid_texture: Option<&Texture2D>) {
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
    grid: &StartingGrid,
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
