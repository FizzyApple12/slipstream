use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use bytes::{Bytes, BytesMut};
use libio::types::midi::MidiMessage;
use log::{debug, warn};
use nusb::{
    DeviceSelector,
    hotplug::HotplugEvent,
    transfer::{Bulk, ControlOut, ControlType, In, Interrupt, Out, Recipient},
};
use thiserror::Error;
use tokio::{fs, task::JoinHandle};
use tokio_stream::StreamExt;

use crate::{
    EP2_IN, EP3_OUT, EP4_IN, EP5_OUT, KEEPALIVE_FRAME, PRODUCT_ID, REKORDBOX_INIT_SEQUENCE,
    VENDOR_COMMANDS, VENDOR_ID,
};

#[derive(Error, Debug)]
pub enum SysFSFindError {
    #[error("Failed to read directory: {0}")]
    ReadError(std::io::Error),
}

pub async fn find_sysfs_device(
    vendor_id: u16,
    product_id: u16,
) -> Result<Option<PathBuf>, SysFSFindError> {
    let mut device_list = fs::read_dir(PathBuf::from("/sys/bus/usb/devices"))
        .await
        .map_err(SysFSFindError::ReadError)?;

    while let Ok(Some(device_entry)) = device_list.next_entry().await {
        debug!("Device: {}", device_entry.path().display());
        debug!(
            "\tVendor ID Path: {}",
            device_entry.path().join("idVendor").display()
        );
        debug!(
            "\tProduct ID Path: {}",
            device_entry.path().join("idProduct").display()
        );

        let Ok(vendor_id_string) = fs::read_to_string(device_entry.path().join("idVendor")).await
        else {
            debug!("\tFailed to read Vendor ID");

            continue;
        };

        let Ok(parsed_vendor_id) = u16::from_str_radix(vendor_id_string.trim(), 16) else {
            debug!("\tFailed to parse Vendor ID");

            continue;
        };

        debug!(
            "\tVendor ID: {} ({parsed_vendor_id})",
            vendor_id_string.trim()
        );

        if parsed_vendor_id != vendor_id {
            continue;
        }

        let Ok(product_id_string) = fs::read_to_string(device_entry.path().join("idProduct")).await
        else {
            debug!("\tFailed to read Product ID");

            continue;
        };

        let Ok(parsed_product_id) = u16::from_str_radix(product_id_string.trim(), 16) else {
            debug!("\tFailed to parse Product ID");

            continue;
        };

        debug!(
            "\tProduct ID: {} ({parsed_product_id})",
            product_id_string.trim()
        );

        if parsed_product_id != product_id {
            continue;
        }

        return Ok(Some(device_entry.path()));
    }

    Ok(None)
}

#[derive(Error, Debug)]
pub enum SysFSUnbindError {
    #[error("Failed to read directory: {0}")]
    ReadError(std::io::Error),
}

pub async fn unbind_all(sysfs_path: &Path) -> Result<bool, SysFSUnbindError> {
    let mut sysfs_list = fs::read_dir(PathBuf::from(&sysfs_path))
        .await
        .map_err(SysFSUnbindError::ReadError)?;

    while let Ok(Some(entry)) = sysfs_list.next_entry().await {
        let Ok(file_name) = entry.file_name().into_string() else {
            continue;
        };

        let Ok(metadata) = entry.metadata().await else {
            continue;
        };

        if !metadata.is_dir() || !file_name.contains(':') {
            continue;
        }

        let Ok(driver_path) = fs::read_link(entry.path().join("driver")).await else {
            continue;
        };

        if let Err(err) =
            fs::write(entry.path().join(&driver_path).join("unbind"), &file_name).await
        {
            warn!(
                "Failed to unbind driver {} from device {file_name}: {err}",
                entry.path().join(&driver_path).display()
            );

            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn wait_for_new_devices() {
    loop {
        match nusb::watch_devices() {
            Ok(mut device_watch) => {
                while let Some(event) = device_watch.next().await {
                    if let HotplugEvent::Connected(_) = event {
                        return;
                    }
                }

                warn!("USB device watcher disconnected, restarting");

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(err) => {
                warn!("Failed to watch for new USB devices, retrying: {err}");

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum DeviceInitError {
    #[error("Failed to find USB device: {0}")]
    FindDevice(SysFSFindError),

    #[error("USB device not found")]
    NotFound,

    #[error("Failed to unbind USB device: {0}")]
    UnbindDevice(SysFSUnbindError),

    #[error("USB device was disconnected during init")]
    Disconnected,

    #[error("Failed to request USB device: {0}")]
    RequestDevice(nusb::Error),

    #[error("Failed to open USB device: {0}")]
    Open(nusb::Error),

    #[error("Failed to claim interface: {0}: {1}")]
    ClaimInterface(u8, nusb::Error),

    #[error("Failed to claim endpoint {0}: {1}")]
    ClaimEndpoint(u8, nusb::Error),

    #[error("Failed to send vendor command: {0}")]
    VendorCommandFailed(nusb::transfer::TransferError),

    #[error("Failed to send vendor command: {0}")]
    InitCommandFailed(std::io::Error),

    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),
}

pub enum DeviceInterfaceEvent {
    Disconnect,
    MidiMessage(MidiMessage),
}

pub struct DeviceInterface {
    midi_sender: tokio::sync::mpsc::UnboundedSender<MidiMessage>,
    midi_receiver: tokio::sync::mpsc::UnboundedReceiver<Bytes>,

    display_sender: tokio::sync::mpsc::UnboundedSender<[u8; 128]>,

    disconnection_signal: tokio::sync::oneshot::Receiver<()>,

    join_handles: Vec<JoinHandle<()>>,
}

impl DeviceInterface {
    pub async fn init_device() -> Result<Self, DeviceInitError> {
        let Some(sysfs_path) = find_sysfs_device(VENDOR_ID, PRODUCT_ID)
            .await
            .map_err(DeviceInitError::FindDevice)?
        else {
            return Err(DeviceInitError::NotFound);
        };

        unbind_all(&sysfs_path)
            .await
            .map_err(DeviceInitError::UnbindDevice)?;

        // the kernel takes some time to unbind everything
        tokio::time::sleep(Duration::from_millis(500)).await;

        let device_info = match nusb::request_device(&[
            DeviceSelector::all().with_vid_pid(VENDOR_ID, PRODUCT_ID)
        ])
        .await
        {
            Ok(Some(device)) => device,
            Ok(None) => {
                return Err(DeviceInitError::Disconnected);
            }
            Err(err) => {
                return Err(DeviceInitError::RequestDevice(err));
            }
        };

        let device = device_info.open().await.map_err(DeviceInitError::Open)?;

        for (command_value, command_index) in VENDOR_COMMANDS {
            device
                .control_out(
                    ControlOut {
                        control_type: ControlType::Vendor,
                        recipient: Recipient::Device,
                        request: 3,
                        value: command_value,
                        index: command_index,
                        data: &[],
                    },
                    Duration::from_millis(100),
                )
                .await
                .map_err(DeviceInitError::VendorCommandFailed)?;

            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        let midi_interface = device
            .claim_interface(4)
            .await
            .map_err(|err| DeviceInitError::ClaimInterface(4, err))?;

        let mut ep2_in = midi_interface
            .endpoint::<Bulk, In>(EP2_IN)
            .map_err(|err| DeviceInitError::ClaimEndpoint(EP2_IN, err))?
            .reader(4096);
        let mut ep3_out = midi_interface
            .endpoint::<Bulk, Out>(EP3_OUT)
            .map_err(|err| DeviceInitError::ClaimEndpoint(EP3_OUT, err))?
            .writer(4096);

        let hid_interface = device
            .claim_interface(5)
            .await
            .map_err(|err| DeviceInitError::ClaimInterface(5, err))?;

        let mut ep4_in = hid_interface
            .endpoint::<Interrupt, In>(EP4_IN)
            .map_err(|err| DeviceInitError::ClaimEndpoint(EP4_IN, err))?
            .reader(4096);
        let mut ep5_out = hid_interface
            .endpoint::<Interrupt, Out>(EP5_OUT)
            .map_err(|err| DeviceInitError::ClaimEndpoint(EP5_OUT, err))?
            .writer(4096);

        for [endpoint, data_string] in REKORDBOX_INIT_SEQUENCE.iter() {
            let data: Vec<u8> = data_string
                .chars()
                .array_chunks::<2>()
                .map(|[high, low]| u8::from_str_radix(&format!("{high}{low}"), 16).unwrap_or(0))
                .collect();

            match endpoint.to_lowercase().as_str() {
                "ep3" => {
                    std::io::Write::write_all(&mut ep3_out, &data)
                        .map_err(DeviceInitError::InitCommandFailed)?;
                }
                "ep5" => {
                    std::io::Write::write_all(&mut ep5_out, &data)
                        .map_err(DeviceInitError::InitCommandFailed)?;
                }
                endpoint => return Err(DeviceInitError::InvalidEndpoint(endpoint.to_string())),
            }

            tokio::time::sleep(Duration::from_millis(2)).await;
        }

        let (midi_sender, mut midi_task_receiver) =
            tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        let (disconnection_sender, disconnection_receiver) = tokio::sync::oneshot::channel();

        let midi_out_task = tokio::task::spawn(async move {
            let mut keepalive_interval = tokio::time::interval(Duration::from_millis(200));

            loop {
                tokio::select! {
                    _ = keepalive_interval.tick() => {
                        if std::io::Write::write_all(&mut ep3_out, &KEEPALIVE_FRAME).is_err()
                            || ep3_out.flush().is_err() {
                            break;
                        }
                    }
                    message = midi_task_receiver.recv() => {
                        if let Some(message) = message {
                            let status = message[0];
                            let cin = status >> 4;

                            let _ = std::io::Write::write_all(
                                &mut ep3_out,
                                &[cin, message[0], message[1], message[2]],
                            );

                            while let Ok(message) = midi_task_receiver.try_recv() {
                                let status = message[0];
                                let cin = status >> 4;

                                let _ = std::io::Write::write_all(
                                    &mut ep3_out,
                                    &[cin, message[0], message[1], message[2]],
                                );
                            }

                            let _ = ep3_out.flush();
                        } else {
                            break;
                        }
                    }
                }
            }

            let _ = disconnection_sender.send(());
        });

        let (midi_task_sender, midi_receiver) = tokio::sync::mpsc::unbounded_channel::<Bytes>();

        let midi_in_task = tokio::task::spawn(async move {
            let mut midi_read_buffer = BytesMut::with_capacity(4096);

            loop {
                match tokio::io::AsyncReadExt::read_buf(&mut ep2_in, &mut midi_read_buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let n = (midi_read_buffer.len() / 4) * 4;
                        if n > 0
                            && midi_task_sender
                                .send(midi_read_buffer.split_to(n).freeze())
                                .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });

        let (display_sender, mut display_task_receiver) =
            tokio::sync::mpsc::unbounded_channel::<[u8; 128]>();

        let display_out_task = tokio::task::spawn(async move {
            while let Some(message) = display_task_receiver.recv().await {
                let _ = std::io::Write::write_all(&mut ep5_out, &message);

                while let Ok(message) = display_task_receiver.try_recv() {
                    let _ = std::io::Write::write_all(&mut ep5_out, &message);
                }

                let _ = ep5_out.flush();
            }
        });

        let state_in_task = tokio::task::spawn(async move {
            let mut state_read_buffer = BytesMut::with_capacity(4096);

            loop {
                match tokio::io::AsyncReadExt::read_buf(&mut ep4_in, &mut state_read_buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let n = (state_read_buffer.len() / 30) * 30;
                        let _ = state_read_buffer.split_to(n);

                        // if n > 0
                        //     && midi_task_sender
                        //         .send(midi_read_buffer.split_to(n).freeze())
                        //         .is_err()
                        // {
                        //     break;
                        // }
                    }
                }
            }
        });

        Ok(DeviceInterface {
            midi_sender,
            midi_receiver,

            display_sender,

            disconnection_signal: disconnection_receiver,

            join_handles: vec![midi_out_task, midi_in_task, display_out_task, state_in_task],
        })
    }

    pub async fn read_event(&mut self) -> DeviceInterfaceEvent {
        loop {
            tokio::select! {
                _ = &mut self.disconnection_signal => {
                    return DeviceInterfaceEvent::Disconnect;
                }
                midi_bytes = self.midi_receiver.recv() => {
                    if let Some(midi_bytes) = midi_bytes {
                        if let Some([raw_cin, channel, note, velocity]) = midi_bytes.get(0..4) {
                            let cin = raw_cin & 0x0f;

                            match cin {
                                0 => {}
                                cin if (0x8..=0xE).contains(&cin) => {
                                    let hi = *channel & 0xf0;

                                    if hi == 0xC0 || hi == 0xD0 {
                                        return DeviceInterfaceEvent::MidiMessage([*channel, *note, 0]);
                                    }

                                    return DeviceInterfaceEvent::MidiMessage([*channel, *note, *velocity]);
                                }
                                _ => {}
                            }
                        }
                    } else {
                        return DeviceInterfaceEvent::Disconnect;
                    }
                }
            }
        }
    }

    pub fn write_midi(&self, message: MidiMessage) {
        let _ = self.midi_sender.send(message);
    }

    pub fn write_jog_status(&self, message: [u8; 35]) {
        let mut full_message = [0u8; 128];

        full_message[0..35].copy_from_slice(&message);

        let _ = self.display_sender.send(full_message);
    }
}

impl Drop for DeviceInterface {
    fn drop(&mut self) {
        while let Some(handle) = self.join_handles.pop() {
            handle.abort();
        }
    }
}
