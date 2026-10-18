use std::f32;

use libdsp::{
    amplitude_to_db,
    audio_loader::TrackAudioData,
    dsp::filters::{BiquadFilter, FilterConfiguration},
};

use crate::{
    AUDIO_CHANNELS, MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH, MIXER_EQ_HIGH_CUTOFF,
    MIXER_EQ_HIGH_OCTAVES, MIXER_EQ_LOW_CUTOFF, MIXER_EQ_LOW_OCTAVES, MIXER_EQ_MID_CENTER,
    MIXER_EQ_MID_Q, MIXER_FILTER_HIGH_PASS_OCTAVES, MIXER_FILTER_LOW_PASS_OCTAVES,
    audio::dsp_pipeline::player::PlayerDSP, math::fader::SingleFader,
    playback::channel::ChannelUpdateResults, types::deck::ChannelState,
};

const BASICALLY_INFINITY: f32 = 26.0;

// i disagree!
#[allow(clippy::type_complexity)]
pub struct ChannelDSP {
    target_sample_rate: u32,

    player: PlayerDSP,

    pub amplitude_history: [[f32; MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH]; AUDIO_CHANNELS],

    gain: f32,
    eq_low_coefficient: f32,
    eq_mid_coefficient: f32,
    eq_high_coefficient: f32,
    filter_coefficient: f32,

    local_master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    master_eq_filters: (
        BiquadFilter<AUDIO_CHANNELS>,
        BiquadFilter<AUDIO_CHANNELS>,
        BiquadFilter<AUDIO_CHANNELS>,
    ),
    master_filter: (BiquadFilter<AUDIO_CHANNELS>, BiquadFilter<AUDIO_CHANNELS>),

    local_touch_cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_eq_filters: (
        BiquadFilter<AUDIO_CHANNELS>,
        BiquadFilter<AUDIO_CHANNELS>,
        BiquadFilter<AUDIO_CHANNELS>,
    ),
    touch_cue_filter: (BiquadFilter<AUDIO_CHANNELS>, BiquadFilter<AUDIO_CHANNELS>),
}

impl ChannelDSP {
    pub fn new(sample_rate: u32) -> ChannelDSP {
        ChannelDSP {
            target_sample_rate: sample_rate,

            player: PlayerDSP::new(sample_rate),

            amplitude_history: [[0.0; MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH]; AUDIO_CHANNELS],

            gain: 0.0,
            eq_low_coefficient: 0.0,
            eq_mid_coefficient: 0.0,
            eq_high_coefficient: 0.0,
            filter_coefficient: 0.0,

            local_master_output_buffers: Default::default(),
            master_eq_filters: (
                BiquadFilter::default(),
                BiquadFilter::default(),
                BiquadFilter::default(),
            ),
            master_filter: (BiquadFilter::default(), BiquadFilter::default()),

            local_touch_cue_output_buffers: Default::default(),
            touch_cue_eq_filters: (
                BiquadFilter::default(),
                BiquadFilter::default(),
                BiquadFilter::default(),
            ),
            touch_cue_filter: (BiquadFilter::default(), BiquadFilter::default()),
        }
    }

    pub fn assign_track_data(&mut self, track_data: Option<Box<TrackAudioData>>) {
        self.reset();

        self.player.assign_track_data(track_data);
    }

    // the definitions of these types make this safe and we need max speed
    #[allow(clippy::indexing_slicing)]
    pub fn generate_samples(
        &mut self,
        channel_state: &ChannelState,
        channel_update_results: &ChannelUpdateResults,
        master_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        cue_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        for master_output_buffer in master_output_buffers.iter_mut() {
            if master_output_buffer.len() < number_samples {
                master_output_buffer.resize(number_samples, 0.0);
            }
        }
        for cue_output_buffer in cue_output_buffers.iter_mut() {
            if cue_output_buffer.len() < number_samples {
                cue_output_buffer.resize(number_samples, 0.0);
            }
        }

        self.player.generate_samples(
            &channel_state.player,
            &channel_update_results.player,
            &mut self.local_master_output_buffers,
            &mut self.local_touch_cue_output_buffers,
            number_samples,
        );

        self.gain = channel_state.gain;
        self.eq_low_coefficient = channel_state.eq_low;
        self.eq_mid_coefficient = channel_state.eq_mid;
        self.eq_high_coefficient = channel_state.eq_high;
        self.filter_coefficient = channel_state.fx;

        self.update_filters();

        let [local_master_left, local_master_right] = &mut self.local_master_output_buffers;
        let [master_left, master_right] = master_output_buffers;

        for (((local_left, local_right), master_left), master_right) in local_master_left
            [0..number_samples]
            .iter_mut()
            .zip(local_master_right[0..number_samples].iter_mut())
            .zip(master_left[0..number_samples].iter_mut())
            .zip(master_right[0..number_samples].iter_mut())
        {
            let root_sample = [*local_left * self.gain, *local_right * self.gain];

            let low_filtered = self.master_eq_filters.0.process(root_sample);
            let mid_filtered = self.master_eq_filters.1.process(low_filtered);
            let high_filtered = self.master_eq_filters.2.process(mid_filtered);

            let filter_low_filtered = self.master_filter.0.process(high_filtered);
            let filter_high_filtered = self.master_filter.1.process(high_filtered);

            let [left_filtered, right_filtered] = if channel_state.fx < -f32::EPSILON {
                filter_low_filtered
            } else if channel_state.fx > f32::EPSILON {
                filter_high_filtered
            } else {
                high_filtered
            };

            self.amplitude_history
                .get_mut(0)
                .unwrap()
                .shift_left([left_filtered]);
            self.amplitude_history
                .get_mut(1)
                .unwrap()
                .shift_left([right_filtered]);

            *local_left = left_filtered;
            *local_right = right_filtered;

            *master_left = left_filtered.fade(channel_state.fade);
            *master_right = right_filtered.fade(channel_state.fade);
        }

        let [touch_cue_left, touch_cue_right] = &mut self.local_touch_cue_output_buffers;
        let [cue_left, cue_right] = cue_output_buffers;

        for (((((cue_left, cue_right), local_left), local_right), master_left), master_right) in
            cue_left[0..number_samples]
                .iter_mut()
                .zip(cue_right[0..number_samples].iter_mut())
                .zip(touch_cue_left[0..number_samples].iter_mut())
                .zip(touch_cue_right[0..number_samples].iter_mut())
                .zip(local_master_left[0..number_samples].iter_mut())
                .zip(local_master_right[0..number_samples].iter_mut())
        {
            let root_sample = [*local_left * self.gain, *local_right * self.gain];

            let low_filtered = self.touch_cue_eq_filters.0.process(root_sample);
            let mid_filtered = self.touch_cue_eq_filters.1.process(low_filtered);
            let high_filtered = self.touch_cue_eq_filters.2.process(mid_filtered);

            let filter_low_filtered = self.touch_cue_filter.0.process(high_filtered);
            let filter_high_filtered = self.touch_cue_filter.1.process(high_filtered);

            let [left_filtered, right_filtered] = if channel_state.fx < -f32::EPSILON {
                filter_low_filtered
            } else if channel_state.fx > f32::EPSILON {
                filter_high_filtered
            } else {
                high_filtered
            };

            *cue_left = left_filtered + if channel_state.cue { *master_left } else { 0.0 };
            *cue_right = right_filtered
                + if channel_state.cue {
                    *master_right
                } else {
                    0.0
                };
        }
    }

    pub fn reset(&mut self) {
        self.master_eq_filters = (
            BiquadFilter::default(),
            BiquadFilter::default(),
            BiquadFilter::default(),
        );
        self.master_filter = (BiquadFilter::default(), BiquadFilter::default());

        self.touch_cue_eq_filters = (
            BiquadFilter::default(),
            BiquadFilter::default(),
            BiquadFilter::default(),
        );
        self.touch_cue_filter = (BiquadFilter::default(), BiquadFilter::default());
    }

    #[allow(clippy::indexing_slicing)]
    fn update_filters(&mut self) {
        self.master_eq_filters
            .0
            .configure(FilterConfiguration::LowShelfDB(
                MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_low_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_LOW_OCTAVES,
            ));
        self.master_eq_filters
            .1
            .configure(FilterConfiguration::PeakDBQ(
                MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_mid_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_MID_Q,
            ));
        self.master_eq_filters
            .2
            .configure(FilterConfiguration::HighShelfDB(
                MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_high_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_HIGH_OCTAVES,
            ));

        // self.master_eq_filters
        //     .0
        //     .configure(FilterConfiguration::LowShelf(
        //         MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_low_coefficient + 1.0),
        //         MIXER_EQ_LOW_OCTAVES,
        //     ));
        // self.master_eq_filters
        //     .1
        //     .configure(FilterConfiguration::PeakQ(
        //         MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_mid_coefficient + 1.0),
        //         MIXER_EQ_MID_Q,
        //     ));
        // self.master_eq_filters
        //     .2
        //     .configure(FilterConfiguration::HighShelf(
        //         MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_high_coefficient + 1.0),
        //         MIXER_EQ_HIGH_OCTAVES,
        //     ));

        self.touch_cue_eq_filters
            .0
            .configure(FilterConfiguration::LowShelfDB(
                MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_low_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_LOW_OCTAVES,
            ));
        self.touch_cue_eq_filters
            .1
            .configure(FilterConfiguration::PeakDBQ(
                MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_mid_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_MID_Q,
            ));
        self.touch_cue_eq_filters
            .2
            .configure(FilterConfiguration::HighShelfDB(
                MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(amplitude_to_db(self.eq_high_coefficient + 1.0).max(-BASICALLY_INFINITY)),
                MIXER_EQ_HIGH_OCTAVES,
            ));

        // self.touch_cue_eq_filters
        //     .0
        //     .configure(FilterConfiguration::LowShelf(
        //         MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_low_coefficient + 1.0),
        //         MIXER_EQ_LOW_OCTAVES,
        //     ));
        // self.touch_cue_eq_filters
        //     .1
        //     .configure(FilterConfiguration::PeakQ(
        //         MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_mid_coefficient + 1.0),
        //         MIXER_EQ_MID_Q,
        //     ));
        // self.touch_cue_eq_filters
        //     .2
        //     .configure(FilterConfiguration::HighShelf(
        //         MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
        //         f64::from(self.eq_high_coefficient + 1.0),
        //         MIXER_EQ_HIGH_OCTAVES,
        //     ));

        self.master_filter.0.configure(FilterConfiguration::LowPass(
            (-f64::from(self.filter_coefficient).min(0.0) / f64::from(self.target_sample_rate))
                / f64::from(self.target_sample_rate),
            MIXER_FILTER_LOW_PASS_OCTAVES,
        ));
        self.master_filter
            .1
            .configure(FilterConfiguration::HighPass(
                (f64::from(self.filter_coefficient).max(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_HIGH_PASS_OCTAVES,
            ));

        self.touch_cue_filter
            .0
            .configure(FilterConfiguration::LowPass(
                (-f64::from(self.filter_coefficient).min(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_LOW_PASS_OCTAVES,
            ));
        self.touch_cue_filter
            .1
            .configure(FilterConfiguration::HighPass(
                (f64::from(self.filter_coefficient).max(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_HIGH_PASS_OCTAVES,
            ));
    }
}
