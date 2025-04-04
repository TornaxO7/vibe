use wgpu::naga::FastHashMap;

type BindingIdx = u32;

#[derive(Debug)]
pub struct BindGroupManager {
    bind_group: wgpu::BindGroup,
    buffers: FastHashMap<BindingIdx, wgpu::Buffer>,
}

impl BindGroupManager {
    pub fn builder(label: Option<&'static str>) -> BindGroupManagerBuilder {
        BindGroupManagerBuilder::new(label)
    }

    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn get_buffer(&self, binding: BindingIdx) -> Option<&wgpu::Buffer> {
        self.buffers.get(&binding)
    }
}

#[derive(Debug)]
pub struct BindGroupManagerBuilder {
    label: Option<&'static str>,

    bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    buffers: FastHashMap<BindingIdx, wgpu::Buffer>,
}

impl BindGroupManagerBuilder {
    fn new(label: Option<&'static str>) -> Self {
        Self {
            label,
            bind_group_layout_entries: vec![],
            buffers: FastHashMap::default(),
        }
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

        if let Some(_) = self.buffers.insert(binding, buffer) {
            panic!("Binding is already used");
        }
    }

    pub fn get_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let label = self.label.map(|label| format!("{} - layout", label));

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: label.as_ref().map(|s| s.as_str()),
            entries: &self.bind_group_layout_entries,
        })
    }

    pub fn build(self, device: &wgpu::Device) -> BindGroupManager {
        let layout = self.get_bind_group_layout(device);

        let bind_group_entries: Vec<wgpu::BindGroupEntry> = self
            .buffers
            .iter()
            .map(|(&binding, buffer)| wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            })
            .collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout: &layout,
            entries: &bind_group_entries,
        });

        BindGroupManager {
            bind_group,
            buffers: self.buffers,
        }
    }
}
