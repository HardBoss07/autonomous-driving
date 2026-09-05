#[derive(Clone, Debug, Default)]
pub struct LapTimes {
    pub current_lap_start_time: f64,
    pub last_lap_time: Option<f32>,
    pub best_lap_time: Option<f32>,
    pub completed_laps: u32,
}

impl LapTimes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn complete_lap(&mut self, current_time: f64) -> Option<f32> {
        if self.current_lap_start_time > 0.0 {
            let lap_time = (current_time - self.current_lap_start_time) as f32;
            self.last_lap_time = Some(lap_time);
            if self.best_lap_time.map_or(true, |best| lap_time < best) {
                self.best_lap_time = Some(lap_time);
            }
            self.completed_laps += 1;
            self.current_lap_start_time = current_time;
            Some(lap_time)
        } else {
            self.current_lap_start_time = current_time;
            None
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
