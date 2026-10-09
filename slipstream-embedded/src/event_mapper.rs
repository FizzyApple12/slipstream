use libdj::types::control::DeckControlEvent;
use libio::types::controller::ControllerEvent;

pub enum UIControlEvent {
    // usb
    USBEjectPress { slot: usize },
    USBEjectRelease { slot: usize },

    // browser
    BrowserEncoderAdjust { delta: f32 },
    BrowserEncoderPress,

    BrowserBackPress,
    BrowserSourcePress,
    BrowserBrowsePress,
    BrowserPlaylistPress,
    BrowserSearchPress,

    MixerMasterFXSelectTouchPress,
    MixerMasterFXSelectTouchRelease,

    MixerChannelFXProximityPress,
    MixerChannelFXProximityRelease,
}

pub enum MappedEvent {
    UIControlEvent(UIControlEvent),
    DeckControlEvent(DeckControlEvent),
}

pub fn map_controller_event(event: ControllerEvent) -> MappedEvent {
    match event {
        ControllerEvent::USBEjectPress { slot } => {
            MappedEvent::UIControlEvent(UIControlEvent::USBEjectPress { slot })
        }
        ControllerEvent::USBEjectRelease { slot } => {
            MappedEvent::UIControlEvent(UIControlEvent::USBEjectRelease { slot })
        }
        ControllerEvent::BrowserEncoderAdjust { delta } => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserEncoderAdjust { delta })
        }
        ControllerEvent::BrowserEncoderPress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserEncoderPress)
        }
        ControllerEvent::BrowserBackPress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserBackPress)
        }
        ControllerEvent::BrowserSourcePress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserSourcePress)
        }
        ControllerEvent::BrowserBrowsePress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserBrowsePress)
        }
        ControllerEvent::BrowserPlaylistPress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserPlaylistPress)
        }
        ControllerEvent::BrowserSearchPress => {
            MappedEvent::UIControlEvent(UIControlEvent::BrowserSearchPress)
        }
        ControllerEvent::MixerMasterGainSet { position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterGainSet { position })
        }
        ControllerEvent::MixerMasterCuePress => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterCuePress)
        }
        ControllerEvent::MixerMasterMasterFXTargetPress => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterMasterFXTargetPress)
        }
        ControllerEvent::MixerMasterFXSelect { delta } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXSelect { delta })
        }
        ControllerEvent::MixerMasterFXSelectTouchPress => {
            MappedEvent::UIControlEvent(UIControlEvent::MixerMasterFXSelectTouchPress)
        }
        ControllerEvent::MixerMasterFXSelectTouchRelease => {
            MappedEvent::UIControlEvent(UIControlEvent::MixerMasterFXSelectTouchRelease)
        }
        ControllerEvent::MixerMasterFXParameterAdjust { delta } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXParameterAdjust { delta })
        }
        ControllerEvent::MixerMasterFXParameterSelectPress => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXParameterSelectPress)
        }
        ControllerEvent::MixerMasterFXBPMAdjust { delta } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXBPMAdjust { delta })
        }
        ControllerEvent::MixerMasterFXBPMSelectPress => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXBPMSelectPress)
        }
        ControllerEvent::MixerMasterFXDepthSet { position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXDepthSet { position })
        }
        ControllerEvent::MixerMasterFXEnablePress => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerMasterFXEnablePress)
        }
        ControllerEvent::MixerChannelFXPress { number } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelFXPress { number })
        }
        ControllerEvent::MixerChannelFXProximityPress => {
            MappedEvent::UIControlEvent(UIControlEvent::MixerChannelFXProximityPress)
        }
        ControllerEvent::MixerChannelFXProximityRelease => {
            MappedEvent::UIControlEvent(UIControlEvent::MixerChannelFXProximityRelease)
        }
        ControllerEvent::MixerCrossfaderSet { position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerCrossfaderSet { position })
        }
        ControllerEvent::MixerHeadphonesMixSet { position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerHeadphonesMixSet { position })
        }
        ControllerEvent::MixerHeadphonesGainSet { position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerHeadphonesGainSet { position })
        }
        ControllerEvent::MixerChannelGainSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelGainSet {
                channel,
                position,
            })
        }
        ControllerEvent::MixerChannelEqHighSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelEqHighSet {
                channel,
                position,
            })
        }
        ControllerEvent::MixerChannelEqMidSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelEqMidSet {
                channel,
                position,
            })
        }
        ControllerEvent::MixerChannelEqLowSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelEqLowSet {
                channel,
                position,
            })
        }
        ControllerEvent::MixerChannelFXSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelFXSet { channel, position })
        }
        ControllerEvent::MixerChannelCuePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelCuePress { channel })
        }
        ControllerEvent::MixerChannelMasterFXTargetPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelMasterFXTargetPress {
                channel,
            })
        }
        ControllerEvent::MixerChannelFaderSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelFaderSet {
                channel,
                position,
            })
        }
        ControllerEvent::MixerChannelCrossfaderAssignAPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelCrossfaderAssignAPress {
                channel,
            })
        }
        ControllerEvent::MixerChannelCrossfaderAssignNonePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelCrossfaderAssignNonePress {
                channel,
            })
        }
        ControllerEvent::MixerChannelCrossfaderAssignBPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::MixerChannelCrossfaderAssignBPress {
                channel,
            })
        }
        ControllerEvent::PlayerJogVelocitySet { channel, velocity } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerJogVelocitySet {
                channel,
                velocity,
            })
        }
        ControllerEvent::PlayerJogSearchVelocitySet { channel, velocity } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerJogSearchVelocitySet {
                channel,
                velocity,
            })
        }
        ControllerEvent::PlayerJogTouchPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerJogTouchPress { channel })
        }
        ControllerEvent::PlayerJogTouchRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerJogTouchRelease { channel })
        }
        ControllerEvent::PlayerQuantizePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerQuantizePress { channel })
        }
        ControllerEvent::PlayerSlipPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerSlipPress { channel })
        }
        ControllerEvent::PlayerPlayPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerPlayPress { channel })
        }
        ControllerEvent::PlayerReversePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerReversePress { channel })
        }
        ControllerEvent::PlayerReverseRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerReverseRelease { channel })
        }
        ControllerEvent::PlayerSlipReversePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerSlipReversePress { channel })
        }
        ControllerEvent::PlayerSlipReverseRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerSlipReverseRelease { channel })
        }
        ControllerEvent::PlayerCuePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerCuePress { channel })
        }
        ControllerEvent::PlayerCueRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerCueRelease { channel })
        }
        ControllerEvent::PlayerAltCuePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerAltCuePress { channel })
        }
        ControllerEvent::PlayerAltCueRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerAltCueRelease { channel })
        }
        ControllerEvent::PlayerBeatJumpBackwardRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatJumpBackwardRelease {
                channel,
            })
        }
        ControllerEvent::PlayerLongBeatJumpBackwardRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerLongBeatJumpBackwardRelease {
                channel,
            })
        }
        ControllerEvent::PlayerBeatJumpForwardRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatJumpForwardRelease {
                channel,
            })
        }
        ControllerEvent::PlayerLongBeatJumpForwardRelease { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerLongBeatJumpForwardRelease {
                channel,
            })
        }
        ControllerEvent::PlayerBeatSyncPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatSyncPress { channel })
        }
        ControllerEvent::PlayerBPMSyncPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBPMSyncPress { channel })
        }
        ControllerEvent::PlayerKeySyncPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerKeySyncPress { channel })
        }
        ControllerEvent::PlayerMasterPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerMasterPress { channel })
        }
        ControllerEvent::PlayerTempoResetPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerTempoResetPress { channel })
        }
        ControllerEvent::PlayerTempoRangePress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerTempoRangePress { channel })
        }
        ControllerEvent::PlayerMasterTempoPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerMasterTempoPress { channel })
        }
        ControllerEvent::PlayerTempoSet { channel, position } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerTempoSet { channel, position })
        }
        ControllerEvent::PlayerBeatLoopInPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatLoopInPress { channel })
        }
        ControllerEvent::PlayerBeatLoopInAdjustPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatLoopInAdjustPress { channel })
        }
        ControllerEvent::PlayerBeatLoopOutPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatLoopOutPress { channel })
        }
        ControllerEvent::PlayerBeatLoopOutAdjustPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatLoopOutAdjustPress {
                channel,
            })
        }
        ControllerEvent::PlayerBeatLoopExitPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerBeatLoopExitPress { channel })
        }
        ControllerEvent::PlayerReLoopPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerReLoopPress { channel })
        }
        ControllerEvent::PlayerInstantLoopPress { channel } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerInstantLoopPress { channel })
        }
        ControllerEvent::PlayerPadPress { channel, number } => {
            MappedEvent::DeckControlEvent(DeckControlEvent::PlayerPadPress { channel, number })
        }
    }
}
