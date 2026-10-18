use std::f64::consts::FRAC_1_SQRT_2;

use libdsp::dsp::filters::{BiquadFilter, FilterConfiguration};

struct Cascade<const CHANNELS: usize> {
    first_stage: BiquadFilter<CHANNELS>,
    second_stage: BiquadFilter<CHANNELS>,
}

impl<const CHANNELS: usize> Cascade<CHANNELS> {
    fn new(config: FilterConfiguration) -> Self {
        let mut first_stage = BiquadFilter::default();
        let mut second_stage = BiquadFilter::default();

        first_stage.configure(config);
        second_stage.configure(config);

        Self {
            first_stage,
            second_stage,
        }
    }

    #[inline]
    fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        self.second_stage.process(self.first_stage.process(input))
    }

    fn reset(&mut self) {
        self.first_stage.reset();
        self.second_stage.reset();
    }
}

pub struct Isolator<const CHANNELS: usize> {
    lp1: Cascade<CHANNELS>,
    hp1: Cascade<CHANNELS>,
    lp2: Cascade<CHANNELS>,
    hp2: Cascade<CHANNELS>,
    ap2: Cascade<CHANNELS>,

    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,
}

impl<const CHANNELS: usize> Isolator<CHANNELS> {
    pub fn new(low_mid_transition: f64, mid_high_transition: f64) -> Self {
        let q = FRAC_1_SQRT_2;

        Self {
            lp1: Cascade::new(FilterConfiguration::LowPassQ(low_mid_transition, q)),
            hp1: Cascade::new(FilterConfiguration::HighPassQ(low_mid_transition, q)),
            lp2: Cascade::new(FilterConfiguration::LowPassQ(mid_high_transition, q)),
            hp2: Cascade::new(FilterConfiguration::HighPassQ(mid_high_transition, q)),
            ap2: Cascade::new(FilterConfiguration::AllPassQ(mid_high_transition, q)),

            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
        }
    }

    pub fn set_gains(&mut self, low: f32, mid: f32, high: f32) {
        self.low_gain = low;
        self.mid_gain = mid;
        self.high_gain = high;
    }

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let low = self.ap2.process(self.lp1.process(input));
        let rest = self.hp1.process(input);
        let mid = self.lp2.process(rest);
        let high = self.hp2.process(rest);

        let mut out = [0.0f32; CHANNELS];

        for (((out, low), mid), high) in out.iter_mut().zip(low).zip(mid).zip(high) {
            *out = low * self.low_gain + mid * self.mid_gain + high * self.high_gain;
        }

        out
    }

    pub fn reset(&mut self) {
        self.lp1.reset();
        self.hp1.reset();
        self.lp2.reset();
        self.hp2.reset();
        self.ap2.reset();
    }
}
