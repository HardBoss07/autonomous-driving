use crate::core::physics::CarInput;
use macroquad::prelude::vec2;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct CarConfig {
    pub max_speed: f32,
    pub engine_force: f32,
    pub brake_force: f32,
    pub drag_coeff: f32,
    pub turn_rate: f32,
    pub grip_normal: f32,
    pub grip_drift: f32,
    pub drift_recovery_rate: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            max_speed: 900.0,
            engine_force: 1800.0,
            brake_force: 2500.0,
            drag_coeff: 0.0012,
            turn_rate: 4.2,
            grip_normal: 0.88,
            grip_drift: 0.12,
            drift_recovery_rate: 3.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CarState {
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub heading: f32,
    pub current_grip: f32,
}

impl CarState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            vel_x: 0.0,
            vel_y: 0.0,
            heading: -PI / 2.0,
            current_grip: 0.88,
        }
    }

    pub fn update(&mut self, input: &CarInput, config: &CarConfig, dt: f32) {
        let forward = vec2(self.heading.cos(), self.heading.sin());
        let right = vec2(-self.heading.sin(), self.heading.cos());

        let current_vel = vec2(self.vel_x, self.vel_y);
        let v_long = current_vel.dot(forward);
        let v_lat = current_vel.dot(right);

        self.update_steering(input.steer, config.turn_rate, v_long, dt);
        self.update_grip(input.is_drifting(), config, dt);

        let new_v_long = self.compute_longitudinal_velocity(v_long, input, config, dt);
        let new_v_lat = self.compute_lateral_velocity(v_lat, dt);

        let new_vel = forward * new_v_long + right * new_v_lat;
        self.vel_x = new_vel.x;
        self.vel_y = new_vel.y;

        self.pos_x += self.vel_x * dt;
        self.pos_y += self.vel_y * dt;
    }

    fn update_steering(&mut self, steer_input: f32, turn_rate: f32, v_long: f32, dt: f32) {
        let turn_threshold = 150.0;
        let turn_factor = (v_long.abs() / turn_threshold).clamp(0.0, 1.0);
        let steering_direction = if v_long < -10.0 { -1.0 } else { 1.0 };

        self.heading += steer_input * turn_rate * steering_direction * turn_factor * dt;
        self.heading = self.heading.rem_euclid(2.0 * PI);
    }

    fn update_grip(&mut self, is_drifting: bool, config: &CarConfig, dt: f32) {
        let target_grip = if is_drifting {
            config.grip_drift
        } else {
            config.grip_normal
        };
        self.current_grip += (target_grip - self.current_grip) * config.drift_recovery_rate * dt;
    }

    fn compute_longitudinal_velocity(
        &self,
        v_long: f32,
        input: &CarInput,
        config: &CarConfig,
        dt: f32,
    ) -> f32 {
        if input.is_straight_braking() {
            let brake_decel = config.brake_force * 1.5 * dt;
            if v_long.abs() <= brake_decel {
                0.0
            } else {
                v_long - v_long.signum() * brake_decel
            }
        } else {
            let f_drive = input.throttle * config.engine_force - input.brake * config.brake_force;
            let f_drag = -config.drag_coeff * v_long * v_long.abs();
            let accel_long = f_drive + f_drag;
            (v_long + accel_long * dt).clamp(-config.max_speed * 0.3, config.max_speed)
        }
    }

    fn compute_lateral_velocity(&self, v_lat: f32, dt: f32) -> f32 {
        v_lat * (1.0f32 - self.current_grip).powf(dt * 60.0)
    }

    pub fn wrap_screen_bounds(&mut self, screen_width: f32, screen_height: f32) {
        if self.pos_x < 0.0 {
            self.pos_x = screen_width;
        } else if self.pos_x > screen_width {
            self.pos_x = 0.0;
        }

        if self.pos_y < 0.0 {
            self.pos_y = screen_height;
        } else if self.pos_y > screen_height {
            self.pos_y = 0.0;
        }
    }

    pub fn speed(&self) -> f32 {
        (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt()
    }
}
