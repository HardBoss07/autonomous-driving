use crate::core::geometry::BoundingBox;
use crate::core::track::Track;
use macroquad::prelude::*;

pub fn draw_track(
    track: &Track,
    grid_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    view_bounds: Option<BoundingBox>,
) {
    if !track.meshes.is_empty() {
        if track.meshes[0].texture.is_some() {
            for (idx, mesh) in track.meshes.iter().enumerate() {
                if let Some(ref view) = view_bounds {
                    if let Some(bbox) = track.mesh_bounding_boxes.get(idx) {
                        if !view.intersects(bbox) {
                            continue;
                        }
                    }
                }
                draw_mesh(mesh);
            }
        } else {
            draw_procedural_fallback_mesh(track);
        }
    }

    draw_outer_walls(track, wall_texture, view_bounds);
    draw_starting_grid(track, grid_texture);
}

fn draw_outer_walls(
    track: &Track,
    wall_texture: Option<&Texture2D>,
    view_bounds: Option<BoundingBox>,
) {
    if wall_texture.is_some() && !track.wall_meshes.is_empty() {
        for (idx, mesh) in track.wall_meshes.iter().enumerate() {
            if let Some(ref view) = view_bounds {
                if let Some(bbox) = track.wall_mesh_bounding_boxes.get(idx) {
                    if !view.intersects(bbox) {
                        continue;
                    }
                }
            }
            draw_mesh(mesh);
        }
    } else if wall_texture.is_none() {
        let wall_core = Color::new(0.85, 0.15, 0.15, 1.0);
        let wall_dark = Color::new(0.12, 0.12, 0.15, 1.0);

        let grid_segs = track.grid_segments();
        let wall_chains: [&[crate::core::track::TrackSegment]; 2] = [grid_segs, &track.segments];

        for chain in wall_chains {
            if chain.len() < 2 {
                continue;
            }
            for i in 0..chain.len() - 1 {
                let s1 = &chain[i];
                let s2 = &chain[i + 1];

                if let Some(ref view) = view_bounds {
                    let min_p = s1.left_bound.min(s2.left_bound).min(s1.right_bound).min(s2.right_bound);
                    let max_p = s1.left_bound.max(s2.left_bound).max(s1.right_bound).max(s2.right_bound);
                    let seg_bb = BoundingBox::new(min_p - vec2(10.0, 10.0), max_p + vec2(10.0, 10.0));
                    if !view.intersects(&seg_bb) {
                        continue;
                    }
                }

                draw_line(
                    s1.left_bound.x,
                    s1.left_bound.y,
                    s2.left_bound.x,
                    s2.left_bound.y,
                    4.0,
                    wall_core,
                );
                let l1_outer = s1.left_bound - s1.normal * 3.0;
                let l2_outer = s2.left_bound - s2.normal * 3.0;
                draw_line(
                    l1_outer.x, l1_outer.y, l2_outer.x, l2_outer.y, 2.0, wall_dark,
                );

                draw_line(
                    s1.right_bound.x,
                    s1.right_bound.y,
                    s2.right_bound.x,
                    s2.right_bound.y,
                    4.0,
                    wall_core,
                );
                let r1_outer = s1.right_bound + s1.normal * 3.0;
                let r2_outer = s2.right_bound + s2.normal * 3.0;
                draw_line(
                    r1_outer.x, r1_outer.y, r2_outer.x, r2_outer.y, 2.0, wall_dark,
                );
            }
        }
    }
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

        let curb_w = 16.0;
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
