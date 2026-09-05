use crate::core::car::config::CarConfig;
use crate::core::car::physics::{
    compute_longitudinal_velocity, integrate_heading, resolve_wall_collision, update_steering,
};
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
        let (new_heading, new_angular_vel) =
            integrate_heading(self.heading, self.angular_velocity, dt);
        self.heading = new_heading;
        self.angular_velocity = new_angular_vel;

        // 2. Query nearest track segment for kerb & wall collision logic with localized temporal window search
        self.handle_track_boundary_interaction(track, &mut drag_coeff, &mut turn_rate, dt);

        let forward = vec2(self.heading.cos(), self.heading.sin());
        let right = vec2(-self.heading.sin(), self.heading.cos());

        let current_vel = vec2(self.vel_x, self.vel_y);
        let v_long = current_vel.dot(forward);
        let v_lat = current_vel.dot(right);

        let is_drifting = input.is_drifting();

        // 3. Steering
        self.heading = update_steering(
            self.heading,
            input.steer,
            turn_rate,
            config.drift_turn_multiplier,
            v_long,
            is_drifting,
            dt,
        );

        // 4. Acceleration / Braking
        let new_v_long = compute_longitudinal_velocity(v_long, input, config, drag_coeff, dt);

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

    fn handle_track_boundary_interaction(
        &mut self,
        track: &Track,
        drag_coeff: &mut f32,
        turn_rate: &mut f32,
        dt: f32,
    ) {
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
                *drag_coeff *= 1.35;
                *turn_rate *= 1.25;
            } else if abs_offset > wall_limit {
                let (new_pos, new_vel, new_ang_vel) = resolve_wall_collision(
                    vec2(self.vel_x, self.vel_y),
                    self.angular_velocity,
                    &seg,
                    lat_offset,
                    wall_limit,
                    dt,
                );
                self.pos_x = new_pos.x;
                self.pos_y = new_pos.y;
                self.vel_x = new_vel.x;
                self.vel_y = new_vel.y;
                self.angular_velocity = new_ang_vel;
            }
        }
    }

    pub fn speed(&self) -> f32 {
        (self.vel_x * self.vel_x + self.vel_y * self.vel_y).sqrt()
    }
}
