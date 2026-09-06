use macroquad::prelude::Vec2;
use std::f32::consts::PI;

pub trait FloatExt {
    fn to_normalized_radians(self) -> f32;
    fn is_near_zero(self) -> bool;
    fn is_near_zero_eps(self, epsilon: f32) -> bool;
}

impl FloatExt for f32 {
    #[inline]
    fn to_normalized_radians(self) -> f32 {
        self.rem_euclid(2.0 * PI)
    }

    #[inline]
    fn is_near_zero(self) -> bool {
        self.abs() < 1e-5
    }

    #[inline]
    fn is_near_zero_eps(self, epsilon: f32) -> bool {
        self.abs() < epsilon
    }
}

pub trait Vec2Ext {
    fn clamp_magnitude(self, max_magnitude: f32) -> Vec2;
    fn left_normal(self) -> Vec2;
    fn right_normal(self) -> Vec2;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn clamp_magnitude(self, max_magnitude: f32) -> Vec2 {
        let length_squared = self.length_squared();
        if length_squared > max_magnitude * max_magnitude && length_squared > 0.0 {
            self * (max_magnitude / length_squared.sqrt())
        } else {
            self
        }
    }

    #[inline]
    fn left_normal(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }

    #[inline]
    fn right_normal(self) -> Vec2 {
        Vec2::new(self.y, -self.x)
    }
}
