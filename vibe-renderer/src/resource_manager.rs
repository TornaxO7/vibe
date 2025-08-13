use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use wgpu::naga::FastHashMap;

/// A little helper struct which stores every buffer, sampler and texture which is
/// for each component and can be accessed with the given ID which is commonly an enum.
#[derive(Debug)]
pub struct ResourceManager<ResourceID>
where
    ResourceID: Clone + Copy + Eq + Hash + Debug,
{
    resources: FastHashMap<ResourceID, Resource>,
}

impl<ResourceID> ResourceManager<ResourceID>
where
    ResourceID: Clone + Copy + Eq + Hash + Debug,
{
    pub fn new() -> Self {
        Self {
            resources: FastHashMap::default(),
        }
    }

    pub fn get_buffer(&self, id: ResourceID) -> Option<&wgpu::Buffer> {
        self.resources.get(&id).map(|resource| match resource {
            Resource::Buffer(buffer) => buffer,
            _ => panic!("There's no buffer with id `{:?}`", id),
        })
    }

    pub fn replace_buffer(&mut self, id: ResourceID, new_buffer: wgpu::Buffer) {
        let resource = self
            .resources
            .get_mut(&id)
            .unwrap_or_else(|| panic!("There's no resource with id `{:?}`", id));

        match resource {
            Resource::Buffer(buffer) => *buffer = new_buffer,
            _ => panic!("There's no buffer with id `{:?}`", id),
        };
    }

    pub fn insert_buffer(&mut self, id: ResourceID, buffer: wgpu::Buffer) {
        if self
            .resources
            .insert(id, Resource::Buffer(buffer))
            .is_some()
        {
            panic!("Id `{:?}` is alread used", id);
        };
    }

    pub fn extend_buffers<const LEN: usize>(&mut self, buffers: [(ResourceID, wgpu::Buffer); LEN]) {
        for (key, buffer) in buffers {
            self.insert_buffer(key, buffer);
        }
    }

    pub fn insert_texture(&mut self, id: ResourceID, texture: wgpu::Texture) {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let resource = Resource::Texture {
            _texture: texture,
            view,
        };

        if self.resources.insert(id, resource).is_some() {
            panic!("Id `{:?}` is already used", id);
        }
    }

    pub fn insert_sampler(&mut self, id: ResourceID, sampler: wgpu::Sampler) {
        if self
            .resources
            .insert(id, Resource::Sampler(sampler))
            .is_some()
        {
            panic!("Id `{:?}` is already used", id);
        }
    }

    pub fn build_bind_group(
        &mut self,
        label: &'static str,
        device: &wgpu::Device,
        binding_resource_map: &HashMap<ResourceID, wgpu::BindGroupLayoutEntry>,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let mut bind_group_entries = Vec::with_capacity(binding_resource_map.len());

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} - Layout", label)),
            entries: &binding_resource_map
                .values()
                .cloned()
                .collect::<Vec<wgpu::BindGroupLayoutEntry>>(),
        });

        for (id, entry) in binding_resource_map {
            let binding = entry.binding;

            let resource = self
                .resources
                .get(id)
                .unwrap_or_else(|| panic!("Id `{:?}` isn't set", id));

            let binding_resource = match resource {
                Resource::Buffer(buffer) => buffer.as_entire_binding(),
                Resource::Texture { view, .. } => wgpu::BindingResource::TextureView(view),
                Resource::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
            };

            bind_group_entries.push(wgpu::BindGroupEntry {
                binding,
                resource: binding_resource,
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &layout,
            entries: &bind_group_entries,
        });

        (bind_group, layout)
    }
}

#[derive(Debug)]
enum Resource {
    Buffer(wgpu::Buffer),
    Texture {
        _texture: wgpu::Texture,
        view: wgpu::TextureView,
    },
    Sampler(wgpu::Sampler),
}
