use rkyv::{Archive, Deserialize, Serialize};

use crate::types::deck::DeckState;

pub type DeckUpdateEvent = Box<
    dyn FnOnce(&mut DeckState, &mut tokio::sync::mpsc::UnboundedSender<DeckControlEvent>) + Send,
>;

#[derive(Clone, Copy, Archive, Serialize, Deserialize, Debug)]
pub enum DeckControlEvent {
    // mixer
    MixerMasterGainSet { position: f32 },
    MixerMasterCuePress,
    MixerMasterMasterFXTargetPress,

    MixerMasterFXSelect { delta: f32 },
    MixerMasterFXParameterAdjust { delta: f32 },
    MixerMasterFXParameterSelectPress,
    MixerMasterFXBPMAdjust { delta: f32 },
    MixerMasterFXBPMSelectPress,
    MixerMasterFXDepthSet { position: f32 },
    MixerMasterFXEnablePress,

    MixerChannelFXPress { number: usize },

    MixerCrossfaderSet { position: f32 },

    MixerHeadphonesMixSet { position: f32 },
    MixerHeadphonesGainSet { position: f32 },

    // mixer channels
    MixerChannelGainSet { channel: usize, position: f32 },

    MixerChannelEqHighSet { channel: usize, position: f32 },
    MixerChannelEqMidSet { channel: usize, position: f32 },
    MixerChannelEqLowSet { channel: usize, position: f32 },

    MixerChannelFXSet { channel: usize, position: f32 },

    MixerChannelCuePress { channel: usize },
    MixerChannelMasterFXTargetPress { channel: usize },

    MixerChannelFaderSet { channel: usize, position: f32 },

    MixerChannelCrossfaderAssignAPress { channel: usize },
    MixerChannelCrossfaderAssignNonePress { channel: usize },
    MixerChannelCrossfaderAssignBPress { channel: usize },

    // player
    PlayerJogVelocitySet { channel: usize, velocity: f32 },
    PlayerJogSearchVelocitySet { channel: usize, velocity: f32 },
    PlayerJogTouchPress { channel: usize },
    PlayerJogTouchRelease { channel: usize },

    PlayerQuantizePress { channel: usize },
    PlayerSlipPress { channel: usize },

    PlayerPlayPress { channel: usize },
    PlayerReversePress { channel: usize },
    PlayerReverseRelease { channel: usize },
    PlayerSlipReversePress { channel: usize },
    PlayerSlipReverseRelease { channel: usize },
    PlayerCuePress { channel: usize },
    PlayerCueRelease { channel: usize },
    PlayerAltCuePress { channel: usize },
    PlayerAltCueRelease { channel: usize },

    PlayerBeatJumpBackwardRelease { channel: usize },
    PlayerLongBeatJumpBackwardRelease { channel: usize },
    PlayerBeatJumpForwardRelease { channel: usize },
    PlayerLongBeatJumpForwardRelease { channel: usize },

    PlayerBeatSyncPress { channel: usize },
    PlayerBPMSyncPress { channel: usize },
    PlayerKeySyncPress { channel: usize },
    PlayerMasterPress { channel: usize },

    PlayerTempoResetPress { channel: usize },
    PlayerTempoRangePress { channel: usize },
    PlayerMasterTempoPress { channel: usize },
    PlayerTempoSet { channel: usize, position: f32 },

    PlayerBeatLoopInPress { channel: usize },
    PlayerBeatLoopInAdjustPress { channel: usize },
    PlayerBeatLoopOutPress { channel: usize },
    PlayerBeatLoopOutAdjustPress { channel: usize },
    PlayerBeatLoopExitPress { channel: usize },
    PlayerReLoopPress { channel: usize },
    PlayerInstantLoopPress { channel: usize },

    PlayerPadPress { channel: usize, number: usize },
}
