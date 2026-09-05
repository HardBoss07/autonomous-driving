pub mod checkpoint;
pub mod grid;
pub mod segment;

pub use checkpoint::CheckpointGate;
pub use grid::{StartGridConfig, StartingGrid};
pub use segment::TrackSegment;

use crate::core::geometry::BoundingBox;
use macroquad::prelude::*;

pub struct Track {
    pub width: f32,
    pub starting_grid: StartingGrid,
    pub start_grid_config: StartGridConfig,
    pub raw_points: Vec<Vec2>,
    pub segments: Vec<TrackSegment>,
    pub cached_grid_segments: Vec<TrackSegment>,
    pub checkpoints: Vec<CheckpointGate>,
    pub is_closed: bool,
    pub meshes: Vec<Mesh>,
    pub mesh_bounding_boxes: Vec<BoundingBox>,
    pub wall_meshes: Vec<Mesh>,
    pub wall_mesh_bounding_boxes: Vec<BoundingBox>,
    pub checkpoint_spacing: f32,
    pub simplify_tolerance: f32,
}

impl Track {
    pub fn new(start_pos: Vec2) -> Self {
        let starting_grid = StartingGrid::new(start_pos);
        let start_grid_config = StartGridConfig::new(start_pos, 0.0);
        let mut track = Self {
            width: starting_grid.width,
            starting_grid,
            start_grid_config,
            raw_points: Vec::new(),
            segments: Vec::new(),
            cached_grid_segments: Vec::new(),
            checkpoints: Vec::new(),
            is_closed: false,
            meshes: Vec::new(),
            mesh_bounding_boxes: Vec::new(),
            wall_meshes: Vec::new(),
            wall_mesh_bounding_boxes: Vec::new(),
            checkpoint_spacing: 350.0,
            simplify_tolerance: 3.0,
        };
        track.cached_grid_segments = track.compute_grid_segments();
        track
    }

    pub fn clear(&mut self) {
        self.raw_points.clear();
        self.segments.clear();
        self.checkpoints.clear();
        self.meshes.clear();
        self.mesh_bounding_boxes.clear();
        self.wall_meshes.clear();
        self.wall_mesh_bounding_boxes.clear();
        self.is_closed = false;
    }

    pub fn add_raw_point(&mut self, point: Vec2) {
        if let Some(&last) = self.raw_points.last() {
            if last.distance(point) < 15.0 {
                return;
            }
        }
        self.raw_points.push(point);
    }

    pub fn remove_raw_point(&mut self, index: usize) {
        if index < self.raw_points.len() {
            self.raw_points.remove(index);
        }
    }

    pub fn insert_raw_point(&mut self, index: usize, point: Vec2) {
        if index <= self.raw_points.len() {
            self.raw_points.insert(index, point);
        }
    }

    pub fn move_raw_point(&mut self, index: usize, new_pos: Vec2) {
        if index < self.raw_points.len() {
            self.raw_points[index] = new_pos;
        }
    }

    pub fn simplify_raw_points(&mut self) {
        if self.simplify_tolerance <= 0.01 || self.raw_points.len() < 3 {
            return;
        }
        self.raw_points = ramer_douglas_peucker(&self.raw_points, self.simplify_tolerance);
    }

    pub fn compute_grid_segments(&self) -> Vec<TrackSegment> {
        let grid = &self.starting_grid;
        let start = grid.end_point();
        let end = grid.start_point();
        let fwd = grid.forward_vector();
        let right = grid.right_vector();
        let half_w = grid.width * 0.5;
        let num_samples = 6;

        let mut segs = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / (num_samples - 1) as f32;
            let center = start.lerp(end, t);
            segs.push(TrackSegment {
                center,
                left_bound: center - right * half_w,
                right_bound: center + right * half_w,
                normal: right,
                tangent: fwd,
                distance_along_track: t * grid.length,
            });
        }
        segs
    }

    pub fn grid_segments(&self) -> &[TrackSegment] {
        &self.cached_grid_segments
    }

    pub fn find_nearest_segment(&self, pos: Vec2) -> Option<(TrackSegment, usize, f32)> {
        let mut best_seg: Option<TrackSegment> = None;
        let mut min_dist_sq = f32::MAX;
        let mut best_idx = 0;

        for (idx, seg) in self.cached_grid_segments.iter().enumerate() {
            let d_sq = seg.center.distance_squared(pos);
            if d_sq < min_dist_sq {
                min_dist_sq = d_sq;
                best_seg = Some(*seg);
                best_idx = idx;
            }
        }

        for (idx, seg) in self.segments.iter().enumerate() {
            let d_sq = seg.center.distance_squared(pos);
            if d_sq < min_dist_sq {
                min_dist_sq = d_sq;
                best_seg = Some(*seg);
                best_idx = idx;
            }
        }

        best_seg.map(|seg| (seg, best_idx, min_dist_sq.sqrt()))
    }

    pub fn find_nearest_segment_localized(
        &self,
        pos: Vec2,
        cached_idx: Option<usize>,
    ) -> Option<(TrackSegment, usize, f32)> {
        if self.segments.is_empty() {
            return self.find_nearest_segment(pos);
        }

        if let Some(center_idx) = cached_idx {
            if center_idx < self.segments.len() {
                let window_radius = 12;
                let start_idx = center_idx.saturating_sub(window_radius);
                let end_idx = (center_idx + window_radius + 1).min(self.segments.len());

                let mut local_best_seg: Option<TrackSegment> = None;
                let mut local_min_dist_sq = f32::MAX;
                let mut local_best_idx = center_idx;

                for idx in start_idx..end_idx {
                    let seg = &self.segments[idx];
                    let d_sq = seg.center.distance_squared(pos);
                    if d_sq < local_min_dist_sq {
                        local_min_dist_sq = d_sq;
                        local_best_seg = Some(*seg);
                        local_best_idx = idx;
                    }
                }

                // If car is within the track/kerb corridor (e.g. 150px squared = 22500), local window is reliable
                if local_min_dist_sq < 22500.0 {
                    return local_best_seg.map(|seg| (seg, local_best_idx, local_min_dist_sq.sqrt()));
                }
            }
        }

        self.find_nearest_segment(pos)
    }

    pub fn rebuild_mesh(
        &mut self,
        samples_per_segment: usize,
        track_texture: Option<&Texture2D>,
        wall_texture: Option<&Texture2D>,
    ) {
        self.segments.clear();
        self.checkpoints.clear();
        self.cached_grid_segments = self.compute_grid_segments();

        if self.raw_points.is_empty() {
            self.meshes.clear();
            self.mesh_bounding_boxes.clear();
            self.wall_meshes.clear();
            self.wall_mesh_bounding_boxes.clear();
            return;
        }

        let grid_start = self.starting_grid.start_point();
        let exit_target = self.starting_grid.exit_target();
        let entry_target = self.starting_grid.entry_target();
        let grid_end = self.starting_grid.end_point();

        let fwd = self.starting_grid.forward_vector();

        self.start_grid_config =
            StartGridConfig::new(grid_start - fwd * 100.0, self.starting_grid.rotation);

        let mut pts = Vec::new();
        pts.push(grid_start);
        pts.push(exit_target);

        let mut raw_iter = self.raw_points.iter().peekable();
        while let Some(&&p) = raw_iter.peek() {
            let proj = (p - grid_start).dot(fwd);
            if proj < 72.0 || p.distance(exit_target) < 20.0 {
                raw_iter.next();
            } else {
                break;
            }
        }

        for &p in raw_iter {
            if let Some(&last) = pts.last() {
                if last.distance(p) > 15.0 {
                    pts.push(p);
                }
            } else {
                pts.push(p);
            }
        }

        if self.is_closed {
            while let Some(&last) = pts.last() {
                if last == grid_start || last == exit_target {
                    break;
                }
                let proj_entry = (last - grid_end).dot(-fwd);
                if proj_entry < 72.0 || last.distance(entry_target) < 20.0 {
                    pts.pop();
                } else {
                    break;
                }
            }

            pts.push(entry_target);
            pts.push(grid_end);
        }

        if pts.len() < 3 {
            return;
        }

        let resampled = resample_points(&pts, 40.0);
        let smoothed = smooth_points_with_fixed_prefix(&resampled, self.is_closed, 2, 2);

        let len = smoothed.len();
        if len < 3 {
            return;
        }

        let mut cumulative_dist = 0.0;
        let mut last_center: Option<Vec2> = None;

        for i in 0..len - 1 {
            let p1 = smoothed[i];
            let p2 = smoothed[i + 1];
            let p0 = if i > 0 {
                smoothed[i - 1]
            } else {
                p1 - (p2 - p1)
            };
            let p3 = if i + 2 < len {
                smoothed[i + 2]
            } else {
                p2 + (p2 - p1)
            };

            for step in 0..samples_per_segment {
                let t = step as f32 / samples_per_segment as f32;
                let center = catmull_rom(p0, p1, p2, p3, t);
                let tangent = catmull_rom_derivative(p0, p1, p2, p3, t).normalize_or_zero();
                let normal = vec2(-tangent.y, tangent.x);

                if let Some(prev) = last_center {
                    cumulative_dist += prev.distance(center);
                }
                last_center = Some(center);

                self.segments.push(TrackSegment {
                    center,
                    left_bound: center - normal * (self.width * 0.5),
                    right_bound: center + normal * (self.width * 0.5),
                    normal,
                    tangent,
                    distance_along_track: cumulative_dist,
                });
            }
        }

        self.generate_checkpoints();
        self.generate_gpu_mesh(track_texture);
        self.generate_wall_meshes(wall_texture);
    }

    fn generate_checkpoints(&mut self) {
        let total_segs = self.segments.len();
        if total_segs < 6 {
            return;
        }

        let margin = 50.0;
        let target_spacing = self.checkpoint_spacing.clamp(100.0, 2000.0);
        let min_curve_gate_distance = (target_spacing * 0.45).max(120.0);
        let mut id = 0;

        let seg0 = &self.segments[0];
        let mut last_gate_pos = seg0.center;
        let mut last_gate_dist_along = seg0.distance_along_track;

        self.checkpoints
            .push(CheckpointGate::from_track_segment(id, seg0, margin));
        id += 1;

        let mut accumulated_angle = 0.0f32;

        for idx in 1..total_segs {
            let prev_seg = &self.segments[idx - 1];
            let curr_seg = &self.segments[idx];

            let dot = prev_seg.tangent.dot(curr_seg.tangent).clamp(-1.0, 1.0);
            let angle_diff = dot.acos();
            accumulated_angle += angle_diff;

            let dist_along = curr_seg.distance_along_track - last_gate_dist_along;
            let direct_dist = curr_seg.center.distance(last_gate_pos);

            let curve_threshold = 0.785;

            let reached_normal_spacing = dist_along >= target_spacing;
            let reached_curve_spacing =
                direct_dist >= min_curve_gate_distance && accumulated_angle >= curve_threshold;

            if reached_normal_spacing || reached_curve_spacing {
                self.checkpoints
                    .push(CheckpointGate::from_track_segment(id, curr_seg, margin));
                id += 1;
                last_gate_pos = curr_seg.center;
                last_gate_dist_along = curr_seg.distance_along_track;
                accumulated_angle = 0.0;
            }
        }
    }

    fn generate_gpu_mesh(&mut self, track_texture: Option<&Texture2D>) {
        self.meshes.clear();
        self.mesh_bounding_boxes.clear();

        if self.segments.len() < 2 {
            return;
        }

        let tile_length = 120.0;
        let chunk_size = 200;

        let total_segments = self.segments.len() - 1;
        let mut idx = 0;

        while idx < total_segments {
            let end_idx = (idx + chunk_size).min(total_segments);

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let mut min_bound = vec2(f32::MAX, f32::MAX);
            let mut max_bound = vec2(f32::MIN, f32::MIN);

            for i in idx..end_idx {
                let seg1 = &self.segments[i];
                let seg2 = &self.segments[i + 1];

                let d1 = seg1.distance_along_track;
                let d2 = seg2.distance_along_track;

                let v1 = (d1 % tile_length) / tile_length;
                let mut v2 = (d2 % tile_length) / tile_length;

                if v2 < v1 {
                    v2 += 1.0;
                }

                min_bound = min_bound
                    .min(seg1.left_bound)
                    .min(seg1.right_bound)
                    .min(seg2.left_bound)
                    .min(seg2.right_bound);
                max_bound = max_bound
                    .max(seg1.left_bound)
                    .max(seg1.right_bound)
                    .max(seg2.left_bound)
                    .max(seg2.right_bound);

                let base_idx = vertices.len() as u16;

                vertices.push(Vertex {
                    position: vec3(seg1.left_bound.x, seg1.left_bound.y, 0.0),
                    uv: vec2(0.0, v1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                vertices.push(Vertex {
                    position: vec3(seg1.right_bound.x, seg1.right_bound.y, 0.0),
                    uv: vec2(1.0, v1),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                vertices.push(Vertex {
                    position: vec3(seg2.left_bound.x, seg2.left_bound.y, 0.0),
                    uv: vec2(0.0, v2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                vertices.push(Vertex {
                    position: vec3(seg2.right_bound.x, seg2.right_bound.y, 0.0),
                    uv: vec2(1.0, v2),
                    color: WHITE.into(),
                    normal: vec4(0.0, 0.0, 1.0, 0.0),
                });

                indices.push(base_idx);
                indices.push(base_idx + 1);
                indices.push(base_idx + 2);

                indices.push(base_idx + 1);
                indices.push(base_idx + 3);
                indices.push(base_idx + 2);
            }

            self.meshes.push(Mesh {
                vertices,
                indices,
                texture: track_texture.cloned(),
            });
            self.mesh_bounding_boxes
                .push(BoundingBox::new(min_bound, max_bound));

            idx = end_idx;
        }
    }

    fn generate_wall_meshes(&mut self, wall_texture: Option<&Texture2D>) {
        self.wall_meshes.clear();
        self.wall_mesh_bounding_boxes.clear();

        let texture = match wall_texture {
            Some(tex) => tex,
            None => return,
        };

        let tile_length = 20.0;
        let wall_width = 8.0;
        let chunk_size = 150;

        let wall_chains: [&[TrackSegment]; 2] = [&self.cached_grid_segments, &self.segments];

        for chain in wall_chains {
            if chain.len() < 2 {
                continue;
            }

            let total_segments = chain.len() - 1;
            let mut idx = 0;

            while idx < total_segments {
                let end_idx = (idx + chunk_size).min(total_segments);

                let mut vertices = Vec::new();
                let mut indices = Vec::new();

                let mut min_bound = vec2(f32::MAX, f32::MAX);
                let mut max_bound = vec2(f32::MIN, f32::MIN);

                for i in idx..end_idx {
                    let s1 = &chain[i];
                    let s2 = &chain[i + 1];

                    let d1 = s1.distance_along_track;
                    let d2 = s2.distance_along_track;

                    let u1 = (d1 % tile_length) / tile_length;
                    let mut u2 = (d2 % tile_length) / tile_length;
                    if u2 < u1 {
                        u2 += 1.0;
                    }

                    // Left Wall Geometry
                    let l1_in = s1.left_bound;
                    let l1_out = s1.left_bound - s1.normal * wall_width;
                    let l2_in = s2.left_bound;
                    let l2_out = s2.left_bound - s2.normal * wall_width;

                    min_bound = min_bound
                        .min(l1_in)
                        .min(l1_out)
                        .min(l2_in)
                        .min(l2_out);
                    max_bound = max_bound
                        .max(l1_in)
                        .max(l1_out)
                        .max(l2_in)
                        .max(l2_out);

                    let base_l = vertices.len() as u16;
                    vertices.push(Vertex {
                        position: vec3(l1_out.x, l1_out.y, 0.0),
                        uv: vec2(0.0, u1),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(l1_in.x, l1_in.y, 0.0),
                        uv: vec2(1.0, u1),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(l2_out.x, l2_out.y, 0.0),
                        uv: vec2(0.0, u2),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(l2_in.x, l2_in.y, 0.0),
                        uv: vec2(1.0, u2),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });

                    indices.push(base_l);
                    indices.push(base_l + 1);
                    indices.push(base_l + 2);
                    indices.push(base_l + 1);
                    indices.push(base_l + 3);
                    indices.push(base_l + 2);

                    // Right Wall Geometry
                    let r1_in = s1.right_bound;
                    let r1_out = s1.right_bound + s1.normal * wall_width;
                    let r2_in = s2.right_bound;
                    let r2_out = s2.right_bound + s2.normal * wall_width;

                    min_bound = min_bound
                        .min(r1_in)
                        .min(r1_out)
                        .min(r2_in)
                        .min(r2_out);
                    max_bound = max_bound
                        .max(r1_in)
                        .max(r1_out)
                        .max(r2_in)
                        .max(r2_out);

                    let base_r = vertices.len() as u16;
                    vertices.push(Vertex {
                        position: vec3(r1_in.x, r1_in.y, 0.0),
                        uv: vec2(0.0, u1),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(r1_out.x, r1_out.y, 0.0),
                        uv: vec2(1.0, u1),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(r2_in.x, r2_in.y, 0.0),
                        uv: vec2(0.0, u2),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });
                    vertices.push(Vertex {
                        position: vec3(r2_out.x, r2_out.y, 0.0),
                        uv: vec2(1.0, u2),
                        color: WHITE.into(),
                        normal: vec4(0.0, 0.0, 1.0, 0.0),
                    });

                    indices.push(base_r);
                    indices.push(base_r + 1);
                    indices.push(base_r + 2);
                    indices.push(base_r + 1);
                    indices.push(base_r + 3);
                    indices.push(base_r + 2);
                }

                if !vertices.is_empty() {
                    self.wall_meshes.push(Mesh {
                        vertices,
                        indices,
                        texture: Some(texture.clone()),
                    });
                    self.wall_mesh_bounding_boxes
                        .push(BoundingBox::new(min_bound, max_bound));
                }

                idx = end_idx;
            }
        }
    }
}

fn perpendicular_distance(point: Vec2, line_a: Vec2, line_b: Vec2) -> f32 {
    let len_sq = line_a.distance_squared(line_b);
    if len_sq < 1e-6 {
        return point.distance(line_a);
    }
    let t = ((point - line_a).dot(line_b - line_a) / len_sq).clamp(0.0, 1.0);
    let projection = line_a + (line_b - line_a) * t;
    point.distance(projection)
}

fn ramer_douglas_peucker(points: &[Vec2], epsilon: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut dmax = 0.0;
    let mut index = 0;
    let end = points.len() - 1;

    for i in 1..end {
        let d = perpendicular_distance(points[i], points[0], points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon {
        let mut rec1 = ramer_douglas_peucker(&points[..=index], epsilon);
        let rec2 = ramer_douglas_peucker(&points[index..], epsilon);

        rec1.pop();
        rec1.extend(rec2);
        rec1
    } else {
        vec![points[0], points[end]]
    }
}

fn resample_points(pts: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result = vec![pts[0]];
    let mut prev = pts[0];
    let mut accumulated = 0.0;

    for &curr in pts.iter().skip(1) {
        let dist = prev.distance(curr);
        if dist < 0.001 {
            continue;
        }

        let dir = (curr - prev) / dist;
        let mut step = spacing - accumulated;

        while step <= dist {
            let new_pt = prev + dir * step;
            result.push(new_pt);
            step += spacing;
        }
        accumulated = dist - (step - spacing);
        prev = curr;
    }

    if let Some(&last) = pts.last() {
        if result.last().map_or(true, |&p| p.distance(last) > 5.0) {
            result.push(last);
        }
    }

    result
}

fn smooth_points_with_fixed_prefix(
    pts: &[Vec2],
    is_closed: bool,
    iterations: usize,
    fixed_prefix_count: usize,
) -> Vec<Vec2> {
    if pts.len() < 3 {
        return pts.to_vec();
    }

    let mut current = pts.to_vec();
    for _ in 0..iterations {
        let mut next = current.clone();
        let len = current.len();

        for i in 1..len - 1 {
            if i < fixed_prefix_count {
                continue;
            }
            next[i] = current[i - 1] * 0.25 + current[i] * 0.5 + current[i + 1] * 0.25;
        }

        if is_closed {
            next[0] = current[len - 1] * 0.25 + current[0] * 0.5 + current[1] * 0.25;
            next[len - 1] = current[len - 2] * 0.25 + current[len - 1] * 0.5 + current[0] * 0.25;
        }

        current = next;
    }

    current
}

fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    0.5 * ((p1 * 2.0)
        + (-p0 + p2) * t
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t * t
        + (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * t * t * t)
}

fn catmull_rom_derivative(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    0.5 * ((-p0 + p2)
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * 2.0 * t
        + (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * 3.0 * t * t)
}
