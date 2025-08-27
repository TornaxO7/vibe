use std::{num::NonZero, sync::Arc, time::Instant};

use anyhow::bail;
use clap::Parser;
use cli::ComponentName;
use vibe_audio::{
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    BarProcessorConfig, SampleProcessor,
};
use vibe_renderer::{
    components::{
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, BarsDescriptor,
        BarsFormat, BarsPlacement, Chessy, ChessyDescriptor, Circle, CircleDescriptor,
        CircleVariant, Component, FragmentCanvas, FragmentCanvasDescriptor, Graph, GraphDescriptor,
        GraphVariant, Radial, RadialDescriptor, RadialVariant, SdfPattern, ShaderCode, ValueNoise,
        ValueNoiseDescriptor,
    },
    Renderer,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::KeyEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

mod cli;

const TURQUOISE: [f32; 4] = [0., 1., 1., 1.];
const DARK_BLUE: [f32; 4] = [0.05, 0., 0.321, 255.];

struct State<'a> {
    renderer: Renderer,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    time: Instant,

    component: Box<dyn Component>,
}

impl<'a> State<'a> {
    pub fn new<'b>(
        window: Window,
        processor: &'b SampleProcessor<SystemAudioFetcher>,
        component_name: ComponentName,
    ) -> anyhow::Result<Self> {
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

        let component = match component_name {
            ComponentName::Aurodio => Ok(Box::new(Aurodio::new(&AurodioDescriptor {
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
                sensitivity: 0.2,
            })) as Box<dyn Component>),
            ComponentName::BarsColorVariant => Bars::new(&BarsDescriptor {
                device: renderer.device(),
                sample_processor: &processor,
                audio_conf: BarProcessorConfig {
                    amount_bars: std::num::NonZero::new(60).unwrap(),
                    sensitivity: 4.,
                    ..Default::default()
                },
                texture_format: surface_config.format,
                max_height: 0.5,
                variant: BarVariant::Color([0., 0., 1., 1.]),
                // placement: BarsPlacement::Custom {
                //     bottom_left_corner: (0.5, 0.5),
                //     width_factor: 0.5,
                //     rotation: cgmath::Deg(45.),
                // },
                placement: BarsPlacement::Bottom,
                format: BarsFormat::BassTreble,
            })
            .map(|bars| Box::new(bars) as Box<dyn Component>),
            ComponentName::BarsFragmentCodeVariant => Bars::new(&BarsDescriptor {
                device: renderer.device(),
                sample_processor: &processor,
                audio_conf: BarProcessorConfig::default(),
                texture_format: surface_config.format,
                max_height: 0.75,
                placement: vibe_renderer::components::BarsPlacement::Top,
                variant: BarVariant::FragmentCode(ShaderCode {
                    language: vibe_renderer::components::ShaderLanguage::Wgsl,
                    source: vibe_renderer::components::ShaderSource::Code(
                        "
                        @fragment
                        fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                            var uv = pos.xy / iResolution.xy;
                            uv.y = 1. - uv.y;
                            uv.x *= iResolution.x / iResolution.y;

                            let col: vec3<f32> = sin(vec3(2., 4., 8.) * iTime * .25) * .2 + .6;
                            return vec4<f32>(col, uv.y);
                        }
                        "
                        .into(),
                    ),
                }),
                format: BarsFormat::BassTreble,
            })
            .map(|bars| Box::new(bars) as Box<dyn Component>),
            ComponentName::BarsPresenceGradientVariant => Bars::new(&BarsDescriptor {
                device: renderer.device(),
                sample_processor: &processor,
                audio_conf: BarProcessorConfig {
                    sensitivity: 4.,
                    amount_bars: NonZero::new(30).unwrap(),
                    ..Default::default()
                },
                texture_format: surface_config.format,
                max_height: 0.5,
                variant: BarVariant::PresenceGradient {
                    high: TURQUOISE,
                    low: DARK_BLUE,
                },
                placement: BarsPlacement::Bottom,
                format: BarsFormat::TrebleBassTreble,
            })
            .map(|bars| Box::new(bars) as Box<dyn Component>),
            ComponentName::ValueNoise => Ok(Box::new(ValueNoise::new(&ValueNoiseDescriptor {
                device: renderer.device(),
                width: size.width,
                height: size.height,
                format: surface_config.format,
                octaves: 6,
                brightness: 0.5,
            })) as Box<dyn Component>),
            ComponentName::CircleCurvedVariant => Ok(Box::new(Circle::new(&CircleDescriptor {
                device: renderer.device(),
                sample_processor: processor,
                audio_conf: vibe_audio::BarProcessorConfig {
                    amount_bars: std::num::NonZero::new(30).unwrap(),
                    ..Default::default()
                },
                texture_format: surface_config.format,
                variant: CircleVariant::Graph {
                    spike_sensitivity: 0.3,
                    color: [0., 1., 1., 1.],
                },

                radius: 0.1,
                rotation: cgmath::Deg(90.),
                position: (0.5, 0.5),
            })) as Box<dyn Component>),
            ComponentName::FragmentCanvas => {
                let fragment_source = ShaderCode {
                    language: vibe_renderer::components::ShaderLanguage::Wgsl,
                    source: vibe_renderer::components::ShaderSource::Code(
                        "
                    @fragment
                    fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                        let uv = pos.xy / iResolution.xy;
                        // return vec4(sin(uv + iTime + freqs[3]) * .5 + .5, 0., 1.0);
                        return vec4(uv, .0, 1.);
                    }
                    "
                        .into(),
                    ),
                };

                FragmentCanvas::new(&FragmentCanvasDescriptor {
                    sample_processor: &processor,
                    audio_conf: vibe_audio::BarProcessorConfig::default(),
                    device: renderer.device(),
                    format: surface_config.format,
                    fragment_code: fragment_source,
                })
                .map(|canvas| Box::new(canvas) as Box<dyn Component>)
            }
            ComponentName::GraphColorVariant => Ok(Box::new(Graph::new(&GraphDescriptor {
                device: renderer.device(),
                sample_processor: processor,
                audio_conf: BarProcessorConfig {
                    amount_bars: NonZero::new(size.width as u16).unwrap(),
                    ..Default::default()
                },
                output_texture_format: surface_config.format,
                variant: GraphVariant::Color([0., 0., 1., 1.]),
                max_height: 0.5,
                smoothness: 0.01,
                placement: vibe_renderer::components::GraphPlacement::Left,
            })) as Box<dyn Component>),
            ComponentName::GraphHorizontalGradientVariant => {
                Ok(Box::new(Graph::new(&GraphDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: BarProcessorConfig {
                        amount_bars: NonZero::new(size.width as u16).unwrap(),
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,
                    variant: GraphVariant::HorizontalGradient {
                        left: [1., 0., 0., 1.],
                        right: [0., 0., 1., 1.],
                    },
                    max_height: 0.5,
                    smoothness: 0.01,
                    placement: vibe_renderer::components::GraphPlacement::Left,
                })) as Box<dyn Component>)
            }
            ComponentName::GraphVerticalGradientVariant => {
                Ok(Box::new(Graph::new(&GraphDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: BarProcessorConfig {
                        amount_bars: NonZero::new(size.width as u16).unwrap(),
                        sensitivity: 0.2,
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,
                    variant: GraphVariant::VerticalGradient {
                        top: [0.012, 0.725, 0.749, 1.],
                        bottom: [0.008, 0.435, 0.447, 1.],
                    },
                    max_height: 0.5,
                    smoothness: 0.01,
                    placement: vibe_renderer::components::GraphPlacement::Top,
                })) as Box<dyn Component>)
            }
            ComponentName::RadialColorVariant => Ok(Box::new(Radial::new(&RadialDescriptor {
                device: renderer.device(),
                processor,
                audio_conf: vibe_audio::BarProcessorConfig {
                    amount_bars: NonZero::new(30).unwrap(),
                    ..Default::default()
                },
                output_texture_format: surface_config.format,

                variant: RadialVariant::Color([1., 0., 0., 1.]),

                init_rotation: cgmath::Deg(90.),
                circle_radius: 0.2,
                bar_height_sensitivity: 0.5,
                bar_width: 0.015,
                position: (0.5, 0.5),
            })) as Box<dyn Component>),

            ComponentName::RadialHeightGradientVariant => {
                Ok(Box::new(Radial::new(&RadialDescriptor {
                    device: renderer.device(),
                    processor,
                    audio_conf: vibe_audio::BarProcessorConfig {
                        amount_bars: NonZero::new(60).unwrap(),
                        sensitivity: 4.0,
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,

                    variant: RadialVariant::HeightGradient {
                        inner: [1., 0., 0., 1.],
                        outer: [1., 1., 1., 1.],
                    },

                    init_rotation: cgmath::Deg(90.),
                    circle_radius: 0.3,
                    bar_height_sensitivity: 1.,
                    bar_width: 0.02,
                    position: (0.5, 0.5),
                })) as Box<dyn Component>)
            }
            ComponentName::ChessyBoxVariant => Ok(Box::new(Chessy::new(&ChessyDescriptor {
                renderer: &renderer,
                sample_processor: processor,
                audio_config: BarProcessorConfig {
                    amount_bars: NonZero::new(10).unwrap(),
                    ..Default::default()
                },
                texture_format: surface_config.format,
                movement_speed: 0.1,
                pattern: SdfPattern::Box,
                zoom_factor: 4.,
            })) as Box<dyn Component>),
        }?;

        Ok(Self {
            time,
            renderer,
            surface,
            window,
            surface_config,
            component,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface
                .configure(self.renderer.device(), &self.surface_config);

            self.component
                .update_resolution(&self.renderer, [new_size.width, new_size.height]);
        }
    }

    pub fn render(
        &mut self,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) -> Result<(), wgpu::SurfaceError> {
        self.component
            .update_audio(self.renderer.queue(), processor);
        self.component
            .update_time(self.renderer.queue(), self.time.elapsed().as_secs_f32());
        let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(&view, &[&self.component]);

        surface_texture.present();
        Ok(())
    }
}

struct App<'a> {
    sample_processor: SampleProcessor<SystemAudioFetcher>,
    state: Option<State<'a>>,
    variant: ComponentName,
}

impl<'a> App<'a> {
    pub fn new<S: AsRef<str>>(
        variant: ComponentName,
        output_device_name: Option<S>,
    ) -> anyhow::Result<Self> {
        let sample_processor = {
            let device = match output_device_name {
                Some(device_name) => {
                    match vibe_audio::util::get_device(device_name.as_ref(), DeviceType::Output)? {
                        Some(device) => device,
                        None => {
                            bail!(
                                concat![
                                    "Available output devices:\n\n{:#?}\n",
                                    "\nThere's no output device called \"{}\".\n",
                                    "Please choose one from the list.\n",
                                ],
                                vibe_audio::util::get_device_names(DeviceType::Output)?,
                                device_name.as_ref()
                            )
                        }
                    }
                }
                None => match vibe_audio::util::get_default_device(DeviceType::Output) {
                    Some(device) => device,
                    None => {
                        bail!(
                            concat![
                                "Available output devices:\n\n{:#?}\n",
                                "\nCoudn't find the default output device on your system.\n",
                                "Please choose one from the list and add it explicitly to the cli invocation.\n"
                            ],
                            vibe_audio::util::get_device_names(DeviceType::Output)?,
                        )
                    }
                },
            };

            let system_audio_fetcher = SystemAudioFetcher::new(&SystemAudioFetcherDescriptor {
                device,
                amount_channels: Some(2),
                ..Default::default()
            })?;

            SampleProcessor::new(system_audio_fetcher)
        };

        Ok(Self {
            sample_processor,
            state: None,
            variant,
        })
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default().with_title("Vibe renderer - Demo"))
            .unwrap();

        self.state = Some(State::new(window, &self.sample_processor, self.variant).unwrap());
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

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    if cli.show_output_devices {
        println!(
            "\nAvailable output devices:\n\n{:#?}\n",
            vibe_audio::util::get_device_names(vibe_audio::util::DeviceType::Output)?
        );
        return Ok(());
    }

    if let Some(component) = cli.component_name {
        let event_loop = EventLoop::new()?;

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = App::new(component, cli.output_device_name)?;
        event_loop.run_app(&mut app).unwrap();
    }

    Ok(())
}
