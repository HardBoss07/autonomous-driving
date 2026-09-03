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
            grip_drift: 0.03, // Lower grip value during drift retains sideways momentum
            drift_recovery_rate: 3.5,
        }
    }
}
