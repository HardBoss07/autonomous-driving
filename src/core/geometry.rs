use macroquad::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod vec2_serde {
    use super::*;

    pub fn serialize<S>(v: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (v.x, v.y).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(Vec2::new(x, y))
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct LineSegment {
    #[serde(with = "vec2_serde")]
    pub a: Vec2,
    #[serde(with = "vec2_serde")]
    pub b: Vec2,
}

impl LineSegment {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }

    pub fn intersects(&self, other: &LineSegment) -> bool {
        let r = self.b - self.a;
        let s = other.b - other.a;
        let r_cross_s = r.perp_dot(s);

        if r_cross_s.abs() < 1e-6 {
            return false;
        }

        let q_p = other.a - self.a;
        let t = q_p.perp_dot(s) / r_cross_s;
        let u = q_p.perp_dot(r) / r_cross_s;

        (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoundingBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}
