use std::{
    collections::HashMap,
    io,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use interprocess::local_socket::{
    GenericFilePath, ToFsName, tokio::Stream, traits::tokio::Stream as _,
};
use libdj::{AUDIO_CHANNELS, types::deck::DeckState};
use log::{debug, info, warn};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, RemoveKind},
};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::Mutex,
    task::{self, JoinHandle},
    time::timeout,
};

use crate::types::{
    controller::ControllerEvent,
    ipc::{ArchivedDriverMessage, DriverInfo, DriverMessage, HostMessage, IPC_DIR},
};

pub enum IPCMessage {
    UpdateDeckState(DeckState),
    UpdateCurrentMeter([[f32; AUDIO_CHANNELS]; 4]),
}

#[derive(Error, Debug)]
pub enum ListenError {
    #[error("Cannot create file: {0}")]
    CreationFailed(std::io::Error),

    #[error("Listen failed: {0}")]
    ListenFailed(notify::Error),

    #[error("Failed to index available drivers: {0}")]
    IndexingDriversFailed(IndexDriversError),
}

#[derive(Error, Debug)]
pub enum SelectDriverError {
    #[error("No driver has that ID")]
    NoDriver,
}

pub struct HostIPC {
    pub drivers: Arc<Mutex<HashMap<String, DriverInfo>>>,

    driver_connection_sender: tokio::sync::mpsc::UnboundedSender<Option<String>>,
    active_driver_watch: tokio::sync::watch::Receiver<Option<String>>,

    ipc_message_sender: tokio::sync::mpsc::UnboundedSender<IPCMessage>,
    controller_event_receiver: tokio::sync::mpsc::UnboundedReceiver<ControllerEvent>,

    socket_watcher_task_handle: Option<JoinHandle<()>>,
    stream_relay_task_handle: Option<JoinHandle<()>>,
}

pub struct HostIPCHandle {
    pub drivers: Arc<Mutex<HashMap<String, DriverInfo>>>,

    driver_connection_sender: tokio::sync::mpsc::UnboundedSender<Option<String>>,
    active_driver_watch: tokio::sync::watch::Receiver<Option<String>>,

    ipc_message_sender: tokio::sync::mpsc::UnboundedSender<IPCMessage>,
}

impl HostIPC {
    pub fn subscribe(&self) -> HostIPCHandle {
        HostIPCHandle {
            drivers: self.drivers.clone(),

            driver_connection_sender: self.driver_connection_sender.clone(),
            active_driver_watch: self.active_driver_watch.clone(),

            ipc_message_sender: self.ipc_message_sender.clone(),
        }
    }

    pub async fn listen() -> Result<HostIPC, ListenError> {
        info!("Listening for driver IPC sockets at {IPC_DIR}");

        let ipc_folder = PathBuf::from(IPC_DIR);

        if !ipc_folder.exists() {
            fs::create_dir_all(ipc_folder.clone())
                .await
                .map_err(ListenError::CreationFailed)?;
        }

        info!("Setting watch on socket folder");

        let (mut watcher, mut event_receiver) =
            async_watcher().map_err(ListenError::ListenFailed)?;

        watcher
            .watch(&ipc_folder, RecursiveMode::Recursive)
            .map_err(ListenError::ListenFailed)?;

        let driver_tab = Arc::new(Mutex::new(HashMap::new()));
        let cloned_driver_tab = driver_tab.clone();

        let cloned_ipc_folder = ipc_folder.clone();

        let (driver_connection_sender, mut driver_connection_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Option<String>>();
        let (active_driver_watch_sender, active_driver_watch_receiver) =
            tokio::sync::watch::channel(None);

        let socket_watcher_task_handle = task::spawn(async move {
            info!("Performing initial driver index");

            while let Err(err) = index_drivers(&cloned_ipc_folder, &cloned_driver_tab).await {
                warn!("Failed to index drivers: {err}");

                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            loop {
                if let Some(notify_event) = event_receiver.recv().await {
                    match notify_event {
                        Ok(event)
                            if event.kind == EventKind::Create(CreateKind::File)
                                || event.kind == EventKind::Remove(RemoveKind::File) =>
                        {
                            info!("IPC folder updated, re-indexing drivers");

                            while let Err(err) =
                                index_drivers(&cloned_ipc_folder, &cloned_driver_tab).await
                            {
                                warn!("Failed to index drivers: {err}");

                                tokio::time::sleep(Duration::from_secs(2)).await;
                            }
                        }
                        Err(err) => {
                            warn!("Driver watcher failed: {err}");

                            loop {
                                info!("Attempting to re-watch socket folder");

                                if let Ok((mut watcher, new_event_receiver)) = async_watcher()
                                    && watcher
                                        .watch(&cloned_ipc_folder, RecursiveMode::Recursive)
                                        .is_ok()
                                {
                                    event_receiver = new_event_receiver;

                                    break;
                                }

                                warn!("Re-watch failed, re-indexing drivers for safety");

                                let _ = index_drivers(&cloned_ipc_folder, &cloned_driver_tab).await;

                                tokio::time::sleep(Duration::from_secs(5)).await;
                            }

                            warn!("Watch reconnected, re-indexing drivers");

                            while let Err(err) =
                                index_drivers(&cloned_ipc_folder, &cloned_driver_tab).await
                            {
                                warn!("Failed to index drivers: {err}");

                                tokio::time::sleep(Duration::from_secs(2)).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        let cloned_driver_tab = driver_tab.clone();

        let (ipc_message_sender, mut ipc_message_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (controller_event_sender, controller_event_receiver) =
            tokio::sync::mpsc::unbounded_channel();

        let stream_relay_task = tokio::spawn(async move {
            let mut selected_driver = None;

            'core_loop: loop {
                info!("Waiting for driver to be selected");

                while selected_driver.is_none() {
                    if let Some(new_selected_driver) = driver_connection_receiver.recv().await {
                        let Some(new_selected_driver) = new_selected_driver else {
                            selected_driver = None;
                            let _ = active_driver_watch_sender.send(None);

                            continue;
                        };

                        let locked_driver_tab = cloned_driver_tab.lock().await;

                        if !locked_driver_tab.contains_key(&new_selected_driver) {
                            warn!("No driver found: {new_selected_driver}");

                            selected_driver = None;
                            let _ = active_driver_watch_sender.send(None);

                            continue;
                        }

                        info!("New driver selected: {new_selected_driver}");

                        selected_driver = Some(new_selected_driver);
                    }
                }

                'reconnection_loop: for i in 0..3 {
                    let Some(currently_selected_driver) = &selected_driver else {
                        continue 'reconnection_loop;
                    };

                    info!("Performing connection attempt {i} to {currently_selected_driver}");

                    let ipc_file: PathBuf =
                        ipc_folder.join(format!("{currently_selected_driver}.socket"));

                    let socket_name = match ipc_file.to_fs_name::<GenericFilePath>() {
                        Ok(socket_name) => socket_name,
                        Err(err) => {
                            warn!("Failed to connect to driver {currently_selected_driver}: {err}");

                            tokio::time::sleep(Duration::from_secs(1)).await;

                            continue 'reconnection_loop;
                        }
                    };

                    let connection = match Stream::connect(socket_name).await {
                        Ok(connection) => connection,
                        Err(err) => {
                            warn!("Failed to connect to driver {currently_selected_driver}: {err}");

                            tokio::time::sleep(Duration::from_secs(1)).await;

                            continue 'reconnection_loop;
                        }
                    };

                    let (receiver, mut sender) = connection.split();

                    let mut receiver = BufReader::new(receiver);
                    let mut received_message = Vec::new();

                    let mut heartbeat_timeout = tokio::time::interval(Duration::from_millis(250));

                    while ipc_message_receiver.try_recv().is_ok() {}

                    let _ =
                        active_driver_watch_sender.send(Some(currently_selected_driver.clone()));

                    info!("Connected to {currently_selected_driver}");

                    loop {
                        tokio::select! {
                            result = tokio::time::timeout(Duration::from_secs(1), receiver.read_until(b'\n', &mut received_message)) => {
                                match result {
                                    Ok(Ok(0)) => {
                                        warn!("Stream hit EOF, disconnecting");

                                        break;
                                    }
                                    Ok(Ok(_)) => {
                                        let _ = received_message.try_remove(received_message.len() - 1);

                                        if let Ok(serialized_message) = STANDARD.decode(&received_message)
                                            && let Ok(archived) = rkyv::access::<ArchivedDriverMessage, rkyv::rancor::Error>(&serialized_message)
                                            && let Ok(deserialized) = rkyv::deserialize::<DriverMessage, rkyv::rancor::Error>(archived) {
                                            debug!("Incoming Message: {deserialized:#?}");

                                            match deserialized {
                                                DriverMessage::Info(_) => {},
                                                DriverMessage::Event(controller_event) => {
                                                    let _ = controller_event_sender.send(controller_event);
                                                },
                                            }
                                        }

                                        received_message.clear();
                                    },
                                    Ok(Err(err)) => {
                                        warn!("Failed to receive message, reconnecting: {err}");

                                        break;
                                    },
                                    Err(_) => {
                                        warn!("Receive message timed out, reconnecting");

                                        break;
                                    },
                                 }
                            }
                            Some(message) = ipc_message_receiver.recv() => {
                                if let Ok(serialized_message) = rkyv::to_bytes::<rkyv::rancor::Error>(&match message {
                                    IPCMessage::UpdateDeckState(deck_state) => HostMessage::UpdateDeckState(deck_state),
                                    IPCMessage::UpdateCurrentMeter(meter) => HostMessage::UpdateCurrentMeter(meter),
                                })
                                    && let Err(err) = sender.write_all((STANDARD.encode(serialized_message) + "\n").as_bytes()).await {
                                    warn!("Failed to send message, reconnecting: {err}");

                                    break;
                                }
                            }
                            _ = heartbeat_timeout.tick() => {
                                if let Ok(serialized_message) = rkyv::to_bytes::<rkyv::rancor::Error>(&HostMessage::Heartbeat)
                                    && let Err(err) = sender.write_all((STANDARD.encode(serialized_message) + "\n").as_bytes()).await {
                                    warn!("Failed to send heartbeat, reconnecting: {err}");

                                    break;
                                }
                            }
                            Some(new_selected_driver) = driver_connection_receiver.recv() => {
                                let Some(new_selected_driver) = new_selected_driver else {
                                    break 'reconnection_loop;
                                };

                                let locked_driver_tab = cloned_driver_tab.lock().await;

                                if !locked_driver_tab.contains_key(&new_selected_driver) {
                                    warn!("No driver found: {new_selected_driver}");

                                    break 'reconnection_loop;
                                }

                                if *currently_selected_driver == new_selected_driver {
                                    continue;
                                }

                                info!("New driver selected: {new_selected_driver}");

                                selected_driver = Some(new_selected_driver);
                                let _ = active_driver_watch_sender.send(None);

                                continue 'core_loop;
                            }
                        }
                    }

                    drop(sender);
                    drop(receiver);

                    tokio::time::sleep(Duration::from_millis(50)).await;
                }

                info!("Disconnected from driver");

                selected_driver = None;
                let _ = active_driver_watch_sender.send(None);
            }
        });

        Ok(HostIPC {
            drivers: driver_tab,

            driver_connection_sender,
            active_driver_watch: active_driver_watch_receiver,

            ipc_message_sender,
            controller_event_receiver,

            socket_watcher_task_handle: Some(socket_watcher_task_handle),
            stream_relay_task_handle: Some(stream_relay_task),
        })
    }

    pub fn get_current_driver_id(&self) -> Option<String> {
        self.active_driver_watch.borrow().clone()
    }

    pub async fn listen_for_driver_change(&mut self) -> Option<String> {
        let _ = self.active_driver_watch.changed().await;

        self.active_driver_watch.borrow_and_update().clone()
    }

    pub fn select_driver_id(&self, id: Option<String>) {
        let _ = self.driver_connection_sender.send(id);
    }

    pub fn send_message(&mut self, event: IPCMessage) -> bool {
        if self.active_driver_watch.borrow().is_some() {
            self.ipc_message_sender.send(event).is_ok()
        } else {
            false
        }
    }

    pub async fn receive_event(&mut self) -> ControllerEvent {
        loop {
            if let Some(message) = self.controller_event_receiver.recv().await {
                return message;
            }
        }
    }
}

impl Drop for HostIPC {
    fn drop(&mut self) {
        if let Some(socket_watcher_task_handle) = self.socket_watcher_task_handle.take() {
            socket_watcher_task_handle.abort();
        }

        if let Some(stream_relay_task_handle) = self.stream_relay_task_handle.take() {
            stream_relay_task_handle.abort();
        }
    }
}

impl HostIPCHandle {
    pub fn get_current_driver_id(&self) -> Option<String> {
        self.active_driver_watch.borrow().clone()
    }

    pub async fn listen_for_driver_change(&mut self) -> Option<String> {
        let _ = self.active_driver_watch.changed().await;

        self.active_driver_watch.borrow_and_update().clone()
    }

    pub fn select_driver_id(&self, id: Option<String>) {
        let _ = self.driver_connection_sender.send(id);
    }

    pub fn send_message(&mut self, event: IPCMessage) -> bool {
        if self.active_driver_watch.borrow().is_some() {
            self.ipc_message_sender.send(event).is_ok()
        } else {
            false
        }
    }
}

fn async_watcher() -> notify::Result<(
    RecommendedWatcher,
    tokio::sync::mpsc::UnboundedReceiver<notify::Result<Event>>,
)> {
    let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

    let watcher = RecommendedWatcher::new(
        move |result| {
            let _ = event_sender.send(result);
        },
        Config::default(),
    )?;

    Ok((watcher, event_receiver))
}

#[derive(Error, Debug)]
pub enum IndexDriversError {
    #[error("Failed to read IPC directory: {0}")]
    ReadFailed(io::Error),
}

async fn index_drivers(
    folder: &Path,
    driver_tab: &Arc<Mutex<HashMap<String, DriverInfo>>>,
) -> Result<(), IndexDriversError> {
    let mut list = fs::read_dir(folder)
        .await
        .map_err(IndexDriversError::ReadFailed)?;

    let mut locked_driver_tab = driver_tab.lock().await;

    let mut drivers_to_prune = locked_driver_tab.clone();

    while let Ok(Some(entry)) = list.next_entry().await {
        debug!("Checking driver: {entry:#?}");

        let Ok(file_type) = entry.file_type().await else {
            debug!("Cannot convert file name");

            continue;
        };

        if !file_type.is_socket() {
            debug!("File is not socket");

            continue;
        }

        let Ok(file_name) = entry.file_name().into_string() else {
            debug!("Cannot convert file name");

            continue;
        };

        let Some((name, extension)) = file_name.rsplit_once('.') else {
            debug!("Cannot split file name");

            continue;
        };

        if !extension.contains("socket") {
            debug!("Not socket");

            continue;
        }

        if locked_driver_tab.contains_key(&name.to_string()) {
            #[allow(clippy::unnecessary_to_owned)]
            let _ = drivers_to_prune.remove(&name.to_string());

            debug!("Reindexed driver {name}");

            continue;
        }

        let driver_info = match query_metadata(folder, name).await {
            Ok(driver_info) => driver_info,
            Err(err) => {
                warn!("Failed to query driver info for {name}: {err}");

                continue;
            }
        };

        locked_driver_tab.insert(name.to_string(), driver_info);

        info!("Added driver {name}");
    }

    for driver_id in drivers_to_prune.keys() {
        info!("Removing driver {driver_id}");

        let _ = locked_driver_tab.remove(driver_id);
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum QueryDriverInfoError {
    #[error("Invalid address: {0}")]
    InvalidAddress(io::Error),

    #[error("Failed to serialize message: {0}")]
    SerializationFailed(rkyv::rancor::Error),

    #[error("Driver didn't respond to info attestation")]
    NoResponse,
}

async fn query_metadata(folder: &Path, id: &str) -> Result<DriverInfo, QueryDriverInfoError> {
    let ipc_file: PathBuf = folder.join(format!("{id}.socket"));

    let socket_name = ipc_file
        .to_fs_name::<GenericFilePath>()
        .map_err(QueryDriverInfoError::InvalidAddress)?;

    for i in 0..3 {
        info!("Probe attempt {i} for {id}");

        let connection =
            timeout(Duration::from_secs(1), Stream::connect(socket_name.clone())).await;

        let Ok(connection) = connection else {
            warn!("Connection timed out");

            tokio::time::sleep(Duration::from_millis(100)).await;

            continue;
        };

        let connection = match connection {
            Ok(connection) => connection,
            Err(err) => {
                warn!("Connection failed: {err}");

                tokio::time::sleep(Duration::from_millis(100)).await;

                continue;
            }
        };

        let mut connection = BufReader::new(connection);

        let mut received_message = Vec::new();

        let serialized_message = rkyv::to_bytes::<rkyv::rancor::Error>(&HostMessage::Heartbeat)
            .map_err(QueryDriverInfoError::SerializationFailed)?;

        if let Err(err) = connection
            .write_all((STANDARD.encode(serialized_message) + "\n").as_bytes())
            .await
        {
            warn!("Failed to send Heartbeat: {err}");

            tokio::time::sleep(Duration::from_millis(100)).await;

            continue;
        }

        let serialized_message = rkyv::to_bytes::<rkyv::rancor::Error>(&HostMessage::GetInfo)
            .map_err(QueryDriverInfoError::SerializationFailed)?;

        if let Err(err) = connection
            .write_all((STANDARD.encode(serialized_message) + "\n").as_bytes())
            .await
        {
            warn!("Failed to send GetInfo: {err}");

            tokio::time::sleep(Duration::from_millis(100)).await;

            continue;
        }
        if let Err(err) = connection.read_until(b'\n', &mut received_message).await {
            warn!("Failed to receive message: {err}");

            tokio::time::sleep(Duration::from_millis(100)).await;

            continue;
        }

        let _ = received_message.try_remove(received_message.len() - 1);

        let serialized_message = match STANDARD.decode(&received_message) {
            Ok(message) => message,
            Err(err) => {
                warn!("Failed to decode message: {err}");

                tokio::time::sleep(Duration::from_millis(100)).await;

                continue;
            }
        };

        let archived =
            match rkyv::access::<ArchivedDriverMessage, rkyv::rancor::Error>(&serialized_message) {
                Ok(message) => message,
                Err(err) => {
                    warn!("Failed to deserialize message: {err}");

                    tokio::time::sleep(Duration::from_millis(100)).await;

                    continue;
                }
            };

        let deserialized = match rkyv::deserialize::<DriverMessage, rkyv::rancor::Error>(archived) {
            Ok(message) => message,
            Err(err) => {
                warn!("Failed to deserialize message: {err}");

                tokio::time::sleep(Duration::from_millis(100)).await;

                continue;
            }
        };

        received_message.clear();

        let _ = connection.shutdown().await;

        if let DriverMessage::Info(driver_info) = deserialized {
            return Ok(driver_info);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(QueryDriverInfoError::NoResponse)
}
