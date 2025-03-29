use std::{sync::Arc, time::Instant};

use shady_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use vibe_renderer::{
    components::{FragmentCanvas, FragmentCanvasDescriptor},
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

    canvas: FragmentCanvas,
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

            let format = capabilities.formats.iter().find(|f| f.is_srgb()).unwrap();

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

        let canvas = {
            let fragment_source = wgpu::ShaderSource::Wgsl(
                "
                    @group(0) @binding(0)
                    var<uniform> iResolution: vec2<f32>;

                    @group(0) @binding(0)
                    var<uniform> iTime: f32;

                    @group(0) @binding(1)
                    var<storage, read> freqs: array<f32>;

                    @fragment
                    fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                        let uv = pos.xy / iResolution.xy;
                        return vec4<f32>(sin(uv + iTime + freqs[3]) * .5 + .5, 0., 1.0);
                    }
            "
                .into(),
            );

            FragmentCanvas::new(&FragmentCanvasDescriptor {
                sample_processor: &processor,
                audio_config: shady_audio::Config::default(),
                device: renderer.device(),
                format: surface_config.format,
                resolution: [size.width, size.height],
                fragment_source,
            })
        };

        Self {
            time,
            renderer,
            surface,
            window,
            surface_config,
            canvas,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface
                .configure(self.renderer.device(), &self.surface_config);
        }
    }

    pub fn render(&mut self, processor: &SampleProcessor) -> Result<(), wgpu::SurfaceError> {
        self.canvas.update_audio(processor, self.renderer.queue());
        self.canvas
            .update_time(self.renderer.queue(), self.time.elapsed().as_secs_f32());
        let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(&view, [&self.canvas]);
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
