pub struct Pitch<const CHANNELS: usize> {
    buffers: Vec<[f32; CHANNELS]>,
    position: usize,

    phase: f32,

    window: usize,

    parameter: f32,
}

impl<const CHANNELS: usize> Pitch<CHANNELS> {
    #[allow(clippy::new_without_default)]
    pub fn new(sample_rate: u32) -> Self {
        let window = (sample_rate as usize) / 100;

        let mut new = Self {
            buffers: vec![[0.0; CHANNELS]; window * 2],
            position: 0,

            phase: 0.0,

            window,

            parameter: 0.0,
        };

        new.set_parameter(0.0);

        new
    }

    pub fn set_parameter(&mut self, parameter: f32) {
        if parameter > 0.0 {
            self.parameter = parameter + 1.0;
        } else {
            self.parameter = (parameter * 0.5) + 1.0;
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::indexing_slicing
    )]
    fn read_delay(&self, delay: f32) -> [f32; CHANNELS] {
        let position = self.position as f32 - delay;
        let position = position.rem_euclid(self.buffers.len() as f32);
        let position_fractional = position.fract();

        let i0 = position.floor() as usize % self.buffers.len();
        let i1 = (i0 + 1) % self.buffers.len();

        let mut out = [0.0; CHANNELS];

        for ((i0_buffer, i1_buffer), out) in
            self.buffers[i0].iter().zip(&self.buffers[i1]).zip(&mut out)
        {
            *out = i0_buffer * (1.0 - position_fractional) + i1_buffer * position_fractional;
        }

        out
    }

    #[inline]
    #[allow(clippy::cast_precision_loss, clippy::indexing_slicing)]
    pub fn process(&mut self, input: [f32; CHANNELS]) -> [f32; CHANNELS] {
        self.buffers[self.position] = input;

        let window = self.window as f32;

        let delay_1 = self.phase * window;
        let delay_1 = self.read_delay(delay_1);

        let delay_2 = ((self.phase + 0.5) % 1.0) * window;
        let delay_2 = self.read_delay(delay_2);

        let crossfade = 1.0 - (2.0 * self.phase - 1.0).abs();

        let mut out = [0.0; CHANNELS];

        for ((delay_1, delay_2), out) in delay_1.iter().zip(&delay_2).zip(&mut out) {
            *out = delay_1 * crossfade + delay_2 * (1.0 - crossfade);
        }

        self.position = (self.position + 1) % self.buffers.len();
        self.phase += (1.0 - self.parameter) / window;
        self.phase = self.phase.rem_euclid(1.0);

        out
    }

    pub fn reset(&mut self) {
        self.buffers = vec![[0.0; CHANNELS]; self.window * 2];

        self.position = 0;

        self.phase = 0.0;
    }
}
