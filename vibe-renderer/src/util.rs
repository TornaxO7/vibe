pub struct SimpleRenderPipelineDescriptor<'a> {
    pub label: &'static str,
    pub layout: &'a wgpu::PipelineLayout,
    pub vertex: wgpu::VertexState<'a>,
    pub fragment: wgpu::FragmentState<'a>,
}

pub fn simple_pipeline_descriptor(
    desc: SimpleRenderPipelineDescriptor,
) -> wgpu::RenderPipelineDescriptor {
    wgpu::RenderPipelineDescriptor {
        label: Some(desc.label),
        layout: Some(desc.layout),
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
