// https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html

use std::{f64::consts::PI, marker::PhantomData};

use num::pow::Pow;

use crate::pipeline::nesting_inlining::{DataExtractor, PipelineNode};

pub enum FilterConfiguration<
    Data,
    ScaledFrequencyDataExtractor,
    BandwidthTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> {
    Disabled(PhantomData<Data>),
    LowPass(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    LowPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    HighPass(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    HighPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    ConstantSkirtGainBandPass(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    ConstantSkirtGainBandPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    ConstantPeakGainBandPass(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    ConstantPeakGainBandPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    Notch(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    NotchQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    AllPass(ScaledFrequencyDataExtractor, BandwidthTargetDataExtractor),
    AllPassQ(ScaledFrequencyDataExtractor, QFactorDataExtractor),
    Peak(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    PeakDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    PeakQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    PeakDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
    LowShelf(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    LowShelfDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    LowShelfQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    LowShelfDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
    HighShelf(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    HighShelfDB(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        BandwidthTargetDataExtractor,
    ),
    HighShelfQ(
        ScaledFrequencyDataExtractor,
        GainDataExtractor,
        QFactorDataExtractor,
    ),
    HighShelfDBQ(
        ScaledFrequencyDataExtractor,
        DecibelGainDataExtractor,
        QFactorDataExtractor,
    ),
}

pub struct BiquadFilter<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    BandwidthTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    BandwidthTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    phantom_data: PhantomData<Data>,
    parent: Parent,
    configuration: FilterConfiguration<
        Data,
        ScaledFrequencyDataExtractor,
        BandwidthTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >,

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

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    BandwidthTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
>
    BiquadFilter<
        CHANNELS,
        Data,
        Parent,
        ScaledFrequencyDataExtractor,
        BandwidthTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    BandwidthTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    pub fn new(
        parent: Parent,
        configuration: FilterConfiguration<
            Data,
            ScaledFrequencyDataExtractor,
            BandwidthTargetDataExtractor,
            QFactorDataExtractor,
            GainDataExtractor,
            DecibelGainDataExtractor,
        >,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            parent,
            configuration,

            a0: 0.0,
            a1: 0.0,
            a2: 0.0,

            b0: 0.0,
            b1: 0.0,
            b2: 0.0,

            x1: [0.0; CHANNELS],
            x2: [0.0; CHANNELS],
            y1: [0.0; CHANNELS],
            y2: [0.0; CHANNELS],
        }
    }
}

impl<
    const CHANNELS: usize,
    Data,
    Parent,
    ScaledFrequencyDataExtractor,
    BandwidthTargetDataExtractor,
    QFactorDataExtractor,
    GainDataExtractor,
    DecibelGainDataExtractor,
> PipelineNode<CHANNELS, Data>
    for BiquadFilter<
        CHANNELS,
        Data,
        Parent,
        ScaledFrequencyDataExtractor,
        BandwidthTargetDataExtractor,
        QFactorDataExtractor,
        GainDataExtractor,
        DecibelGainDataExtractor,
    >
where
    Parent: PipelineNode<CHANNELS, Data>,
    ScaledFrequencyDataExtractor: DataExtractor<Data, f64>,
    BandwidthTargetDataExtractor: DataExtractor<Data, f64>,
    QFactorDataExtractor: DataExtractor<Data, f64>,
    GainDataExtractor: DataExtractor<Data, f64>,
    DecibelGainDataExtractor: DataExtractor<Data, f64>,
{
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::manual_midpoint
    )]
    fn update(&mut self, data: &Data) {
        match &self.configuration {
            FilterConfiguration::Disabled(_) => {
                self.b0 = 1.0;
                self.b1 = 0.0;
                self.b2 = 0.0;
                self.a0 = 1.0;
                self.a1 = 0.0;
                self.a2 = 0.0;
            }
            FilterConfiguration::LowPass(scaled_frequency_extractor, bandwidth_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.b1 = (1.0 - omega_zero.cos()) as f32;
                self.b2 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::LowPassQ(scaled_frequency_extractor, q_factor_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.b1 = (1.0 - omega_zero.cos()) as f32;
                self.b2 = ((1.0 - omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::HighPass(scaled_frequency_extractor, bandwidth_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.b1 = -(1.0 + omega_zero.cos()) as f32;
                self.b2 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::HighPassQ(scaled_frequency_extractor, q_factor_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.b1 = -(1.0 + omega_zero.cos()) as f32;
                self.b2 = ((1.0 + omega_zero.cos()) / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantSkirtGainBandPass(
                scaled_frequency_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = (omega_zero.sin() / 2.0) as f32;
                self.b1 = 0.0;
                self.b2 = -(omega_zero.sin() / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantSkirtGainBandPassQ(
                scaled_frequency_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = (omega_zero.sin() / 2.0) as f32;
                self.b1 = 0.0;
                self.b2 = -(omega_zero.sin() / 2.0) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantPeakGainBandPass(
                scaled_frequency_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = alpha as f32;
                self.b1 = 0.0;
                self.b2 = -alpha as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::ConstantPeakGainBandPassQ(
                scaled_frequency_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = alpha as f32;
                self.b1 = 0.0;
                self.b2 = -alpha as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::Notch(scaled_frequency_extractor, bandwidth_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = 1.0;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = 1.0;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::NotchQ(scaled_frequency_extractor, q_factor_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = 1.0;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = 1.0;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::AllPass(scaled_frequency_extractor, bandwidth_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                self.b0 = (1.0 - alpha) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 + alpha) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::AllPassQ(scaled_frequency_extractor, q_factor_extractor) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                self.b0 = (1.0 - alpha) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 + alpha) as f32;
                self.a0 = (1.0 + alpha) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - alpha) as f32;
            }
            FilterConfiguration::Peak(
                scaled_frequency_extractor,
                gain_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = (gain_extractor)(data);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakDB(
                scaled_frequency_extractor,
                db_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakQ(
                scaled_frequency_extractor,
                gain_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = (gain_extractor)(data);

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::PeakDBQ(
                scaled_frequency_extractor,
                db_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

                self.b0 = (1.0 + (alpha * gain)) as f32;
                self.b1 = (-2.0 * omega_zero.cos()) as f32;
                self.b2 = (1.0 - (alpha * gain)) as f32;
                self.a0 = (1.0 + (alpha / gain)) as f32;
                self.a1 = (-2.0 * omega_zero.cos()) as f32;
                self.a2 = (1.0 - (alpha / gain)) as f32;
            }
            FilterConfiguration::LowShelf(
                scaled_frequency_extractor,
                gain_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = (gain_extractor)(data);

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
            FilterConfiguration::LowShelfDB(
                scaled_frequency_extractor,
                db_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

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
            FilterConfiguration::LowShelfQ(
                scaled_frequency_extractor,
                gain_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = (gain_extractor)(data);

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
            FilterConfiguration::LowShelfDBQ(
                scaled_frequency_extractor,
                db_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

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
            FilterConfiguration::HighShelf(
                scaled_frequency_extractor,
                gain_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = (gain_extractor)(data);

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
            FilterConfiguration::HighShelfDB(
                scaled_frequency_extractor,
                db_extractor,
                bandwidth_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_bandwidth(omega_zero, (bandwidth_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

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
            FilterConfiguration::HighShelfQ(
                scaled_frequency_extractor,
                gain_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = (gain_extractor)(data);

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
            FilterConfiguration::HighShelfDBQ(
                scaled_frequency_extractor,
                db_extractor,
                q_factor_extractor,
            ) => {
                let omega_zero = calculate_omega_zero((scaled_frequency_extractor)(data));

                let alpha = calculate_alpha_with_q_factor(omega_zero, (q_factor_extractor)(data));

                let gain = calculate_sqrt_gain((db_extractor)(data));

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
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self, clock: usize) -> [f32; CHANNELS] {
        let mut parent_data = self.parent.next(clock);

        for ((((input, x1), x2), y1), y2) in parent_data
            .iter_mut()
            .zip(&mut self.x1)
            .zip(&mut self.x2)
            .zip(&mut self.y1)
            .zip(&mut self.y2)
        {
            let y0 = ((self.b0 / self.a0) * *input)
                + ((self.b1 / self.a0) * *x1)
                + ((self.b2 / self.a0) * *x2)
                - ((self.a1 / self.a0) * *y1)
                - ((self.a2 / self.a0) * *y2);

            *y2 = *y1;
            *y1 = y0;
            *x2 = *x1;
            *x1 = *input;

            *input = y0;
        }

        parent_data
    }

    fn reset(&mut self) {
        self.x1 = [0.0; CHANNELS];
        self.x2 = [0.0; CHANNELS];
        self.y1 = [0.0; CHANNELS];
        self.y2 = [0.0; CHANNELS];

        self.parent.reset();
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
