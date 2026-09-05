use crate::core::car::{CarConfig, CarState};
use crate::core::geometry::BoundingBox;
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
    pub wall_texture: Option<Texture2D>,
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

        let wall_texture = load_texture("assets/textures/wall.png").await.ok();
        if let Some(ref texture) = wall_texture {
            texture.set_filter(FilterMode::Nearest);
        }

        let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
        let mut track = Track::new(screen_center);
        track.rebuild_mesh(5, track_texture.as_ref(), wall_texture.as_ref());

        let (spawn_pos, spawn_heading) = track.start_grid_config.get_anchor_transform(0);
        let mut car = CarState::new(spawn_pos.x, spawn_pos.y);
        car.heading = spawn_heading;

        Self {
            mode: AppMode::Driving,
            car,
            config: CarConfig::default(),
            track,
            editor: TrackEditor::new(),
            skidmark_manager: SkidmarkManager::new(20.0),
            car_texture,
            track_texture,
            grid_texture,
            wall_texture,
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
                    self.car
                        .update(&input, &self.config, &self.track, dt, current_time);

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

                    let view_half_w = (screen_width() * 0.5) / self.driving_zoom;
                    let view_half_h = (screen_height() * 0.5) / self.driving_zoom;
                    let view_bounds = BoundingBox::new(
                        self.camera_target - vec2(view_half_w, view_half_h),
                        self.camera_target + vec2(view_half_w, view_half_h),
                    );

                    debug::draw_grid(64.0, screen_width() * 4.0, screen_height() * 4.0);
                    track_render::draw_track(
                        &self.track,
                        self.grid_texture.as_ref(),
                        self.wall_texture.as_ref(),
                        Some(view_bounds),
                    );

                    self.skidmark_manager.draw();

                    if self.show_checkpoints {
                        debug::draw_checkpoints(
                            &self.track.checkpoints,
                            self.car.timing.next_checkpoint_idx,
                            Some(view_bounds),
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
                    let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);

                    let world_mouse =
                        self.camera_target + (screen_mouse - screen_center) / self.editor_zoom;

                    self.editor.handle_input(
                        &mut self.track,
                        self.track_texture.as_ref(),
                        self.wall_texture.as_ref(),
                        world_mouse,
                        screen_mouse,
                        &mut self.camera_target,
                        &mut self.editor_zoom,
                        dt,
                    );

                    clear_background(Color::new(0.05, 0.05, 0.07, 1.0));

                    set_camera(&Camera2D {
                        target: self.camera_target,
                        zoom: vec2(
                            (2.0 / screen_width()) * self.editor_zoom,
                            (2.0 / screen_height()) * self.editor_zoom,
                        ),
                        ..Default::default()
                    });

                    let view_half_w = (screen_width() * 0.5) / self.editor_zoom;
                    let view_half_h = (screen_height() * 0.5) / self.editor_zoom;
                    let view_bounds = BoundingBox::new(
                        self.camera_target - vec2(view_half_w, view_half_h),
                        self.camera_target + vec2(view_half_w, view_half_h),
                    );

                    debug::draw_grid(64.0, screen_width() * 10.0, screen_height() * 10.0);
                    track_render::draw_track(
                        &self.track,
                        self.grid_texture.as_ref(),
                        self.wall_texture.as_ref(),
                        Some(view_bounds),
                    );
                    self.editor
                        .draw_snap_previews(&self.track, world_mouse, self.editor_zoom);

                    set_default_camera();

                    let done = self.editor.update_and_draw_ui(
                        &mut self.track,
                        &mut self.editor_zoom,
                        self.track_texture.as_ref(),
                        self.wall_texture.as_ref(),
                    );

                    if self.editor.show_help_overlay {
                        self.editor.draw_help_overlay();
                    }

                    if done {
                        self.track.rebuild_mesh(
                            5,
                            self.track_texture.as_ref(),
                            self.wall_texture.as_ref(),
                        );
                        self.mode = AppMode::Driving;

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
