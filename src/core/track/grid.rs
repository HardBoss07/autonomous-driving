use crate::core::geometry::vec2_serde;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct StartGridConfig {
    #[serde(with = "vec2_serde")]
    pub origin: Vec2,
    pub heading: f32,
    pub row_spacing: f32,
    pub col_spacing: f32,
}

impl StartGridConfig {
    pub fn new(origin: Vec2, heading: f32) -> Self {
        Self {
            origin,
            heading,
            row_spacing: 50.0,
            col_spacing: 40.0,
        }
    }

    pub fn get_anchor_transform(&self, slot_index: usize) -> (Vec2, f32) {
        let forward = Vec2::new(self.heading.cos(), self.heading.sin());
        let right = Vec2::new(-self.heading.sin(), self.heading.cos());

        let row = (slot_index / 2) as f32;
        let col = (slot_index % 2) as f32;

        let local_x = (col - 0.5) * self.col_spacing;
        let local_y = -row * self.row_spacing;

        let pos = self.origin + (right * local_x) + (forward * local_y);
        (pos, self.heading)
    }
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
