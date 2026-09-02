use crate::core::geometry::LineSegment;
use crate::core::track::CheckpointGate;
use macroquad::prelude::Vec2;

#[derive(Clone, Debug, Default)]
pub struct TimingState {
    pub next_checkpoint_idx: usize,
    pub current_lap_start_time: f64,
    pub sector_start_time: f64,
    pub current_sector: usize,
    pub current_sector_times: [Option<f32>; 3],
    pub best_sector_times: [Option<f32>; 3],
    pub last_lap_time: Option<f32>,
    pub best_lap_time: Option<f32>,
    pub completed_laps: u32,
}

impl TimingState {
    pub fn update(
        &mut self,
        prev_pos: Vec2,
        curr_pos: Vec2,
        checkpoints: &[CheckpointGate],
        current_time: f64,
    ) {
        if checkpoints.is_empty() {
            return;
        }

        let total_gates = checkpoints.len();
        let s1_idx = total_gates / 3;
        let s2_idx = (total_gates * 2) / 3;

        let motion = LineSegment::new(prev_pos, curr_pos);
        let target_gate = &checkpoints[self.next_checkpoint_idx];

        if motion.intersects(&target_gate.line) {
            let hit_idx = self.next_checkpoint_idx;

            if hit_idx == 0 {
                if self.current_lap_start_time > 0.0 {
                    let sec3_time = (current_time - self.sector_start_time) as f32;
                    self.current_sector_times[2] = Some(sec3_time);
                    if self.best_sector_times[2].map_or(true, |b| sec3_time < b) {
                        self.best_sector_times[2] = Some(sec3_time);
                    }

                    let lap_time = (current_time - self.current_lap_start_time) as f32;
                    self.last_lap_time = Some(lap_time);
                    if self.best_lap_time.map_or(true, |best| lap_time < best) {
                        self.best_lap_time = Some(lap_time);
                    }
                    self.completed_laps += 1;
                }

                self.current_lap_start_time = current_time;
                self.sector_start_time = current_time;
                self.current_sector = 0;
                self.next_checkpoint_idx = 1 % total_gates;
            } else {
                if hit_idx == s1_idx && s1_idx > 0 {
                    let sec1_time = (current_time - self.sector_start_time) as f32;
                    self.current_sector_times[0] = Some(sec1_time);
                    if self.best_sector_times[0].map_or(true, |b| sec1_time < b) {
                        self.best_sector_times[0] = Some(sec1_time);
                    }
                    self.sector_start_time = current_time;
                    self.current_sector = 1;
                } else if hit_idx == s2_idx && s2_idx > s1_idx {
                    let sec2_time = (current_time - self.sector_start_time) as f32;
                    self.current_sector_times[1] = Some(sec2_time);
                    if self.best_sector_times[1].map_or(true, |b| sec2_time < b) {
                        self.best_sector_times[1] = Some(sec2_time);
                    }
                    self.sector_start_time = current_time;
                    self.current_sector = 2;
                }

                self.next_checkpoint_idx = (self.next_checkpoint_idx + 1) % total_gates;
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
