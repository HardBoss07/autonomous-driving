use macroquad::prelude::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct TrackSegment {
    pub center: Vec2,
    pub left_bound: Vec2,
    pub right_bound: Vec2,
    pub normal: Vec2,
    pub tangent: Vec2,
    pub distance_along_track: f32,
}
