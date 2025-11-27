//! Some helper utilities which can be used in the whole crate.
pub const DEFAULT_SAMPLER_DESCRIPTOR: wgpu::SamplerDescriptor = wgpu::SamplerDescriptor {
    label: None,
    address_mode_u: wgpu::AddressMode::MirrorRepeat,
    address_mode_v: wgpu::AddressMode::MirrorRepeat,
    address_mode_w: wgpu::AddressMode::MirrorRepeat,
    mipmap_filter: wgpu::FilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    mag_filter: wgpu::FilterMode::Linear,
    lod_min_clamp: 0.0,
    lod_max_clamp: 32.0,
    compare: None,
    anisotropy_clamp: 1,
    border_color: None,
};

pub struct SimpleRenderPipelineDescriptor<'a> {
    pub label: &'static str,
    pub layout: Option<&'a wgpu::PipelineLayout>,
    pub vertex: wgpu::VertexState<'a>,
    pub fragment: wgpu::FragmentState<'a>,
}

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
        multiview: None,
        cache: None,
    }
}

/// A little helper function to create a bind group layout entry for buffers.
pub const fn buffer(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BufferBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub const fn texture(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub const fn sampler(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

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
