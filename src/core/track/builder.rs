use crate::core::geometry::BoundingBox;
use crate::core::track::checkpoint::CheckpointGate;
use crate::core::track::curve::{
    catmull_rom, catmull_rom_derivative, resample_points, smooth_points_with_fixed_prefix,
};
use crate::core::track::grid::StartingGrid;
use crate::core::track::segment::TrackSegment;
use macroquad::prelude::*;

pub fn prepare_track_points(
    raw_points: &[Vec2],
    starting_grid: &StartingGrid,
    is_closed: bool,
) -> Vec<Vec2> {
    let grid_start = starting_grid.start_point();
    let exit_target = starting_grid.exit_target();
    let entry_target = starting_grid.entry_target();
    let grid_end = starting_grid.end_point();
    let forward = starting_grid.forward_vector();

    let mut points = Vec::new();
    points.push(grid_start);
    points.push(exit_target);

    let mut raw_iter = raw_points.iter().peekable();
    while let Some(&&point) = raw_iter.peek() {
        let projection = (point - grid_start).dot(forward);
        if projection < 72.0 || point.distance(exit_target) < 20.0 {
            raw_iter.next();
        } else {
            break;
        }
    }

    for &point in raw_iter {
        if let Some(&last_point) = points.last() {
            if last_point.distance(point) > 15.0 {
                points.push(point);
            }
        } else {
            points.push(point);
        }
    }

    if is_closed {
        while let Some(&last_point) = points.last() {
            if last_point == grid_start || last_point == exit_target {
                break;
            }
            let projection_entry = (last_point - grid_end).dot(-forward);
            if projection_entry < 72.0 || last_point.distance(entry_target) < 20.0 {
                points.pop();
            } else {
                break;
            }
        }

        points.push(entry_target);
        points.push(grid_end);
    }

    points
}

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
        let point_current = smoothed[index];
        let point_next = smoothed[index + 1];
        let point_prev = if index > 0 {
            smoothed[index - 1]
        } else {
            point_current - (point_next - point_current)
        };
        let point_future = if index + 2 < point_count {
            smoothed[index + 2]
        } else {
            point_next + (point_next - point_current)
        };

        for step in 0..samples_per_segment {
            let step_ratio = step as f32 / samples_per_segment as f32;
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
            let normal = vec2(-tangent.y, tangent.x);

            if let Some(prev) = last_center {
                cumulative_distance += prev.distance(center);
            }
            last_center = Some(center);

            segments.push(TrackSegment {
                center,
                left_bound: center - normal * (track_width * 0.5),
                right_bound: center + normal * (track_width * 0.5),
                normal,
                tangent,
                distance_along_track: cumulative_distance,
            });
        }
    }

    segments
}

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

        let dot_product = previous_segment
            .tangent
            .dot(current_segment.tangent)
            .clamp(-1.0, 1.0);
        let angle_difference = dot_product.acos();
        accumulated_angle += angle_difference;

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

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut min_bound = vec2(f32::MAX, f32::MAX);
        let mut max_bound = vec2(f32::MIN, f32::MIN);

        for index in chunk_index..end_index {
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

            indices.push(base_index);
            indices.push(base_index + 1);
            indices.push(base_index + 2);

            indices.push(base_index + 1);
            indices.push(base_index + 3);
            indices.push(base_index + 2);
        }

        meshes.push(Mesh {
            vertices,
            indices,
            texture: track_texture.cloned(),
        });
        bounding_boxes.push(BoundingBox::new(min_bound, max_bound));

        chunk_index = end_index;
    }

    (meshes, bounding_boxes)
}

pub fn generate_wall_meshes(
    cached_grid_segments: &[TrackSegment],
    segments: &[TrackSegment],
    wall_texture: Option<&Texture2D>,
) -> (Vec<Mesh>, Vec<BoundingBox>) {
    let mut wall_meshes = Vec::new();
    let mut wall_bounding_boxes = Vec::new();

    let texture = match wall_texture {
        Some(tex) => tex,
        None => return (wall_meshes, wall_bounding_boxes),
    };

    let tile_length = 20.0;
    let wall_width = 8.0;
    let chunk_size = 150;

    let wall_chains: [&[TrackSegment]; 2] = [cached_grid_segments, segments];

    for chain in wall_chains {
        if chain.len() < 2 {
            continue;
        }

        let total_segments = chain.len() - 1;
        let mut chunk_index = 0;

        while chunk_index < total_segments {
            let end_index = (chunk_index + chunk_size).min(total_segments);

            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let mut min_bound = vec2(f32::MAX, f32::MAX);
            let mut max_bound = vec2(f32::MIN, f32::MIN);

            for index in chunk_index..end_index {
                let segment_first = &chain[index];
                let segment_second = &chain[index + 1];

                let distance_first = segment_first.distance_along_track;
                let distance_second = segment_second.distance_along_track;

                let texture_u1 = (distance_first % tile_length) / tile_length;
                let mut texture_u2 = (distance_second % tile_length) / tile_length;
                if texture_u2 < texture_u1 {
                    texture_u2 += 1.0;
                }

                // Left Wall Geometry
                let left_in_first = segment_first.left_bound;
                let left_out_first = segment_first.left_bound - segment_first.normal * wall_width;
                let left_in_second = segment_second.left_bound;
                let left_out_second =
                    segment_second.left_bound - segment_second.normal * wall_width;

                min_bound = min_bound
                    .min(left_in_first)
                    .min(left_out_first)
                    .min(left_in_second)
                    .min(left_out_second);
                max_bound = max_bound
                    .max(left_in_first)
                    .max(left_out_first)
                    .max(left_in_second)
                    .max(left_out_second);

                let base_left = vertices.len() as u16;
                vertices.push(Vertex {
                    position: vec3(left_out_first.x, left_out_first.y, 0.0),
                    uv: vec2(0.0, texture_u1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(left_in_first.x, left_in_first.y, 0.0),
                    uv: vec2(1.0, texture_u1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(left_out_second.x, left_out_second.y, 0.0),
                    uv: vec2(0.0, texture_u2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(left_in_second.x, left_in_second.y, 0.0),
                    uv: vec2(1.0, texture_u2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                indices.push(base_left);
                indices.push(base_left + 1);
                indices.push(base_left + 2);
                indices.push(base_left + 1);
                indices.push(base_left + 3);
                indices.push(base_left + 2);

                // Right Wall Geometry
                let right_in_first = segment_first.right_bound;
                let right_out_first = segment_first.right_bound + segment_first.normal * wall_width;
                let right_in_second = segment_second.right_bound;
                let right_out_second =
                    segment_second.right_bound + segment_second.normal * wall_width;

                min_bound = min_bound
                    .min(right_in_first)
                    .min(right_out_first)
                    .min(right_in_second)
                    .min(right_out_second);
                max_bound = max_bound
                    .max(right_in_first)
                    .max(right_out_first)
                    .max(right_in_second)
                    .max(right_out_second);

                let base_right = vertices.len() as u16;
                vertices.push(Vertex {
                    position: vec3(right_in_first.x, right_in_first.y, 0.0),
                    uv: vec2(0.0, texture_u1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(right_out_first.x, right_out_first.y, 0.0),
                    uv: vec2(1.0, texture_u1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(right_in_second.x, right_in_second.y, 0.0),
                    uv: vec2(0.0, texture_u2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });
                vertices.push(Vertex {
                    position: vec3(right_out_second.x, right_out_second.y, 0.0),
                    uv: vec2(1.0, texture_u2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                indices.push(base_right);
                indices.push(base_right + 1);
                indices.push(base_right + 2);
                indices.push(base_right + 1);
                indices.push(base_right + 3);
                indices.push(base_right + 2);
            }

            if !vertices.is_empty() {
                wall_meshes.push(Mesh {
                    vertices,
                    indices,
                    texture: Some(texture.clone()),
                });
                wall_bounding_boxes.push(BoundingBox::new(min_bound, max_bound));
            }

            chunk_index = end_index;
        }
    }

    (wall_meshes, wall_bounding_boxes)
}
