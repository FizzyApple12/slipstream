pub mod effects;

use crate::{
    audio::dsp_pipeline::filters::fx::effects::{
        bitcrush::Bitcrush, dub_echo::DubEcho, filter::Filter, noise::Noise, pitch::Pitch,
        space::Space,
    },
    types::deck::{ChannelFXEffect, MasterFXEffect},
};

pub struct MasterFX<const CHANNELS: usize> {}

impl<const CHANNELS: usize> MasterFX<CHANNELS> {
    pub fn new(_sample_rate: u32) -> Self {
        Self {}
    }

    pub fn set_effect(&mut self, _effect: MasterFXEffect) {}

    pub fn set_parameter(&mut self, _parameter: f32) {}

    pub fn set_depth(&mut self, _percent: f32) {}

    pub fn set_bpm(&mut self, _bpm: f32) {}

    #[inline]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        input
    }

    pub fn reset(&mut self) {}
}

pub struct ChannelFX<const CHANNELS: usize> {
    effect: ChannelFXEffect,

    noise: Noise<CHANNELS>,

    filter: Filter<CHANNELS>,
    bitcrush: Bitcrush<CHANNELS>,
    pitch: Pitch<CHANNELS>,

    dub_echo: DubEcho<CHANNELS>,
    space: Space<CHANNELS>,
}

impl<const CHANNELS: usize> ChannelFX<CHANNELS> {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            effect: ChannelFXEffect::None,

            noise: Noise::<CHANNELS>::new(sample_rate),

            filter: Filter::<CHANNELS>::new(sample_rate),
            bitcrush: Bitcrush::<CHANNELS>::new(sample_rate),
            pitch: Pitch::<CHANNELS>::new(sample_rate),

            dub_echo: DubEcho::<CHANNELS>::new(sample_rate),
            space: Space::<CHANNELS>::new(sample_rate),
        }
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        self.noise.set_parameter(parameter);

        self.filter.set_parameter(parameter);
        self.bitcrush.set_parameter(parameter);
        self.pitch.set_parameter(parameter);

        self.dub_echo.set_parameter(parameter);
        self.space.set_parameter(parameter);
    }

    pub fn set_effect(&mut self, effect: ChannelFXEffect) {
        self.effect = effect;
    }

    #[inline]
    pub fn process_pre_eq(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let noise = self.noise.process(input);

        match self.effect {
            ChannelFXEffect::Noise => noise,
            _ => input,
        }
    }

    #[inline]
    pub fn process_pre_fader(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let filter = self.filter.process(input);
        let bitcrush = self.bitcrush.process(input);
        let pitch = self.pitch.process(input);

        match self.effect {
            ChannelFXEffect::Filter => filter,
            ChannelFXEffect::Bitcrush => bitcrush,
            ChannelFXEffect::Pitch => pitch,
            _ => input,
        }
    }

    #[inline]
    pub fn process_pre_mixer(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        let dub_echo = self.dub_echo.process(input);
        let space = self.space.process(input);

        match self.effect {
            ChannelFXEffect::DubEcho => dub_echo,
            ChannelFXEffect::Space => space,
            _ => input,
        }
    }

    pub fn reset(&mut self) {
        self.noise.reset();

        self.filter.reset();
        self.bitcrush.reset();
        self.pitch.reset();

        self.dub_echo.reset();
        self.space.reset();
    }
}
