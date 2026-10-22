use libdj::types::deck::{BeatSyncMode, DeckState, PlayState};
use timecode::Timecode;

use crate::{leds::LEDFlashTimers, usb::DeviceInterface};

// we use the same safe code a lot here, and there's a lot of code here to
// handle, this is probably cleanupable later
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::indexing_slicing,
    clippy::cast_sign_loss
)]
pub fn set_deck_screens(
    deck_state: &DeckState,
    flash_timers: &LEDFlashTimers,
    device_interface: &mut DeviceInterface,
) {
    for (channel, channel_state) in deck_state.mixer_channels.iter().enumerate() {
        let player_state = &channel_state.player;

        // 0b
        // - loaded
        // - ???
        // - show bpm
        // - show tempo percent
        // - ???
        // - show cue point
        // - ???
        // - ???
        let displayed_information = if player_state.current_track.is_some() {
            0x80 + (if player_state.get_current_bpm().is_some() {
                0x20
            } else {
                0x00
            }) + 0x10
                + (if player_state.cue_time.is_some() {
                    0x04
                } else {
                    0x00
                })
        } else {
            0x00
        };

        let jog_illuminated = if let Some((_, current_track)) = &player_state.current_track {
            let reference_time = if let Some(track_analysis) = &player_state.current_track_analysis
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
                true
            } else if player_state.time >= reference_time - timecode::Duration::from_seconds(15) {
                flash_timers.fast
            } else if player_state.time >= reference_time - timecode::Duration::from_seconds(30) {
                flash_timers.slow
            } else {
                true
            }
        } else {
            false
        };

        // 0b
        // - jog wheel illuminated
        // - ???
        // - sync
        // - master
        // - ???
        // - ???
        // - ???
        // - ???
        let status = (if jog_illuminated { 0x80 } else { 0x00 })
            + (if let Some(master_channel) = deck_state.master_channel
                && master_channel == channel
            {
                0x10
            } else {
                0x00
            })
            + match player_state.beat_sync {
                BeatSyncMode::Off => 0x00,
                BeatSyncMode::BPMSync => {
                    if flash_timers.fast {
                        0x20
                    } else {
                        0x00
                    }
                }
                BeatSyncMode::BeatSync => 0x20,
            };

        let (position, duration): (u32, u32) = if let Some((_, track)) = &player_state.current_track
        {
            let track_position = player_state.time.nanoseconds.max(0) as u64;
            let position_minutes = ((track_position / 60_000_000_000) & 0xff) as u32;
            let position_seconds =
                (((track_position % 60_000_000_000) / 1_000_000_000) & 0xff) as u32;
            let position_milliseconds =
                (((track_position % 1_000_000_000) / 1_000_000) & 0xffff) as u32;

            let track_duration = track.duration.max(0) as u64;
            let duration_minutes = ((track_duration / 60_000_000_000) & 0xff) as u32;
            let duration_seconds =
                (((track_duration % 60_000_000_000) / 1_000_000_000) & 0xff) as u32;
            let duration_milliseconds =
                (((track_duration % 1_000_000_000) / 1_000_000) & 0xffff) as u32;

            (
                (position_minutes << 24) + (position_seconds << 16) + position_milliseconds,
                (duration_minutes << 24) + (duration_seconds << 16) + duration_milliseconds,
            )
        } else {
            (0, 0x06_1b_fa_01)
        };

        let bpm: u32 = if let Some(bpm) = player_state.get_current_bpm() {
            ((bpm * 100.0).round() as u32).min(25599)
        } else {
            0
        };

        let tempo = (player_state.tempo_percent - 1.0) * 100.0;
        let tempo = (tempo * 100.0).round() as i32;
        let tempo = tempo.clamp(-0x8000, 0x7FFF) as i16 as u16;

        // todo: figure out the mapping between key and number
        // 0x92 = key 4A
        // let key = if let Some((_, current_track)) = &player_state.current_track {
        //     0x80 + current_track
        //         .key
        //         .shift(if player_state.master_tempo {
        //             0
        //         } else {
        //             player_state.get_tempo_keyshift().round() as i32
        //         })
        //         .to_semitones() as u8
        // } else {
        //     0
        // };
        let key = 0;

        // key shift (0x0d is 0)
        // let keyshift = 0x0d; //(128.0 + player_state.get_tempo_keyshift()).round() as
        // u8;
        let keyshift = 0;

        // (deck_state.mixer_channels[2].fade * 255.0).round() as u8,
        // (deck_state.mixer_channels[0].fade * 255.0).round() as u8,
        // (deck_state.mixer_channels[1].fade * 255.0).round() as u8,
        // (deck_state.mixer_channels[3].fade * 255.0).round() as u8,

        device_interface.write_jog_status([
            [0x10, 0x20, 0x30, 0x40][channel],
            0x27, // command
            displayed_information,
            status,
            match player_state.tempo_range {
                libdj::types::deck::TempoRange::SixPercent => 0,
                libdj::types::deck::TempoRange::TenPercent => 1,
                libdj::types::deck::TempoRange::SixteenPercent => 2,
                libdj::types::deck::TempoRange::OneHundredPercent => 3,
                libdj::types::deck::TempoRange::YesPercent => 4,
            },
            ((position >> 24) & 0xff) as u8,
            ((position >> 16) & 0xff) as u8,
            (position & 0xff) as u8,
            ((position >> 8) & 3) as u8,
            ((duration >> 24) & 0xff) as u8,
            ((duration >> 16) & 0xff) as u8,
            ((duration >> 8) & 0xff) as u8,
            (duration & 0xff) as u8,
            (bpm / 100) as u8,
            (((bpm / 10) % 10) as u8) << 4,
            ((bpm % 10) as u8),
            (tempo & 0xff) as u8,
            (tempo >> 8) as u8,
            0x00, // ???
            0x00, // ???
            0x0e, // ???
            0x00, // ???
            0x00, // ???
            0x00, // ???
            0x00, // ???
            0x80, // ???
            0x00, // ???
            0x00, // ???
            0x00, // ???
            if player_state.current_track.is_some() {
                key
            } else {
                0x80
            },
            keyshift,
            [0x02, 0x01, 0x04, 0x03][channel],
            0xe0, // ??
            0x01, // ??
            0x00, // ??
        ]);
    }
}
