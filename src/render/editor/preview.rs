use crate::core::track::Track;
use macroquad::prelude::*;

pub fn draw_grid_targets(track: &Track, world_mouse: Vec2, snap_distance: f32) {
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
        let color = if dist < snap_distance { GREEN } else { YELLOW };
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

pub fn draw_checkpoint_gates(track: &Track) {
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
}

pub fn draw_track_spline_preview(track: &Track) {
    if track.raw_points.len() >= 2 {
        for i in 0..track.raw_points.len() - 1 {
            let p1 = track.raw_points[i];
            let p2 = track.raw_points[i + 1];
            draw_line(p1.x, p1.y, p2.x, p2.y, 1.5, Color::new(0.3, 0.7, 1.0, 0.5));
        }
    }
}

pub fn draw_nodes(
    track: &Track,
    selected_node_index: Option<usize>,
    hovered_node_index: Option<usize>,
    camera_zoom: f32,
) {
    let node_radius = (8.0 / camera_zoom.max(0.2)).clamp(4.0, 20.0);
    for (index, &point) in track.raw_points.iter().enumerate() {
        let mut color = SKYBLUE;
        if Some(index) == selected_node_index {
            color = RED;
        } else if Some(index) == hovered_node_index {
            color = GOLD;
        }

        draw_circle(point.x, point.y, node_radius, color);
        draw_circle_lines(point.x, point.y, node_radius + 2.0, 1.5, WHITE);
    }
}
