use super::{Fetcher, SampleBuffer};
use crate::DEFAULT_SAMPLE_RATE;
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Errors which can occur while creating [crate::fetcher::SystemAudioFetcher].
#[derive(thiserror::Error, Debug)]
pub enum SystemAudioError {
    /// No default audio device could be found to fetch from.
    #[error("Couldn't retrieve default output dev")]
    NoDefaultDevice,

    /// No default configuration could be found of the default output device.
    #[error("Couldn't retrieve any config of the output stream of the default device.")]
    NoAvailableOutputConfigs,

    #[error("Couldn't get supported output config of device: {0}")]
    SupportedStreamConfigError(#[from] cpal::SupportedStreamConfigsError),

    #[error("Couldn't build an audio stream:\n{0}")]
    BuildOutputStreamError(#[from] cpal::BuildStreamError),
}

pub struct Descriptor {
    pub device: cpal::Device,
    pub sample_rate: cpal::SampleRate,
    pub sample_format: Option<cpal::SampleFormat>,
    pub amount_channels: Option<u16>,
}

impl Default for Descriptor {
    fn default() -> Self {
        let device = crate::util::get_default_device(crate::util::DeviceType::Output)
            .expect("Default output device is set in the system");

        Self {
            device,
            sample_rate: DEFAULT_SAMPLE_RATE,
            sample_format: None,
            amount_channels: None,
        }
    }
}

/// Fetcher for the system audio.
pub struct SystemAudio {
    sample_buffer: Arc<Mutex<SampleBuffer>>,
    channels: u16,
    _stream: cpal::Stream,
}

impl SystemAudio {
    pub fn new(desc: &Descriptor) -> Result<Self, SystemAudioError> {
        let device = &desc.device;
        let stream_config = {
            let mut matching_configs: Vec<_> = desc
                .device
                .supported_output_configs()?
                .filter(|conf| {
                    let matching_sample_format = desc
                        .sample_format
                        .map(|sample_format| sample_format == conf.sample_format())
                        .unwrap_or(true);
                    let matching_amount_channels = desc
                        .amount_channels
                        .map(|amount| amount == conf.channels())
                        .unwrap_or(true);

                    matching_sample_format && matching_amount_channels
                })
                .collect();

            matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));
            let supported_stream_config = matching_configs
                .into_iter()
                .next()
                .ok_or(SystemAudioError::NoAvailableOutputConfigs)?;

            supported_stream_config
                .try_with_sample_rate(desc.sample_rate)
                .unwrap_or(supported_stream_config.with_max_sample_rate())
                .config()
        };

        let sample_rate = stream_config.sample_rate;
        let channels = stream_config.channels;

        debug!("Stream config: {:#?}", stream_config);

        let sample_buffer = Arc::new(Mutex::new(SampleBuffer::new(sample_rate)));

        let stream = {
            let stream = device.build_input_stream(
                &stream_config,
                {
                    let buffer = sample_buffer.clone();
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer.lock().unwrap();
                        buf.push_before(data);
                    }
                },
                |err| panic!("`shady-audio`: {}", err),
                None,
            )?;
            stream.play().expect("Start listening to audio");
            stream
        };

        Ok(Self {
            _stream: stream,
            channels,
            sample_buffer,
        })
    }
}

impl Drop for SystemAudio {
    /// Closes the audio stream before it gets dropped.
    ///
    /// **Panics** if it couldn't close the stream correctly.
    fn drop(&mut self) {
        self._stream.pause().expect("Stop stream");
    }
}

impl Fetcher for SystemAudio {
    fn sample_buffer(&self) -> Arc<Mutex<SampleBuffer>> {
        self.sample_buffer.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

// #[instrument(skip_all)]
// fn default_output_config(
//     device: &cpal::Device,
// ) -> Result<SupportedStreamConfigRange, SystemAudioError> {
//     let mut matching_configs: Vec<_> = device
//         .supported_output_configs()
//         .expect(concat![
//             "Eh... somehow `shady-audio` couldn't get any supported output configs of your audio device.\n",
//             "Could it be that you are running \"pure\" pulseaudio?\n",
//             "Only ALSA and JACK are supported for audio processing :("
//         ])
//         .collect();

//     matching_configs.sort_by(|a, b| a.cmp_default_heuristics(b));
//     matching_configs
//         .into_iter()
//         .next()
//         .ok_or(SystemAudioError::NoAvailableOutputConfigs)
// }
