use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

use crate::audio::dsp_pipeline::filters::fx::effects::{filter::cutoff_normalized, reverb::Reverb};

pub struct Space<const CHANNELS: usize> {
    low: BiquadFilter<CHANNELS>,
    high: BiquadFilter<CHANNELS>,

    reverb: Reverb<CHANNELS>,

    sample_rate: u32,

    parameter: f64,
}

impl<const CHANNELS: usize> Space<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        let mut new = Self {
            low: BiquadFilter::default(),
            high: BiquadFilter::default(),

            reverb: Reverb::<CHANNELS>::new(sample_rate),

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

        self.reverb.set_params(
            0.5,
            0.5,
            parameter.abs() * 0.35,
            parameter.abs() * 0.5,
            1.0 - (parameter.abs() * 0.5),
        );
    }

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        // let low_filter = self.low.process(input);

        // let high_filter = self.high.process(input);

        // let input = if self.parameter < -f64::EPSILON {
        //     low_filter
        // } else if self.parameter > f64::EPSILON {
        //     high_filter
        // } else {
        //     input
        // };

        self.reverb.process(input)
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.high.reset();
    }
}
