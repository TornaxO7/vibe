//! Some helper utilities which can be used in the whole crate.

/// A simple [wgpu::SamplerDescriptor] with defaults which I think make sense... but probably really useless...
pub const DEFAULT_SAMPLER_DESCRIPTOR: wgpu::SamplerDescriptor = wgpu::SamplerDescriptor {
    label: None,
    address_mode_u: wgpu::AddressMode::MirrorRepeat,
    address_mode_v: wgpu::AddressMode::MirrorRepeat,
    address_mode_w: wgpu::AddressMode::MirrorRepeat,
    mipmap_filter: wgpu::MipmapFilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    mag_filter: wgpu::FilterMode::Linear,
    lod_min_clamp: 0.0,
    lod_max_clamp: 32.0,
    compare: None,
    anisotropy_clamp: 1,
    border_color: None,
};

/// Basically [wgpu::RenderPipelineDescriptor] but with some attributes set with values which I really often set.
pub struct SimpleRenderPipelineDescriptor<'a> {
    /// The label of the pipeline descriptor.
    ///
    /// Yes. I want one, so no `None` here >:)
    pub label: &'static str,
    pub layout: Option<&'a wgpu::PipelineLayout>,
    pub vertex: wgpu::VertexState<'a>,
    pub fragment: wgpu::FragmentState<'a>,
}

/// This should be probably be replaced with a [std::convert::From] for [SimpleRenderPipelineDescriptor]...
pub fn simple_pipeline_descriptor(
    desc: SimpleRenderPipelineDescriptor,
) -> wgpu::RenderPipelineDescriptor {
    wgpu::RenderPipelineDescriptor {
        label: Some(desc.label),
        layout: desc.layout,
        vertex: desc.vertex.clone(),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(desc.fragment.clone()),
        multiview_mask: None,
        cache: None,
    }
}

/// A little helper function which loads the given image into a [wgpu::Texture] and returns it.
///
/// Helps you to avoid the boilerplate of
/// 1. Creating the texture
/// 2. Copy the data of the [image::DynamicImage] to the [wgpu::Texture]
pub fn load_img_to_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: &image::DynamicImage,
) -> wgpu::Texture {
    let rgba8 = img.to_rgba8();

    let format = wgpu::TextureFormat::Rgba8Unorm;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Image texture"),
        size: wgpu::Extent3d {
            width: rgba8.width(),
            height: rgba8.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        rgba8.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(
                format
                    .block_copy_size(Some(wgpu::TextureAspect::All))
                    .unwrap()
                    * texture.width(),
            ),
            rows_per_image: Some(rgba8.height()),
        },
        texture.size(),
    );

    texture
}
