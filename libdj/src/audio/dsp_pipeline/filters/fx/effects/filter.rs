const MIN_HZ: f64 = 20.0;
const MAX_HZ: f64 = 20_000.0;

pub fn cutoff_normalized(x: f64, sample_rate: u32) -> f64 {
    let hz = MIN_HZ * (MAX_HZ / MIN_HZ).powf(x);

    (hz / f64::from(sample_rate)).min(0.49)
}

use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

pub struct Filter<const CHANNELS: usize> {
    low: BiquadFilter<CHANNELS>,
    high: BiquadFilter<CHANNELS>,

    sample_rate: u32,

    parameter: f64,
}

impl<const CHANNELS: usize> Filter<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        let mut new = Self {
            low: BiquadFilter::default(),
            high: BiquadFilter::default(),

            sample_rate,

            parameter: 0.0,
        };

        new.set_parameter(0.0);

        new
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        self.parameter = f64::from(parameter);

        let q = FRAC_1_SQRT_2;

        self.low.configure(FilterConfiguration::LowPassQ(
            cutoff_normalized(1.0 + self.parameter.min(0.0), self.sample_rate),
            q,
        ));
        self.high.configure(FilterConfiguration::HighPassQ(
            cutoff_normalized(self.parameter.max(0.0), self.sample_rate),
            q,
        ));
    }

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let low_filter = self.low.process(input);

        let high_filter = self.high.process(input);

        if self.parameter < -f64::EPSILON {
            low_filter
        } else if self.parameter > f64::EPSILON {
            high_filter
        } else {
            input
        }
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.high.reset();
    }
}
