use crate::texture_generation::TextureGenerator;

pub struct EdgeDistanceMap<'a> {
    pub src: &'a image::DynamicImage,
}

impl<'a> TextureGenerator for EdgeDistanceMap<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let src_texture = {
            let src_img = self.src.to_rgba8();
            let (width, height) = src_img.dimensions();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Source image texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 1,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                src_img.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(std::mem::size_of::<[u8; 4]>() as u32 * width),
                    rows_per_image: Some(height),
                },
                texture.size(),
            );

            texture
        };

        // apply gray scale and (gaussian) blur to source
        let src = gray_scale_and_blur(
            &src_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            device,
            queue,
        );
        todo!()
    }
}

fn gray_scale_and_blur(
    src: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::Texture {
    let src_texture = src.texture();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Gray scaled and gaussian blurred source"),
        size: src_texture.size(),
        mip_level_count: src_texture.mip_level_count(),
        sample_count: src_texture.sample_count(),
        dimension: src_texture.dimension(),
        format: wgpu::TextureFormat::R16Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let kernel = {
        fn gauss(sigma: f32, x: f32, y: f32) -> f32 {
            (1. / (2. * std::f32::consts::PI * sigma * sigma))
                * std::f32::consts::E.powf(-(x * x + y * y) / (2. * sigma * sigma))
        }
    };

    texture
}
