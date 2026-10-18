pub mod resampler;

use std::{array, f32};

use libdsp::audio_loader::TrackAudioData;
use phase_vocoder_testing::{Config, Stretcher};
use timecode::Timecode;

use crate::{
    AUDIO_CHANNELS, AUDIO_NEUTRAL_GAIN_DB,
    audio::dsp_pipeline::track_processor::resampler::Resampler,
};

pub struct TrackProcessor {
    target_sample_rate: u32,

    stretch_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    flush_disposal_buffers: [Vec<f32>; AUDIO_CHANNELS],
    stretcher: Stretcher,
    stretcher_has_config: bool,

    resampler_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    resampler: Resampler,
}

impl TrackProcessor {
    #[allow(clippy::cast_precision_loss)]
    pub fn new(sample_rate: u32) -> TrackProcessor {
        let mut stretcher_config = Config::new(sample_rate as f32, AUDIO_CHANNELS);
        stretcher_config.formant_compensation = true;
        stretcher_config.formant_width_hz = 300.0;

        TrackProcessor {
            target_sample_rate: sample_rate,

            stretch_input_buffers: Default::default(),
            flush_disposal_buffers: Default::default(),
            stretcher: Stretcher::new(stretcher_config),
            stretcher_has_config: false,

            resampler_input_buffers: Default::default(),
            resampler: Resampler::default(),
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing,
        clippy::cast_lossless,
        clippy::needless_range_loop,
        clippy::too_many_arguments
    )]
    #[allow(clippy::println_empty_string)]
    pub fn perform_sample_stretch_interpolate(
        &mut self,
        track_data: &TrackAudioData,
        start_time: Timecode,
        end_time: Timecode,
        wrap_times: Option<(Timecode, Timecode, usize)>,
        stretch: bool,
        pitch: f32,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        for output_buffer in output_buffers.iter_mut() {
            if output_buffer.len() < number_samples {
                output_buffer.resize(number_samples, 0.0);
            }
        }

        let samples_retrieved = track_data.read_samples(
            AUDIO_NEUTRAL_GAIN_DB,
            start_time,
            end_time,
            wrap_times,
            &mut self.stretch_input_buffers,
        );

        let resample_ratio = track_data.sample_rate as f64 / self.target_sample_rate as f64;

        let stretch_output_samples = if stretch {
            (number_samples as f64 * resample_ratio).round() as usize
        } else {
            samples_retrieved
        };

        for input_buffer in &mut self.resampler_input_buffers {
            if input_buffer.len() < stretch_output_samples {
                input_buffer.resize(stretch_output_samples, 0.0);
            }
        }

        if samples_retrieved == 0 {
            for input_buffer in &mut self.resampler_input_buffers {
                input_buffer.fill(0.0);
            }
        } else {
            for buffer in &mut self.resampler_input_buffers {
                buffer.clear();
            }

            self.stretcher
                .set_speed((samples_retrieved as f64) / (stretch_output_samples as f64));
            self.stretcher.set_pitch_semitones(pitch as f64);

            self.stretcher.push(&array::from_fn::<_, 2, _>(|index| {
                &self.stretch_input_buffers[index][0..samples_retrieved]
            }));

            self.stretcher
                .pull(&mut self.resampler_input_buffers, stretch_output_samples);
        }

        self.resampler.process(
            &self.resampler_input_buffers,
            stretch_output_samples,
            output_buffers,
            number_samples,
        );
    }

    pub fn reset(&mut self) {
        if self.stretcher_has_config {
            self.stretcher.flush(&mut self.flush_disposal_buffers);
            self.stretcher.reset();
        }

        self.resampler.reset();
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn setup(&mut self, track_data: &TrackAudioData) {
        let mut stretcher_config = Config::new(track_data.sample_rate as f32, AUDIO_CHANNELS);
        stretcher_config.formant_compensation = true;
        stretcher_config.formant_width_hz = 300.0;
        self.stretcher = Stretcher::new(stretcher_config);
        self.stretcher.reset();

        self.stretcher_has_config = true;

        self.reset();
    }
}
