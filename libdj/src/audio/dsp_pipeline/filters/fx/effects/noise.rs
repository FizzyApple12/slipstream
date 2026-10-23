use std::{
    f64::consts::FRAC_1_SQRT_2,
    time::{SystemTime, UNIX_EPOCH},
};

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

use crate::audio::dsp_pipeline::filters::fx::effects::filter::cutoff_normalized;

pub struct Noise<const CHANNELS: usize> {
    low: BiquadFilter<CHANNELS>,
    high: BiquadFilter<CHANNELS>,

    sample_rate: u32,

    feedback_buffer: [u32; 4],

    parameter: f64,
}

impl<const CHANNELS: usize> Noise<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let a = x ^ (x >> 31);

        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let b = x ^ (x >> 31);

        #[allow(clippy::cast_possible_truncation)]
        let mut new = Self {
            low: BiquadFilter::default(),
            high: BiquadFilter::default(),

            sample_rate,

            feedback_buffer: [a as u32, (a >> 32) as u32, b as u32, (b >> 32) as u32],

            parameter: 0.0,
        };

        new.set_parameter(0.0);

        new
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        self.parameter = f64::from(parameter);

        let q = FRAC_1_SQRT_2;

        self.low.configure(FilterConfiguration::LowPassQ(
            cutoff_normalized(1.0 - (1.0 + self.parameter.min(0.0)), self.sample_rate),
            q,
        ));
        self.high.configure(FilterConfiguration::HighPassQ(
            cutoff_normalized(1.0 - self.parameter.max(0.0), self.sample_rate),
            q,
        ));
    }

    #[inline]
    pub fn process(&mut self, mut input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let bits =
            ((self.feedback_buffer[0].wrapping_add(self.feedback_buffer[3])) >> 9) | 0x3F80_0000;
        let noise = f32::from_bits(bits) * 2.0 - 3.0;
        let noise = [noise; CHANNELS];

        let t = self.feedback_buffer[1] << 9;
        self.feedback_buffer[2] ^= self.feedback_buffer[0];
        self.feedback_buffer[3] ^= self.feedback_buffer[1];
        self.feedback_buffer[1] ^= self.feedback_buffer[2];
        self.feedback_buffer[0] ^= self.feedback_buffer[3];
        self.feedback_buffer[2] ^= t;
        self.feedback_buffer[3] = self.feedback_buffer[3].rotate_left(11);

        let low_filter = self.low.process(noise);

        let high_filter = self.high.process(noise);

        if self.parameter < -f64::EPSILON {
            for (input, low_filter) in input.iter_mut().zip(&low_filter) {
                *input += *low_filter * 0.1;
            }
        } else if self.parameter > f64::EPSILON {
            for (input, high_filter) in input.iter_mut().zip(&high_filter) {
                *input += *high_filter * 0.1;
            }
        }

        input
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.high.reset();
    }
}
