mod compute_distance_map;
mod double_threshold;
mod edge_detection;
mod edge_tracking;
mod flag_cleanup;
mod gaussian_blur;
mod gray_scale;
mod non_maximation_suppression;

use crate::texture_generation::{
    edge_distance_map::{
        compute_distance_map::ComputeDistanceMapDescriptor,
        double_threshold::DoubleThresholdDescriptor, edge_detection::EdgeDetectionDescriptor,
        edge_tracking::EdgeTrackingDescriptor, gaussian_blur::GaussianBlurDescriptor,
        non_maximation_suppression::NMSDescriptor,
    },
    TextureGenerator,
};

const WORKGROUP_SIZE: u32 = 16;

pub struct EdgeDistanceMap<'a> {
    pub src: &'a image::DynamicImage,
}

impl<'a> TextureGenerator for EdgeDistanceMap<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture1 = gray_scale::apply(gray_scale::GrayScaleDescriptor {
            src: self.src,
            device,
            queue,
        });
        let texture2 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture 2"),
            size: texture1.size(),
            mip_level_count: texture1.mip_level_count(),
            sample_count: texture1.sample_count(),
            dimension: texture1.dimension(),
            format: texture1.format(),
            usage: texture1.usage(),
            view_formats: &[],
        });

        let max_side_length = texture1.width().max(texture1.height());

        let tv1 = texture1.create_view(&wgpu::TextureViewDescriptor::default());
        let tv2 = texture2.create_view(&wgpu::TextureViewDescriptor::default());

        gaussian_blur::apply(GaussianBlurDescriptor {
            device,
            queue,
            src: tv1.clone(),
            dst: tv2.clone(),
            sigma: 1.6,
            kernel_size: 5,
        });

        non_maximation_suppression::apply(NMSDescriptor {
            device,
            queue,
            edge_infos: edge_detection::apply(EdgeDetectionDescriptor {
                device,
                queue,
                src: tv2.clone(),
            }),
            dst: tv1.clone(),
        });

        double_threshold::apply(DoubleThresholdDescriptor {
            device,
            queue,
            src: tv1.clone(),
            dst: tv2.clone(),
            high_threshold_ratio: 0.75,
            low_threshold_ratio: 0.2,
        });

        edge_tracking::apply(EdgeTrackingDescriptor {
            device,
            queue,
            src: tv2.clone(),
            dst: tv1.clone(),
            iterations: max_side_length,
        });

        flag_cleanup::apply(flag_cleanup::FlagCleanupDescriptor {
            device,
            queue,
            src: tv1.clone(),
            dst: tv2.clone(),
        });

        compute_distance_map::apply(ComputeDistanceMapDescriptor {
            device,
            queue,
            src: tv2.clone(),
            dst: tv1.clone(),
            iterations: max_side_length,
        });

        texture1
    }
}

fn start_computing(
    label_prefix: &'static str,
    device: &wgpu::Device,
    dst: &wgpu::Texture,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
) {
    let command_encoder_label = format!("{}: Command encoder", label_prefix);
    let pass_label = format!("{}: Compute pass", label_prefix);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(&command_encoder_label),
    });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&pass_label),
            timestamp_writes: None,
        });

        pass.set_bind_group(0, bind_group, &[]);
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(
            dst.width().div_ceil(WORKGROUP_SIZE),
            dst.height().div_ceil(WORKGROUP_SIZE),
            1,
        );
    }

    queue.submit(std::iter::once(encoder.finish()));
}
