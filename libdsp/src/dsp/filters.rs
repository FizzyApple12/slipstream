use std::f64::consts::PI;

use num::pow::Pow;

#[derive(Clone, Copy)]
pub enum FilterConfiguration {
    Disabled,
    LowPass(f64, f64),
    LowPassQ(f64, f64),
    HighPass(f64, f64),
    HighPassQ(f64, f64),
    ConstantSkirtGainBandPass(f64, f64),
    ConstantSkirtGainBandPassQ(f64, f64),
    ConstantPeakGainBandPass(f64, f64),
    ConstantPeakGainBandPassQ(f64, f64),
    Notch(f64, f64),
    NotchQ(f64, f64),
    AllPass(f64, f64),
    AllPassQ(f64, f64),
    Peak(f64, f64, f64),
    PeakDB(f64, f64, f64),
    PeakQ(f64, f64, f64),
    PeakDBQ(f64, f64, f64),
    LowShelf(f64, f64, f64),
    LowShelfDB(f64, f64, f64),
    LowShelfQ(f64, f64, f64),
    LowShelfDBQ(f64, f64, f64),
    HighShelf(f64, f64, f64),
    HighShelfDB(f64, f64, f64),
    HighShelfQ(f64, f64, f64),
    HighShelfDBQ(f64, f64, f64),
}

pub struct BiquadFilter<const CHANNELS: usize> {
    a0: f32,
    a1: f32,
    a2: f32,

    b0: f32,
    b1: f32,
    b2: f32,

    x1: [f32; CHANNELS],
    x2: [f32; CHANNELS],
    y1: [f32; CHANNELS],
    y2: [f32; CHANNELS],
}

impl<const CHANNELS: usize> Default for BiquadFilter<CHANNELS> {
    fn default() -> Self {
        Self {
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,

            x1: [0.0; CHANNELS],
            x2: [0.0; CHANNELS],

            y1: [0.0; CHANNELS],
            y2: [0.0; CHANNELS],
        }
    }
}

impl<const CHANNELS: usize> BiquadFilter<CHANNELS> {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::manual_midpoint
    )]
    pub fn configure(&mut self, config: FilterConfiguration) {
        match config {
            FilterConfiguration::Disabled => {
                self.b0 = 1.0;
                self.b1 = 0.0;
                self.b2 = 0.0;
                self.a0 = 1.0;
                self.a1 = 0.0;
                self.a2 = 0.0;
            }
            FilterConfiguration::LowPass(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.b1 = (1.0 - omega_zero.cos()) as f32;
                self.b2 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::LowPassQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.b1 = (1.0 - omega_zero.cos()) as f32;
                self.b2 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::HighPass(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.b1 = -(1.0 + omega_zero.cos()) as f32;
                self.b2 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::HighPassQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.b1 = -(1.0 + omega_zero.cos()) as f32;
                self.b2 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantSkirtGainBandPass(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = (omega_zero.sin() / 2.0) as f32;
                self.b1 = 0.0;
                self.b2 = -(omega_zero.sin() / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantSkirtGainBandPassQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = (omega_zero.sin() / 2.0) as f32;
                self.b1 = 0.0;
                self.b2 = -(omega_zero.sin() / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantPeakGainBandPass(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = alpha as f32;
                self.b1 = 0.0;
                self.b2 = -alpha as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantPeakGainBandPassQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = alpha as f32;
                self.b1 = 0.0;
                self.b2 = -alpha as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::Notch(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = 1.0;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = 1.0;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::NotchQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = 1.0;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = 1.0;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::AllPass(scaled_frequency, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = (1.0 - alpha) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 + alpha) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::AllPassQ(scaled_frequency, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = (1.0 - alpha) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 + alpha) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::Peak(scaled_frequency, gain, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakDB(scaled_frequency, db, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakQ(scaled_frequency, gain, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakDBQ(scaled_frequency, db, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::LowShelf(scaled_frequency, gain, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = (gain
                    * ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0)
                        - ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0)
                    + ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::LowShelfDB(scaled_frequency, db, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (gain
                    * ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0)
                        - ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0)
                    + ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::LowShelfQ(scaled_frequency, gain, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = (gain
                    * ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0)
                        - ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0)
                    + ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::LowShelfDBQ(scaled_frequency, db, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (gain
                    * ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0)
                        - ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0)
                    + ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::HighShelf(scaled_frequency, gain, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                self.b0 = (gain
                    * ((gain + 1.0)
                        + ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0)
                    - ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::HighShelfDB(scaled_frequency, db, bandwidth) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_bandwidth(omega_zero, bandwidth);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (gain
                    * ((gain + 1.0)
                        + ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0)
                    - ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::HighShelfQ(scaled_frequency, gain, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                self.b0 = (gain
                    * ((gain + 1.0)
                        + ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0)
                    - ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
            FilterConfiguration::HighShelfDBQ(scaled_frequency, db, q_factor) => {
                let omega_zero = calculate_omega_zero(scaled_frequency);

                let alpha = calculate_alpha_with_q_factor(omega_zero, q_factor);

                let gain = calculate_sqrt_gain(db);

                self.b0 = (gain
                    * ((gain + 1.0)
                        + ((gain - 1.0) * omega_zero.cos())
                        + (2.0 * gain.sqrt() * alpha))) as f32;
                self.b1 = (2.0 * gain * ((gain - 1.0) - ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.b2 = (gain
                    * ((gain + 1.0) + ((gain - 1.0) * omega_zero.cos())
                        - (2.0 * gain.sqrt() * alpha))) as f32;
                self.a0 = ((gain + 1.0) - ((gain - 1.0) * omega_zero.cos())
                    + (2.0 * gain.sqrt() * alpha)) as f32;
                self.a1 = (-2.0 * ((gain - 1.0) + ((gain + 1.0) * omega_zero.cos()))) as f32;
                self.a2 = ((gain + 1.0)
                    - ((gain - 1.0) * omega_zero.cos())
                    - (2.0 * gain.sqrt() * alpha)) as f32;
            }
        }

        self.b0 /= self.a0;
        self.b1 /= self.a0;
        self.b2 /= self.a0;
        self.a1 /= self.a0;
        self.a2 /= self.a0;
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn process(&mut self, mut input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        for ((((input, x1), x2), y1), y2) in input
            .iter_mut()
            .zip(&mut self.x1)
            .zip(&mut self.x2)
            .zip(&mut self.y1)
            .zip(&mut self.y2)
        {
            let y0 = (self.b0 * *input) + (self.b1 * *x1) + (self.b2 * *x2)
                - (self.a1 * *y1)
                - (self.a2 * *y2);

            *y2 = *y1;
            *y1 = y0;
            *x2 = *x1;
            *x1 = *input;

            *input = y0;
        }

        input
    }

    pub fn reset(&mut self) {
        self.x1 = [0.0; CHANNELS];
        self.x2 = [0.0; CHANNELS];
        self.y1 = [0.0; CHANNELS];
        self.y2 = [0.0; CHANNELS];
    }
}

fn calculate_sqrt_gain(db_gain: f64) -> f64 {
    10.0f64.pow(db_gain / 40.0)
}

fn calculate_omega_zero(scaled_frequency: f64) -> f64 {
    2.0 * PI * scaled_frequency
}

fn calculate_alpha_with_q_factor(omega_zero: f64, q: f64) -> f64 {
    omega_zero.sin() / (2.0 * q)
}

fn calculate_alpha_with_bandwidth(omega_zero: f64, bandwidth: f64) -> f64 {
    omega_zero.sin() * ((2.0f64.log2() / 2.0) * bandwidth * (omega_zero / omega_zero.sin())).sinh()
}
