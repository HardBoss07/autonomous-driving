use crate::core::track::segment::TrackSegment;
use macroquad::prelude::Vec2;

pub fn find_nearest_segment(
    cached_grid_segments: &[TrackSegment],
    segments: &[TrackSegment],
    pos: Vec2,
) -> Option<(TrackSegment, usize, f32)> {
    let mut best_segment: Option<TrackSegment> = None;
    let mut min_distance_sq = f32::MAX;
    let mut best_index = 0;

    for (index, segment) in cached_grid_segments.iter().enumerate() {
        let distance_sq = segment.center.distance_squared(pos);
        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            best_segment = Some(*segment);
            best_index = index;
        }
    }

    for (index, segment) in segments.iter().enumerate() {
        let distance_sq = segment.center.distance_squared(pos);
        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            best_segment = Some(*segment);
            best_index = index;
        }
    }

    best_segment.map(|segment| (segment, best_index, min_distance_sq.sqrt()))
}

pub fn find_nearest_segment_localized(
    cached_grid_segments: &[TrackSegment],
    segments: &[TrackSegment],
    pos: Vec2,
    cached_idx: Option<usize>,
) -> Option<(TrackSegment, usize, f32)> {
    if segments.is_empty() {
        return find_nearest_segment(cached_grid_segments, segments, pos);
    }

    if let Some(center_idx) = cached_idx {
        if center_idx < segments.len() {
            let window_radius = 12;
            let start_idx = center_idx.saturating_sub(window_radius);
            let end_idx = (center_idx + window_radius + 1).min(segments.len());

            let mut local_best_seg: Option<TrackSegment> = None;
            let mut local_min_dist_sq = f32::MAX;
            let mut local_best_idx = center_idx;

            for idx in start_idx..end_idx {
                let seg = &segments[idx];
                let d_sq = seg.center.distance_squared(pos);
                if d_sq < local_min_dist_sq {
                    local_min_dist_sq = d_sq;
                    local_best_seg = Some(*seg);
                    local_best_idx = idx;
                }
            }

            if local_min_dist_sq < 22500.0 {
                return local_best_seg.map(|seg| (seg, local_best_idx, local_min_dist_sq.sqrt()));
            }
        }
    }

    find_nearest_segment(cached_grid_segments, segments, pos)
}
