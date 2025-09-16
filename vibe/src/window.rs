use std::{
    path::PathBuf,
    sync::{mpsc::Receiver, Arc},
    time::Instant,
};

use anyhow::{bail, Context};
use notify::{INotifyWatcher, Watcher};
use tracing::{error, warn};
use vibe_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use vibe_renderer::{
    components::{Component, ShaderCodeError},
    Renderer, RendererDescriptor,
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, keyboard::Key,
    window::Window,
};

use crate::{
    output::config::{component::ComponentConfig, OutputConfig},
    types::size::Size,
};

struct State<'a> {
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,

    components: Vec<Box<dyn Component<SystemAudioFetcher>>>,
}

impl State<'_> {
    pub fn new(window: Window, renderer: &Renderer) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();

        let surface = renderer.instance().create_surface(window.clone()).unwrap();

        let surface_config =
            crate::output::get_surface_config(renderer.adapter(), &surface, Size::from(size));
        surface.configure(renderer.device(), &surface_config);

        Self {
            surface,
            surface_config,
            window,
            components: Vec::new(),
        }
    }

    pub fn refresh_components(
        &mut self,
        renderer: &Renderer,
        processor: &SampleProcessor<SystemAudioFetcher>,
        comp_configs: &[ComponentConfig],
    ) -> Result<(), ShaderCodeError> {
        let mut new_components = Vec::with_capacity(comp_configs.len());

        for config in comp_configs.iter() {
            let mut component =
                config.to_component(renderer, processor, self.surface_config.format)?;

            component.update_resolution(
                renderer,
                [self.surface_config.width, self.surface_config.height],
            );

            new_components.push(component);
        }

        self.components = new_components;
        Ok(())
    }

    pub fn resize(&mut self, new_size: Size, renderer: &Renderer) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface
                .configure(renderer.device(), &self.surface_config);

            for component in self.components.iter_mut() {
                component.update_resolution(renderer, [new_size.width, new_size.height]);
            }
        }
    }

    pub fn render(&self, renderer: &Renderer) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        renderer.render(&view, &self.components);

        surface_texture.present();
        Ok(())
    }
}

struct OutputRenderer<'a> {
    processor: SampleProcessor<SystemAudioFetcher>,
    renderer: Renderer,
    state: Option<State<'a>>,

    output_config: OutputConfig,
    output_name: String,
    lookup_paths: Vec<PathBuf>,
    watcher: INotifyWatcher,
    rx: Receiver<notify::Result<notify::Event>>,
    time: Instant,
}

impl OutputRenderer<'_> {
    pub fn new(output_name: String) -> anyhow::Result<Self> {
        let config = crate::config::load()?;

        let renderer = Renderer::new(&RendererDescriptor::from(config.graphics_config.clone()));
        let processor = config.sample_processor(Some(2))?;

        let (output_config_path, output_config) = {
            let Some((path, config)) = crate::output::config::load(&output_name) else {
                bail!(
                    "The config file for '{}' does not exist. Can't start hot reloading.`",
                    output_name
                );
            };

            match config {
                Ok(config) => (path, config),
                Err(err) => {
                    error!("{:?}", err);
                    (
                        path,
                        OutputConfig {
                            enable: true,
                            components: Vec::new(),
                        },
                    )
                }
            }
        };

        let lookup_paths = output_config.external_paths();

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        for path in lookup_paths.iter() {
            watcher.watch(path, notify::RecursiveMode::NonRecursive)?;
        }

        // don't forget to watch the actual config file as well
        watcher.watch(&output_config_path, notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            renderer,
            processor,
            state: None,

            watcher,
            lookup_paths,
            rx,
            output_config,
            output_name,
            time: Instant::now(),
        })
    }

    pub fn config_is_modified(&self) -> bool {
        let events: Vec<notify::Result<notify::Event>> = self.rx.try_iter().collect();

        for event in events {
            let event = event.unwrap_or_else(|err| {
                error!("Something happened while checking if any specifique files have been modified:\n{}", err);
                panic!();
            });

            if event.kind.is_modify() || event.kind.is_create() {
                return true;
            }
        }

        false
    }

    // Returns `Err` if something un-saveable happened. => Signal for exiting
    pub fn refresh_config(&mut self) -> anyhow::Result<()> {
        self.output_config = {
            let Some((path, output_config)) = crate::output::config::load(&self.output_name) else {
                bail!(
                    "The config file of your output '{}' got removed. `vibe` will stop rendering...",
                    self.output_name
                );
            };

            let _ = self.watcher.unwatch(&path);
            self.watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .context("Start watching the output config file.")?;

            match output_config {
                Ok(conf) => conf,
                Err(err) => {
                    error!("{:?}", err);
                    return Ok(());
                }
            }
        };

        // refresh lookup paths
        while let Some(path) = self.lookup_paths.pop() {
            let _ = self.watcher.unwatch(&path);
        }

        // add all paths within the config file as well
        for path in self.output_config.external_paths() {
            self.lookup_paths.push(path.clone());

            if let Err(err) = self
                .watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
            {
                bail!(
                    "Couldn't start watching file '{}':\n{}",
                    path.to_string_lossy(),
                    err
                );
            }
        }

        // update components to render
        if let Some(state) = self.state.as_mut() {
            if let Err(err) = state.refresh_components(
                &self.renderer,
                &self.processor,
                &self.output_config.components,
            ) {
                error!("{}", err);
            }
        }

        Ok(())
    }
}

impl ApplicationHandler for OutputRenderer<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                winit::window::WindowAttributes::default()
                    .with_title(format!("vibe - {}", &self.output_name)),
            )
            .expect("Create window");

        self.state = Some(State::new(window, &self.renderer));

        if let Err(err) = self.refresh_config() {
            error!("{:?}", err);
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.config_is_modified() {
            if let Err(err) = self.refresh_config() {
                error!("{:?}", err);
                event_loop.exit();
                return;
            }
        }

        let state = self.state.as_mut().unwrap();

        match event {
            WindowEvent::RedrawRequested => {
                state.window.request_redraw();

                self.processor.process_next_samples();
                for component in state.components.iter_mut() {
                    component.update_time(self.renderer.queue(), self.time.elapsed().as_secs_f32());
                    component.update_audio(self.renderer.queue(), &self.processor);
                }

                match state.render(&self.renderer) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Timeout) => {
                        error!("Surface timeout");
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        unreachable!("Dayum, you don't have any memory left for rendering....");
                    }
                    Err(err) => {
                        warn!("{}", err);
                    }
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(Size::from(new_size), &self.renderer);
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Character("q".into()) =>
            {
                event_loop.exit()
            }
            _ => {}
        }
    }
}

pub fn run(output_name: String) -> anyhow::Result<()> {
    let mut app = OutputRenderer::new(output_name)?;
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app)?;
    Ok(())
}
