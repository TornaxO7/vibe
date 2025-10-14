mod gaussian_blur;
mod gray_scale;
mod light_threshold;

use crate::texture_generation::{
    light_sources::light_threshold::{LightThreshold, LightThresholdDescriptor},
    TextureGenerator,
};
use gaussian_blur::{GaussianBlur, GaussianBlurDescriptor};
use gray_scale::{GrayScale, GrayScaleDescriptor};
use tracing::info_span;
use tracing_indicatif::{span_ext::IndicatifSpanExt, style::ProgressStyle};

const WORKGROUP_SIZE: u32 = 16;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

pub struct LightSources<'a> {
    pub src: &'a image::DynamicImage,

    pub light_threshold: f32,
}

impl<'a> TextureGenerator for LightSources<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let (texture1, texture2) = {
            let desc = wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: self.src.width(),
                    height: self.src.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            };

            let t1 = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture 1"),
                ..desc.clone()
            });

            let t2 = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture 2"),
                ..desc.clone()
            });

            (t1, t2)
        };

        let tv1 = texture1.create_view(&wgpu::TextureViewDescriptor::default());
        let tv2 = texture2.create_view(&wgpu::TextureViewDescriptor::default());

        let steps = [
            GrayScale::step(GrayScaleDescriptor {
                device,
                queue,
                src: self.src,
                dst: tv1.clone(),
            }),
            GaussianBlur::step(GaussianBlurDescriptor {
                device,

                src: tv1.clone(),
                dst: tv2.clone(),

                sigma: 10.,
                kernel_size: 49,
            }),
            LightThreshold::step(LightThresholdDescriptor {
                device,
                src: tv2.clone(),
                dst: tv1.clone(),
                threshold: self.light_threshold,
            }),
        ];

        // start creating first steps
        {
            let span = info_span!("Computing");
            span.pb_set_length(steps.iter().map(|step| step.amount_steps()).sum::<u32>() as u64);
            span.pb_set_message("Doing first steps for detecting light sources.");
            span.pb_set_style(&ProgressStyle::default_bar());
            let _enter = span.enter();

            for step in steps {
                step.compute(
                    device,
                    queue,
                    tv1.texture().width().div_ceil(WORKGROUP_SIZE),
                    tv1.texture().height().div_ceil(WORKGROUP_SIZE),
                )
            }
        }

        todo!()
    }
}
