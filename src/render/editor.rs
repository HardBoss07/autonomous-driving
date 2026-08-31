use crate::core::track::Track;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditorTool {
    SelectMove,
    PenDraw,
}

pub struct TrackEditor {
    pub active_tool: EditorTool,
    pub is_drawing: bool,
    pub is_dragging_start: bool,
    pub snap_distance: f32,
    pub ui_rect: Rect,
}

impl TrackEditor {
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::PenDraw,
            is_drawing: false,
            is_dragging_start: false,
            snap_distance: 90.0,
            ui_rect: Rect::new(15.0, 15.0, 230.0, 260.0),
        }
    }

    pub fn is_mouse_over_ui(&self, mouse_pos: Vec2) -> bool {
        self.ui_rect.contains(mouse_pos) || root_ui().is_mouse_over(mouse_pos)
    }

    pub fn update_and_draw_ui(&mut self, track: &mut Track, camera_zoom: &mut f32) -> bool {
        let mut exit_editor = false;

        widgets::Window::new(
            hash!(),
            vec2(self.ui_rect.x, self.ui_rect.y),
            vec2(self.ui_rect.w, self.ui_rect.h),
        )
        .label("Track Builder Tools")
        .ui(&mut root_ui(), |ui| {
            if ui.button(None, "Tool: Pen Draw") {
                self.active_tool = EditorTool::PenDraw;
                self.is_drawing = false;
            }
            if ui.button(None, "Tool: Select / Move") {
                self.active_tool = EditorTool::SelectMove;
                self.is_drawing = false;
            }

            ui.separator();
            if ui.button(None, "Clear Track Curve") {
                track.clear();
                self.is_drawing = false;
            }

            ui.slider(hash!(), "Zoom", 0.3..1.5, camera_zoom);

            ui.separator();
            if ui.button(None, "Save & Drive") {
                exit_editor = true;
                self.is_drawing = false;
            }
        });

        exit_editor
    }

    pub fn handle_input(
        &mut self,
        track: &mut Track,
        track_tex: Option<&Texture2D>,
        world_mouse: Vec2,
        screen_mouse: Vec2,
    ) {
        let over_ui = self.is_mouse_over_ui(screen_mouse);

        let exit_target = track.starting_grid.exit_target();
        let entry_target = track.starting_grid.entry_target();

        match self.active_tool {
            EditorTool::SelectMove => {
                let grid_pos = track.starting_grid.position;

                if !over_ui
                    && is_mouse_button_pressed(MouseButton::Left)
                    && world_mouse.distance(grid_pos) < 70.0
                {
                    self.is_dragging_start = true;
                }

                if is_mouse_button_released(MouseButton::Left) {
                    self.is_dragging_start = false;
                }

                if self.is_dragging_start {
                    track.starting_grid.position = world_mouse;
                    track.rebuild_mesh(3, track_tex);
                }

                let wheel_y = mouse_wheel().1;
                if wheel_y != 0.0 && !over_ui {
                    track.starting_grid.rotation += wheel_y.signum() * 0.1;
                    track.rebuild_mesh(3, track_tex);
                }
            }
            EditorTool::PenDraw => {
                if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
                    self.is_drawing = true;

                    if track.raw_points.is_empty() {
                        track.add_raw_point(exit_target);
                    }
                }

                if self.is_drawing && is_mouse_button_down(MouseButton::Left) {
                    let target = if world_mouse.distance(entry_target) < self.snap_distance {
                        entry_target
                    } else {
                        world_mouse
                    };

                    track.add_raw_point(target);
                    track.rebuild_mesh(2, track_tex);
                }

                if is_mouse_button_released(MouseButton::Left) && self.is_drawing {
                    self.is_drawing = false;

                    if let Some(&last) = track.raw_points.last() {
                        if last.distance(entry_target) < self.snap_distance {
                            if let Some(last_mut) = track.raw_points.last_mut() {
                                *last_mut = entry_target;
                            }
                            track.is_closed = true;
                        }
                    }
                    track.rebuild_mesh(5, track_tex);
                }
            }
        }
    }

    pub fn draw_snap_previews(&self, track: &Track, world_mouse: Vec2) {
        if self.active_tool != EditorTool::PenDraw {
            return;
        }

        let grid_start = track.starting_grid.start_point();
        let grid_end = track.starting_grid.end_point();
        let exit_target = track.starting_grid.exit_target();
        let entry_target = track.starting_grid.entry_target();

        draw_line(
            grid_start.x,
            grid_start.y,
            exit_target.x,
            exit_target.y,
            2.0,
            GREEN,
        );
        draw_line(
            grid_end.x,
            grid_end.y,
            entry_target.x,
            entry_target.y,
            2.0,
            GREEN,
        );

        if track.raw_points.is_empty() {
            draw_circle_lines(exit_target.x, exit_target.y, 28.0, 3.0, GREEN);
            draw_text(
                "Start Target (Exit)",
                exit_target.x - 55.0,
                exit_target.y - 35.0,
                16.0,
                GREEN,
            );
        } else {
            let dist = world_mouse.distance(entry_target);
            let color = if dist < self.snap_distance {
                GREEN
            } else {
                YELLOW
            };
            draw_circle_lines(entry_target.x, entry_target.y, 28.0, 3.0, color);
            draw_text(
                "Finish Target (Entry)",
                entry_target.x - 60.0,
                entry_target.y - 35.0,
                16.0,
                color,
            );
        }
    }
}
