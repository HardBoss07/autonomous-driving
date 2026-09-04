#[derive(Clone, Copy, Debug)]
pub struct CarConfig {
    pub max_speed: f32,
    pub engine_force: f32,
    pub brake_force: f32,
    pub drag_coeff: f32,
    pub turn_rate: f32,
    pub drift_turn_multiplier: f32,
    pub grip_normal: f32,
}

impl Default for CarConfig {
    fn default() -> Self {
        Self {
            max_speed: 700.0,
            engine_force: 1800.0,
            brake_force: 2500.0,
            drag_coeff: 0.0012,
            turn_rate: 3.8,
            drift_turn_multiplier: 1.2,
            grip_normal: 0.94,
        }
    }
}
