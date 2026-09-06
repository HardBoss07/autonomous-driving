use macroquad::prelude::*;

#[inline]
pub fn draw_quad(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2, color: Color) {
    draw_triangle(v1, v2, v3, color);
    draw_triangle(v1, v3, v4, color);
}
