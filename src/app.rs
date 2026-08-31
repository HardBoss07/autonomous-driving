use crate::core::car::{CarConfig, CarState};
use crate::core::physics::CarInput;
use crate::render::{debug, ui};
use macroquad::prelude::*;

pub struct App {
    car: CarState,
    config: CarConfig,
    car_texture: Option<Texture2D>,
}

impl App {
    pub async fn new() -> Self {
        let car_texture = load_texture("assets/textures/car.png").await.ok();
        if let Some(ref texture) = car_texture {
            texture.set_filter(FilterMode::Nearest);
        }

        Self {
            car: CarState::new(screen_width() / 2.0, screen_height() / 2.0),
            config: CarConfig::default(),
            car_texture,
        }
    }

    pub async fn run_loop(&mut self) {
        loop {
            let dt = get_frame_time().min(0.05);

            // Process Input & Physics State Update
            let input = CarInput::read_keyboard();
            self.car.update(&input, &self.config, dt);
            self.car.wrap_screen_bounds(screen_width(), screen_height());

            // Render World Layer
            clear_background(Color::new(0.08, 0.08, 0.10, 1.0));
            debug::draw_grid(64.0, screen_width(), screen_height());
            debug::draw_car(&self.car, self.car_texture.as_ref(), input.handbrake);
            debug::draw_drift_indicator(&self.car, input.is_drifting());

            // Render Telemetry & Controls UI
            ui::draw_telemetry(&self.car, input.is_drifting());
            ui::draw_tuning(&mut self.config);

            next_frame().await;
        }
    }
}
