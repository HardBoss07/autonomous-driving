use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct TrackSegment {
    pub center: Vec2,
    pub left_bound: Vec2,
    pub right_bound: Vec2,
    pub normal: Vec2,
    pub tangent: Vec2,
    pub distance_along_track: f32,
}

#[derive(Clone, Debug)]
pub struct StartingGrid {
    pub position: Vec2,
    pub rotation: f32,
    pub width: f32,
    pub length: f32,
}

impl StartingGrid {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            width: 140.0,
            length: 220.0,
        }
    }

    pub fn forward_vector(&self) -> Vec2 {
        vec2(self.rotation.cos(), self.rotation.sin())
    }

    pub fn right_vector(&self) -> Vec2 {
        let fwd = self.forward_vector();
        vec2(-fwd.y, fwd.x)
    }

    pub fn start_point(&self) -> Vec2 {
        self.position + self.forward_vector() * (self.length * 0.5)
    }

    pub fn end_point(&self) -> Vec2 {
        self.position - self.forward_vector() * (self.length * 0.5)
    }

    pub fn exit_target(&self) -> Vec2 {
        self.start_point() + self.forward_vector() * 70.0
    }

    pub fn entry_target(&self) -> Vec2 {
        self.end_point() - self.forward_vector() * 70.0
    }
}

pub struct Track {
    pub width: f32,
    pub starting_grid: StartingGrid,
    pub raw_points: Vec<Vec2>,
    pub segments: Vec<TrackSegment>,
    pub is_closed: bool,
    pub meshes: Vec<Mesh>,
}

impl Track {
    pub fn new(start_pos: Vec2) -> Self {
        let starting_grid = StartingGrid::new(start_pos);
        Self {
            width: starting_grid.width,
            starting_grid,
            raw_points: Vec::new(),
            segments: Vec::new(),
            is_closed: false,
            meshes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.raw_points.clear();
        self.segments.clear();
        self.meshes.clear();
        self.is_closed = false;
    }

    pub fn add_raw_point(&mut self, point: Vec2) {
        if let Some(&last) = self.raw_points.last() {
            if last.distance(point) < 20.0 {
                return;
            }
        }
        self.raw_points.push(point);
    }

    pub fn rebuild_mesh(&mut self, samples_per_segment: usize, track_texture: Option<&Texture2D>) {
        self.segments.clear();

        if self.raw_points.is_empty() {
            return;
        }

        let grid_start = self.starting_grid.start_point();
        let grid_end = self.starting_grid.end_point();
        let exit_target = self.starting_grid.exit_target();
        let entry_target = self.starting_grid.entry_target();

        let mut pts = Vec::new();
        pts.push(grid_end);
        pts.push(grid_start);
        pts.push(exit_target);

        let fixed_prefix_count = pts.len();

        for &p in &self.raw_points {
            if let Some(&last) = pts.last() {
                if last.distance(p) > 2.0 {
                    pts.push(p);
                }
            } else {
                pts.push(p);
            }
        }

        if self.is_closed {
            let suffix = [entry_target, grid_end, grid_start];
            for &p in &suffix {
                if let Some(&last) = pts.last() {
                    if last.distance(p) > 2.0 {
                        pts.push(p);
                    }
                } else {
                    pts.push(p);
                }
            }
        }

        if pts.len() < 3 {
            return;
        }

        let resampled = resample_points(&pts, 40.0);
        let smoothed =
            smooth_points_with_fixed_prefix(&resampled, self.is_closed, 2, fixed_prefix_count);

        let len = smoothed.len();
        if len < 3 {
            return;
        }

        let mut cumulative_dist = 0.0;
        let mut last_center: Option<Vec2> = None;

        for i in 1..len - 1 {
            let p1 = smoothed[i];
            let p2 = smoothed[i + 1];
            let p0 = smoothed[i - 1];
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

        self.generate_gpu_mesh(track_texture);
    }

    fn generate_gpu_mesh(&mut self, track_texture: Option<&Texture2D>) {
        self.meshes.clear();

        if self.segments.len() < 2 {
            return;
        }

        let tile_length = 120.0;
        let chunk_size = 200; // Max quads per batch to stay under Macroquad's draw call limit

        let total_segments = self.segments.len() - 1;
        let mut idx = 0;

        while idx < total_segments {
            let end_idx = (idx + chunk_size).min(total_segments);

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

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

            idx = end_idx;
        }
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
