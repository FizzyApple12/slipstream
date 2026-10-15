use std::{path::PathBuf, time::Duration};

use base64::{Engine, engine::general_purpose::STANDARD};
use interprocess::local_socket::{
    GenericFilePath, ListenerNonblockingMode, ListenerOptions, ToFsName,
    tokio::{RecvHalf, SendHalf},
    traits::tokio::{Listener, Stream},
};
use libdj::{AUDIO_CHANNELS, types::deck::DeckState};
use log::{debug, info, warn};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use crate::types::{
    controller::ControllerEvent,
    ipc::{ArchivedHostMessage, DriverInfo, DriverMessage, HostMessage, IPC_DIR},
};

pub enum IPCMessage {
    UpdateDeckState(DeckState),
    UpdateCurrentMeter([[f32; AUDIO_CHANNELS]; 4]),
}

#[derive(Error, Debug)]
pub enum AdvertisementError {
    #[error("Invalid address: {0}")]
    InvalidAddress(std::io::Error),

    #[error("Listen failed: {0}")]
    ListenFailed(std::io::Error),

    #[error("Cannot create file: {0}")]
    CreationFailed(std::io::Error),
}

pub struct DriverIPC {
    sender: tokio::sync::mpsc::UnboundedSender<ControllerEvent>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<IPCMessage>,

    disconnect_signal: Option<tokio::sync::oneshot::Sender<()>>,
}

impl DriverIPC {
    pub async fn advertise(driver_info: DriverInfo) -> Result<Self, AdvertisementError> {
        info!(
            "Registering driver IPC to {IPC_DIR}{}.socket",
            driver_info.id
        );

        let ipc_folder = PathBuf::from(IPC_DIR);

        if !ipc_folder.exists() {
            fs::create_dir_all(ipc_folder.clone())
                .await
                .map_err(AdvertisementError::CreationFailed)?;
        }

        let ipc_file: PathBuf = ipc_folder.join(format!("{}.socket", driver_info.id));

        if ipc_file.exists() {
            warn!("Socket already found, attempting to reclaim");

            fs::remove_file(ipc_file.clone())
                .await
                .map_err(AdvertisementError::CreationFailed)?;
        }

        let socket_name = ipc_file
            .to_fs_name::<GenericFilePath>()
            .map_err(AdvertisementError::InvalidAddress)?;

        info!("Starting listener");

        let listener = ListenerOptions::new()
            .name(socket_name)
            .nonblocking(ListenerNonblockingMode::Both)
            .create_tokio()
            .map_err(AdvertisementError::ListenFailed)?;

        let (ipc_message_sender, ipc_message_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (controller_event_sender, mut controller_event_receiver) =
            tokio::sync::mpsc::unbounded_channel();

        let (disconnect_sender, mut disconnect_receiver) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let mut stream_send_half: SendHalf;
            let mut stream_recv_half: BufReader<RecvHalf>;
            let mut stream_read_buffer: Vec<u8>;

            let heartbeat_timeout = tokio::time::sleep(Duration::from_secs(1));
            tokio::pin!(heartbeat_timeout);

            'outer_loop: loop {
                loop {
                    info!("Waiting for host connection");

                    match listener.accept().await {
                        Ok(connection) => {
                            let (receiver, sender) = connection.split();

                            let receiver = BufReader::new(receiver);

                            stream_send_half = sender;
                            stream_recv_half = receiver;
                            stream_read_buffer = Vec::new();

                            info!("Accepted connection");

                            break;
                        }
                        Err(err) => {
                            warn!("Error accepting connection: {err}");
                        }
                    }
                }

                loop {
                    tokio::select! {
                        result = tokio::time::timeout(Duration::from_secs(1), stream_recv_half.read_until(b'\n', &mut stream_read_buffer)) => {
                            match result {
                                Ok(Ok(0)) => {
                                    warn!("Stream hit EOF, disconnecting");

                                    break;
                                }
                                Ok(Ok(_)) => {
                                    let _ = stream_read_buffer.try_remove(stream_read_buffer.len().saturating_sub(1));

                                    if let Ok(serialized_message) = STANDARD.decode(&stream_read_buffer)
                                        && let Ok(archived) = rkyv::access::<ArchivedHostMessage, rkyv::rancor::Error>(
                                            &serialized_message,
                                        )
                                        && let Ok(deserialized) =
                                            rkyv::deserialize::<HostMessage, rkyv::rancor::Error>(archived)
                                    {
                                        debug!("Incoming Message: {deserialized:#?}");

                                        match deserialized {
                                            HostMessage::GetInfo => {
                                                if let Ok(serialized_event) = rkyv::to_bytes::<rkyv::rancor::Error>(&DriverMessage::Info(driver_info.clone())) {
                                                    let _ = stream_send_half.write_all(
                                                        (STANDARD.encode(serialized_event.clone()) + "\n")
                                                            .as_bytes(),
                                                    ).await;
                                                }
                                            },
                                            HostMessage::Heartbeat => {
                                                heartbeat_timeout.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(1));
                                            },
                                            HostMessage::UpdateDeckState(deck_state) => {
                                                let _ = ipc_message_sender.send(IPCMessage::UpdateDeckState(deck_state));
                                            },
                                            HostMessage::UpdateCurrentMeter(meter_values) => {
                                                let _ = ipc_message_sender.send(IPCMessage::UpdateCurrentMeter(meter_values));
                                            },
                                        }
                                    }

                                    stream_read_buffer.clear();
                                },
                                Ok(Err(err)) => {
                                    warn!("Failed to receive message, disconnecting: {err}");

                                    break;
                                },
                                Err(_) => {
                                    warn!("Receive message timed out, disconnecting");

                                    break;
                                },
                            }
                        }
                        Some(event) = controller_event_receiver.recv() => {
                            debug!("Sending Message: {event:#?}");

                            if let Ok(serialized_event) = rkyv::to_bytes::<rkyv::rancor::Error>(&DriverMessage::Event(event)) {
                                let _ = stream_send_half.write_all(
                                    (STANDARD.encode(serialized_event.clone()) + "\n")
                                        .as_bytes(),
                                ).await;
                            }
                        }
                        // () = &mut heartbeat_timeout => {
                        //     warn!("Heartbeat overrun, closing connection");

                        //     break;
                        // }
                        _ = &mut disconnect_receiver => {
                            info!("Disconnected requested");

                            break 'outer_loop;
                        }
                    }
                }
            }

            drop(listener);
        });

        Ok(DriverIPC {
            sender: controller_event_sender,
            receiver: ipc_message_receiver,

            disconnect_signal: Some(disconnect_sender),
        })
    }

    pub fn send_event(&mut self, event: ControllerEvent) {
        let _ = self.sender.send(event);
    }

    pub async fn receive_message(&mut self) -> IPCMessage {
        loop {
            if let Some(message) = self.receiver.recv().await {
                return message;
            }
        }
    }
}

impl Drop for DriverIPC {
    fn drop(&mut self) {
        if let Some(disconnect_signal) = self.disconnect_signal.take() {
            let _ = disconnect_signal.send(());
        }
    }
}
