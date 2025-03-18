use std::{num::NonZero, ops::Range};

use shady_audio::{BarProcessor, Config, SampleProcessor};
use tracing::error;

use super::Resource;

const _DESCRIPTION: &str = "\
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.";

pub struct AudioDesc<'a> {
    pub device: &'a wgpu::Device,
    pub processor: &'a SampleProcessor,
    pub amount_bars: NonZero<usize>,
    pub freq_range: Range<NonZero<u16>>,
    pub binding: u32,
}

impl<'a> AsRef<AudioDesc<'a>> for AudioDesc<'a> {
    fn as_ref(&self) -> &AudioDesc<'a> {
        self
    }
}

pub struct Audio {
    bar_processor: BarProcessor,
    bar_values: Box<[f32]>,
    buffer: wgpu::Buffer,
    binding: u32,
}

impl Audio {
    pub fn new<'a>(desc_ref: impl AsRef<AudioDesc<'a>>) -> Self {
        let desc = desc_ref.as_ref();

        let amount_bars = usize::from(desc.amount_bars);

        let audio_buffer = vec![0f32; amount_bars].into_boxed_slice();

        let buffer = desc.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Audio buffer"),
            size: (std::mem::size_of::<f32>() * amount_bars) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bar_processor = {
            let mut bar_processor = BarProcessor::new(
                desc.processor,
                Config {
                    amount_bars: desc.amount_bars,
                    freq_range: desc.freq_range.clone(),
                    ..Default::default()
                },
            );

            // check if the frequency range is big enough
            {
                let buffer = bar_processor.process_bars(&desc.processor);

                if buffer.len() < amount_bars {
                    error!("Your given frequency range ({}Hz to {}Hz) is too small to display {} bars. Please either decrease the amount of bars or increase the frequency range!", desc.freq_range.start, desc.freq_range.end, desc.amount_bars);
                    panic!("Invalid audio config");
                }
            }

            bar_processor
        };

        Self {
            bar_processor,
            bar_values: audio_buffer,
            buffer,
            binding: desc.binding,
        }
    }

    pub fn fetch_bar_values(&mut self, processor: &SampleProcessor) {
        let bar_values = self.bar_processor.process_bars(processor);
        self.bar_values.copy_from_slice(bar_values);
    }
}

impl Resource for Audio {
    fn bind_group_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: wgpu::BindingResource::Buffer(self.buffer.as_entire_buffer_binding()),
        }
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.bar_values));
    }
}

// impl TemplateGenerator for Audio {
//     fn write_wgsl_template(
//         writer: &mut dyn std::fmt::Write,
//         bind_group_index: u32,
//     ) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// {}
// @group({}) @binding({})
// var<storage, read> iAudio: array<f32>;
// ",
//             DESCRIPTION, bind_group_index, BINDING,
//         ))
//     }

//     fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// {}
// layout(binding = {}) readonly buffer iAudio {{
//     float[] freqs;
// }};
// ",
//             DESCRIPTION, BINDING
//         ))
//     }
// }
