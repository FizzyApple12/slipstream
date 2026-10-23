use timecode::{Duration, Timecode};

use crate::{
    MIN_LOOP_SIZE_NANOSECONDS,
    types::deck::{
        BeatLoopAdjustMode, BeatSyncMode, ChannelFXEffect, CrossFaderSide, DeckState,
        MasterFXChannel, MasterFXEffect, PlayDirection, PlayState, TempoRange,
    },
};

pub fn mixer_master_cue_press(deck_state: &mut DeckState) {
    deck_state.master_cue = !deck_state.master_cue;
}

pub fn mixer_master_fx_target_master_press(deck_state: &mut DeckState) {
    deck_state.master_fx.channel = MasterFXChannel::Master;
}

pub fn mixer_master_fx_enable_press(deck_state: &mut DeckState) {
    deck_state.master_fx.enabled = !deck_state.master_fx.enabled;
}

pub fn mixer_channel_cue_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cue = !channel.cue;
    }
}

pub fn mixer_master_fx_effect_press(deck_state: &mut DeckState, effect: usize) {
    deck_state.master_fx.effect = match effect {
        0 => MasterFXEffect::LowCutEcho,
        1 => MasterFXEffect::Echo,
        2 => MasterFXEffect::MultiTapDelay,
        3 => MasterFXEffect::Spiral,
        4 => MasterFXEffect::Reverb,
        5 => MasterFXEffect::Transgate,
        6 => MasterFXEffect::EnigmaJet,
        7 => MasterFXEffect::Flanger,
        8 => MasterFXEffect::Phaser,
        9 => MasterFXEffect::Stretch,
        10 => MasterFXEffect::SlipRoll,
        11 => MasterFXEffect::Roll,
        _ => {
            return;
        }
    }
}

pub fn mixer_master_fx_target_channel_press(deck_state: &mut DeckState, channel: usize) {
    if deck_state.mixer_channels.get(channel).is_some() {
        deck_state.master_fx.channel = MasterFXChannel::Channel(channel);
    }
}

pub fn mixer_channel_fx_effect_press(deck_state: &mut DeckState, effect: usize) {
    deck_state.channel_fx_effect = match effect {
        0 => ChannelFXEffect::Space,
        1 => ChannelFXEffect::DubEcho,
        2 => ChannelFXEffect::Bitcrush,
        3 => ChannelFXEffect::Pitch,
        4 => ChannelFXEffect::Noise,
        5 => ChannelFXEffect::Filter,
        _ => {
            return;
        }
    }
}

pub fn mixer_channel_fx_effect_release(deck_state: &mut DeckState, effect: usize) {
    if deck_state.channel_fx_effect
        == match effect {
            0 => ChannelFXEffect::Space,
            1 => ChannelFXEffect::DubEcho,
            2 => ChannelFXEffect::Bitcrush,
            3 => ChannelFXEffect::Pitch,
            4 => ChannelFXEffect::Noise,
            5 => ChannelFXEffect::Filter,
            _ => {
                return;
            }
        }
    {
        deck_state.channel_fx_effect = ChannelFXEffect::None;
    }
}

pub fn mixer_channel_crossfader_assign_a_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::A;
    }
}

pub fn mixer_channel_crossfader_assign_none_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::None;
    }
}

pub fn mixer_channel_crossfader_assign_b_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::B;
    }
}

pub fn player_quantize_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.quanitze = !channel.player.quanitze;
    }
}

pub fn player_slip_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.slip = !channel.player.slip;
    }
}

pub fn player_gate_cue_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.gate_cue = !channel.player.gate_cue;
    }
}

pub fn player_play_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        match channel.player.play_state {
            PlayState::Cue | PlayState::Stop => {
                if channel.player.slip_playing {
                    channel.player.time = channel.player.slip_time;
                }

                channel.player.play_state = PlayState::Play;
            }
            PlayState::Play => channel.player.play_state = PlayState::Stop,
        }
    }
}

pub fn player_reverse_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.play_direction = PlayDirection::Reverse;
    }
}

pub fn player_reverse_release(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if channel.player.play_direction == PlayDirection::SlipReverse {
            channel.player.time = channel.player.slip_time;
        }

        channel.player.play_direction = PlayDirection::Forward;
    }
}

pub fn player_slip_reverse_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.play_direction = PlayDirection::SlipReverse;
    }
}

pub fn player_slip_reverse_release(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if channel.player.play_direction == PlayDirection::SlipReverse {
            channel.player.time = channel.player.slip_time;
        }

        channel.player.play_direction = PlayDirection::Forward;
    }
}

pub fn player_cue_press(deck_state: &mut DeckState, channel: usize, alt: bool) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if alt {
            channel.player.time = Timecode::zero();
            channel.player.cue_time = None;
        } else {
            match channel.player.play_state {
                PlayState::Stop => {
                    if let Some(cue_time) = channel.player.cue_time
                        && channel.player.time == cue_time
                    {
                        channel.player.play_state = PlayState::Cue;
                    } else {
                        let cue_time = if channel.player.quanitze {
                            channel
                                .player
                                .get_closest_beat()
                                .map_or(channel.player.time, |beat| beat.time)
                        } else {
                            channel.player.time
                        };

                        channel.player.play_state = PlayState::Stop;
                        channel.player.cue_time = Some(cue_time);
                        channel.player.time = cue_time;
                    }
                }
                PlayState::Play => {
                    let cue_time = channel
                        .player
                        .cue_time
                        .unwrap_or(if channel.player.quanitze {
                            if let Some(ref track_analysis) = channel.player.current_track_analysis
                                && let Some(first_beat) = track_analysis.beat_grid.first()
                            {
                                first_beat.time
                            } else {
                                Timecode::zero()
                            }
                        } else {
                            Timecode::zero()
                        });

                    channel.player.play_state = PlayState::Stop;
                    channel.player.cue_time = Some(cue_time);
                    channel.player.time = cue_time;
                }
                PlayState::Cue => {}
            }
        }
    }
}

pub fn player_cue_release(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && channel.player.play_state == PlayState::Cue
    {
        channel.player.play_state = PlayState::Stop;

        if let Some(position) = channel.player.cue_time {
            channel.player.time = position;
        }
    }
}

pub fn player_beat_sync_press(deck_state: &mut DeckState, mode: BeatSyncMode, channel: usize) {
    let Some(beat_sync) = deck_state
        .mixer_channels
        .get_mut(channel)
        .map(|channel| channel.player.beat_sync)
    else {
        return;
    };

    let new_beat_sync = match beat_sync {
        BeatSyncMode::Off => {
            if deck_state.master_channel.is_none() {
                deck_state.master_channel = Some(channel);
            }

            mode
        }
        BeatSyncMode::BPMSync | BeatSyncMode::BeatSync => {
            if let Some(master_channel) = deck_state.master_channel
                && master_channel == channel
            {
                deck_state.find_new_master(&[channel]);
            }

            BeatSyncMode::Off
        }
    };

    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.beat_sync = new_beat_sync;
    }
}

pub fn player_key_sync_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.key_sync = !channel.player.key_sync;
    }
}

pub fn player_master_press(deck_state: &mut DeckState, channel: usize) {
    if deck_state.mixer_channels.get(channel).is_some() {
        deck_state.master_channel = Some(channel);
    }
}

pub fn player_tempo_reset_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.tempo_reset = !channel.player.tempo_reset;
    }
}

pub fn player_tempo_range_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.tempo_range = match channel.player.tempo_range {
            TempoRange::SixPercent => TempoRange::TenPercent,
            TempoRange::TenPercent => TempoRange::SixteenPercent,
            TempoRange::SixteenPercent => TempoRange::OneHundredPercent,
            TempoRange::OneHundredPercent => TempoRange::YesPercent,
            TempoRange::YesPercent => TempoRange::SixPercent,
        };

        channel.player.tempo_slider_is_accurate = false;
    }
}

pub fn player_master_tempo_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.master_tempo = !channel.player.master_tempo;
    }
}

pub fn player_beat_loop_in_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if let Some(loop_start) = channel.player.beat_loop_start
            && let Some(loop_end) = channel.player.beat_loop_end
        {
            if (loop_end.nanoseconds - loop_start.nanoseconds) > MIN_LOOP_SIZE_NANOSECONDS {
                let new_end = loop_start + ((loop_end - loop_start) / 2);

                channel.player.beat_loop_end = Some(new_end);
                channel.player.last_beat_loop = Some((loop_start, new_end));
            }
        } else {
            let loop_start_time = if channel.player.quanitze {
                channel
                    .player
                    .get_closest_beat()
                    .map_or(channel.player.time, |beat| beat.time)
            } else {
                channel.player.time
            };

            if let Some(loop_end) = channel.player.beat_loop_end
                && loop_start_time >= loop_end
            {
                channel.player.beat_loop_end = None;
            }

            channel.player.beat_loop_start = Some(loop_start_time);

            if let Some(loop_start) = channel.player.beat_loop_start
                && let Some(loop_end) = channel.player.beat_loop_end
            {
                channel.player.last_beat_loop = Some((loop_start, loop_end));
            }
        }
    }
}

pub fn player_beat_loop_in_adjust_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if channel.player.beat_loop_adjust_mode == BeatLoopAdjustMode::In {
            channel.player.beat_loop_adjust_mode = BeatLoopAdjustMode::None;
        } else {
            channel.player.beat_loop_adjust_mode = BeatLoopAdjustMode::In;
        }
    }
}

pub fn player_beat_loop_out_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if let Some(loop_start) = channel.player.beat_loop_start
            && let Some(loop_end) = channel.player.beat_loop_end
        {
            let new_end = loop_start + ((loop_end - loop_start) * 2);

            channel.player.beat_loop_end = Some(new_end);
            channel.player.last_beat_loop = Some((loop_start, new_end));
        } else {
            let loop_end_time = if channel.player.quanitze {
                channel
                    .player
                    .get_closest_beat()
                    .map_or(channel.player.time, |beat| beat.time)
            } else {
                channel.player.time
            };

            if let Some(loop_start) = channel.player.beat_loop_start
                && loop_end_time <= loop_start
            {
                channel.player.beat_loop_start = None;
            }

            channel.player.beat_loop_end = Some(loop_end_time);

            if let Some(loop_start) = channel.player.beat_loop_start
                && let Some(loop_end) = channel.player.beat_loop_end
            {
                channel.player.last_beat_loop = Some((loop_start, loop_end));
            }
        }
    }
}

pub fn player_beat_loop_out_adjust_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if channel.player.beat_loop_adjust_mode == BeatLoopAdjustMode::Out {
            channel.player.beat_loop_adjust_mode = BeatLoopAdjustMode::None;
        } else {
            channel.player.beat_loop_adjust_mode = BeatLoopAdjustMode::Out;
        }
    }
}

pub fn player_beat_loop_exit_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && channel.player.beat_loop_start.is_some()
        && channel.player.beat_loop_end.is_some()
    {
        channel.player.beat_loop_end = None;
    }
}

pub fn player_re_loop_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && let Some((last_loop_start, last_loop_end)) = channel.player.last_beat_loop
    {
        channel.player.beat_loop_start = Some(last_loop_start);
        channel.player.beat_loop_end = Some(last_loop_end);

        if channel.player.time.nanoseconds < last_loop_start.nanoseconds
            || channel.player.time.nanoseconds > last_loop_end.nanoseconds
        {
            channel.player.time = last_loop_start;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_hot_cue_press(deck_state: &mut DeckState, channel: usize, number: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && let Some(analysis) = &channel.player.current_track_analysis
        && let Some(cue) = analysis
            .hot_cues
            .iter()
            .find(|cue| cue.number == number as u32)
    {
        channel.player.cue_time = Some(cue.time);
        channel.player.time = cue.time;

        if channel.player.gate_cue {
            channel.player.play_state = PlayState::Cue;
        } else {
            channel.player.play_state = PlayState::Play;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_hot_cue_release(deck_state: &mut DeckState, channel: usize, number: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && channel.player.gate_cue
        && channel.player.play_state == PlayState::Cue
        && let Some(analysis) = &channel.player.current_track_analysis
        && let Some(cue) = analysis
            .hot_cues
            .iter()
            .find(|cue| cue.number == number as u32)
        && let Some(cue_time) = channel.player.cue_time
        && cue.time == cue_time
    {
        channel.player.play_state = PlayState::Stop;
        channel.player.time = cue.time;
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_beat_jump_press(deck_state: &mut DeckState, channel: usize, distance: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && let Some(bpm) = channel.player.get_current_source_bpm()
    {
        if channel.player.beat_sync == BeatSyncMode::BeatSync {
            let fractional_distance = distance.abs().fract();

            if fractional_distance > f32::EPSILON && fractional_distance < (1.0 - f32::EPSILON) {
                channel.player.beat_sync = BeatSyncMode::BPMSync;
            }
        }

        let distance = ((60_000_000_000.0 / bpm) * distance).floor() as i64;

        channel.player.time += Duration::from_nanoseconds(distance);
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_instant_loop_press(deck_state: &mut DeckState, channel: usize, length: f32) {
    if length.abs() < f32::EPSILON {
        return;
    }

    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && let Some(bpm) = channel.player.get_current_source_bpm()
    {
        let length = Duration::from_nanoseconds(((60_000_000_000.0 / bpm) * length).floor() as i64);

        if let Some(loop_start) = channel.player.beat_loop_start
            && channel.player.beat_loop_end.is_some()
        {
            let new_end = loop_start + length;

            channel.player.beat_loop_end = Some(new_end);

            channel.player.last_beat_loop = Some((loop_start, new_end));
        } else {
            let loop_start_time = if channel.player.quanitze {
                channel
                    .player
                    .get_closest_beat()
                    .map_or(channel.player.time, |beat| beat.time)
            } else {
                channel.player.time
            };

            if let Some(loop_end) = channel.player.beat_loop_end
                && loop_start_time >= loop_end
            {
                channel.player.beat_loop_end = None;
            }

            channel.player.beat_loop_start = Some(loop_start_time);
            channel.player.beat_loop_end = Some(loop_start_time + length);

            channel.player.last_beat_loop = Some((loop_start_time, loop_start_time + length));
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_key_shift_press(deck_state: &mut DeckState, channel: usize, shift: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.keyshift = shift;

        channel.player.key_sync = false;
    }
}
