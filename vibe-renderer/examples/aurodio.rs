use std::{num::NonZero, sync::Arc, time::Instant};

use shady_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use vibe_renderer::{
    components::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor, Component},
    Renderer,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::KeyEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

struct State<'a> {
    renderer: Renderer,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    time: Instant,

    aurodio: Aurodio,
}

impl<'a> State<'a> {
    pub fn new<'b>(window: Window, processor: &'b SampleProcessor) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();
        let time = Instant::now();

        let renderer = Renderer::new(&vibe_renderer::RendererDescriptor::default());
        let surface = renderer.instance().create_surface(window.clone()).unwrap();

        let surface_config = {
            let capabilities = surface.get_capabilities(renderer.adapter());

            let format = capabilities.formats.iter().find(|f| !f.is_srgb()).unwrap();

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: *format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
                view_formats: vec![],
            }
        };

        surface.configure(renderer.device(), &surface_config);

        let aurodio = Aurodio::new(&AurodioDescriptor {
            renderer: &renderer,
            sample_processor: &processor,
            texture_format: surface_config.format,
            layers: &[
                AurodioLayerDescriptor {
                    freq_range: NonZero::new(50).unwrap()..NonZero::new(250).unwrap(),
                    zoom_factor: 3.,
                },
                AurodioLayerDescriptor {
                    freq_range: NonZero::new(500).unwrap()..NonZero::new(2_000).unwrap(),
                    zoom_factor: 5.,
                },
                AurodioLayerDescriptor {
                    freq_range: NonZero::new(4_000).unwrap()..NonZero::new(6_000).unwrap(),
                    zoom_factor: 10.,
                },
            ],
            base_color: [0., 0.5, 0.5],
            movement_speed: 0.005,
            easing: shady_audio::StandardEasing::OutCubic,
            sensitivity: 0.2,
        });

        Self {
            time,
            renderer,
            surface,
            window,
            surface_config,
            aurodio,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface
                .configure(self.renderer.device(), &self.surface_config);

            self.aurodio
                .update_resolution(&self.renderer, [new_size.width, new_size.height]);
        }
    }

    pub fn render(&mut self, processor: &SampleProcessor) -> Result<(), wgpu::SurfaceError> {
        self.aurodio.update_audio(self.renderer.queue(), processor);
        self.aurodio
            .update_time(self.renderer.queue(), self.time.elapsed().as_secs_f32());
        let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(&view, &[&self.aurodio]);

        surface_texture.present();
        Ok(())
    }
}

struct App<'a> {
    sample_processor: SampleProcessor,
    state: Option<State<'a>>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            sample_processor: SampleProcessor::new(
                SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap(),
            ),
            state: None,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        self.state = Some(State::new(window, &self.sample_processor));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        self.sample_processor.process_next_samples();

        match event {
            winit::event::WindowEvent::Resized(new_size) => state.resize(new_size),
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                state.render(&self.sample_processor).unwrap();
                state.window.request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent { logical_key, .. } if logical_key.to_text() == Some("q") => {
                    event_loop.exit()
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
