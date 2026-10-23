use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

pub struct DubEcho<const CHANNELS: usize> {
    low: BiquadFilter<CHANNELS>,
    high: BiquadFilter<CHANNELS>,

    sample_rate: u32,

    parameter: f64,

    buffer: Vec<[f32; CHANNELS]>,
    position: usize,
}

impl<const CHANNELS: usize> DubEcho<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        let delay_length = sample_rate as usize / 5;

        let mut new = Self {
            low: BiquadFilter::default(),
            high: BiquadFilter::default(),

            sample_rate,

            parameter: 0.0,

            buffer: vec![[0.0; CHANNELS]; delay_length],
            position: 0,
        };

        new.set_parameter(0.0);

        new
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        self.parameter = f64::from(parameter);

        let (low_cutoff, high_cutoff) = if parameter > 0.0 {
            (
                20000.0,
                20.0 * (1000.0f64 / 20.0f64).powf(self.parameter.abs()),
            )
        } else {
            (
                20000.0 * (8000.0f64 / 20000.0f64).powf(self.parameter.abs()),
                20.0,
            )
        };

        let q = FRAC_1_SQRT_2;

        self.low.configure(FilterConfiguration::LowPassQ(
            (low_cutoff / f64::from(self.sample_rate)).min(0.49),
            q,
        ));
        self.high.configure(FilterConfiguration::HighPassQ(
            (high_cutoff / f64::from(self.sample_rate)).min(0.49),
            q,
        ));
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::indexing_slicing)]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let delayed = self.buffer[self.position];

        let filtered = self.high.process(self.low.process(delayed));

        for ((input, filtered), buffer) in input
            .iter()
            .zip(&filtered)
            .zip(&mut self.buffer[self.position])
        {
            *buffer = *input + *filtered * (self.parameter.abs() as f32);
        }

        self.position = (self.position + 1) % self.buffer.len();

        let mut out = [0.0; CHANNELS];

        for ((input, delayed), out) in input.iter().zip(&delayed).zip(&mut out) {
            *out = *input + *delayed * self.parameter.abs() as f32;
        }

        out
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.high.reset();
    }
}
