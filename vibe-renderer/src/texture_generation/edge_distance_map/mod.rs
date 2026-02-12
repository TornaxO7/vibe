mod compute_distance_map;
mod double_threshold;
mod edge_detection;
mod edge_tracking;
mod flag_cleanup;
mod gaussian_blur;
mod gray_scale;
mod non_maximation_suppression;

use crate::{
    texture_generation::{
        edge_distance_map::{
            compute_distance_map::{ComputeDistanceMap, ComputeDistanceMapDescriptor},
            double_threshold::{DoubleThreshold, DoubleThresholdDescriptor},
            edge_tracking::{EdgeTracking, EdgeTrackingDescriptor},
            flag_cleanup::{FlagCleanup, FlagCleanupDescriptor},
            gaussian_blur::{GaussianBlur, GaussianBlurDescriptor},
            gray_scale::{GrayScale, GrayScaleDescriptor},
            non_maximation_suppression::{Nms, NmsDescriptor},
        },
        TextureGenerator,
    },
};
use std::hash::{DefaultHasher, Hash, Hasher};
use tracing::info_span;
use tracing_indicatif::{span_ext::IndicatifSpanExt, style::ProgressStyle};

trait EdgeDistanceMapStep {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32);

    fn amount_steps(&self) -> u32;
}

const WORKGROUP_SIZE: u32 = 16;
/// The texture format used for the returned texture of `EdgeDistanceMap`.
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

pub struct EdgeDistanceMap<'a> {
    pub src: &'a image::DynamicImage,
    pub high_threshold_ratio: f32,
    pub low_threshold_ratio: f32,

    pub sigma: f32,
    pub kernel_size: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> crate::cache::Cacheable for EdgeDistanceMap<'a> {
    fn subpath(&self) -> std::path::PathBuf {
        let img_hash = {
            let mut hasher = DefaultHasher::new();
            self.src.to_rgba8().as_raw().hash(&mut hasher);
            hasher.finish()
        };

        format!("{}", img_hash).into()
    }

    fn checksum(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.src.to_rgba8().as_raw().hash(&mut hasher);

        let bytes: u32 = self.high_threshold_ratio.to_bits();
        bytes.hash(&mut hasher);

        let bytes: u32 = self.low_threshold_ratio.to_bits();
        bytes.hash(&mut hasher);

        let bytes = self.sigma.to_bits();
        bytes.hash(&mut hasher);

        self.kernel_size.hash(&mut hasher);

        hasher.finish()
    }

    fn format(&self) -> wgpu::TextureFormat {
        TEXTURE_FORMAT
    }
}

impl<'a> TextureGenerator for EdgeDistanceMap<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        assert!(self.kernel_size % 2 == 1);

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
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            };

            let texture1 = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture 1"),
                ..desc.clone()
            });

            let texture2 = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture 2"),
                ..desc.clone()
            });

            (texture1, texture2)
        };

        let max_side_length = self.src.height().max(self.src.width());

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

                sigma: self.sigma,
                kernel_size: self.kernel_size,
            }),
            Nms::step(NmsDescriptor {
                device,
                src: tv2.clone(),
                dst: tv1.clone(),
            }),
            DoubleThreshold::step(DoubleThresholdDescriptor {
                device,
                src: tv1.clone(),
                dst: tv2.clone(),

                high_threshold_ratio: self.high_threshold_ratio,
                low_threshold_ratio: self.low_threshold_ratio,
            }),
            EdgeTracking::step(EdgeTrackingDescriptor {
                device,
                src: tv2.clone(),
                dst: tv1.clone(),

                iterations: max_side_length,
            }),
            FlagCleanup::step(FlagCleanupDescriptor {
                device,
                src: tv1.clone(),
                dst: tv2.clone(),
            }),
            ComputeDistanceMap::step(ComputeDistanceMapDescriptor {
                device,
                src: tv2.clone(),
                dst: tv1.clone(),
                iterations: max_side_length,
            }),
        ];

        // start generating
        let span = info_span!("Computing");
        span.pb_set_length(steps.iter().map(|step| step.amount_steps()).sum::<u32>() as u64);
        span.pb_set_message("Generating distance map");
        span.pb_set_style(&ProgressStyle::default_bar());
        let _enter = span.enter();

        for step in steps {
            step.compute(
                device,
                queue,
                tv1.texture().width().div_ceil(WORKGROUP_SIZE),
                tv1.texture().height().div_ceil(WORKGROUP_SIZE),
            );
        }

        texture1
    }
}

impl<'a> TextureGenerator for &'a EdgeDistanceMap<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        (*self).generate(device, queue)
    }
}
