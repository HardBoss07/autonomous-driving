use macroquad::prelude::*;

pub fn perpendicular_distance(point: Vec2, line_a: Vec2, line_b: Vec2) -> f32 {
    let length_squared = line_a.distance_squared(line_b);
    if length_squared < 1e-6 {
        return point.distance(line_a);
    }
    let projection_factor =
        ((point - line_a).dot(line_b - line_a) / length_squared).clamp(0.0, 1.0);
    let projection = line_a + (line_b - line_a) * projection_factor;
    point.distance(projection)
}

pub fn ramer_douglas_peucker(points: &[Vec2], epsilon: f32) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut maximum_distance = 0.0;
    let mut split_index = 0;
    let end_index = points.len() - 1;

    for index in 1..end_index {
        let distance = perpendicular_distance(points[index], points[0], points[end_index]);
        if distance > maximum_distance {
            split_index = index;
            maximum_distance = distance;
        }
    }

    if maximum_distance > epsilon {
        let mut recursive_left = ramer_douglas_peucker(&points[..=split_index], epsilon);
        let recursive_right = ramer_douglas_peucker(&points[split_index..], epsilon);

        recursive_left.pop();
        recursive_left.extend(recursive_right);
        recursive_left
    } else {
        vec![points[0], points[end_index]]
    }
}

pub fn resample_points(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut result = vec![points[0]];
    let mut previous_point = points[0];
    let mut accumulated_distance = 0.0;

    for &current_point in points.iter().skip(1) {
        let segment_distance = previous_point.distance(current_point);
        if segment_distance < 0.001 {
            continue;
        }

        let direction = (current_point - previous_point) / segment_distance;
        let mut step = spacing - accumulated_distance;

        while step <= segment_distance {
            let new_point = previous_point + direction * step;
            result.push(new_point);
            step += spacing;
        }
        accumulated_distance = segment_distance - (step - spacing);
        previous_point = current_point;
    }

    if let Some(&last_point) = points.last() {
        if result
            .last()
            .map_or(true, |&p| p.distance(last_point) > 5.0)
        {
            result.push(last_point);
        }
    }

    result
}

pub fn smooth_points_with_fixed_prefix(
    points: &[Vec2],
    is_closed: bool,
    iterations: usize,
    fixed_prefix_count: usize,
) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut current = points.to_vec();
    for _ in 0..iterations {
        let mut next = current.clone();
        let length = current.len();

        for index in 1..length - 1 {
            if index < fixed_prefix_count {
                continue;
            }
            next[index] =
                current[index - 1] * 0.25 + current[index] * 0.5 + current[index + 1] * 0.25;
        }

        if is_closed {
            next[0] = current[length - 1] * 0.25 + current[0] * 0.5 + current[1] * 0.25;
            next[length - 1] =
                current[length - 2] * 0.25 + current[length - 1] * 0.5 + current[0] * 0.25;
        }

        current = next;
    }

    current
}

pub fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    0.5 * ((p1 * 2.0)
        + (-p0 + p2) * t
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t * t
        + (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * t * t * t)
}

pub fn catmull_rom_derivative(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    0.5 * ((-p0 + p2)
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * 2.0 * t
        + (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * 3.0 * t * t)
}
