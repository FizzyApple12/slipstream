#![feature(iter_array_chunks)]

pub mod leds;
pub mod logger;
pub mod screens;
pub mod usb;
use std::{collections::BTreeMap, sync::LazyLock, time::Duration};

use clap::Parser;
use libdj::types::deck::DeckState;
use libio::{
    driver_tools::{
        ipc::{DriverIPC, IPCMessage},
        midi::{addr_channel_value_to_midi, midi_to_addr_value},
        midi_mapper::{MidiMapping, load_midi_mapping, process_midi_command},
    },
    types::{controller::ControllerEvent, ipc::DriverInfo},
};
use log::{error, info, warn};

use crate::{
    leds::{LEDFlashTimers, set_deck_leds},
    logger::setup_logger,
    screens::set_deck_screens,
    usb::{DeviceInitError, DeviceInterface, DeviceInterfaceEvent, wait_for_new_devices},
};

pub const MIDI_MAPPING: &str = include_str!("../ddj-flx10.txt");

pub const VENDOR_ID: u16 = 0x2B73;
pub const PRODUCT_ID: u16 = 0x0041;

pub const EP2_IN: u8 = 0x82;
pub const EP3_OUT: u8 = 0x03;
pub const EP4_IN: u8 = 0x84;
pub const EP5_OUT: u8 = 0x05;

pub const VENDOR_COMMANDS: [(u16, u16); 7] = [
    (0x0100, 0xC028),
    (0x0000, 0xC029),
    (0x0200, 0xC013),
    (0x0000, 0xC02B),
    (0x0100, 0xC026),
    (0x0000, 0xC01D),
    (0x0100, 0xC027),
];

pub const KEEPALIVE_FRAME: [u8; 16] = [
    0x04, 0xf0, 0x00, 0x40, 0x04, 0x05, 0x00, 0x00, 0x04, 0x04, 0x01, 0x00, 0x07, 0x50, 0x00, 0xf7,
];

pub static REKORDBOX_INIT_SEQUENCE: LazyLock<Vec<[String; 2]>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../rekordbox_init_sequence.json"))
        .expect("Invalid rekordbox init sequence")
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadMode {
    HotCue,
    BeatJump,
    BeatLoop,
    KeyShift,
}

#[derive(Debug, Clone)]
pub struct InternalPlayerState {
    pub pad_mode: PadMode,
}

impl Default for InternalPlayerState {
    fn default() -> Self {
        Self {
            pad_mode: PadMode::HotCue,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalMixerState {
    pub player: InternalPlayerState,
}

#[allow(clippy::derivable_impls)]
impl Default for InternalMixerState {
    fn default() -> Self {
        Self {
            player: InternalPlayerState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalDeckState {
    pub mixer_channels: [InternalMixerState; 4],
}

#[allow(clippy::derivable_impls)]
impl Default for InternalDeckState {
    fn default() -> Self {
        Self {
            mixer_channels: [
                InternalMixerState::default(),
                InternalMixerState::default(),
                InternalMixerState::default(),
                InternalMixerState::default(),
            ],
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    log_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    setup_logger(args.log_path)?;

    let _ = LazyLock::force(&REKORDBOX_INIT_SEQUENCE);

    let midi_mapping = load_midi_mapping(MIDI_MAPPING);

    loop {
        info!("Connecting to DDJ-FLX10...");

        let mut device_interfaces = match DeviceInterface::init_device().await {
            Ok(device_interfaces) => device_interfaces,
            Err(DeviceInitError::NotFound) => {
                warn!("USB device not found");
                info!("Waiting for new USB devices to become available...");

                wait_for_new_devices().await;

                continue;
            }
            Err(err) => {
                warn!("Failed to init USB device, retrying: {err}");

                tokio::time::sleep(Duration::from_secs(1)).await;

                continue;
            }
        };

        info!("Port connected, starting IPC socket...");

        let mut ipc = match DriverIPC::advertise(DriverInfo {
            id: "ddj_flx10".to_string(),
            name: "DDJ-FLX10".to_string(),
            update_rate_hz: 150,
        })
        .await
        {
            Ok(ipc) => ipc,
            Err(err) => {
                error!("IPC Error: {err}");

                return Err(format!("IPC Error: {err}").into());
            }
        };

        info!("IPC running, starting message relay.");

        event_loop(&mut ipc, &midi_mapping, &mut device_interfaces).await;

        info!("Device disconnected, cleaning up...");

        drop(ipc);

        drop(device_interfaces);

        info!("Waiting for new USB devices to become available...");

        wait_for_new_devices().await;
    }
}

async fn event_loop(
    ipc: &mut DriverIPC,
    midi_mapping: &MidiMapping,
    connection: &mut DeviceInterface,
) {
    let mut current_deck_state: DeckState = DeckState::default();
    let mut current_internal_deck_state: InternalDeckState = InternalDeckState::default();

    let mut controller_midi_state: BTreeMap<(&str, usize), (u8, u8)> = BTreeMap::new();

    let mut led_flash_timers = LEDFlashTimers {
        slow: false,
        mid: false,
        fast: false,
    };

    let mut slow_flash_interval = tokio::time::interval(Duration::from_millis(500));
    let mut mid_flash_interval = tokio::time::interval(Duration::from_millis(250));
    let mut fast_flash_interval = tokio::time::interval(Duration::from_millis(100));

    let mut update_jog_interval = tokio::time::interval(Duration::from_millis(20));

    loop {
        tokio::select! {
            _ = slow_flash_interval.tick() => {
                led_flash_timers.slow = !led_flash_timers.slow;

                set_deck_leds(&current_deck_state, &current_internal_deck_state, &led_flash_timers, connection);
            }
            _ = mid_flash_interval.tick() => {
                led_flash_timers.mid = !led_flash_timers.mid;

                set_deck_leds(&current_deck_state, &current_internal_deck_state, &led_flash_timers, connection);
            }
            _ = fast_flash_interval.tick() => {
                led_flash_timers.fast = !led_flash_timers.fast;

                set_deck_leds(&current_deck_state, &current_internal_deck_state, &led_flash_timers, connection);
            }
            _ = update_jog_interval.tick() => {
                set_deck_screens(&current_deck_state, &led_flash_timers, connection);
            }
            event = connection.read_event() => {
                match event {
                    DeviceInterfaceEvent::Disconnect => {
                        break;
                    },
                    DeviceInterfaceEvent::MidiMessage(midi_message) => {
                        let (address, value) = midi_to_addr_value(midi_message);

                        if let Some(event) = process_midi_command(address, value, midi_mapping, &mut controller_midi_state) {
                            postprocess_command(event, &current_deck_state, &mut current_internal_deck_state, &led_flash_timers, connection, ipc);
                        }
                    },
                }
            }
            ipc_event = ipc.receive_message() => {
                match ipc_event {
                    IPCMessage::Connected => {
                        // todo: replay analog knob events
                    },
                    IPCMessage::Disconnected => {
                        current_deck_state = DeckState::default();

                        set_deck_leds(&current_deck_state, &current_internal_deck_state, &led_flash_timers, connection);
                        set_deck_screens(&current_deck_state, &led_flash_timers, connection);

                        for deck in 0..4 {
                            connection.write_midi(addr_channel_value_to_midi(0xB002, deck, 0));
                        }
                    },
                    IPCMessage::UpdateDeckState(deck_state) => {
                        current_deck_state = deck_state;

                        set_deck_leds(&current_deck_state, &current_internal_deck_state, &led_flash_timers, connection);
                    },
                    IPCMessage::UpdateCurrentMeter(amplitudes) => {
                        for (deck, amplitudes) in amplitudes.into_iter().enumerate() {
                            let [left, right] = amplitudes;

                            let amplitude = left.max(right).abs().clamp(0.0, 127.0);

                            // this function sucks because the ddj-flx10 level meter makes zero sense
                            // note from future me: this doesn't even work correctly and i'm loosing my mind
                            let out_value = if amplitude >= 0.535 {
                                17.2043 * amplitude + 109.7957
                            } else if amplitude >= 0.38 {
                                70.8677 * amplitude + 81.03226
                            } else if amplitude >= 0.2675 {
                                62.22222 * amplitude + 84.35556
                            } else if amplitude >= 0.19 {
                                90.32258 * amplitude + 76.83871
                            } else if amplitude >= 0.135 {
                                127.27273 * amplitude + 69.81818
                            } else if amplitude >= 0.0975 {
                                186.66667 * amplitude + 61.8
                            } else if amplitude >= 0.0675 {
                                266.66667 * amplitude + 54.0
                            } else if amplitude >= 0.0075 {
                                366.66667 * amplitude + 47.25
                            } else {
                                6666.6667 * amplitude
                            };

                            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                            let out_value = out_value.round() as u8;

                            #[allow(clippy::cast_possible_truncation)]
                            connection.write_midi(addr_channel_value_to_midi(0xB002, deck as u16, out_value));
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::indexing_slicing)]
fn postprocess_command(
    event: ControllerEvent,
    deck_state: &DeckState,
    internal_deck_state: &mut InternalDeckState,
    _flash_timers: &LEDFlashTimers,
    device_interface: &mut DeviceInterface,
    ipc: &mut DriverIPC,
) {
    match event {
        ControllerEvent::DriverInternalButton { id } => {
            log::warn!("todo: bind internal driver event button_{id}");
        }
        ControllerEvent::DriverInternalButtonWithInteger { id, number } => match (id, number) {
            (deck @ (0..=3), mode @ (0..=3)) => {
                internal_deck_state.mixer_channels[deck].player.pad_mode = [
                    PadMode::HotCue,
                    PadMode::BeatJump,
                    PadMode::BeatLoop,
                    PadMode::KeyShift,
                ][mode];

                #[allow(clippy::cast_possible_truncation)]
                match internal_deck_state.mixer_channels[deck].player.pad_mode {
                    PadMode::HotCue => {
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x901B,
                            deck as u16,
                            127,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x9020,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906D,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906F,
                            deck as u16,
                            0,
                        ));
                    }
                    PadMode::BeatJump => {
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x901B,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x9020,
                            deck as u16,
                            127,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906D,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906F,
                            deck as u16,
                            0,
                        ));
                    }
                    PadMode::BeatLoop => {
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x901B,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x9020,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906D,
                            deck as u16,
                            127,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906F,
                            deck as u16,
                            0,
                        ));
                    }
                    PadMode::KeyShift => {
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x901B,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x9020,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906D,
                            deck as u16,
                            0,
                        ));
                        device_interface.write_midi(addr_channel_value_to_midi(
                            0x906F,
                            deck as u16,
                            127,
                        ));
                    }
                }
            }
            (id, number) => log::warn!("todo: bind internal driver events integer_{id}({number})"),
        },
        ControllerEvent::DriverInternalButtonWithFloat { id, number } => {
            log::warn!("todo: bind internal driver events float_{id}({number})");
        }
        ControllerEvent::DriverInternalButtonWithAbsolute { id, position } => {
            log::warn!("todo: bind internal driver events absolute_{id}({position})");
        }
        ControllerEvent::DriverInternalButtonWithRelative { id, delta } => {
            log::warn!("todo: bind internal driver events relative_{id}({delta})");
        }
        ControllerEvent::PlayerBeatLoopExitPress { channel } => {
            if let Some(player_channel) = deck_state.mixer_channels.get(channel)
                && player_channel.player.beat_loop_start.is_some()
                && player_channel.player.beat_loop_end.is_some()
            {
                ipc.send_event(ControllerEvent::PlayerBeatLoopExitPress { channel });
            } else {
                ipc.send_event(ControllerEvent::PlayerInstantLoopPress {
                    channel,
                    length: 4.0,
                });
            }
        }
        event => {
            ipc.send_event(event);
        }
    }
}
