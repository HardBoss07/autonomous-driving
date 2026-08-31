use crate::core::track::Track;
use macroquad::prelude::*;

pub fn draw_track(track: &Track, grid_texture: Option<&Texture2D>) {
    if !track.meshes.is_empty() {
        if track.meshes[0].texture.is_some() {
            for mesh in &track.meshes {
                draw_mesh(mesh);
            }
        } else {
            draw_procedural_fallback_mesh(track);
        }
    }

    draw_starting_grid(track, grid_texture);
}

fn draw_procedural_fallback_mesh(track: &Track) {
    if track.segments.len() < 2 {
        return;
    }

    for i in 0..track.segments.len() - 1 {
        let seg1 = &track.segments[i];
        let seg2 = &track.segments[i + 1];

        draw_quad(
            seg1.left_bound,
            seg1.right_bound,
            seg2.right_bound,
            seg2.left_bound,
            Color::new(0.2, 0.2, 0.22, 1.0),
        );

        let curb_w = 12.0;
        let stripe_idx = (seg1.distance_along_track / 32.0) as i32;
        let curb_color = if stripe_idx % 2 == 0 {
            Color::new(0.85, 0.15, 0.15, 1.0)
        } else {
            WHITE
        };

        let l1_out = seg1.left_bound - seg1.normal * curb_w;
        let l2_out = seg2.left_bound - seg2.normal * curb_w;
        draw_quad(seg1.left_bound, l1_out, l2_out, seg2.left_bound, curb_color);

        let r1_out = seg1.right_bound + seg1.normal * curb_w;
        let r2_out = seg2.right_bound + seg2.normal * curb_w;
        draw_quad(
            seg1.right_bound,
            r1_out,
            r2_out,
            seg2.right_bound,
            curb_color,
        );
    }
}

fn draw_starting_grid(track: &Track, grid_texture: Option<&Texture2D>) {
    let grid = &track.starting_grid;
    let right = grid.right_vector();
    let half_w = grid.width * 0.5;

    let front_center = grid.start_point();
    let front_left = front_center - right * half_w;
    let front_right = front_center + right * half_w;

    let back_center = grid.end_point();
    let back_left = back_center - right * half_w;
    let back_right = back_center + right * half_w;

    if let Some(texture) = grid_texture {
        let v_fl = Vertex {
            position: vec3(front_left.x, front_left.y, 0.0),
            uv: vec2(0.0, 0.0),
            color: WHITE.into(),
            normal: vec4(0.0, 0.0, 1.0, 0.0),
        };
        let v_fr = Vertex {
            position: vec3(front_right.x, front_right.y, 0.0),
            uv: vec2(1.0, 0.0),
            color: WHITE.into(),
            normal: vec4(0.0, 0.0, 1.0, 0.0),
        };
        let v_br = Vertex {
            position: vec3(back_right.x, back_right.y, 0.0),
            uv: vec2(1.0, 1.0),
            color: WHITE.into(),
            normal: vec4(0.0, 0.0, 1.0, 0.0),
        };
        let v_bl = Vertex {
            position: vec3(back_left.x, back_left.y, 0.0),
            uv: vec2(0.0, 1.0),
            color: WHITE.into(),
            normal: vec4(0.0, 0.0, 1.0, 0.0),
        };

        let grid_mesh = Mesh {
            vertices: vec![v_fl, v_fr, v_br, v_bl],
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: Some(texture.clone()),
        };

        draw_mesh(&grid_mesh);
    } else {
        draw_quad(
            back_left,
            back_right,
            front_right,
            front_left,
            Color::new(0.18, 0.18, 0.2, 1.0),
        );

        let checkers = 10;
        let check_w = grid.width / checkers as f32;
        let check_h = 16.0;

        for i in 0..checkers {
            let offset = -half_w + (i as f32 * check_w);
            let c_pos = front_center + right * (offset + check_w * 0.5);
            let color = if i % 2 == 0 { WHITE } else { BLACK };

            draw_rectangle_ex(
                c_pos.x - check_w * 0.5,
                c_pos.y - check_h * 0.5,
                check_w,
                check_h,
                DrawRectangleParams {
                    color,
                    rotation: grid.rotation,
                    offset: vec2(0.5, 0.5),
                },
            );
        }
    }
}

fn draw_quad(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2, color: Color) {
    draw_triangle(v1, v2, v3, color);
    draw_triangle(v1, v3, v4, color);
}
