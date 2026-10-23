const LOW_MIN_HZ: f64 = 3000.0;
const LOW_MAX_HZ: f64 = 12_000.0;

fn low_cutoff_normalized(x: f64, sample_rate: u32) -> f64 {
    let hz = LOW_MIN_HZ * (LOW_MAX_HZ / LOW_MIN_HZ).powf(x);

    (hz / f64::from(sample_rate)).min(0.49)
}

const HIGH_MIN_HZ: f64 = 20.0;
const HIGH_MAX_HZ: f64 = 3_000.0;

fn high_cutoff_normalized(x: f64, sample_rate: u32) -> f64 {
    let hz = HIGH_MIN_HZ * (HIGH_MAX_HZ / HIGH_MIN_HZ).powf(x);

    (hz / f64::from(sample_rate)).min(0.49)
}

use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

pub struct Bitcrush<const CHANNELS: usize> {
    low: BiquadFilter<CHANNELS>,
    high: BiquadFilter<CHANNELS>,

    sample_rate: u32,

    parameter: f64,

    phase: f32,
    held: [f32; CHANNELS],
}

impl<const CHANNELS: usize> Bitcrush<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        let mut new = Self {
            low: BiquadFilter::default(),
            high: BiquadFilter::default(),

            sample_rate,

            parameter: 0.0,

            phase: 0.0,
            held: [0.0; CHANNELS],
        };

        new.set_parameter(0.0);

        new
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        self.parameter = f64::from(parameter);

        let q = FRAC_1_SQRT_2;

        self.low.configure(FilterConfiguration::LowPassQ(
            low_cutoff_normalized(self.parameter.abs(), self.sample_rate),
            q,
        ));

        self.high.configure(FilterConfiguration::HighPassQ(
            high_cutoff_normalized(self.parameter.max(0.0), self.sample_rate),
            q,
        ));
    }

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        #[allow(clippy::cast_possible_truncation)]
        let depth = self.parameter.abs() as f32;

        let bits = 24.0 - depth * 16.0;
        let factor = (depth * 5.0).exp2();
        let drive = 1.0 + depth * 3.0;
        let mix = (depth * 4.0).min(1.0);

        let mut driven = [0.0; CHANNELS];

        for (input, driven) in input.iter().zip(&mut driven) {
            *driven = (*input * drive).tanh();
        }

        self.phase += 1.0 / factor;

        if self.phase >= 1.0 {
            self.phase -= 1.0;

            let levels = (bits - 1.0).exp2();

            for (driven, held) in driven.iter().zip(&mut self.held) {
                *held = (*driven * levels).round() / levels;
            }
        }

        let low_filter = self.low.process(self.held);

        let mut bitcrush = [0.0; CHANNELS];

        for ((input, low_filter), bitcrush) in input.iter().zip(&low_filter).zip(&mut bitcrush) {
            *bitcrush = input * (1.0 - mix) + low_filter * mix;
        }

        let high_filter = self.high.process(bitcrush);

        if self.parameter < -f64::EPSILON {
            bitcrush
        } else if self.parameter > f64::EPSILON {
            high_filter
        } else {
            input
        }
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.high.reset();

        self.phase = 1.0;
        self.held = [0.0; CHANNELS];
    }
}
