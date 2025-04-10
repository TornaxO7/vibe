use std::{
    path::PathBuf,
    sync::{mpsc::Receiver, Arc},
    time::Instant,
};

use anyhow::Context;
use notify::{INotifyWatcher, Watcher};
use shady_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use tracing::{error, info, warn};
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

    components: Vec<Box<dyn Component>>,
}

impl<'a> State<'a> {
    pub fn new<'b>(window: Window, renderer: &'b Renderer) -> Self {
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
        processor: &SampleProcessor,
        comp_configs: &[ComponentConfig],
    ) -> Result<(), ShaderCodeError> {
        let mut new_components = Vec::with_capacity(comp_configs.len());

        for config in comp_configs.iter() {
            let component = config.to_component(renderer, processor, self.surface_config.format)?;
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
                component.update_resolution(renderer.queue(), [new_size.width, new_size.height]);
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
    processor: SampleProcessor,
    renderer: Renderer,
    state: Option<State<'a>>,

    output_config: OutputConfig,
    output_name: String,
    lookup_paths: Vec<PathBuf>,
    watcher: INotifyWatcher,
    rx: Receiver<notify::Result<notify::Event>>,
    time: Instant,
}

impl<'a> OutputRenderer<'a> {
    pub fn new(output_name: String) -> anyhow::Result<Self> {
        let renderer = {
            let config = crate::config::load()?;
            Renderer::new(&RendererDescriptor::from(config.graphics_config))
        };

        let processor =
            SampleProcessor::new(SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap());

        let output_config = {
            let Some(config) = crate::output::config::load(&output_name) else {
                anyhow::bail!(
                    "The config file for '{}' does not exist. Can't start hot reloading.`",
                    output_name
                );
            };

            config?
        };

        let lookup_paths = output_config.hot_reloading_paths();

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        for path in lookup_paths.iter() {
            watcher.watch(path, notify::RecursiveMode::NonRecursive)?;
        }

        // don't forget to watch the actual config file as well
        let output_config_path = crate::output::config::get_path(&output_name);
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

    pub fn refresh_config(&mut self) {
        self.output_config = {
            let Some(output_config) = crate::output::config::load(&self.output_name) else {
                error!(
                    "The config file of your output '{}' got removed. `vibe` will stop rendering...",
                    self.output_name
                );
                panic!();
            };

            match output_config {
                Ok(conf) => conf,
                Err(err) => {
                    error!("{:?}", err);
                    return;
                }
            }
        };

        // refresh lookup paths
        while let Some(path) = self.lookup_paths.pop() {
            self.watcher.unwatch(&path).unwrap_or_else(|err| {
                error!("Couldn't unwatch paths of the previous config: {}", err);
                panic!();
            });
        }

        // don't forget the config path as well
        let output_config_file_path = crate::output::config::get_path(&self.output_name);
        let _ = self.watcher.unwatch(&output_config_file_path);
        self.watcher
            .watch(
                &output_config_file_path,
                notify::RecursiveMode::NonRecursive,
            )
            .unwrap();

        // add all paths within the config file as well
        for path in self.output_config.hot_reloading_paths() {
            self.lookup_paths.push(path.clone());
            self.watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .unwrap_or_else(|err| {
                    error!(
                        "Couldn't start watching file '{}':\n{}",
                        path.to_string_lossy(),
                        err
                    );
                    panic!();
                });
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
    }
}

impl<'a> ApplicationHandler for OutputRenderer<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(winit::window::WindowAttributes::default())
            .expect("Create window");

        self.state = Some(State::new(window, &self.renderer));
        self.refresh_config();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.config_is_modified() {
            self.refresh_config();
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
                        error!("Surface timout");
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
