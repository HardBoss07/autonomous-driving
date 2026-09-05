use crate::core::car::config::CarConfig;
use crate::core::physics::CarInput;
use crate::core::track::TrackSegment;
use macroquad::prelude::Vec2;
use std::f32::consts::PI;

pub fn integrate_heading(heading: f32, mut angular_velocity: f32, delta_time: f32) -> (f32, f32) {
    let mut new_heading = heading + angular_velocity * delta_time;
    new_heading = new_heading.rem_euclid(2.0 * PI);
    angular_velocity *= (1.0f32 - 10.0 * delta_time).max(0.0);
    (new_heading, angular_velocity)
}

pub fn update_steering(
    mut heading: f32,
    steer_input: f32,
    turn_rate: f32,
    drift_multiplier: f32,
    v_long: f32,
    is_drifting: bool,
    delta_time: f32,
) -> f32 {
    let turn_threshold = 120.0;
    let turn_factor = (v_long.abs() / turn_threshold).clamp(0.0, 1.0);
    let steering_direction = if v_long < -10.0 { -1.0 } else { 1.0 };

    let active_turn_rate = if is_drifting {
        turn_rate * drift_multiplier
    } else {
        turn_rate
    };

    heading += steer_input * active_turn_rate * steering_direction * turn_factor * delta_time;
    heading.rem_euclid(2.0 * PI)
}

pub fn compute_longitudinal_velocity(
    v_long: f32,
    input: &CarInput,
    config: &CarConfig,
    drag_coeff: f32,
    delta_time: f32,
) -> f32 {
    if input.is_straight_braking() {
        let brake_decel = config.brake_force * 1.5 * delta_time;
        if v_long.abs() <= brake_decel {
            0.0
        } else {
            v_long - v_long.signum() * brake_decel
        }
    } else {
        let f_drive = input.throttle * config.engine_force - input.brake * config.brake_force;
        let f_drag = -drag_coeff * v_long * v_long.abs();
        let accel_long = f_drive + f_drag;
        (v_long + accel_long * delta_time).clamp(-config.max_speed * 0.3, config.max_speed)
    }
}

pub fn resolve_wall_collision(
    current_velocity: Vec2,
    angular_velocity: f32,
    segment: &TrackSegment,
    lateral_offset: f32,
    wall_limit: f32,
    delta_time: f32,
) -> (Vec2, Vec2, f32) {
    let wall_normal = if lateral_offset > 0.0 {
        -segment.normal
    } else {
        segment.normal
    };

    let mut position = segment.center - wall_normal * (wall_limit - 1.5);
    let mut velocity = current_velocity;
    let mut new_angular_velocity = angular_velocity;

    let speed = velocity.length();
    let v_normal_magnitude = velocity.dot(wall_normal);

    if v_normal_magnitude < 0.0 {
        let v_normal = wall_normal * v_normal_magnitude;
        let v_tangent = velocity - v_normal;
        let impact_speed = -v_normal_magnitude;

        let normal_ratio = if speed > 5.0 {
            impact_speed / speed
        } else {
            0.0
        };

        if normal_ratio < 0.35 {
            // Slide smoothly along wall
            velocity = v_tangent * (1.0 - 1.8 * delta_time).max(0.2);
            position += wall_normal * 12.0 * delta_time;
        } else {
            // Rebound velocity and impart torque
            let new_v_normal = -0.25 * v_normal;
            let new_v_tangent = 0.55 * v_tangent;
            velocity = new_v_normal + new_v_tangent;

            let spin_direction = if v_tangent.dot(segment.tangent) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let torque_bias = if lateral_offset > 0.0 { -1.0 } else { 1.0 };

            new_angular_velocity += torque_bias * spin_direction * impact_speed * 0.035;
        }
    }

    (position, velocity, new_angular_velocity)
}
