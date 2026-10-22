use libdj::types::deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, PlayDirection, PlayState};
use libio::driver_tools::midi::{addr_channel_value_to_midi, addr_value_to_midi};

use crate::{InternalDeckState, usb::DeviceInterface};

pub struct LEDFlashTimers {
    pub slow: bool,
    pub mid: bool,
    pub fast: bool,
}

const PAD_ADDRESS_TABLE: [u16; 16] = [
    0x9000, 0x9001, 0x9002, 0x9003, 0x9004, 0x9005, 0x9006, 0x9007, 0x9008, 0x9009, 0x900A, 0x900B,
    0x900C, 0x900D, 0x900E, 0x900F,
];

// sorted to be in ascending key order
const PAD_ADDRESS_TABLE_KEY_SHIFT: [u16; 16] = [
    0x900F, 0x900E, 0x900D, 0x900C, 0x9007, 0x9006, 0x9005, 0x9004, 0x9000, 0x9001, 0x9002, 0x9003,
    0x9008, 0x9009, 0x900A, 0x900B,
];

// we use the same safe code a lot here, and there's a lot of code here to
// handle, this is probably cleanupable later
#[allow(
    clippy::cast_possible_truncation,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::bool_to_int_with_if
)]
pub fn set_deck_leds(
    deck_state: &DeckState,
    internal_deck_state: &InternalDeckState,
    flash_timers: &LEDFlashTimers,
    device_interface: &mut DeviceInterface,
) {
    // master cue
    device_interface.write_midi(addr_value_to_midi(
        0x9662,
        if deck_state.master_cue { 127 } else { 0 },
    ));
    device_interface.write_midi(addr_value_to_midi(
        0x9663,
        if deck_state.master_cue { 127 } else { 0 },
    ));

    for (channel, (channel_state, _internal_channel_state)) in deck_state
        .mixer_channels
        .iter()
        .zip(&internal_deck_state.mixer_channels)
        .enumerate()
    {
        let player_state = &channel_state.player;
        // let internal_player_state = &internal_channel_state.player;

        // cue
        #[allow(clippy::indexing_slicing)]
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9054,
            channel as u16,
            if channel_state.cue || player_state.touch_cue_time.is_some() {
                127
            } else {
                0
            },
        ));

        let is_loaded = player_state.current_track.is_some();

        // master channel led
        if let Some(master_channel) = deck_state.master_channel {
            device_interface.write_midi(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                if channel == master_channel { 127 } else { 0 },
            ));
        } else {
            device_interface.write_midi(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                0,
            ));
        }

        // vinyl
        device_interface.write_midi(addr_channel_value_to_midi(0x9017, channel as u16, 127));

        // beat sync
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9058,
            channel as u16,
            match player_state.beat_sync {
                BeatSyncMode::Off => 0,
                BeatSyncMode::BPMSync => {
                    if flash_timers.mid {
                        127
                    } else {
                        0
                    }
                }
                BeatSyncMode::BeatSync => 127,
            },
        ));

        // key sync
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9065,
            channel as u16,
            if player_state.key_sync { 127 } else { 0 },
        ));

        // quantize
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9035,
            channel as u16,
            if player_state.quanitze { 127 } else { 0 },
        ));

        // slip
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9040,
            channel as u16,
            if player_state.slip { 127 } else { 0 },
        ));

        // reverse
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9038,
            channel as u16,
            if player_state.play_direction == PlayDirection::Forward {
                0
            } else {
                127
            },
        ));

        if is_loaded {
            // play
            match player_state.play_state {
                PlayState::Play => {
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x900B,
                        channel as u16,
                        127,
                    ));
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x9047,
                        channel as u16,
                        127,
                    ));
                }
                PlayState::Cue | PlayState::Stop => {
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x900B,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x9047,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                }
            }

            // cue
            match player_state.play_state {
                PlayState::Play | PlayState::Cue => {
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x900C,
                        channel as u16,
                        127,
                    ));
                    device_interface.write_midi(addr_channel_value_to_midi(
                        0x9048,
                        channel as u16,
                        127,
                    ));
                }
                PlayState::Stop => {
                    if let Some(cue_time) = player_state.cue_time {
                        if player_state.time == cue_time {
                            device_interface.write_midi(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                127,
                            ));
                            device_interface.write_midi(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                127,
                            ));
                        } else {
                            device_interface.write_midi(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                            device_interface.write_midi(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                        }
                    } else {
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x900C,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x9048,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                    }
                }
            }
        } else {
            device_interface.write_midi(addr_channel_value_to_midi(0x900B, channel as u16, 0));
            device_interface.write_midi(addr_channel_value_to_midi(0x9047, channel as u16, 0));

            device_interface.write_midi(addr_channel_value_to_midi(0x900C, channel as u16, 0));
            device_interface.write_midi(addr_channel_value_to_midi(0x9048, channel as u16, 0));
        }

        // beat loop
        match (player_state.beat_loop_start, player_state.beat_loop_end) {
            (None, None) => {
                // in

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9010,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9012,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(0x904C, channel as u16, 0));

                // out

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    127,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    if player_state.last_beat_loop.is_some() {
                        127
                    } else {
                        0
                    },
                ));

                // 4 beat

                device_interface.write_midi(addr_channel_value_to_midi(0x9014, channel as u16, 0));

                device_interface.write_midi(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), None) => {
                // in
                let in_state = if flash_timers.mid { 127 } else { 0 };
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9010,
                    channel as u16,
                    in_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9012,
                    channel as u16,
                    in_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904C,
                    channel as u16,
                    in_state,
                ));

                // out

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    127,
                ));

                // 4 beat

                device_interface.write_midi(addr_channel_value_to_midi(0x9014, channel as u16, 0));

                device_interface.write_midi(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (None, Some(_)) => {
                // in

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9010,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9012,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904C,
                    channel as u16,
                    127,
                ));

                // out
                let out_state = if flash_timers.mid { 127 } else { 0 };
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat

                device_interface.write_midi(addr_channel_value_to_midi(0x9014, channel as u16, 0));

                device_interface.write_midi(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), Some(_)) => {
                // in
                let in_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::In {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
                };
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9010,
                    channel as u16,
                    in_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9012,
                    channel as u16,
                    in_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904C,
                    channel as u16,
                    in_state,
                ));

                // out
                let out_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::Out {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
                };
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                device_interface.write_midi(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9014,
                    channel as u16,
                    127,
                ));

                device_interface.write_midi(addr_channel_value_to_midi(
                    0x9050,
                    channel as u16,
                    127,
                ));
            }
        }

        // tempo reset
        device_interface.write_midi(addr_channel_value_to_midi(
            0x9041,
            channel as u16,
            if player_state.tempo_reset { 127 } else { 0 },
        ));

        // master tempo
        device_interface.write_midi(addr_value_to_midi(
            [0x9F20, 0x9F21, 0x9F22, 0x9F23][channel],
            if player_state.master_tempo { 127 } else { 0 },
        ));

        // pads
        // colors:
        // 0x00 = dim
        // 0x01 - 0x9f = bright colors
        // 0xa0 = off
        // 0xa1 = white
        // 0xa2 - 0xfe = dim colors
        // 0xff = din white

        let pad_channel = [0x7, 0x9, 0xb, 0xd][channel];

        for (i, pad_address) in PAD_ADDRESS_TABLE.iter().enumerate() {
            let pad_address = *pad_address;

            if let Some(analysis) = &channel_state.player.current_track_analysis
                && let Some(cue) = analysis.hot_cues.iter().find(|cue| cue.number == i as u32)
            {
                device_interface.write_midi(addr_channel_value_to_midi(
                    pad_address,
                    pad_channel,
                    cue.number as u8 + 1,
                ));
            } else {
                device_interface.write_midi(addr_channel_value_to_midi(
                    pad_address,
                    pad_channel,
                    0xa0,
                ));
            }
        }

        for (i, pad_address) in PAD_ADDRESS_TABLE.iter().enumerate() {
            let pad_address = *pad_address | 0x20;

            device_interface.write_midi(addr_channel_value_to_midi(
                pad_address,
                pad_channel,
                if i % 2 == 0 { 0x60 } else { 0x66 },
            ));
        }

        for pad_address in PAD_ADDRESS_TABLE {
            let pad_address = pad_address | 0x60;

            device_interface.write_midi(addr_channel_value_to_midi(pad_address, pad_channel, 0x60));
        }

        let current_key_shift = channel_state.player.keyshift.round() as i32;

        for (i, pad_address) in PAD_ADDRESS_TABLE_KEY_SHIFT.iter().enumerate() {
            let pad_address = *pad_address | 0x70;

            #[allow(clippy::cast_possible_wrap)]
            let key_shift = i as i32 - 7;

            device_interface.write_midi(addr_channel_value_to_midi(
                pad_address,
                pad_channel,
                if key_shift == current_key_shift {
                    if flash_timers.slow { 0xa1 } else { 0xff }
                } else if key_shift > 0 {
                    0x60
                } else if key_shift < 0 {
                    0x66
                } else {
                    0x63
                },
            ));
        }

        // // cue point
        // if let Some(cue_point) = player_state.cue_time {
        //     // 1_800_000_000 for 33.3 rpm
        //     let cue_point: u16 =
        // (((cue_point.nanoseconds.rem_euclid(1_800_000_000))         /
        // 1_800_000_000)
        //         * 0x0267) as u16;

        //     device_interface.write_midi(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         ((cue_point >> 8) & 0x00FF) as u8,
        //     ));
        //     device_interface.write_midi(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         (cue_point & 0x00FF) as u8,
        //     ));
        // } else {
        //     device_interface.write_midi(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         0x7F,
        //     ));
        //     device_interface.write_midi(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         0x7F,
        //     ));
        // }
    }
}
