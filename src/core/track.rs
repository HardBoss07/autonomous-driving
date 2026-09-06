pub mod builder;
pub mod checkpoint;
pub mod curve;
pub mod grid;
pub mod query;
pub mod segment;

pub use checkpoint::CheckpointGate;
pub use curve::{
    catmull_rom, catmull_rom_derivative, perpendicular_distance, ramer_douglas_peucker,
    resample_points, smooth_points_with_fixed_prefix,
};
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
        let forward = grid.forward_vector();
        let right = grid.right_vector();
        let half_width = grid.width * 0.5;
        let num_samples = 6;

        let mut segments = Vec::with_capacity(num_samples);
        for index in 0..num_samples {
            let ratio = index as f32 / (num_samples - 1) as f32;
            let center = start.lerp(end, ratio);
            segments.push(TrackSegment {
                center,
                left_bound: center - right * half_width,
                right_bound: center + right * half_width,
                normal: right,
                tangent: forward,
                distance_along_track: ratio * grid.length,
            });
        }
        segments
    }

    #[inline]
    pub fn grid_segments(&self) -> &[TrackSegment] {
        &self.cached_grid_segments
    }

    pub fn find_nearest_segment(&self, pos: Vec2) -> Option<(TrackSegment, usize, f32)> {
        query::find_nearest_segment(&self.cached_grid_segments, &self.segments, pos)
    }

    pub fn find_nearest_segment_localized(
        &self,
        pos: Vec2,
        cached_idx: Option<usize>,
    ) -> Option<(TrackSegment, usize, f32)> {
        query::find_nearest_segment_localized(
            &self.cached_grid_segments,
            &self.segments,
            pos,
            cached_idx,
        )
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
        let forward = self.starting_grid.forward_vector();
        self.start_grid_config =
            StartGridConfig::new(grid_start - forward * 100.0, self.starting_grid.rotation);

        let prepared_points =
            builder::prepare_track_points(&self.raw_points, &self.starting_grid, self.is_closed);
        self.segments = builder::generate_track_segments(
            &prepared_points,
            self.is_closed,
            self.width,
            samples_per_segment,
        );

        self.checkpoints = builder::generate_checkpoints(&self.segments, self.checkpoint_spacing);

        let (meshes, bboxes) = builder::generate_gpu_mesh(&self.segments, track_texture);
        self.meshes = meshes;
        self.mesh_bounding_boxes = bboxes;

        let (wall_meshes, wall_bboxes) =
            builder::generate_wall_meshes(&self.cached_grid_segments, &self.segments, wall_texture);
        self.wall_meshes = wall_meshes;
        self.wall_mesh_bounding_boxes = wall_bboxes;
    }
}
