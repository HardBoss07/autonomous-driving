use crate::core::car::{CarConfig, CarState};
use crate::core::physics::CarInput;
use crate::core::track::Track;
use crate::render::editor::TrackEditor;
use crate::render::skidmarks::SkidmarkManager;
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
    pub skidmark_manager: SkidmarkManager,
    pub car_texture: Option<Texture2D>,
    pub track_texture: Option<Texture2D>,
    pub grid_texture: Option<Texture2D>,
    pub driving_zoom: f32,
    pub editor_zoom: f32,
    pub camera_target: Vec2,
    pub show_checkpoints: bool,
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

        let (spawn_pos, spawn_heading) = track.start_grid_config.get_anchor_transform(0);
        let mut car = CarState::new(spawn_pos.x, spawn_pos.y);
        car.heading = spawn_heading;

        Self {
            mode: AppMode::Driving,
            car,
            config: CarConfig::default(),
            track,
            editor: TrackEditor::new(),
            skidmark_manager: SkidmarkManager::new(20.0), // Fade duration of 20 seconds
            car_texture,
            track_texture,
            grid_texture,
            driving_zoom: 1.0,
            editor_zoom: 0.5,
            camera_target: spawn_pos,
            show_checkpoints: false,
        }
    }

    pub async fn run_loop(&mut self) {
        loop {
            let dt = get_frame_time().min(0.05);
            let current_time = get_time();

            match self.mode {
                AppMode::Driving => {
                    // Keybinds: R to reset to starting grid & clear stats, C to toggle checkpoints debug
                    if is_key_pressed(KeyCode::R) {
                        let (spawn_pos, spawn_heading) =
                            self.track.start_grid_config.get_anchor_transform(0);
                        self.car.reset_to_grid(spawn_pos, spawn_heading);
                        self.camera_target = spawn_pos;
                        self.skidmark_manager.clear();
                    }

                    if is_key_pressed(KeyCode::C) {
                        self.show_checkpoints = !self.show_checkpoints;
                    }

                    let input = CarInput::read_keyboard();
                    self.car.update(
                        &input,
                        &self.config,
                        &self.track.checkpoints,
                        dt,
                        current_time,
                    );

                    // Update skid marks during braking
                    self.skidmark_manager.update(&self.car, &input, dt);

                    let car_pos = vec2(self.car.pos_x, self.car.pos_y);
                    let follow_speed = 6.0;
                    self.camera_target +=
                        (car_pos - self.camera_target) * (follow_speed * dt).min(1.0);

                    clear_background(Color::new(0.08, 0.08, 0.10, 1.0));

                    set_camera(&Camera2D {
                        target: self.camera_target,
                        zoom: vec2(
                            (2.0 / screen_width()) * self.driving_zoom,
                            (2.0 / screen_height()) * self.driving_zoom,
                        ),
                        ..Default::default()
                    });

                    debug::draw_grid(64.0, screen_width() * 4.0, screen_height() * 4.0);
                    track_render::draw_track(&self.track, self.grid_texture.as_ref());

                    // Render skid marks underneath the car sprite
                    self.skidmark_manager.draw();

                    if self.show_checkpoints {
                        debug::draw_checkpoints(
                            &self.track.checkpoints,
                            self.car.timing.next_checkpoint_idx,
                        );
                    }

                    let is_drifting = input.is_drifting();
                    debug::draw_car(&self.car, self.car_texture.as_ref(), input.handbrake);
                    debug::draw_drift_indicator(&self.car, is_drifting);

                    set_default_camera();

                    ui::draw_telemetry(&self.car, is_drifting);
                    ui::draw_tuning(&mut self.config);

                    if ui::draw_editor_toggle_button() {
                        self.mode = AppMode::TrackEditor;
                    }
                }
                AppMode::TrackEditor => {
                    let mouse_pos = mouse_position();
                    let screen_mouse = vec2(mouse_pos.0, mouse_pos.1);
                    let world_mouse = vec2(
                        (mouse_pos.0 - screen_width() * 0.5) / self.editor_zoom
                            + screen_width() * 0.5,
                        (mouse_pos.1 - screen_height() * 0.5) / self.editor_zoom
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
                            (2.0 / screen_width()) * self.editor_zoom,
                            (2.0 / screen_height()) * self.editor_zoom,
                        ),
                        ..Default::default()
                    });

                    debug::draw_grid(64.0, screen_width() * 4.0, screen_height() * 4.0);
                    track_render::draw_track(&self.track, self.grid_texture.as_ref());
                    self.editor.draw_snap_previews(&self.track, world_mouse);

                    set_default_camera();

                    let done = self
                        .editor
                        .update_and_draw_ui(&mut self.track, &mut self.editor_zoom);

                    if done {
                        self.track.rebuild_mesh(5, self.track_texture.as_ref());
                        self.mode = AppMode::Driving;

                        // Reset car state & clear old laptimes and skid marks for new track
                        let (spawn_pos, heading) =
                            self.track.start_grid_config.get_anchor_transform(0);
                        self.car.reset_to_grid(spawn_pos, heading);
                        self.camera_target = spawn_pos;
                        self.skidmark_manager.clear();
                    }
                }
            }

            next_frame().await;
        }
    }
}
