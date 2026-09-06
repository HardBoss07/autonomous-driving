use crate::core::track::checkpoint::CheckpointGate;
use crate::core::track::segment::TrackSegment;

pub fn generate_checkpoints(
    segments: &[TrackSegment],
    checkpoint_spacing: f32,
) -> Vec<CheckpointGate> {
    let total_segments = segments.len();
    if total_segments < 6 {
        return Vec::new();
    }

    let mut checkpoints = Vec::new();
    let margin = 50.0;
    let target_spacing = checkpoint_spacing.clamp(100.0, 2000.0);
    let min_curve_gate_distance = (target_spacing * 0.45).max(120.0);
    let mut checkpoint_id = 0;

    let segment_zero = &segments[0];
    let mut last_gate_position = segment_zero.center;
    let mut last_gate_distance_along = segment_zero.distance_along_track;

    checkpoints.push(CheckpointGate::from_track_segment(
        checkpoint_id,
        segment_zero,
        margin,
    ));
    checkpoint_id += 1;

    let mut accumulated_angle = 0.0f32;

    for index in 1..total_segments {
        let previous_segment = &segments[index - 1];
        let current_segment = &segments[index];

        accumulated_angle += compute_angle_difference(previous_segment, current_segment);

        let distance_along = current_segment.distance_along_track - last_gate_distance_along;
        let direct_distance = current_segment.center.distance(last_gate_position);

        let curve_threshold = 0.785;
        let reached_normal_spacing = distance_along >= target_spacing;
        let reached_curve_spacing =
            direct_distance >= min_curve_gate_distance && accumulated_angle >= curve_threshold;

        if reached_normal_spacing || reached_curve_spacing {
            checkpoints.push(CheckpointGate::from_track_segment(
                checkpoint_id,
                current_segment,
                margin,
            ));
            checkpoint_id += 1;
            last_gate_position = current_segment.center;
            last_gate_distance_along = current_segment.distance_along_track;
            accumulated_angle = 0.0;
        }
    }

    checkpoints
}

fn compute_angle_difference(
    previous_segment: &TrackSegment,
    current_segment: &TrackSegment,
) -> f32 {
    let dot_product = previous_segment
        .tangent
        .dot(current_segment.tangent)
        .clamp(-1.0, 1.0);
    dot_product.acos()
}
