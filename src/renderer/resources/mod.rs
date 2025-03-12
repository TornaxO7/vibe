mod audio;
mod resolution;
mod time;

use std::fmt;

use audio::Audio;
use resolution::Resolution;
use time::Time;

use tracing::instrument;
use wgpu::Device;

use super::template::TemplateGenerator;

#[repr(u32)]
enum BindingValue {
    Audio,
    Resolution,
    Time,
}

trait ResourceInstantiator: Resource + TemplateGenerator {
    fn new(device: &wgpu::Device) -> Self;
}

trait Resource {
    fn update_buffer(&self, queue: &wgpu::Queue);

    fn binding(&self) -> u32;

    fn buffer_type(&self) -> wgpu::BufferBindingType;

    fn buffer(&self) -> &wgpu::Buffer;
}

pub struct Resources {
    pub audio: Audio,
    pub resolution: Resolution,
    pub time: Time,

    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Resources {
    #[instrument(level = "trace", skip_all)]
    pub fn new(device: &wgpu::Device) -> Self {
        let audio = Audio::new(device);
        let resolution = Resolution::new(device);
        let time = Time::new(device);

        let resources = [&audio as &dyn Resource, &resolution, &time];

        let bind_group_layout = bind_group_layout(device, resources);
        let bind_group = bind_group(device, &bind_group_layout, resources);

        Self {
            audio,
            resolution,
            time,

            bind_group,
            bind_group_layout,
        }
    }
}

fn bind_group_layout<'a>(
    device: &Device,
    resources: impl IntoIterator<Item = &'a dyn Resource>,
) -> wgpu::BindGroupLayout {
    let entries: Vec<wgpu::BindGroupLayoutEntry> = resources
        .into_iter()
        .map(|resource| bind_group_layout_entry(resource))
        .collect();

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Shady bind group layout"),
        entries: &entries,
    })
}

fn bind_group<'a>(
    device: &Device,
    layout: &wgpu::BindGroupLayout,
    resources: impl IntoIterator<Item = &'a dyn Resource> + Clone,
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = resources
        .into_iter()
        .map(|resource| wgpu::BindGroupEntry {
            binding: resource.binding(),
            resource: resource.buffer().as_entire_binding(),
        })
        .collect();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shady bind group"),
        layout: &layout,
        entries: &entries,
    })
}

impl TemplateGenerator for Resources {
    fn write_wgsl_template(
        writer: &mut dyn fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        Audio::write_wgsl_template(writer, bind_group_index)?;
        Resolution::write_wgsl_template(writer, bind_group_index)?;
        Time::write_wgsl_template(writer, bind_group_index)?;

        Ok(())
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        Audio::write_glsl_template(writer)?;
        Resolution::write_glsl_template(writer)?;
        Time::write_glsl_template(writer)?;

        Ok(())
    }
}

fn bind_group_layout_entry<'a>(resource: &'a dyn Resource) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: resource.binding(),
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: resource.buffer_type(),
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_uniform_buffer<'a>(label: &'a str, device: &Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_storage_buffer<'a>(label: &'a str, device: &Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
