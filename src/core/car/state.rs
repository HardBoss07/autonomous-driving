use crate::core::car::config::CarConfig;
use crate::core::physics::CarInput;
use crate::core::timing::TimingState;
use crate::core::track::Track;
use macroquad::prelude::{Vec2, vec2};
use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct CarState {
    pub pos_x: f32,
    pub pos_y: f32,
    pub prev_pos_x: f32,
    pub prev_pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub heading: f32,
    pub angular_velocity: f32,
    pub timing: TimingState,
    pub nearest_segment_index: Option<usize>,
}

impl CarState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            prev_pos_x: x,
            prev_pos_y: y,
            vel_x: 0.0,
            vel_y: 0.0,
            heading: -PI / 2.0,
            angular_velocity: 0.0,
            timing: TimingState::default(),
            nearest_segment_index: None,
        }
    }

    pub fn update(
        &mut self,
        input: &CarInput,
        config: &CarConfig,
        track: &Track,
        dt: f32,
        current_time: f64,
    ) {
        self.prev_pos_x = self.pos_x;
        self.prev_pos_y = self.pos_y;

        let mut turn_rate = config.turn_rate;
        let mut drag_coeff = config.drag_coeff;

        // 1. Smoothly integrate angular velocity into heading with exponential damping
        self.heading += self.angular_velocity * dt;
        self.heading = self.heading.rem_euclid(2.0 * PI);
        self.angular_velocity *= (1.0f32 - 10.0 * dt).max(0.0);

        // 2. Query nearest track segment for kerb & wall collision logic with localized temporal window search
        if let Some((seg, idx, _dist)) = track.find_nearest_segment_localized(
            vec2(self.pos_x, self.pos_y),
            self.nearest_segment_index,
        ) {
            self.nearest_segment_index = Some(idx);
            let offset_vec = vec2(self.pos_x, self.pos_y) - seg.center;
            let lat_offset = offset_vec.dot(seg.normal);
            let abs_offset = lat_offset.abs();

            let tarmac_limit = 54.0; // 140px width - (2 * 16px kerb) = 108px tarmac (54px half-width)
            let wall_limit = 70.0; // 140px track boundary (70px half-width)

            if abs_offset > tarmac_limit && abs_offset <= wall_limit {
                // Riding 16px Kerb: Rumble slows down top speed slightly but apex clipping improves turn-in rotation
                drag_coeff *= 1.35;
                turn_rate *= 1.25;
            } else if abs_offset > wall_limit {
                // Wall Collision & Anti-Wall-Riding Physics
                let wall_normal = if lat_offset > 0.0 {
                    -seg.normal
                } else {
                    seg.normal
                };

                // Gently clamp car position to wall boundary
                let clamped_pos = seg.center - wall_normal * (wall_limit - 1.5);
                self.pos_x = clamped_pos.x;
                self.pos_y = clamped_pos.y;

                let vel = vec2(self.vel_x, self.vel_y);
                let speed = vel.length();
                let v_normal_mag = vel.dot(wall_normal);

                if v_normal_mag < 0.0 {
                    let v_normal = wall_normal * v_normal_mag;
                    let v_tangent = vel - v_normal;
                    let impact_speed = -v_normal_mag;

                    let normal_ratio = if speed > 5.0 {
                        impact_speed / speed
                    } else {
                        0.0
                    };

                    if normal_ratio < 0.35 {
                        // SHALLOW IMPACT / PARALLEL SCRAPING:
                        // Slide smoothly along wall with realistic friction without spinning out
                        let new_v_tangent = v_tangent * (1.0 - 1.8 * dt).max(0.2);
                        let new_vel = new_v_tangent;

                        self.vel_x = new_vel.x;
                        self.vel_y = new_vel.y;

                        // Slight repulsive nudge away from wall so car doesn't stick
                        self.pos_x += wall_normal.x * 12.0 * dt;
                        self.pos_y += wall_normal.y * 12.0 * dt;
                    } else {
                        // HARSH ANGLE IMPACT:
                        // Rebound velocity and impart smooth physical angular momentum (torque)
                        let new_v_normal = -0.25 * v_normal;
                        let new_v_tangent = 0.55 * v_tangent;

                        let new_vel = new_v_normal + new_v_tangent;
                        self.vel_x = new_vel.x;
                        self.vel_y = new_vel.y;

                        // Apply continuous angular velocity torque instead of instant rotation jump
                        let spin_direction = if v_tangent.dot(seg.tangent) >= 0.0 {
                            1.0
                        } else {
                            -1.0
                        };
                        let torque_bias = if lat_offset > 0.0 { -1.0 } else { 1.0 };

                        self.angular_velocity +=
                            torque_bias * spin_direction * impact_speed * 0.035;
                    }
                }
            }
        }

        let forward = vec2(self.heading.cos(), self.heading.sin());
        let right = vec2(-self.heading.sin(), self.heading.cos());

        let current_vel = vec2(self.vel_x, self.vel_y);
        let v_long = current_vel.dot(forward);
        let v_lat = current_vel.dot(right);

        let is_drifting = input.is_drifting();

        // 3. Steering
        self.update_steering(
            input.steer,
            turn_rate,
            config.drift_turn_multiplier,
            v_long,
            is_drifting,
            dt,
        );

        // 4. Acceleration / Braking
        let new_v_long = self.compute_longitudinal_velocity(v_long, input, config, drag_coeff, dt);

        // 5. Lateral grip damping
        let new_v_lat = v_lat * (1.0f32 - config.grip_normal).powf(dt * 60.0);

        let new_vel = forward * new_v_long + right * new_v_lat;
        self.vel_x = new_vel.x;
        self.vel_y = new_vel.y;

        self.pos_x += self.vel_x * dt;
        self.pos_y += self.vel_y * dt;

        self.timing.update(
            vec2(self.prev_pos_x, self.prev_pos_y),
            vec2(self.pos_x, self.pos_y),
            &track.checkpoints,
            current_time,
        );
    }

    pub fn reset_to_grid(&mut self, pos: Vec2, heading: f32) {
        self.pos_x = pos.x;
        self.pos_y = pos.y;
        self.prev_pos_x = pos.x;
        self.prev_pos_y = pos.y;
        self.vel_x = 0.0;
        self.vel_y = 0.0;
        self.heading = heading;
        self.angular_velocity = 0.0;
        self.nearest_segment_index = None;
        self.timing.reset();
    }

    fn update_steering(
        &mut self,
        steer_input: f32,
        turn_rate: f32,
        drift_multiplier: f32,
        v_long: f32,
        is_drifting: bool,
        dt: f32,
    ) {
        let turn_threshold = 120.0;
        let turn_factor = (v_long.abs() / turn_threshold).clamp(0.0, 1.0);
        let steering_direction = if v_long < -10.0 { -1.0 } else { 1.0 };

        let active_turn_rate = if is_drifting {
            turn_rate * drift_multiplier
        } else {
            turn_rate
        };

        self.heading += steer_input * active_turn_rate * steering_direction * turn_factor * dt;
        self.heading = self.heading.rem_euclid(2.0 * PI);
    }

    fn compute_longitudinal_velocity(
        &self,
        v_long: f32,
        input: &CarInput,
        config: &CarConfig,
        drag_coeff: f32,
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
            let f_drag = -drag_coeff * v_long * v_long.abs();
            let accel_long = f_drive + f_drag;
            (v_long + accel_long * dt).clamp(-config.max_speed * 0.3, config.max_speed)
        }
    }

    pub fn speed(&self) -> f32 {
        (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt()
    }
}
