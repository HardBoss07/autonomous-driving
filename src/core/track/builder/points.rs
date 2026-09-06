use crate::core::track::grid::StartingGrid;
use macroquad::prelude::Vec2;

pub fn prepare_track_points(
    raw_points: &[Vec2],
    starting_grid: &StartingGrid,
    is_closed: bool,
) -> Vec<Vec2> {
    let grid_start = starting_grid.start_point();
    let exit_target = starting_grid.exit_target();
    let entry_target = starting_grid.entry_target();
    let grid_end = starting_grid.end_point();
    let forward = starting_grid.forward_vector();

    let mut points = Vec::new();
    points.push(grid_start);
    points.push(exit_target);

    let mut raw_iter = raw_points.iter().peekable();
    while let Some(&&point) = raw_iter.peek() {
        let projection = (point - grid_start).dot(forward);
        if projection < 72.0 || point.distance(exit_target) < 20.0 {
            raw_iter.next();
        } else {
            break;
        }
    }

    for &point in raw_iter {
        if let Some(&last_point) = points.last() {
            if last_point.distance(point) > 15.0 {
                points.push(point);
            }
        } else {
            points.push(point);
        }
    }

    if is_closed {
        while let Some(&last_point) = points.last() {
            if last_point == grid_start || last_point == exit_target {
                break;
            }
            let projection_entry = (last_point - grid_end).dot(-forward);
            if projection_entry < 72.0 || last_point.distance(entry_target) < 20.0 {
                points.pop();
            } else {
                break;
            }
        }

        points.push(entry_target);
        points.push(grid_end);
    }

    points
}
