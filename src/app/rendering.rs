use crate::core::geometry::BoundingBox;
use crate::core::physics::CarInput;
use crate::core::track::Track;
use crate::render::editor::TrackEditor;
use crate::render::skidmarks::SkidmarkManager;
use crate::render::{debug, track_render, ui};
use macroquad::prelude::*;

pub fn render_driving(
    camera_target: Vec2,
    driving_zoom: f32,
    track: &Track,
    grid_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    skidmark_manager: &SkidmarkManager,
    car: &crate::core::car::CarState,
    car_texture: Option<&Texture2D>,
    input: &CarInput,
    show_checkpoints: bool,
    config: &mut crate::core::car::CarConfig,
) {
    clear_background(Color::new(0.08, 0.08, 0.10, 1.0));

    set_camera(&Camera2D {
        target: camera_target,
        zoom: vec2(
            (2.0 / screen_width()) * driving_zoom,
            (2.0 / screen_height()) * driving_zoom,
        ),
        ..Default::default()
    });

    let view_half_w = (screen_width() * 0.5) / driving_zoom;
    let view_half_h = (screen_height() * 0.5) / driving_zoom;
    let view_bounds = BoundingBox::new(
        camera_target - vec2(view_half_w, view_half_h),
        camera_target + vec2(view_half_w, view_half_h),
    );

    debug::draw_grid(64.0, screen_width() * 4.0, screen_height() * 4.0);
    track_render::draw_track(track, grid_texture, wall_texture, Some(view_bounds));

    skidmark_manager.draw();

    if show_checkpoints {
        debug::draw_checkpoints(
            &track.checkpoints,
            car.timing.next_checkpoint_idx,
            Some(view_bounds),
        );
    }

    let is_drifting = input.is_drifting();
    debug::draw_car(car, car_texture, input.handbrake);
    debug::draw_drift_indicator(car, is_drifting);

    set_default_camera();

    ui::draw_telemetry(car, is_drifting);
    ui::draw_tuning(config);
}

pub fn render_editor(
    camera_target: Vec2,
    editor_zoom: f32,
    track: &Track,
    grid_texture: Option<&Texture2D>,
    wall_texture: Option<&Texture2D>,
    editor: &TrackEditor,
    world_mouse: Vec2,
) {
    clear_background(Color::new(0.05, 0.05, 0.07, 1.0));

    set_camera(&Camera2D {
        target: camera_target,
        zoom: vec2(
            (2.0 / screen_width()) * editor_zoom,
            (2.0 / screen_height()) * editor_zoom,
        ),
        ..Default::default()
    });

    let view_half_w = (screen_width() * 0.5) / editor_zoom;
    let view_half_h = (screen_height() * 0.5) / editor_zoom;
    let view_bounds = BoundingBox::new(
        camera_target - vec2(view_half_w, view_half_h),
        camera_target + vec2(view_half_w, view_half_h),
    );

    debug::draw_grid(64.0, screen_width() * 10.0, screen_height() * 10.0);
    track_render::draw_track(track, grid_texture, wall_texture, Some(view_bounds));
    editor.draw_snap_previews(track, world_mouse, editor_zoom);

    set_default_camera();
}
