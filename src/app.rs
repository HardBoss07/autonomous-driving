use crate::core::car::{CarConfig, CarState};
use crate::core::physics::CarInput;
use crate::core::track::Track;
use crate::render::editor::TrackEditor;
use crate::render::{debug, track_render, ui};
use macroquad::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum AppMode {
    Driving,
    TrackEditor,
}

pub struct App {
    pub mode: AppMode,
    pub car: CarState,
    pub config: CarConfig,
    pub track: Track,
    pub editor: TrackEditor,
    pub car_texture: Option<Texture2D>,
    pub track_texture: Option<Texture2D>,
    pub grid_texture: Option<Texture2D>,
    pub camera_zoom: f32,
}

impl App {
    pub async fn new() -> Self {
        let car_texture = load_texture("assets/textures/car.png").await.ok();
        if let Some(ref texture) = car_texture {
            texture.set_filter(FilterMode::Nearest);
        }

        let track_texture = load_texture("assets/textures/track.png").await.ok();
        if let Some(ref texture) = track_texture {
            texture.set_filter(FilterMode::Nearest);
        }

        let grid_texture = load_texture("assets/textures/start_grid.png").await.ok();
        if let Some(ref texture) = grid_texture {
            texture.set_filter(FilterMode::Nearest);
        }

        let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
        let mut track = Track::new(screen_center);
        track.rebuild_mesh(5, track_texture.as_ref());

        Self {
            mode: AppMode::Driving,
            car: CarState::new(screen_center.x, screen_center.y),
            config: CarConfig::default(),
            track,
            editor: TrackEditor::new(),
            car_texture,
            track_texture,
            grid_texture,
            camera_zoom: 0.6,
        }
    }

    pub async fn run_loop(&mut self) {
        loop {
            let dt = get_frame_time().min(0.05);

            match self.mode {
                AppMode::Driving => {
                    let input = CarInput::read_keyboard();
                    self.car.update(&input, &self.config, dt);
                    self.car.wrap_screen_bounds(screen_width(), screen_height());

                    clear_background(Color::new(0.08, 0.08, 0.10, 1.0));
                    debug::draw_grid(64.0, screen_width(), screen_height());

                    track_render::draw_track(&self.track, self.grid_texture.as_ref());

                    let is_drifting = input.is_drifting();
                    debug::draw_car(&self.car, self.car_texture.as_ref(), input.handbrake);
                    debug::draw_drift_indicator(&self.car, is_drifting);

                    ui::draw_telemetry(&self.car, is_drifting);
                    ui::draw_tuning(&mut self.config);

                    if ui::draw_editor_toggle_button() {
                        self.mode = AppMode::TrackEditor;
                        self.car.pos_x = self.track.starting_grid.position.x;
                        self.car.pos_y = self.track.starting_grid.position.y;
                        self.car.heading = self.track.starting_grid.rotation;
                        self.car.vel_x = 0.0;
                        self.car.vel_y = 0.0;
                    }
                }
                AppMode::TrackEditor => {
                    let mouse_pos = mouse_position();
                    let screen_mouse = vec2(mouse_pos.0, mouse_pos.1);
                    let world_mouse = vec2(
                        (mouse_pos.0 - screen_width() * 0.5) / self.camera_zoom
                            + screen_width() * 0.5,
                        (mouse_pos.1 - screen_height() * 0.5) / self.camera_zoom
                            + screen_height() * 0.5,
                    );

                    self.editor.handle_input(
                        &mut self.track,
                        self.track_texture.as_ref(),
                        world_mouse,
                        screen_mouse,
                    );

                    clear_background(Color::new(0.05, 0.05, 0.07, 1.0));

                    set_camera(&Camera2D {
                        target: vec2(screen_width() * 0.5, screen_height() * 0.5),
                        zoom: vec2(
                            (2.0 / screen_width()) * self.camera_zoom,
                            (2.0 / screen_height()) * self.camera_zoom,
                        ),
                        ..Default::default()
                    });

                    debug::draw_grid(64.0, screen_width() * 2.0, screen_height() * 2.0);
                    track_render::draw_track(&self.track, self.grid_texture.as_ref());
                    self.editor.draw_snap_previews(&self.track, world_mouse);

                    set_default_camera();

                    let done = self
                        .editor
                        .update_and_draw_ui(&mut self.track, &mut self.camera_zoom);
                    if done {
                        self.track.rebuild_mesh(5, self.track_texture.as_ref());
                        self.mode = AppMode::Driving;
                    }
                }
            }

            next_frame().await;
        }
    }
}
