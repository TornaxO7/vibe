use image::RgbaImage;
use vibe_renderer::{components::Component, Renderer, RendererDescriptor};

mod aurodio;

mod bar_color_variant;
mod bar_fragment_code_variant;
mod bar_presence_gradient_variant;

mod fragment_canvas;

mod graph_color_variant;
mod graph_horizontal_gradient_variant;
mod graph_vertical_gradient_variant;

mod circle_graph;

mod radial_color_variant;
mod radial_height_gradient_variant;

mod chessy;

mod live_wallpaper_pulse_edges;

const PIXEL_SIZE: u32 = std::mem::size_of::<u32>() as u32;

// some colors
const BLUE: [f32; 4] = [0., 0., 1., 1.];
const RED: [f32; 4] = [1., 0., 0., 1.];
const WHITE: [f32; 4] = [1f32; 4];

pub struct Tester<'a> {
    pub output_width: u32,
    pub output_height: u32,

    pub renderer: Renderer,

    output_texture_desc: wgpu::TextureDescriptor<'a>,
    output_texture: wgpu::Texture,
    output_buffer: wgpu::Buffer,
}

impl<'a> Tester<'a> {
    pub fn new(width: u32, height: u32) -> Self {
        let renderer = Renderer::new(&RendererDescriptor {
            fallback_to_software_rendering: true,
            ..Default::default()
        });
        let output_width = width;
        let output_height = height;

        let output_texture_desc = wgpu::TextureDescriptor {
            label: Some("Output texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        let output_texture = renderer.device().create_texture(&output_texture_desc);

        let output_buffer = {
            renderer.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Output buffer"),
                size: (PIXEL_SIZE * width * height) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };

        Self {
            renderer,
            output_width,
            output_height,
            output_texture,
            output_texture_desc,
            output_buffer,
        }
    }

    pub fn output_texture_format(&self) -> wgpu::TextureFormat {
        self.output_texture.format()
    }

    pub fn render(&mut self, component: impl Component) -> RgbaImage {
        let view = self
            .output_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .renderer
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Tester render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            component.render_with_renderpass(&mut render_pass);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(PIXEL_SIZE * self.output_width),
                    rows_per_image: Some(self.output_height),
                },
            },
            self.output_texture_desc.size,
        );

        self.renderer.queue().submit(Some(encoder.finish()));

        let rgba_image = {
            let buffer_slice = self.output_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });

            self.renderer
                .device()
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .unwrap();
            rx.recv().unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            RgbaImage::from_raw(self.output_width, self.output_height, data.to_vec()).unwrap()
        };

        self.output_buffer.unmap();

        rgba_image
    }
}

impl<'a> Default for Tester<'a> {
    fn default() -> Self {
        Self::new(256, 256)
    }
}
