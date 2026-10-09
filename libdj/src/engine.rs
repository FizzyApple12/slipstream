use std::{
    sync::{Arc, nonpoison::RwLock},
    time::Duration,
};

use libdsp::audio_loader::TrackAudioData;
use thiserror::Error;
use tokio::{sync::watch::Ref, task::JoinHandle};

use crate::{
    audio::{
        AudioPipelineCreationError, AudioPipelineRuntimeError, AudioPipelineRuntimeStatus,
        AudioSystem, AudioSystemChannels, FindDeviceError,
    },
    types::{
        control::{DeckControlEvent, DeckUpdateEvent},
        deck::DeckState,
    },
};

#[derive(Error, Debug, Clone, Copy)]
pub enum DJEngineRuntimeError {
    #[error("Audio Host Disconnected")]
    AudioHostDisconnected,

    #[error("Audio Device Disconnected")]
    AudioDeviceDisconnected,

    #[error("Invalid Input")]
    InvalidInput,

    #[error("Insufficient Permissions")]
    InsufficientPermissions,

    #[error("Resources Exhausted")]
    ResourcesExhausted,

    #[error("Audio Stream Invalidated")]
    AudioStreamInvalidated,

    #[error("Audio Configuration Not Supported")]
    UnsupportedAudioConfiguration,

    #[error("Audio Operation Not Supported")]
    UnsupportedAudioOperation,

    #[error("A fatal audio error occurred")]
    Fatal,
}

#[derive(Clone, Copy)]
pub enum DJEngineRuntimeStatus {
    AudioInitialising,
    AudioInitError(AudioPipelineCreationError),
    Running,
    AudioRuntimeError(AudioPipelineRuntimeError),
}

#[derive(Clone)]
pub struct DJEngineHandle {
    runtime_state_receiver: tokio::sync::watch::Receiver<DJEngineRuntimeStatus>,
    deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,

    deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdateEvent>,
    control_event_sender: tokio::sync::mpsc::UnboundedSender<DeckControlEvent>,

    loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
}

impl DJEngineHandle {
    pub async fn wait_for_new_deck_state_changed(&mut self) {
        let _ = self.deck_state_receiver.changed().await;
    }

    pub async fn wait_for_new_deck_state(&mut self) -> Ref<'_, DeckState> {
        let _ = self.deck_state_receiver.changed().await;

        self.deck_state_receiver.borrow_and_update()
    }

    pub fn deck_state_changed(&self) -> bool {
        self.deck_state_receiver.has_changed().unwrap_or(false)
    }

    pub fn get_deck_state(&mut self) -> Ref<'_, DeckState> {
        self.deck_state_receiver.borrow_and_update()
    }

    pub async fn wait_for_new_status(&mut self) -> DJEngineRuntimeStatus {
        let _ = self.runtime_state_receiver.changed().await;

        *self.runtime_state_receiver.borrow_and_update()
    }

    pub fn status_changed(&self) -> bool {
        self.runtime_state_receiver.has_changed().unwrap_or(false)
    }

    pub fn get_status(&mut self) -> DJEngineRuntimeStatus {
        *self.runtime_state_receiver.borrow_and_update()
    }

    pub fn send_update_event(&self, update_event: DeckUpdateEvent) {
        let _ = self.deck_update_sender.send(update_event);
    }

    pub fn send_control_event(&self, control_event: DeckControlEvent) {
        let _ = self.control_event_sender.send(control_event);
    }

    pub fn send_loaded_track(&self, channel: usize, track_audio_data: Option<Box<TrackAudioData>>) {
        let _ = self.loaded_track_sender.send((channel, track_audio_data));
    }
}

pub struct DJEngine {
    manager_handle: JoinHandle<()>,

    runtime_state_receiver: tokio::sync::watch::Receiver<DJEngineRuntimeStatus>,
    deck_state_receiver: tokio::sync::watch::Receiver<DeckState>,

    deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdateEvent>,
    control_event_sender: tokio::sync::mpsc::UnboundedSender<DeckControlEvent>,

    loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
}

impl DJEngine {
    // this will be replaced in the future with a proper DJ Engine builder
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (deck_state_sender, deck_state_receiver) =
            tokio::sync::watch::channel(DeckState::default());
        let (control_event_sender, control_event_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (deck_update_sender, deck_update_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (loaded_track_sender, loaded_track_receiver) = tokio::sync::mpsc::unbounded_channel();

        let (runtime_state_sender, runtime_state_receiver) =
            tokio::sync::watch::channel(DJEngineRuntimeStatus::AudioInitialising);

        let channels = Arc::new(RwLock::new(AudioSystemChannels {
            deck_state_sender,
            control_event_sender: control_event_sender.clone(),
            control_event_receiver,
            deck_update_receiver,
            loaded_track_receiver,
        }));

        let manager_handle = tokio::task::spawn(async move {
            'system_loop: loop {
                let _ = runtime_state_sender.send(DJEngineRuntimeStatus::AudioInitialising);

                let mut audio_system = AudioSystem::default();

                'pipeline_loop: loop {
                    let cloned_channels = channels.clone();

                    let device =
                        match audio_system.find_output_device_by_name_substring("DDJ-FLX10") {
                            Ok(device) => device,
                            Err(err) => {
                                match err {
                                    FindDeviceError::HostNotAvailable => {
                                        let _ = runtime_state_sender.send(
                                            DJEngineRuntimeStatus::AudioInitError(
                                                AudioPipelineCreationError::HostNotAvailable,
                                            ),
                                        );
                                    }
                                    FindDeviceError::Fatal => {
                                        let _ = runtime_state_sender.send(
                                            DJEngineRuntimeStatus::AudioInitError(
                                                AudioPipelineCreationError::Fatal,
                                            ),
                                        );
                                    }
                                }

                                drop(audio_system);

                                tokio::time::sleep(Duration::from_millis(1000)).await;

                                continue 'system_loop;
                            }
                        };

                    let mut pipeline_handle =
                        match audio_system.create_full_audio_pipeline(cloned_channels, device) {
                            Ok(pipeline_handle) => pipeline_handle,
                            Err(err) => {
                                let _ = runtime_state_sender
                                    .send(DJEngineRuntimeStatus::AudioInitError(err));

                                match err {
                                    AudioPipelineCreationError::HostNotAvailable
                                    | AudioPipelineCreationError::Fatal
                                    | AudioPipelineCreationError::InsufficientPermissions => {
                                        drop(audio_system);

                                        tokio::time::sleep(Duration::from_millis(1000)).await;

                                        continue 'system_loop;
                                    }
                                    _ => {
                                        tokio::time::sleep(Duration::from_millis(500)).await;

                                        continue 'pipeline_loop;
                                    }
                                }
                            }
                        };

                    let _ = runtime_state_sender.send(DJEngineRuntimeStatus::Running);

                    'status_loop: loop {
                        let AudioPipelineRuntimeStatus::Err(err) =
                            pipeline_handle.wait_for_new_status().await
                        else {
                            continue 'status_loop;
                        };

                        let _ = runtime_state_sender
                            .send(DJEngineRuntimeStatus::AudioRuntimeError(err));

                        match err {
                            AudioPipelineRuntimeError::HostDisconnected
                            | AudioPipelineRuntimeError::InsufficientPermissions
                            | AudioPipelineRuntimeError::ResourcesExhausted
                            | AudioPipelineRuntimeError::Fatal => {
                                drop(audio_system);

                                tokio::time::sleep(Duration::from_millis(1000)).await;

                                continue 'system_loop;
                            }
                            _ => {
                                tokio::time::sleep(Duration::from_millis(500)).await;

                                continue 'pipeline_loop;
                            }
                        }
                    }
                }
            }
        });

        DJEngine {
            manager_handle,

            runtime_state_receiver,
            deck_state_receiver,

            deck_update_sender,
            control_event_sender,

            loaded_track_sender,
        }
    }

    pub fn subscribe(&self) -> DJEngineHandle {
        DJEngineHandle {
            runtime_state_receiver: self.runtime_state_receiver.clone(),
            deck_state_receiver: self.deck_state_receiver.clone(),
            deck_update_sender: self.deck_update_sender.clone(),
            control_event_sender: self.control_event_sender.clone(),
            loaded_track_sender: self.loaded_track_sender.clone(),
        }
    }

    pub async fn wait_for_new_deck_state(&mut self) -> Ref<'_, DeckState> {
        let _ = self.deck_state_receiver.changed().await;

        self.deck_state_receiver.borrow_and_update()
    }

    pub fn deck_state_changed(&self) -> bool {
        self.deck_state_receiver.has_changed().unwrap_or(false)
    }

    pub fn get_deck_state(&mut self) -> Ref<'_, DeckState> {
        self.deck_state_receiver.borrow_and_update()
    }

    pub async fn wait_for_new_status(&mut self) -> DJEngineRuntimeStatus {
        let _ = self.runtime_state_receiver.changed().await;

        *self.runtime_state_receiver.borrow_and_update()
    }

    pub fn status_changed(&self) -> bool {
        self.runtime_state_receiver.has_changed().unwrap_or(false)
    }

    pub fn get_status(&mut self) -> DJEngineRuntimeStatus {
        *self.runtime_state_receiver.borrow_and_update()
    }

    pub fn send_update_event(&self, update_event: DeckUpdateEvent) {
        let _ = self.deck_update_sender.send(update_event);
    }

    pub fn send_control_event(&self, control_event: DeckControlEvent) {
        let _ = self.control_event_sender.send(control_event);
    }

    pub fn send_loaded_track(&self, channel: usize, track_audio_data: Option<Box<TrackAudioData>>) {
        let _ = self.loaded_track_sender.send((channel, track_audio_data));
    }
}

impl Drop for DJEngine {
    fn drop(&mut self) {
        self.manager_handle.abort();
    }
}
