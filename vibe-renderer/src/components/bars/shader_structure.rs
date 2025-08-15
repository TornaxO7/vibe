pub const PIPELINE_LAYOUT: wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor {
    label: Some("Yes"),

    #[rustfmt::skip]
    entries: &[
        buffer(0, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform),
    ],
};
