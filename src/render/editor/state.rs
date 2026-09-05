use macroquad::prelude::*;

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

pub struct EditorHistory {
    pub undo_stack: Vec<EditorSnapshot>,
    pub redo_stack: Vec<EditorSnapshot>,
}

impl EditorHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn pop_undo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot> {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(snapshot)
        } else {
            None
        }
    }

    pub fn pop_redo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot> {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(snapshot)
        } else {
            None
        }
    }
}

pub struct EditorState {
    pub active_tool: EditorTool,
    pub is_drawing: bool,
    pub is_dragging_start: bool,
    pub is_dragging_node: bool,
    pub is_rotating_grid: bool,
    pub selected_node_index: Option<usize>,
    pub hovered_node_index: Option<usize>,
    pub snap_distance: f32,
    pub ui_rect: Rect,
    pub history: EditorHistory,
    pub show_help_overlay: bool,
    pub last_mouse_world_pan: Option<Vec2>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::PenDraw,
            is_drawing: false,
            is_dragging_start: false,
            is_dragging_node: false,
            is_rotating_grid: false,
            selected_node_index: None,
            hovered_node_index: None,
            snap_distance: 90.0,
            ui_rect: Rect::new(15.0, 15.0, 260.0, 520.0),
            history: EditorHistory::new(),
            show_help_overlay: true,
            last_mouse_world_pan: None,
        }
    }
}
