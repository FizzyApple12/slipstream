use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

struct Comb<const CHANNELS: usize> {
    buffer: Vec<[f32; CHANNELS]>,
    position: usize,

    feedback: f32,

    input_filter: BiquadFilter<CHANNELS>,
}

impl<const CHANNELS: usize> Comb<CHANNELS> {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![[0.0; CHANNELS]; size.max(1)],
            position: 0,
            feedback: 0.5,
            input_filter: BiquadFilter::default(),
        }
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn set_damp_hz(&mut self, frequency: f32, sample_rate: u32) {
        self.input_filter.configure(FilterConfiguration::LowPassQ(
            (f64::from(frequency) / f64::from(sample_rate)).min(0.49),
            FRAC_1_SQRT_2,
        ));
    }

    #[inline]
    #[allow(clippy::indexing_slicing)]
    fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let out = self.buffer[self.position];
        let damped = self.input_filter.process(out);

        for ((input, damped), buffer) in input
            .iter()
            .zip(&damped)
            .zip(&mut self.buffer[self.position])
        {
            *buffer = *input + *damped * self.feedback;
        }

        self.position = (self.position + 1) % self.buffer.len();

        out
    }
}

struct SchroederAllPass<const CHANNELS: usize> {
    buffer: Vec<[f32; CHANNELS]>,
    position: usize,
}

impl<const CHANNELS: usize> SchroederAllPass<CHANNELS> {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![[0.0; CHANNELS]; size.max(1)],
            position: 0,
        }
    }

    #[inline]
    #[allow(clippy::indexing_slicing)]
    fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let mut delayed = self.buffer[self.position];

        for ((input, delayed), buffer) in input
            .iter()
            .zip(&delayed)
            .zip(&mut self.buffer[self.position])
        {
            *buffer = *input + *delayed * 0.5;
        }

        self.position = (self.position + 1) % self.buffer.len();

        for (input, delayed) in input.iter().zip(&mut delayed) {
            *delayed -= *input;
        }

        delayed
    }
}

const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;

const DAMP_HZ_BRIGHT: f32 = 12_000.0;
const DAMP_HZ_DARK: f32 = 2_000.0;

pub struct Reverb<const CHANNELS: usize> {
    sample_rate: u32,

    combs: Vec<Comb<CHANNELS>>,
    allpass: Vec<SchroederAllPass<CHANNELS>>,
    input_filter: BiquadFilter<CHANNELS>,

    wet1: f32,
    wet2: f32,

    dry: f32,
}

impl<const CHANNELS: usize> Reverb<CHANNELS> {
    pub fn new(sample_rate: u32) -> Self {
        let mut new = Self {
            sample_rate,

            combs: COMB_TUNING
                .iter()
                .map(|&n| Comb::new(n + STEREO_SPREAD))
                .collect(),
            allpass: ALLPASS_TUNING
                .iter()
                .map(|&n| SchroederAllPass::new(n))
                .collect(),
            input_filter: BiquadFilter::default(),

            wet1: 0.0,
            wet2: 0.0,

            dry: 0.0,
        };

        new.set_params(0.5, 0.5, 0.3, 1.0, 0.7);

        new
    }

    pub fn set_params(&mut self, room_size: f32, damping: f32, width: f32, wet: f32, dry: f32) {
        let feedback = room_size * 0.28 + 0.7;

        let damp_hz = DAMP_HZ_BRIGHT * (DAMP_HZ_DARK / DAMP_HZ_BRIGHT).powf(damping);

        for comb in &mut self.combs {
            comb.set_feedback(feedback);
            comb.set_damp_hz(damp_hz, self.sample_rate);
        }

        let wet = wet * 3.0;

        self.wet1 = wet * (width / 2.0 + 0.5);
        self.wet2 = wet * ((1.0 - width) / 2.0);

        self.dry = dry;
    }

    pub fn set_low_cut(&mut self, frequency: f32) {
        self.input_filter.configure(FilterConfiguration::LowPassQ(
            (f64::from(frequency) / f64::from(self.sample_rate)).min(0.49),
            FRAC_1_SQRT_2,
        ));
    }

    pub fn set_high_cut(&mut self, frequency: f32) {
        self.input_filter.configure(FilterConfiguration::HighPassQ(
            (f64::from(frequency) / f64::from(self.sample_rate)).min(0.49),
            FRAC_1_SQRT_2,
        ));
    }

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let input = self.input_filter.process(input);

        let mut out = [0.0; CHANNELS];

        for comb in &mut self.combs {
            let combed = comb.process(input);

            for (combed, out) in combed.iter().zip(&mut out) {
                *out += *combed;
            }
        }

        for allpass in &mut self.allpass {
            out = allpass.process(out);
        }

        let mut out_reverse = out;
        out_reverse.reverse();

        for ((out, out_reverse), input) in out.iter_mut().zip(&out_reverse).zip(&input) {
            *out = *out * self.wet1 + *out_reverse * self.wet2 + *input * self.dry;
        }

        out
    }
}
