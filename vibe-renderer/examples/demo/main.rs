mod cli;
mod texture_component;

use std::{num::NonZero, str::FromStr, sync::Arc, time::Instant};

use anyhow::bail;
use cgmath::Deg;
use clap::Parser;
use cli::ComponentName;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vibe_audio::{
    cpal::DeviceId,
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    BarProcessorConfig, SampleProcessor,
};
use vibe_renderer::{
    components::{
        live_wallpaper::pulse_edges::{PulseEdges, PulseEdgesDescriptor},
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, BarsDescriptor,
        BarsFormat, BarsPlacement, Chessy, ChessyDescriptor, Circle, CircleDescriptor,
        CircleVariant, FragmentCanvas, FragmentCanvasDescriptor, Graph, GraphDescriptor,
        GraphFormat, GraphVariant, Radial, RadialDescriptor, RadialFormat, RadialVariant,
        ShaderCode,
    },
    texture_generation::{SdfMask, SdfPattern, ValueNoise},
    ComponentAudio, Renderer,
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::KeyEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::texture_component::{TextureComponent, TextureComponentDescriptor};

const TURQUOISE: [f32; 4] = [0., 1., 1., 1.];
const DARK_BLUE: [f32; 4] = [0.05, 0., 0.321, 255.];
const BLUE: [f32; 4] = [0., 0., 1., 1.];
const RED: [f32; 4] = [1., 0., 0., 1.];
const WHITE: [f32; 4] = [1f32; 4];

struct State<'a> {
    renderer: Renderer,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    time: Instant,

    component: Box<dyn ComponentAudio<SystemAudioFetcher>>,
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

        let component: Box<dyn ComponentAudio<SystemAudioFetcher>> = match component_name {
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
                base_color: [0., 0.5, 0.5].into(),
                movement_speed: 0.005,
                sensitivity: 0.2,
                seed: None,
            })) as Box<dyn ComponentAudio<_>>),
            ComponentName::BarsColorVariant => Bars::new(&BarsDescriptor {
                renderer: &renderer,
                sample_processor: &processor,
                audio_conf: BarProcessorConfig {
                    amount_bars: std::num::NonZero::new(60).unwrap(),
                    sensitivity: 4.,
                    ..Default::default()
                },
                texture_format: surface_config.format,
                max_height: 0.5,
                variant: BarVariant::Color([0., 0., 1., 1.].into()),
                // placement: BarsPlacement::Custom {
                //     bottom_left_corner: (0.5, 0.5),
                //     width_factor: 0.5,
                //     rotation: cgmath::Deg(45.),
                // },
                placement: BarsPlacement::Bottom,
                format: BarsFormat::BassTreble,
            })
            .map(|bars| Box::new(bars) as Box<dyn ComponentAudio<_>>),
            ComponentName::BarsPresenceGradientVariant => Bars::new(&BarsDescriptor {
                renderer: &renderer,
                sample_processor: &processor,
                audio_conf: BarProcessorConfig {
                    sensitivity: 4.,
                    amount_bars: NonZero::new(30).unwrap(),
                    ..Default::default()
                },
                texture_format: surface_config.format,
                max_height: 0.25,
                variant: BarVariant::PresenceGradient {
                    high: TURQUOISE.into(),
                    low: DARK_BLUE.into(),
                },
                placement: BarsPlacement::Custom {
                    bottom_left_corner: (0., 0.5),
                    width: NonZero::new(100).unwrap(),
                    rotation: Deg(0.),
                    height_mirrored: true,
                },
                format: BarsFormat::TrebleBassTreble,
            })
            .map(|bars| Box::new(bars) as Box<dyn ComponentAudio<_>>),
            ComponentName::CircleCurvedVariant => Ok(Box::new(Circle::new(&CircleDescriptor {
                renderer: &renderer,
                sample_processor: processor,
                audio_conf: vibe_audio::BarProcessorConfig {
                    amount_bars: std::num::NonZero::new(30).unwrap(),
                    ..Default::default()
                },
                texture_format: surface_config.format,
                variant: CircleVariant::Graph {
                    spike_sensitivity: 0.3,
                    color: TURQUOISE.into(),
                },

                radius: 0.1,
                rotation: cgmath::Deg(90.),
                position: (0.5, 0.5),
            })) as Box<dyn ComponentAudio<_>>),
            ComponentName::FragmentCanvas => {
                let fragment_source = ShaderCode {
                    language: vibe_renderer::components::ShaderLanguage::Wgsl,
                    source: vibe_renderer::components::ShaderSource::Code(
                        "
                    @fragment
                    fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                        let uv = pos.xy / iResolution.xy - iMouse.xy;
                        return vec4(abs(uv), .0, 1.);
                    }
                    "
                        .into(),
                    ),
                };

                FragmentCanvas::new(&FragmentCanvasDescriptor {
                    sample_processor: &processor,
                    audio_conf: vibe_audio::BarProcessorConfig::default(),
                    renderer: &renderer,
                    format: surface_config.format,

                    img: None,
                    fragment_code: fragment_source,
                })
                .map(|fc| Box::new(fc) as Box<dyn ComponentAudio<_>>)
            }
            ComponentName::GraphColorVariant => Ok(Box::new(Graph::new(&GraphDescriptor {
                renderer: &renderer,
                sample_processor: processor,
                audio_conf: BarProcessorConfig::default(),
                output_texture_format: surface_config.format,
                variant: GraphVariant::Color(BLUE.into()),
                max_height: 0.5,
                format: GraphFormat::BassTreble,
                // placement: vibe_renderer::components::GraphPlacement::Bottom,
                placement: vibe_renderer::components::GraphPlacement::Custom {
                    bottom_left_corner: [0.25, 0.2],
                    rotation: Deg(-45.),
                    amount_bars: NonZero::new(500).unwrap(),
                },
            })) as Box<dyn ComponentAudio<_>>),
            ComponentName::GraphHorizontalGradientVariant => {
                Ok(Box::new(Graph::new(&GraphDescriptor {
                    renderer: &renderer,
                    sample_processor: processor,
                    audio_conf: BarProcessorConfig {
                        sensitivity: 4.0,
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,
                    variant: GraphVariant::HorizontalGradient {
                        left: RED.into(),
                        right: BLUE.into(),
                    },
                    max_height: 0.5,
                    format: GraphFormat::BassTreble,
                    placement: vibe_renderer::components::GraphPlacement::Bottom,
                })) as Box<dyn ComponentAudio<_>>)
            }
            ComponentName::GraphVerticalGradientVariant => {
                Ok(Box::new(Graph::new(&GraphDescriptor {
                    renderer: &renderer,
                    sample_processor: processor,
                    audio_conf: BarProcessorConfig {
                        amount_bars: NonZero::new(256).unwrap(),
                        sensitivity: 4.0,
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,
                    variant: GraphVariant::VerticalGradient {
                        top: RED.into(),
                        bottom: BLUE.into(),
                    },
                    max_height: 0.5,
                    format: GraphFormat::BassTrebleBass,
                    placement: vibe_renderer::components::GraphPlacement::Bottom,
                    // placement: vibe_renderer::components::GraphPlacement::Custom {
                    //     bottom_left_corner: [0.5, 0.2],
                    //     rotation: Deg(-45.),
                    // },
                })) as Box<dyn ComponentAudio<_>>)
            }
            ComponentName::RadialColorVariant => Ok(Box::new(Radial::new(&RadialDescriptor {
                renderer: &renderer,
                processor,
                audio_conf: vibe_audio::BarProcessorConfig {
                    amount_bars: NonZero::new(60).unwrap(),
                    sensitivity: 4.0,
                    ..Default::default()
                },
                output_texture_format: surface_config.format,

                variant: RadialVariant::Color(RED.into()),

                init_rotation: cgmath::Deg(90.),
                circle_radius: 0.2,
                bar_height_sensitivity: 0.5,
                bar_width: 0.015,
                position: (0.5, 0.5),
                format: RadialFormat::TrebleBass,
            })) as Box<dyn ComponentAudio<_>>),

            ComponentName::RadialHeightGradientVariant => {
                Ok(Box::new(Radial::new(&RadialDescriptor {
                    renderer: &renderer,
                    processor,
                    audio_conf: vibe_audio::BarProcessorConfig {
                        amount_bars: NonZero::new(60).unwrap(),
                        sensitivity: 4.0,
                        ..Default::default()
                    },
                    output_texture_format: surface_config.format,

                    variant: RadialVariant::HeightGradient {
                        inner: RED.into(),
                        outer: WHITE.into(),
                    },

                    init_rotation: cgmath::Deg(90.),
                    circle_radius: 0.3,
                    bar_height_sensitivity: 1.,
                    bar_width: 0.02,
                    position: (0.5, 0.5),
                    format: RadialFormat::TrebleBass,
                })) as Box<dyn ComponentAudio<_>>)
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
            })) as Box<dyn ComponentAudio<_>>),

            ComponentName::TextureValueNoise => {
                let texture = renderer.generate(&ValueNoise {
                    texture_size: 256,
                    octaves: 7,
                    seed: None,
                });

                Ok(Box::new(TextureComponent::new(&TextureComponentDescriptor {
                    device: renderer.device(),
                    texture,
                    format: surface_config.format,
                })) as Box<dyn ComponentAudio<_>>)
            }
            ComponentName::TextureSdf => {
                let texture = renderer.generate(&SdfMask {
                    texture_size: 256,
                    pattern: SdfPattern::Box,
                });

                Ok(Box::new(TextureComponent::new(&TextureComponentDescriptor {
                    device: renderer.device(),
                    texture,
                    format: surface_config.format,
                })) as Box<dyn ComponentAudio<_>>)
            }
            ComponentName::WallpaperPulseEdges => Ok(Box::new(
                PulseEdges::new(&PulseEdgesDescriptor {
                    renderer: &renderer,
                    sample_processor: &processor,
                    img: image::ImageReader::open("./assets/castle.jpg")
                        .unwrap()
                        .decode()
                        .unwrap(),

                    freq_range: NonZero::new(100).unwrap()..NonZero::new(250).unwrap(),
                    audio_sensitivity: 8.,
                    texture_format: surface_config.format,

                    low_threshold_ratio: 0.4,
                    high_threshold_ratio: 0.6,
                    wallpaper_brightness: 0.5,
                    edge_width: 0.3,
                    pulse_brightness: 1.5,
                    sigma: 10.,
                    kernel_size: 49,
                })
                .unwrap(),
            ) as Box<dyn ComponentAudio<_>>),
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

        self.renderer.render(&view, &[self.component.as_ref()]);

        surface_texture.present();
        Ok(())
    }

    pub fn update_mouse_pos(&mut self, new_pos: PhysicalPosition<f64>) {
        let rel_x = new_pos.x as f32 / self.surface_config.width as f32;
        let rel_y = new_pos.y as f32 / self.surface_config.height as f32;

        self.component
            .update_mouse_position(self.renderer.queue(), (rel_x, rel_y));
    }
}

struct App<'a> {
    sample_processor: SampleProcessor<SystemAudioFetcher>,
    state: Option<State<'a>>,
    variant: ComponentName,
}

impl<'a> App<'a> {
    pub fn new(variant: ComponentName, device_id: Option<DeviceId>) -> anyhow::Result<Self> {
        let sample_processor = {
            let device = match device_id {
                Some(device_id) => {
                    match vibe_audio::util::get_device(device_id.clone(), DeviceType::Output)? {
                        Some(device) => device,
                        None => {
                            bail!(
                                concat![
                                    "Available output devices:\n\n{:#?}\n",
                                    "\nThere's no output device called \"{}\".\n",
                                    "Please choose one from the list.\n",
                                ],
                                vibe_audio::util::get_device_ids(DeviceType::Output)?,
                                device_id.to_string()
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
                            vibe_audio::util::get_device_ids(DeviceType::Output)?,
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
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                state.update_mouse_pos(position);
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    init_logging();
    let cli = cli::Cli::parse();

    if cli.show_output_devices {
        println!(
            "\nAvailable output devices:\n\n{:#?}\n",
            vibe_audio::util::get_device_ids(vibe_audio::util::DeviceType::Output)?
        );
        return Ok(());
    }

    if let Some(component) = cli.component_name {
        let event_loop = EventLoop::new()?;

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        let mut app = App::new(
            component,
            cli.output_device_id
                .map(|id| DeviceId::from_str(&id).unwrap()),
        )?;
        event_loop.run_app(&mut app).unwrap();
    }

    Ok(())
}

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::builder().parse("vibe_renderer=info").unwrap());

    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::fmt()
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_env_filter(env_filter)
        .without_time()
        .pretty()
        .finish()
        .with(indicatif_layer)
        .init();

    tracing::debug!("Debug logging enabled");
}
