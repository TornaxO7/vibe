use wgpu::naga::FastHashMap;

type BindingIdx = u32;

#[derive(Debug)]
pub struct BindGroupManager {
    bind_group: Option<wgpu::BindGroup>,
    label: Option<&'static str>,

    bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    buffers: FastHashMap<BindingIdx, wgpu::Buffer>,
    textures: FastHashMap<BindingIdx, (wgpu::Texture, wgpu::TextureView)>,
    samplers: FastHashMap<BindingIdx, wgpu::Sampler>,
}

impl BindGroupManager {
    pub fn new(label: Option<&'static str>) -> Self {
        Self {
            label,
            bind_group: None,
            bind_group_layout_entries: Vec::new(),
            buffers: FastHashMap::default(),
            textures: FastHashMap::default(),
            samplers: FastHashMap::default(),
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        self.bind_group.as_ref().unwrap()
    }

    pub fn get_buffer(&self, binding: BindingIdx) -> Option<&wgpu::Buffer> {
        self.buffers.get(&binding)
    }

    pub fn replace_buffer(&mut self, binding: BindingIdx, new_buffer: wgpu::Buffer) {
        let buffer = self.buffers.get_mut(&binding).unwrap();
        *buffer = new_buffer;
    }

    pub fn insert_buffer(
        &mut self,
        binding: BindingIdx,
        visibility: wgpu::ShaderStages,
        buffer: wgpu::Buffer,
    ) {
        // add bind group layout entry
        {
            let usage = buffer.usage();
            let ty = if usage.contains(wgpu::BufferUsages::UNIFORM) {
                wgpu::BufferBindingType::Uniform
            } else if usage.contains(wgpu::BufferUsages::STORAGE) {
                wgpu::BufferBindingType::Storage { read_only: true }
            } else {
                unimplemented!("Eh... houston, we've got a problem!");
            };

            self.bind_group_layout_entries
                .push(wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
        }

        if self.buffers.insert(binding, buffer).is_some() {
            panic!("Binding ({}) is already used.", binding);
        }
    }

    pub fn insert_texture(
        &mut self,
        binding: BindingIdx,
        visibility: wgpu::ShaderStages,
        texture: wgpu::Texture,
    ) {
        self.bind_group_layout_entries
            .push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        if self.textures.insert(binding, (texture, view)).is_some() {
            panic!("Binding {} is already used.", binding);
        }
    }

    pub fn insert_sampler(
        &mut self,
        binding: BindingIdx,
        visibility: wgpu::ShaderStages,
        sampler: wgpu::Sampler,
    ) {
        self.bind_group_layout_entries
            .push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });

        if self.samplers.insert(binding, sampler).is_some() {
            panic!("Binding {} is already used.", binding);
        }
    }

    pub fn get_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let label = self.label.map(|label| format!("{} - layout", label));

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: label.as_deref(),
            entries: &self.bind_group_layout_entries,
        })
    }

    pub fn build_bind_group(&mut self, device: &wgpu::Device) {
        let layout = self.get_bind_group_layout(device);

        let mut bind_group_entries = Vec::new();

        for (binding, buffer) in self.buffers.iter() {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: buffer.as_entire_binding(),
            })
        }

        for (binding, (_texture, view)) in self.textures.iter() {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::TextureView(view),
            })
        }

        for (binding, sampler) in self.samplers.iter() {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout: &layout,
            entries: &bind_group_entries,
        });

        self.bind_group = Some(bind_group);
    }
}
