use crate::types::deck::{PlayerState, TempoRange};

impl PlayerState {
    pub fn get_actual_slider_tempo(&self) -> f32 {
        let (range_min, range_max) = match self.tempo_range {
            TempoRange::SixPercent => (1.0 - 0.06, 1.06),
            TempoRange::TenPercent => (1.0 - 0.10, 1.10),
            TempoRange::SixteenPercent => (1.0 - 0.16, 1.16),
            TempoRange::OneHundredPercent => (0.0, 2.0),
            TempoRange::YesPercent => (0.0, 250.0),
        };

        (self.tempo_slider_position + 1.0) * (range_max - range_min) / 2.0 + range_min
    }

    pub fn get_tempo_keyshift(&self) -> f32 {
        if self.master_tempo {
            0.0
        } else {
            12.0 * (1.0 + self.tempo_percent).log2()
        }
    }
}
