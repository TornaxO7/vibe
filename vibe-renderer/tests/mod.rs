use colored::Colorize;
use image::{buffer::ConvertBuffer, ImageReader, RgbaImage};
use std::{io::Cursor, path::Path};
use vibe_audio::SampleProcessor;
use vibe_renderer::{ComponentAudio, Renderer, RendererDescriptor};

mod aurodio;

mod bar_color_variant;
mod bar_presence_gradient_variant;

mod fragment_canvas;

mod graph_color_variant;
mod graph_horizontal_gradient_variant;
mod graph_vertical_gradient_variant;

mod circle_graph;

mod radial_color_variant;
mod radial_height_gradient_variant;

mod chessy;

mod live_wallpaper_light_sources;
mod live_wallpaper_pulse_edges;

mod fetcher;

use fetcher::TestFetcher;

const PIXEL_SIZE: u32 = std::mem::size_of::<u32>() as u32;
/// The environment variable which needs to be set to create and save the diff images of the tests.
const DIFF_ENV: &str = "VIBE_TEST_SAVE_DIFF";
const DIFF_PATH_PREFIX: &str = "/tmp/vibe_test_diffs";

// some colors
const BLUE: [f32; 4] = [0., 0., 1., 1.];
const RED: [f32; 4] = [1., 0., 0., 1.];
const WHITE: [f32; 4] = [1f32; 4];

pub struct Tester<'a> {
    pub output_width: u32,
    pub output_height: u32,

    pub renderer: Renderer,
    pub sample_processor: SampleProcessor<TestFetcher>,

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
        let sample_processor = {
            let mut sample_processor = SampleProcessor::new(TestFetcher::new());
            sample_processor.process_next_samples();
            sample_processor
        };

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
            sample_processor,
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

    /// Renders the given component and returns the rendered image
    pub fn render<C: ComponentAudio<TestFetcher>>(&self, component: &mut C) -> RgbaImage {
        component.update_resolution(&self.renderer, [self.output_width, self.output_height]);
        component.update_audio(self.renderer.queue(), &self.sample_processor);
        component.update_time(self.renderer.queue(), 100.);

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

    pub fn evaluate<C: ComponentAudio<TestFetcher>>(
        &self,
        component: &mut C,
        reference: &'static [u8],
        id: &str,
        threshold: f32,
    ) {
        assert!(
            (0. ..=1.).contains(&threshold),
            "The threshold must be within the range [0, 1]!"
        );

        let test_img: image::RgbImage = self.render(component).convert();

        let test_flip_img =
            nv_flip::FlipImageRgb8::with_data(test_img.width(), test_img.height(), &test_img);

        let ref_img = ImageReader::new(Cursor::new(reference))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb8();

        let ref_flip_img =
            nv_flip::FlipImageRgb8::with_data(ref_img.width(), ref_img.height(), &ref_img);

        let error_map = nv_flip::flip(
            ref_flip_img,
            test_flip_img,
            nv_flip::DEFAULT_PIXELS_PER_DEGREE,
        );

        // save diff
        if std::env::var(DIFF_ENV).ok().is_some() {
            let visualized = error_map.apply_color_lut(&nv_flip::magma_lut());

            let diff_img = image::RgbImage::from_raw(
                visualized.width(),
                visualized.height(),
                visualized.to_vec(),
            )
            .unwrap();

            let prefix = format!("{}/{}", DIFF_PATH_PREFIX, id);
            std::fs::create_dir_all(&prefix).unwrap();

            test_img.save(&format!("{}/rendered.png", prefix)).unwrap();
            ref_img.save(&format!("{}/reference.png", prefix)).unwrap();
            diff_img.save(&format!("{}/diff.png", prefix)).unwrap();

            println!("Saved diff to {}*", prefix.blue());
        }

        let pool = nv_flip::FlipPool::from_image(&error_map);

        assert!(
            pool.mean() < 0.001,
            "Got mean of {}. Set the `{}` variable to see the diffs in `{}`.",
            pool.mean(),
            DIFF_ENV.yellow(),
            DIFF_PATH_PREFIX.yellow()
        );
    }

    /// A little helper function to create the reference file of a component.
    pub fn create_reference_img<C, P>(&self, component: &mut C, dest: P)
    where
        C: ComponentAudio<TestFetcher>,
        P: AsRef<Path>,
    {
        let img = self.render(component);
        img.save(dest).unwrap();
    }
}

impl<'a> Default for Tester<'a> {
    fn default() -> Self {
        let size = 256;
        Self::new(size, size)
    }
}
