use crate::core::track::Track;
use crate::render::track_render::mesh::primitives::draw_quad;
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

        draw_curbs(segment_first, segment_second);
    }
}

fn draw_curbs(
    segment_first: &crate::core::track::TrackSegment,
    segment_second: &crate::core::track::TrackSegment,
) {
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
