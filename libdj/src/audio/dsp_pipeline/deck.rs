use std::f32;

use libdsp::audio_loader::TrackAudioData;

use crate::{
    AUDIO_CHANNELS, MIXER_CHANNELS,
    audio::dsp_pipeline::{channel::ChannelDSP, filters::fx::MasterFX},
    math::fader::CrossFader,
    playback::deck::DeckUpdateResults,
    types::deck::{DeckState, MasterFXChannel},
};

pub struct DeckDSP {
    _target_sample_rate: u32,

    pub channels: Vec<ChannelDSP>,
    master_output_buffers: [[Vec<f32>; AUDIO_CHANNELS]; MIXER_CHANNELS],
    cue_output_buffers: [[Vec<f32>; AUDIO_CHANNELS]; MIXER_CHANNELS],

    master_fx: MasterFX<AUDIO_CHANNELS>,
}

impl DeckDSP {
    pub fn new(sample_rate: u32) -> DeckDSP {
        DeckDSP {
            _target_sample_rate: sample_rate,

            channels: vec![
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
            ],
            master_output_buffers: Default::default(),
            cue_output_buffers: Default::default(),

            master_fx: MasterFX::new(sample_rate),
        }
    }

    pub fn assign_track_data(&mut self, channel: usize, track_data: Option<Box<TrackAudioData>>) {
        let _ = self
            .channels
            .get_mut(channel)
            .map(|channel| channel.assign_track_data(track_data));
    }

    // the definitions of these types make this safe and we need max speed
    #[allow(clippy::indexing_slicing)]
    pub fn generate_samples(
        &mut self,
        deck_state: &DeckState,
        deck_update_results: &DeckUpdateResults,
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

        let channel_fx_effect = deck_state.channel_fx_effect;

        for (channel_number, channel) in self.channels.iter_mut().enumerate() {
            channel.generate_samples(
                &deck_state.mixer_channels[channel_number],
                &deck_update_results.channels[channel_number],
                channel_fx_effect,
                &mut self.master_output_buffers[channel_number],
                &mut self.cue_output_buffers[channel_number],
                number_samples,
            );
        }

        self.master_fx.set_effect(deck_state.master_fx.effect);
        self.master_fx.set_parameter(deck_state.master_fx.parameter);
        self.master_fx.set_depth(deck_state.master_fx.depth);

        'bpm_set_block: {
            match &deck_state.master_fx.channel {
                MasterFXChannel::Channel(channel) => {
                    if let Some(channel) = deck_state.mixer_channels.get(*channel)
                        && let Some(bpm) = channel.player.get_current_bpm()
                    {
                        self.master_fx.set_bpm(bpm);

                        break 'bpm_set_block;
                    }

                    if let Some(channel) = &deck_state.master_channel
                        && let Some(channel) = deck_state.mixer_channels.get(*channel)
                        && let Some(bpm) = channel.player.get_current_bpm()
                    {
                        self.master_fx.set_bpm(bpm);

                        break 'bpm_set_block;
                    }

                    for channel in &deck_state.mixer_channels {
                        if channel.fade > f32::EPSILON
                            && let Some(bpm) = channel.player.get_current_bpm()
                        {
                            self.master_fx.set_bpm(bpm);

                            break 'bpm_set_block;
                        }
                    }

                    self.master_fx.set_bpm(deck_state.master_fx.bpm);
                }
                MasterFXChannel::Master => {
                    if let Some(channel) = &deck_state.master_channel
                        && let Some(channel) = deck_state.mixer_channels.get(*channel)
                        && let Some(bpm) = channel.player.get_current_bpm()
                    {
                        self.master_fx.set_bpm(bpm);

                        break 'bpm_set_block;
                    }

                    for channel in &deck_state.mixer_channels {
                        if channel.fade > f32::EPSILON
                            && let Some(bpm) = channel.player.get_current_bpm()
                        {
                            self.master_fx.set_bpm(bpm);

                            break 'bpm_set_block;
                        }
                    }

                    self.master_fx.set_bpm(deck_state.master_fx.bpm);
                }
            }
        }

        // i cant wait to replace this :sob:
        let [local_channel_0_left, local_channel_0_right] = &self.master_output_buffers[0];
        let [local_channel_1_left, local_channel_1_right] = &self.master_output_buffers[1];
        let [local_channel_2_left, local_channel_2_right] = &self.master_output_buffers[2];
        let [local_channel_3_left, local_channel_3_right] = &self.master_output_buffers[3];
        let [master_left, master_right] = master_output_buffers;

        for (
            (
                (
                    ((master_left, master_right), (local_channel_3_left, local_channel_3_right)),
                    (local_channel_2_left, local_channel_2_right),
                ),
                (local_channel_1_left, local_channel_1_right),
            ),
            (local_channel_0_left, local_channel_0_right),
        ) in master_left[0..number_samples]
            .iter_mut()
            .zip(&mut master_right[0..number_samples])
            .zip(local_channel_0_left.iter().zip(local_channel_0_right))
            .zip(local_channel_1_left.iter().zip(local_channel_1_right))
            .zip(local_channel_2_left.iter().zip(local_channel_2_right))
            .zip(local_channel_3_left.iter().zip(local_channel_3_right))
        {
            let channel_0 = [
                local_channel_0_left.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[0].cross_fader_side,
                ),
                local_channel_0_right.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[0].cross_fader_side,
                ),
            ];
            let [channel_0_left, channel_0_right] =
                if deck_state.master_fx.channel == MasterFXChannel::Channel(0) {
                    self.master_fx.process(channel_0)
                } else {
                    channel_0
                };

            let channel_1 = [
                local_channel_1_left.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[1].cross_fader_side,
                ),
                local_channel_1_right.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[1].cross_fader_side,
                ),
            ];
            let [channel_1_left, channel_1_right] =
                if deck_state.master_fx.channel == MasterFXChannel::Channel(1) {
                    self.master_fx.process(channel_1)
                } else {
                    channel_1
                };

            let channel_2 = [
                local_channel_2_left.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[2].cross_fader_side,
                ),
                local_channel_2_right.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[2].cross_fader_side,
                ),
            ];
            let [channel_2_left, channel_2_right] =
                if deck_state.master_fx.channel == MasterFXChannel::Channel(2) {
                    self.master_fx.process(channel_2)
                } else {
                    channel_2
                };

            let channel_3 = [
                local_channel_3_left.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[3].cross_fader_side,
                ),
                local_channel_3_right.crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[3].cross_fader_side,
                ),
            ];
            let [channel_3_left, channel_3_right] =
                if deck_state.master_fx.channel == MasterFXChannel::Channel(3) {
                    self.master_fx.process(channel_3)
                } else {
                    channel_3
                };

            let master_out = [
                channel_0_left + channel_1_left + channel_2_left + channel_3_left,
                channel_0_right + channel_1_right + channel_2_right + channel_3_right,
            ];
            let [final_master_left, final_master_right] =
                if deck_state.master_fx.channel == MasterFXChannel::Master {
                    self.master_fx.process(master_out)
                } else {
                    master_out
                };

            *master_left = final_master_left;
            *master_right = final_master_right;
        }

        for (buffer_index, cue_output_buffer) in cue_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in
                cue_output_buffer[0..number_samples].iter_mut().enumerate()
            {
                *sample = self.cue_output_buffers[0][buffer_index][sample_number]
                    + self.cue_output_buffers[1][buffer_index][sample_number]
                    + self.cue_output_buffers[2][buffer_index][sample_number]
                    + self.cue_output_buffers[3][buffer_index][sample_number];
            }
        }
    }

    pub fn reset(&mut self) {
        for channel in &mut self.channels {
            channel.reset();
        }
    }
}
