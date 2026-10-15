use libdj::{AUDIO_CHANNELS, types::deck::DeckState};
use rkyv::{Archive, Deserialize, Serialize};

use crate::types::controller::ControllerEvent;

pub const IPC_DIR: &str = "/tmp/slipstream_ipc/";

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct DriverInfo {
    pub id: String,
    pub name: String,
    pub update_rate_hz: usize,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum DriverMessage {
    Info(DriverInfo),
    Event(ControllerEvent),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum HostMessage {
    GetInfo,
    Heartbeat,
    UpdateDeckState(DeckState),
    UpdateCurrentMeter([[f32; AUDIO_CHANNELS]; 4]),
}
