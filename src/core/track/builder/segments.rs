use crate::core::geometry::Vec2Ext;
use crate::core::track::curve::{
    catmull_rom, catmull_rom_derivative, resample_points, smooth_points_with_fixed_prefix,
};
use crate::core::track::segment::TrackSegment;
use macroquad::prelude::Vec2;

pub fn generate_track_segments(
    points: &[Vec2],
    is_closed: bool,
    track_width: f32,
    samples_per_segment: usize,
) -> Vec<TrackSegment> {
    if points.len() < 3 {
        return Vec::new();
    }

    let resampled = resample_points(points, 40.0);
    let smoothed = smooth_points_with_fixed_prefix(&resampled, is_closed, 2, 2);

    let point_count = smoothed.len();
    if point_count < 3 {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut cumulative_distance = 0.0;
    let mut last_center: Option<Vec2> = None;

    for index in 0..point_count - 1 {
        let (point_prev, point_current, point_next, point_future) =
            get_segment_control_points(&smoothed, index, point_count);

        for step in 0..samples_per_segment {
            let step_ratio = step as f32 / samples_per_segment as f32;
            let segment = build_subsegment(
                point_prev,
                point_current,
                point_next,
                point_future,
                step_ratio,
                track_width,
                &mut cumulative_distance,
                &mut last_center,
            );
            segments.push(segment);
        }
    }

    segments
}

fn get_segment_control_points(
    points: &[Vec2],
    index: usize,
    point_count: usize,
) -> (Vec2, Vec2, Vec2, Vec2) {
    let point_current = points[index];
    let point_next = points[index + 1];
    let point_prev = if index > 0 {
        points[index - 1]
    } else {
        point_current - (point_next - point_current)
    };
    let point_future = if index + 2 < point_count {
        points[index + 2]
    } else {
        point_next + (point_next - point_current)
    };
    (point_prev, point_current, point_next, point_future)
}

fn build_subsegment(
    point_prev: Vec2,
    point_current: Vec2,
    point_next: Vec2,
    point_future: Vec2,
    step_ratio: f32,
    track_width: f32,
    cumulative_distance: &mut f32,
    last_center: &mut Option<Vec2>,
) -> TrackSegment {
    let center = catmull_rom(
        point_prev,
        point_current,
        point_next,
        point_future,
        step_ratio,
    );
    let tangent = catmull_rom_derivative(
        point_prev,
        point_current,
        point_next,
        point_future,
        step_ratio,
    )
    .normalize_or_zero();
    let normal = tangent.left_normal();

    if let Some(prev) = *last_center {
        *cumulative_distance += prev.distance(center);
    }
    *last_center = Some(center);

    TrackSegment {
        center,
        left_bound: center - normal * (track_width * 0.5),
        right_bound: center + normal * (track_width * 0.5),
        normal,
        tangent,
        distance_along_track: *cumulative_distance,
    }
}
