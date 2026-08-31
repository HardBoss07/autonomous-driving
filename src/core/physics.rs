use macroquad::input::{KeyCode, is_key_down};

#[derive(Clone, Copy, Debug, Default)]
pub struct CarInput {
    pub throttle: f32,
    pub brake: f32,
    pub steer: f32,
    pub handbrake: bool,
}

impl CarInput {
    pub fn read_keyboard() -> Self {
        let mut input = Self::default();

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            input.throttle = 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            input.brake = 1.0;
        }

        let mut steer = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            steer -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            steer += 1.0;
        }
        input.steer = steer;
        input.handbrake = is_key_down(KeyCode::Space);

        input
    }

    pub fn is_steering(&self) -> bool {
        self.steer.abs() > 0.01
    }

    pub fn is_drifting(&self) -> bool {
        self.handbrake && self.is_steering()
    }

    pub fn is_straight_braking(&self) -> bool {
        self.handbrake && !self.is_steering()
    }
}
