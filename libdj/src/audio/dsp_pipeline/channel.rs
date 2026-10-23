use std::f32;

use libdsp::audio_loader::TrackAudioData;

use crate::{
    AUDIO_CHANNELS, MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH, MIXER_ISOLATOR_LOW_MID_TRANSITION,
    MIXER_ISOLATOR_MID_HIGH_TRANSITION,
    audio::dsp_pipeline::{
        filters::{fx::ChannelFX, isolator::Isolator},
        player::PlayerDSP,
    },
    math::fader::SingleFader,
    playback::channel::ChannelUpdateResults,
    types::deck::{ChannelFXEffect, ChannelState},
};

pub struct ChannelDSP {
    _target_sample_rate: u32,

    player: PlayerDSP,

    pub amplitude_history: [[f32; MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH]; AUDIO_CHANNELS],

    gain: f32,

    local_master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    master_isolator: Isolator<AUDIO_CHANNELS>,
    master_channel_fx: ChannelFX<AUDIO_CHANNELS>,

    local_touch_cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_isolator: Isolator<AUDIO_CHANNELS>,
    touch_cue_channel_fx: ChannelFX<AUDIO_CHANNELS>,
}

impl ChannelDSP {
    pub fn new(sample_rate: u32) -> ChannelDSP {
        ChannelDSP {
            _target_sample_rate: sample_rate,

            player: PlayerDSP::new(sample_rate),

            amplitude_history: [[0.0; MIXER_AMPLITUDE_MEASUREMENT_HISTORY_LENGTH]; AUDIO_CHANNELS],

            gain: 0.0,

            local_master_output_buffers: Default::default(),
            master_isolator: Isolator::new(
                MIXER_ISOLATOR_LOW_MID_TRANSITION / f64::from(sample_rate),
                MIXER_ISOLATOR_MID_HIGH_TRANSITION / f64::from(sample_rate),
            ),
            master_channel_fx: ChannelFX::new(sample_rate),

            local_touch_cue_output_buffers: Default::default(),
            touch_cue_isolator: Isolator::new(
                MIXER_ISOLATOR_LOW_MID_TRANSITION / f64::from(sample_rate),
                MIXER_ISOLATOR_MID_HIGH_TRANSITION / f64::from(sample_rate),
            ),
            touch_cue_channel_fx: ChannelFX::new(sample_rate),
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
        channel_fx_effect: ChannelFXEffect,
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

        self.master_isolator.set_gains(
            channel_state.eq_low + 1.0,
            channel_state.eq_mid + 1.0,
            channel_state.eq_high + 1.0,
        );

        self.master_channel_fx.set_effect(channel_fx_effect);
        self.master_channel_fx.set_parameter(channel_state.fx);

        self.touch_cue_isolator.set_gains(
            channel_state.eq_low + 1.0,
            channel_state.eq_mid + 1.0,
            channel_state.eq_high + 1.0,
        );

        self.touch_cue_channel_fx.set_effect(channel_fx_effect);
        self.touch_cue_channel_fx.set_parameter(channel_state.fx);

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

            let effected = self.master_channel_fx.process_pre_eq(root_sample);

            let filtered = self.master_isolator.process(effected);

            let [left_effected, right_effected] =
                self.master_channel_fx.process_pre_fader(filtered);

            self.amplitude_history
                .get_mut(0)
                .unwrap()
                .shift_left([left_effected]);
            self.amplitude_history
                .get_mut(1)
                .unwrap()
                .shift_left([right_effected]);

            *local_left = left_effected;
            *local_right = right_effected;

            let faded = [
                left_effected.fade(channel_state.fade),
                right_effected.fade(channel_state.fade),
            ];

            let [left_effected, right_effected] = self.master_channel_fx.process_pre_mixer(faded);

            *master_left = left_effected;
            *master_right = right_effected;
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

            let effected = self.touch_cue_channel_fx.process_pre_eq(root_sample);

            let filtered = self.touch_cue_isolator.process(effected);

            let [left_effected, right_effected] =
                self.touch_cue_channel_fx.process_pre_fader(filtered);

            *cue_left = left_effected + if channel_state.cue { *master_left } else { 0.0 };
            *cue_right = right_effected
                + if channel_state.cue {
                    *master_right
                } else {
                    0.0
                };
        }
    }

    pub fn reset(&mut self) {
        self.master_isolator.reset();
        self.master_channel_fx.reset();

        self.touch_cue_isolator.reset();
        self.touch_cue_channel_fx.reset();
    }
}
