use crate::core::track::Track;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditorTool {
    SelectMove,
    PenDraw,
    NodePlace,
    NodeDelete,
}

#[derive(Clone, Debug)]
pub struct EditorSnapshot {
    pub raw_points: Vec<Vec2>,
    pub grid_position: Vec2,
    pub grid_rotation: f32,
    pub is_closed: bool,
}

pub struct TrackEditor {
    pub active_tool: EditorTool,
    pub is_drawing: bool,
    pub is_dragging_start: bool,
    pub is_dragging_node: bool,
    pub selected_node_index: Option<usize>,
    pub hovered_node_index: Option<usize>,
    pub snap_distance: f32,
    pub ui_rect: Rect,
    pub undo_stack: Vec<EditorSnapshot>,
    pub redo_stack: Vec<EditorSnapshot>,
    pub show_help_overlay: bool,
    pub last_mouse_world_pan: Option<Vec2>,
}

impl TrackEditor {
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::PenDraw,
            is_drawing: false,
            is_dragging_start: false,
            is_dragging_node: false,
            selected_node_index: None,
            hovered_node_index: None,
            snap_distance: 90.0,
            ui_rect: Rect::new(15.0, 15.0, 260.0, 480.0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_help_overlay: true,
            last_mouse_world_pan: None,
        }
    }

    pub fn is_mouse_over_ui(&self, mouse_pos: Vec2) -> bool {
        self.ui_rect.contains(mouse_pos) || root_ui().is_mouse_over(mouse_pos)
    }

    pub fn save_snapshot(&mut self, track: &Track) {
        let snapshot = EditorSnapshot {
            raw_points: track.raw_points.clone(),
            grid_position: track.starting_grid.position,
            grid_rotation: track.starting_grid.rotation,
            is_closed: track.is_closed,
        };
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, track: &mut Track, track_tex: Option<&Texture2D>) {
        if let Some(snapshot) = self.undo_stack.pop() {
            let current = EditorSnapshot {
                raw_points: track.raw_points.clone(),
                grid_position: track.starting_grid.position,
                grid_rotation: track.starting_grid.rotation,
                is_closed: track.is_closed,
            };
            self.redo_stack.push(current);

            track.raw_points = snapshot.raw_points;
            track.starting_grid.position = snapshot.grid_position;
            track.starting_grid.rotation = snapshot.grid_rotation;
            track.is_closed = snapshot.is_closed;
            track.rebuild_mesh(5, track_tex);
        }
    }

    pub fn redo(&mut self, track: &mut Track, track_tex: Option<&Texture2D>) {
        if let Some(snapshot) = self.redo_stack.pop() {
            let current = EditorSnapshot {
                raw_points: track.raw_points.clone(),
                grid_position: track.starting_grid.position,
                grid_rotation: track.starting_grid.rotation,
                is_closed: track.is_closed,
            };
            self.undo_stack.push(current);

            track.raw_points = snapshot.raw_points;
            track.starting_grid.position = snapshot.grid_position;
            track.starting_grid.rotation = snapshot.grid_rotation;
            track.is_closed = snapshot.is_closed;
            track.rebuild_mesh(5, track_tex);
        }
    }

    pub fn update_and_draw_ui(
        &mut self,
        track: &mut Track,
        camera_zoom: &mut f32,
        track_tex: Option<&Texture2D>,
    ) -> bool {
        let mut exit_editor = false;

        widgets::Window::new(
            hash!(),
            vec2(self.ui_rect.x, self.ui_rect.y),
            vec2(self.ui_rect.w, self.ui_rect.h),
        )
        .label("Track Builder Tools")
        .ui(&mut root_ui(), |ui| {
            ui.label(None, "--- EDITING TOOLS ---");
            if ui.button(
                None,
                if self.active_tool == EditorTool::PenDraw {
                    "> Freehand Pen Draw"
                } else {
                    "Tool: Freehand Pen"
                },
            ) {
                self.active_tool = EditorTool::PenDraw;
                self.is_drawing = false;
            }
            if ui.button(
                None,
                if self.active_tool == EditorTool::NodePlace {
                    "> Click Node Place"
                } else {
                    "Tool: Click Node"
                },
            ) {
                self.active_tool = EditorTool::NodePlace;
                self.is_drawing = false;
            }
            if ui.button(
                None,
                if self.active_tool == EditorTool::SelectMove {
                    "> Select / Move"
                } else {
                    "Tool: Select / Move"
                },
            ) {
                self.active_tool = EditorTool::SelectMove;
                self.is_drawing = false;
            }
            if ui.button(
                None,
                if self.active_tool == EditorTool::NodeDelete {
                    "> Delete Node"
                } else {
                    "Tool: Delete Node"
                },
            ) {
                self.active_tool = EditorTool::NodeDelete;
                self.is_drawing = false;
            }

            ui.separator();
            ui.label(None, "--- TRACK PROPERTIES ---");
            let loop_text = if track.is_closed {
                "Loop State: Closed (Toggle Open)"
            } else {
                "Loop State: Open (Toggle Close)"
            };
            if ui.button(None, loop_text) {
                self.save_snapshot(track);
                track.is_closed = !track.is_closed;
                track.rebuild_mesh(5, track_tex);
            }

            if ui.button(None, "Clear Track Curve") {
                self.save_snapshot(track);
                track.clear();
                self.is_drawing = false;
            }

            ui.separator();
            ui.label(None, "--- AI CHECKPOINT GATES ---");
            let mut spacing_f32 = track.checkpoint_spacing as f32;
            ui.slider(hash!(), "Gate Spacing", 5.0..40.0, &mut spacing_f32);
            if (spacing_f32 as usize) != track.checkpoint_spacing {
                track.checkpoint_spacing = spacing_f32 as usize;
                track.rebuild_mesh(5, track_tex);
            }

            ui.separator();
            ui.label(None, "--- VIEWPORT & HISTORY ---");
            ui.slider(hash!(), "Zoom Level", 0.05..2.5, camera_zoom);

            if ui.button(None, "Undo Action (Ctrl+Z)") {
                self.undo(track, track_tex);
            }
            if ui.button(None, "Redo Action (Ctrl+Y)") {
                self.redo(track, track_tex);
            }

            let help_btn_text = if self.show_help_overlay {
                "Hide Controls HUD"
            } else {
                "Show Controls HUD"
            };
            if ui.button(None, help_btn_text) {
                self.show_help_overlay = !self.show_help_overlay;
            }

            ui.separator();
            if ui.button(None, "Save Track & Drive!") {
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
        camera_target: &mut Vec2,
        camera_zoom: &mut f32,
        dt: f32,
    ) {
        let over_ui = self.is_mouse_over_ui(screen_mouse);

        let mut pan_dir = vec2(0.0, 0.0);
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            pan_dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            pan_dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            pan_dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            pan_dir.x += 1.0;
        }

        if pan_dir.length() > 0.0 {
            let speed = 750.0 / *camera_zoom;
            *camera_target += pan_dir.normalize() * speed * dt;
        }

        if is_mouse_button_pressed(MouseButton::Middle) {
            self.last_mouse_world_pan = Some(screen_mouse);
        }
        if is_mouse_button_down(MouseButton::Middle) {
            if let Some(last_pos) = self.last_mouse_world_pan {
                let delta = (screen_mouse - last_pos) / *camera_zoom;
                *camera_target -= delta;
                self.last_mouse_world_pan = Some(screen_mouse);
            }
        } else {
            self.last_mouse_world_pan = None;
        }

        let wheel_y = mouse_wheel().1;
        if wheel_y != 0.0 && !over_ui {
            let zoom_factor = if wheel_y > 0.0 { 1.15 } else { 0.85 };
            let old_zoom = *camera_zoom;
            let new_zoom = (old_zoom * zoom_factor).clamp(0.05, 3.0);

            if (new_zoom - old_zoom).abs() > 0.0001 {
                let screen_center = vec2(screen_width() * 0.5, screen_height() * 0.5);
                let mouse_offset = screen_mouse - screen_center;
                let world_before = *camera_target + mouse_offset / old_zoom;

                *camera_zoom = new_zoom;
                *camera_target = world_before - mouse_offset / new_zoom;
            }
        }

        if is_key_pressed(KeyCode::F) && !over_ui {
            if !track.raw_points.is_empty() {
                let mut min_p = track.raw_points[0];
                let mut max_p = track.raw_points[0];
                for p in &track.raw_points {
                    min_p = min_p.min(*p);
                    max_p = max_p.max(*p);
                }
                *camera_target = (min_p + max_p) * 0.5;
            } else {
                *camera_target = track.starting_grid.position;
            }
        }

        let ctrl_down = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        if ctrl_down && is_key_pressed(KeyCode::Z) {
            self.undo(track, track_tex);
            return;
        }
        if ctrl_down && is_key_pressed(KeyCode::Y) {
            self.redo(track, track_tex);
            return;
        }

        self.hovered_node_index = None;
        let hit_radius = 25.0 / (*camera_zoom).max(0.2);
        let mut min_dist = hit_radius;
        for (idx, p) in track.raw_points.iter().enumerate() {
            let dist = world_mouse.distance(*p);
            if dist < min_dist {
                min_dist = dist;
                self.hovered_node_index = Some(idx);
            }
        }

        let exit_target = track.starting_grid.exit_target();
        let entry_target = track.starting_grid.entry_target();

        if !over_ui && is_mouse_button_pressed(MouseButton::Right) {
            if let Some(idx) = self.hovered_node_index {
                self.save_snapshot(track);
                track.remove_raw_point(idx);
                self.hovered_node_index = None;
                self.selected_node_index = None;
                track.rebuild_mesh(5, track_tex);
                return;
            }
        }

        match self.active_tool {
            EditorTool::SelectMove => {
                let grid_pos = track.starting_grid.position;

                if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
                    if world_mouse.distance(grid_pos) < 70.0 {
                        self.save_snapshot(track);
                        self.is_dragging_start = true;
                    } else if let Some(idx) = self.hovered_node_index {
                        self.save_snapshot(track);
                        self.selected_node_index = Some(idx);
                        self.is_dragging_node = true;
                    } else {
                        self.selected_node_index = None;
                    }
                }

                if is_mouse_button_released(MouseButton::Left) {
                    if self.is_dragging_start || self.is_dragging_node {
                        track.rebuild_mesh(5, track_tex);
                    }
                    self.is_dragging_start = false;
                    self.is_dragging_node = false;
                }

                if self.is_dragging_start {
                    track.starting_grid.position = world_mouse;
                    track.rebuild_mesh(3, track_tex);
                }

                if self.is_dragging_node {
                    if let Some(idx) = self.selected_node_index {
                        track.move_raw_point(idx, world_mouse);
                        track.rebuild_mesh(3, track_tex);
                    }
                }

                if wheel_y != 0.0 && !over_ui && self.is_dragging_start {
                    track.starting_grid.rotation += wheel_y.signum() * 0.1;
                    track.rebuild_mesh(3, track_tex);
                }
            }
            EditorTool::NodePlace => {
                if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
                    self.save_snapshot(track);

                    if track.raw_points.is_empty() {
                        track.add_raw_point(exit_target);
                    }

                    if world_mouse.distance(entry_target) < self.snap_distance
                        && track.raw_points.len() > 2
                    {
                        track.add_raw_point(entry_target);
                        track.is_closed = true;
                    } else {
                        track.add_raw_point(world_mouse);
                    }

                    track.rebuild_mesh(5, track_tex);
                }
            }
            EditorTool::NodeDelete => {
                if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
                    if let Some(idx) = self.hovered_node_index {
                        self.save_snapshot(track);
                        track.remove_raw_point(idx);
                        self.hovered_node_index = None;
                        track.rebuild_mesh(5, track_tex);
                    }
                }
            }
            EditorTool::PenDraw => {
                if !over_ui && is_mouse_button_pressed(MouseButton::Left) {
                    self.save_snapshot(track);
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

    pub fn draw_snap_previews(&self, track: &Track, world_mouse: Vec2, camera_zoom: f32) {
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

        for gate in &track.checkpoints {
            draw_line(
                gate.line.a.x,
                gate.line.a.y,
                gate.line.b.x,
                gate.line.b.y,
                1.5,
                Color::new(0.0, 0.8, 1.0, 0.4),
            );
        }

        if track.raw_points.len() >= 2 {
            for i in 0..track.raw_points.len() - 1 {
                let p1 = track.raw_points[i];
                let p2 = track.raw_points[i + 1];
                draw_line(p1.x, p1.y, p2.x, p2.y, 1.5, Color::new(0.3, 0.7, 1.0, 0.5));
            }
        }

        let node_radius = (8.0 / camera_zoom.max(0.2)).clamp(4.0, 20.0);
        for (idx, &p) in track.raw_points.iter().enumerate() {
            let mut color = SKYBLUE;
            if Some(idx) == self.selected_node_index {
                color = RED;
            } else if Some(idx) == self.hovered_node_index {
                color = GOLD;
            }

            draw_circle(p.x, p.y, node_radius, color);
            draw_circle_lines(p.x, p.y, node_radius + 2.0, 1.5, WHITE);
        }
    }

    pub fn draw_help_overlay(&self) {
        let x = screen_width() - 280.0;
        let y = 15.0;
        let w = 265.0;
        let h = 230.0;

        draw_rectangle(x, y, w, h, Color::new(0.02, 0.02, 0.05, 0.85));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::new(0.3, 0.5, 0.9, 0.8));

        let font_size = 15.0;
        let line_height = 20.0;
        let mut cur_y = y + 25.0;

        draw_text("EDITOR CONTROLS HUD", x + 12.0, cur_y, 16.0, GOLD);
        cur_y += line_height + 5.0;

        let keybinds = [
            ("WASD / Arrows", "Pan Camera"),
            ("Mouse Scroll", "Zoom to Cursor"),
            ("Middle Drag", "Pan Canvas"),
            ("Key [ F ]", "Focus Track Center"),
            ("Left Click", "Draw / Place / Drag"),
            ("Right Click", "Quick Delete Node"),
            ("Ctrl + Z / Y", "Undo / Redo"),
        ];

        for (key, action) in keybinds {
            draw_text(key, x + 12.0, cur_y, font_size, WHITE);
            draw_text(action, x + 130.0, cur_y, font_size, LIGHTGRAY);
            cur_y += line_height;
        }
    }
}
