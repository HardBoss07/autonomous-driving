use crate::core::car::state::CarState;
use crate::core::physics::CarInput;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SkidSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub alpha: f32,
    pub thickness: f32,
}

pub struct SkidmarkManager {
    pub segments: Vec<SkidSegment>,
    pub last_left_wheel: Option<Vec2>,
    pub last_right_wheel: Option<Vec2>,
    pub fade_duration: f32,
    pub max_alpha: f32,
}

impl SkidmarkManager {
    pub fn new(fade_duration: f32) -> Self {
        Self {
            segments: Vec::new(),
            last_left_wheel: None,
            last_right_wheel: None,
            fade_duration,
            max_alpha: 0.8,
        }
    }

    pub fn get_rear_wheel_positions(car: &CarState) -> (Vec2, Vec2) {
        let car_pos = vec2(car.pos_x, car.pos_y);
        let forward = vec2(car.heading.cos(), car.heading.sin());
        let right = vec2(-car.heading.sin(), car.heading.cos());

        let rear_axle = forward * -16.0;
        let half_track = right * 10.0;

        let left_wheel = car_pos + rear_axle - half_track;
        let right_wheel = car_pos + rear_axle + half_track;

        (left_wheel, right_wheel)
    }

    pub fn update(&mut self, car: &CarState, input: &CarInput, dt: f32) {
        let alpha_decay = (self.max_alpha / self.fade_duration) * dt;
        for segment in &mut self.segments {
            segment.alpha -= alpha_decay;
        }
        self.segments.retain(|s| s.alpha > 0.0);

        let is_braking = input.brake > 0.05 || input.is_straight_braking();
        let is_drifting = input.is_drifting();
        let is_moving = car.speed() > 30.0;

        if (is_braking || is_drifting) && is_moving {
            let (curr_left, curr_right) = Self::get_rear_wheel_positions(car);

            if let (Some(prev_left), Some(prev_right)) =
                (self.last_left_wheel, self.last_right_wheel)
            {
                if prev_left.distance_squared(curr_left) > 0.01 {
                    self.segments.push(SkidSegment {
                        start: prev_left,
                        end: curr_left,
                        alpha: self.max_alpha,
                        thickness: 4.0,
                    });
                    self.segments.push(SkidSegment {
                        start: prev_right,
                        end: curr_right,
                        alpha: self.max_alpha,
                        thickness: 4.0,
                    });
                }
            }

            self.last_left_wheel = Some(curr_left);
            self.last_right_wheel = Some(curr_right);
        } else {
            self.last_left_wheel = None;
            self.last_right_wheel = None;
        }
    }

    pub fn draw(&self) {
        for seg in &self.segments {
            draw_line(
                seg.start.x,
                seg.start.y,
                seg.end.x,
                seg.end.y,
                seg.thickness,
                Color::new(0.1, 0.1, 0.12, seg.alpha),
            );
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.last_left_wheel = None;
        self.last_right_wheel = None;
    }
}
