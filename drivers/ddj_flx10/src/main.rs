pub mod leds;
pub mod logger;

use std::{collections::BTreeMap, time::Duration};

use libdj::types::deck::DeckState;
use libio::{
    driver_tools::{
        ipc::{DriverIPC, IPCMessage},
        midi::{addr_channel_value_to_midi, midi_to_addr_value},
        midi_mapper::{MidiMapping, load_midi_mapping, process_midi_command},
    },
    types::{ipc::DriverInfo, midi::MidiMessage},
};
use log::{error, info, warn};
use midir::{MidiInput, MidiInputConnection, MidiOutput, SendError};
use tokio::task::JoinHandle;

use crate::{
    leds::{LEDFlashTimers, set_deck_leds},
    logger::setup_logger,
};

const MIDI_MAPPING: &str = include_str!("../ddj-flx10.txt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    let midi_mapping = load_midi_mapping(MIDI_MAPPING);

    loop {
        info!("Connecting to DDJ-FLX10 MIDI...");

        let mut connection = match connect() {
            Ok(connection) => connection,
            Err(err) => {
                warn!("Failed to connect to MIDI: {err}");
                info!("Waiting for new MIDI ports to become available...");

                wait_for_new_devices().await;

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

        midi_loop(&mut ipc, &midi_mapping, &mut connection).await;

        info!("Midi disconnected, cleaning up...");

        drop(ipc);

        connection.input_connection.close();
        connection.output_task.abort();

        info!("Waiting for new MIDI ports to become available...");

        wait_for_new_devices().await;
    }
}

async fn wait_for_new_devices() {
    // todo: replace with a waiter when midir gets device listening support
    tokio::time::sleep(Duration::from_secs(10)).await;
}

struct MidiConnection {
    input_connection: MidiInputConnection<()>,
    output_task: JoinHandle<()>,

    sender: tokio::sync::mpsc::UnboundedSender<MidiMessage>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<MidiMessage>,

    disconnection_signal: tokio::sync::oneshot::Receiver<()>,
}

fn connect() -> Result<MidiConnection, Box<dyn std::error::Error>> {
    let (disconnection_sender, disconnection_receiver) = tokio::sync::oneshot::channel();

    let (midi_input_sender, midi_input_receiver) =
        tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

    let (midi_output_sender, mut midi_output_receiver) =
        tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

    let midi_input = MidiInput::new("slipstream-midi-input")?;

    let input_ports = midi_input.ports();

    let Some(target_input_port) = input_ports.iter().find(|port| {
        midi_input
            .port_name(port)
            .is_ok_and(|name| name.contains("DDJ-FLX10"))
    }) else {
        return Err("No input port found".into());
    };

    let input_port_name = midi_input.port_name(target_input_port)?;

    let midi_output = MidiOutput::new("slipstream-midi-output")?;

    let output_ports = midi_output.ports();

    let Some(target_output_port) = output_ports.iter().find(|port| {
        midi_output
            .port_name(port)
            .is_ok_and(|name| name.contains("DDJ-FLX10"))
    }) else {
        return Err("No output port found".into());
    };

    let output_port_name = midi_output.port_name(target_output_port)?;

    let midi_input = midi_input.connect(
        target_input_port,
        &input_port_name,
        move |_, message, ()| {
            if let [byte_zero, byte_one, byte_two] = *message {
                let _ = midi_input_sender.send([byte_zero, byte_one, byte_two]);
            }
        },
        (),
    )?;

    let mut connection = midi_output.connect(target_output_port, &output_port_name)?;

    let midi_output = tokio::task::spawn(async move {
        while let Some(message) = &midi_output_receiver.recv().await {
            if let Err(err) = connection.send(message) {
                match err {
                    SendError::InvalidData(_) => {}
                    SendError::Other(_) => {
                        break;
                    }
                }
            }
        }

        let _ = disconnection_sender.send(());
    });

    Ok(MidiConnection {
        input_connection: midi_input,
        output_task: midi_output,

        sender: midi_output_sender,
        receiver: midi_input_receiver,

        disconnection_signal: disconnection_receiver,
    })
}

async fn midi_loop(
    ipc: &mut DriverIPC,
    midi_mapping: &MidiMapping,
    connection: &mut MidiConnection,
) {
    let mut current_deck_state: DeckState = DeckState::default();

    let mut controller_state: BTreeMap<(&str, usize), (u8, u8)> = BTreeMap::new();

    let mut led_flash_timers = LEDFlashTimers {
        slow: false,
        mid: false,
        fast: false,
    };

    let mut slow_flash_interval = tokio::time::interval(Duration::from_millis(500));
    let mut mid_flash_interval = tokio::time::interval(Duration::from_millis(250));
    let mut fast_flash_interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = slow_flash_interval.tick() => {
                led_flash_timers.slow = !led_flash_timers.slow;

                set_deck_leds(&current_deck_state, &led_flash_timers, &connection.sender);
            }
            _ = mid_flash_interval.tick() => {
                led_flash_timers.mid = !led_flash_timers.mid;

                set_deck_leds(&current_deck_state, &led_flash_timers, &connection.sender);
            }
            _ = fast_flash_interval.tick() => {
                led_flash_timers.fast = !led_flash_timers.fast;

                set_deck_leds(&current_deck_state, &led_flash_timers, &connection.sender);
            }
            midi_message = connection.receiver.recv() => {
                if let Some(midi_message) = midi_message {
                    let (address, value) = midi_to_addr_value(midi_message);

                    process_midi_command(address, value, midi_mapping, &mut controller_state, ipc);
                }
            }
            _ = &mut connection.disconnection_signal => {
                break;
            }
            ipc_event = ipc.receive_message() => {
                match ipc_event {
                    IPCMessage::UpdateDeckState(deck_state) => {
                        current_deck_state = deck_state;

                        set_deck_leds(&current_deck_state, &led_flash_timers, &connection.sender);
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
                            let _ = connection.sender.send(addr_channel_value_to_midi(0xB002, deck as u16, out_value));
                        }
                    }
                }
            }
        }
    }
}
