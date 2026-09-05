use crate::core::geometry::LineSegment;
use crate::core::track::segment::TrackSegment;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointGate {
    pub id: usize,
    pub line: LineSegment,
}

impl CheckpointGate {
    pub fn new(id: usize, line: LineSegment) -> Self {
        Self { id, line }
    }

    pub fn from_track_segment(id: usize, segment: &TrackSegment, margin: f32) -> Self {
        let a = segment.left_bound - segment.normal * margin;
        let b = segment.right_bound + segment.normal * margin;
        Self {
            id,
            line: LineSegment::new(a, b),
        }
    }

    pub fn center(&self) -> Vec2 {
        (self.line.a + self.line.b) * 0.5
    }

    pub fn width(&self) -> f32 {
        self.line.a.distance(self.line.b)
    }

    pub fn direction(&self) -> Vec2 {
        (self.line.b - self.line.a).normalize_or_zero()
    }

    pub fn normal(&self) -> Vec2 {
        let dir = self.direction();
        vec2(-dir.y, dir.x)
    }

    pub fn bounding_box(&self) -> crate::core::geometry::BoundingBox {
        let min = self.line.a.min(self.line.b);
        let max = self.line.a.max(self.line.b);
        crate::core::geometry::BoundingBox::new(min, max)
    }
}
