use libdj::types::deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, PlayDirection, PlayState};
use libio::{
    driver_tools::midi::{addr_channel_value_to_midi, addr_value_to_midi},
    types::midi::MidiMessage,
};
use timecode::Timecode;

pub struct LEDFlashTimers {
    pub slow: bool,
    pub mid: bool,
    pub fast: bool,
}

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
    flash_timers: &LEDFlashTimers,
    midi_output: &tokio::sync::mpsc::UnboundedSender<MidiMessage>,
) {
    // master cue
    let _ = midi_output.send(addr_value_to_midi(
        0x9662,
        if deck_state.master_cue { 127 } else { 0 },
    ));
    let _ = midi_output.send(addr_value_to_midi(
        0x9663,
        if deck_state.master_cue { 127 } else { 0 },
    ));

    for (channel, channel_state) in deck_state.mixer_channels.iter().enumerate() {
        let player_state = &channel_state.player;

        // cue
        #[allow(clippy::indexing_slicing)]
        let _ = midi_output.send(addr_channel_value_to_midi(
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
            let _ = midi_output.send(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                if channel == master_channel { 127 } else { 0 },
            ));
        } else {
            let _ = midi_output.send(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                0,
            ));
        }

        // vinyl
        let _ = midi_output.send(addr_channel_value_to_midi(0x9017, channel as u16, 127));

        // beat sync
        let _ = midi_output.send(addr_channel_value_to_midi(
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
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9065,
            channel as u16,
            if player_state.key_sync { 127 } else { 0 },
        ));

        // quantize
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9035,
            channel as u16,
            if player_state.quanitze { 127 } else { 0 },
        ));

        // slip
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9040,
            channel as u16,
            if player_state.slip { 127 } else { 0 },
        ));

        // reverse
        let _ = midi_output.send(addr_channel_value_to_midi(
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
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x900B, channel as u16, 127));
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x9047, channel as u16, 127));
                }
                PlayState::Cue | PlayState::Stop => {
                    let _ = midi_output.send(addr_channel_value_to_midi(
                        0x900B,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                    let _ = midi_output.send(addr_channel_value_to_midi(
                        0x9047,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                }
            }

            // cue
            match player_state.play_state {
                PlayState::Play | PlayState::Cue => {
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x900C, channel as u16, 127));
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x9048, channel as u16, 127));
                }
                PlayState::Stop => {
                    if let Some(cue_time) = player_state.cue_time {
                        if player_state.time == cue_time {
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                127,
                            ));
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                127,
                            ));
                        } else {
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                        }
                    } else {
                        let _ = midi_output.send(addr_channel_value_to_midi(
                            0x900C,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                        let _ = midi_output.send(addr_channel_value_to_midi(
                            0x9048,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                    }
                }
            }
        } else {
            let _ = midi_output.send(addr_channel_value_to_midi(0x900B, channel as u16, 0));
            let _ = midi_output.send(addr_channel_value_to_midi(0x9047, channel as u16, 0));

            let _ = midi_output.send(addr_channel_value_to_midi(0x900C, channel as u16, 0));
            let _ = midi_output.send(addr_channel_value_to_midi(0x9048, channel as u16, 0));
        }

        // beat loop
        match (player_state.beat_loop_start, player_state.beat_loop_end) {
            (None, None) => {
                // in
                let _ = midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, 0));

                // out
                let _ = midi_output.send(addr_channel_value_to_midi(0x9011, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9013, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    if player_state.last_beat_loop.is_some() {
                        127
                    } else {
                        0
                    },
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), None) => {
                // in
                let in_state = if flash_timers.mid { 127 } else { 0 };
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, in_state));

                // out
                let _ = midi_output.send(addr_channel_value_to_midi(0x9011, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9013, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904D, channel as u16, 127));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (None, Some(_)) => {
                // in
                let _ = midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, 127));

                // out
                let out_state = if flash_timers.mid { 127 } else { 0 };
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), Some(_)) => {
                // in
                let in_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::In {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
                };
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, in_state));

                // out
                let out_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::Out {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
                };
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 127));
            }
        }

        // tempo reset
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9041,
            channel as u16,
            if player_state.tempo_reset { 127 } else { 0 },
        ));

        // master tempo
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F20, 0x9F21, 0x9F22, 0x9F23][channel],
            if player_state.master_tempo { 127 } else { 0 },
        ));

        // --------- jog display stuff ---------

        // display on
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F5D, 0x9F5E, 0x9F5F, 0x9F60][channel],
            0x00,
            // if player_state.current_track.is_some() {
            //     0x00
            // } else {
            //     0x7F
            // },
        ));

        // jog illumination
        let color = if channel_state.fade > 0.00001 {
            0x02
        } else {
            0x01
        };

        let _ = midi_output.send(addr_value_to_midi(
            [0xBF09, 0xBF0A, 0xBF0B, 0xBF0C][channel],
            if let Some((_, current_track)) = &player_state.current_track {
                let reference_time = if let Some(track_analysis) =
                    &player_state.current_track_analysis
                    && let Some(last_beat) = track_analysis.beat_grid.last()
                {
                    last_beat.time
                } else {
                    Timecode::from_nanoseconds(current_track.duration)
                };

                if player_state.jog_hold
                    || player_state.jog_wait
                    || player_state.time >= reference_time
                    || player_state.play_state == PlayState::Stop
                {
                    color
                } else if player_state.time >= reference_time - timecode::Duration::from_seconds(15)
                {
                    if flash_timers.fast { color } else { 0x00 }
                } else if player_state.time >= reference_time - timecode::Duration::from_seconds(30)
                {
                    if flash_timers.slow { color } else { 0x00 }
                } else {
                    color
                }
            } else {
                0x00
            },
        ));

        // jog position
        // 1_800_000_000 for 33.3 rpm
        // let jog_position: u16 =
        // (((player_state.time.nanoseconds.rem_euclid(1_800_000_000))
        //     / 1_800_000_000)
        //     * 0x0267) as u16;

        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF10, 0xBF11, 0xBF12, 0xBF13][channel],
        //     ((jog_position >> 8) & 0x00FF) as u8,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF30, 0xBF31, 0xBF32, 0xBF33][channel],
        //     (jog_position & 0x00FF) as u8,
        // ));

        // cue point
        // if let Some(cue_point) = player_state.cue_time {
        //     // 1_800_000_000 for 33.3 rpm
        //     let cue_point: u16 =
        // (((cue_point.nanoseconds.rem_euclid(1_800_000_000))         /
        // 1_800_000_000)
        //         * 0x0267) as u16;

        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         ((cue_point >> 8) & 0x00FF) as u8,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         (cue_point & 0x00FF) as u8,
        //     ));
        // } else {
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         0x7F,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         0x7F,
        //     ));
        // }

        // time
        // let track_time_seconds = player_state.time.nanoseconds /
        // 1_000_000_000; let track_time_minutes = (track_time_seconds /
        // 60).unsigned_abs() as u8; let track_time_seconds =
        // (track_time_seconds % 60).unsigned_abs() as u8;

        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF42, 0xBF44, 0xBF46, 0xBF48][channel],
        //     track_time_minutes,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF43, 0xBF45, 0xBF47, 0xBF49][channel],
        //     track_time_seconds,
        // ));

        // bpm
        // if let Some(bpm) = player_state.get_current_bpm() {
        //     // 19983 == 0x4E0F
        //     let cue_point = (bpm.clamp(0.0, 999.9) * 19983.0).floor() as u16;

        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
        //         ((cue_point >> 8) & 0x00FF) as u8,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
        //         (cue_point & 0x00FF) as u8,
        //     ));
        // } else {
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
        //         0,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
        //         0,
        //     ));
        // }

        // tempo percent
        // 19983 == 0x4E0F
        // let tempo = (((player_state.tempo_percent - 1.0) * (19983.0 / 2.0)) +
        // (19983.0 / 2.0))     .floor() as u16;
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF18, 0xBF19, 0xBF1A, 0xBF1B][channel],
        //     ((tempo >> 8) & 0x00FF) as u8,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF38, 0xBF39, 0xBF3A, 0xBF3B][channel],
        //     (tempo & 0x00FF) as u8,
        // ));
    }
}
