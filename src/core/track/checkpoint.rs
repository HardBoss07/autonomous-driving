use crate::core::geometry::LineSegment;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointGate {
    pub id: usize,
    pub line: LineSegment,
}
