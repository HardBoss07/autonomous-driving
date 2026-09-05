#[derive(Clone, Debug, Default)]
pub struct SectorTimes {
    pub current_sector: usize,
    pub sector_start_time: f64,
    pub current_sector_times: [Option<f32>; 3],
    pub best_sector_times: [Option<f32>; 3],
}

impl SectorTimes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_sector(&mut self, sector_index: usize, current_time: f64) {
        if sector_index < 3 {
            let sector_time = (current_time - self.sector_start_time) as f32;
            self.current_sector_times[sector_index] = Some(sector_time);
            if self.best_sector_times[sector_index].map_or(true, |best| sector_time < best) {
                self.best_sector_times[sector_index] = Some(sector_time);
            }
        }
        self.sector_start_time = current_time;
        self.current_sector = (sector_index + 1) % 3;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
