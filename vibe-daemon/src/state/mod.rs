use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};

use anyhow::Context;
use shady_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
};
use std::{collections::HashMap, ptr::NonNull};
use tracing::{debug, error, info, warn};
use vibe_renderer::Renderer;
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    Connection, Proxy, QueueHandle,
};

use crate::{
    config::ConfigError,
    output::{config::OutputConfig, OutputCtx},
    types::size::Size,
};

pub struct State {
    pub run: bool,

    output_state: OutputState,
    registry_state: RegistryState,
    layer_shell: LayerShell,
    compositor_state: CompositorState,

    renderer: Renderer,
    sample_processor: SampleProcessor,

    outputs: HashMap<WlOutput, OutputCtx>,
}

impl State {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> anyhow::Result<Self> {
        let Ok(layer_shell) = LayerShell::bind(globals, qh) else {
            error!(concat![
                "Your compositor doesn't seem to implement the wlr_layer_shell protocol but this is required for this program to run. ",
                "Here's a list of compositors which implements this protocol: <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support>\n"
            ]);

            panic!("wlr_layer_shell protocol is not supported by compositor.");
        };

        let sample_processor =
            SampleProcessor::new(SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap());

        let vibe_config = crate::config::load().unwrap_or_else(|err| {
            let config_path = crate::get_config_path();
            let default_config = crate::config::Config::default();

            match err {
                ConfigError::IO(io_err) =>
                {
                    match io_err.kind() {
                        std::io::ErrorKind::NotFound => {
                            if let Err(err) = default_config.save() {
                                warn!("Couldn't save default config file: {:?}", err);
                            }
                        }
                        _other => {
                            warn!("{}. Fallback to default config file", io_err);
                        }
                    };
                },
                ConfigError::Serde(serde_err) => {
                    let backup_path = {
                        let mut path = config_path.clone();
                        path.set_extension("back");
                        path
                    };

                    warn!(
                        "{:?} {} will be backup to {} and the default config will be saved and used.",
                        serde_err,
                        config_path.to_string_lossy(),
                        backup_path.to_string_lossy()
                    );

                    if let Err(err) = std::fs::copy(&config_path, &backup_path) {
                        warn!("Couldn't backup config file: {:?}. Won't create new config file.", err);
                    } else if let Err(err) = default_config.save() {
                        warn!("Couldn't create default config file: {:?}", err);
                    };
                }
            };

            default_config
        });

        let renderer = Renderer::new(&vibe_config.graphics_config);

        Ok(Self {
            run: true,
            compositor_state: CompositorState::bind(globals, qh).unwrap(),
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            layer_shell,
            renderer,

            sample_processor,

            outputs: HashMap::new(),
        })
    }

    pub fn render(&self, output: &OutputCtx, qh: &QueueHandle<Self>) {
        // update the buffers for the next frame
        {
            self.global_resources
                .update_ressource_buffers(self.renderer.queue());

            for shader in output.shaders.iter() {
                shader
                    .resources
                    .update_ressource_buffers(self.renderer.queue());
            }
        }

        let global_bind_groups = [
            self.global_resources.bind_group(),
            output.resources.bind_group(),
        ];

        match output.surface().get_current_texture() {
            Ok(surface_texture) => {
                self.renderer.render(
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    global_bind_groups,
                    output.render_shaders(),
                );
                surface_texture.present();
                output.request_redraw(qh);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => unreachable!("Out of memory"),
            Err(wgpu::SurfaceError::Timeout) => {
                error!("A frame took too long to be present")
            }
            Err(err) => warn!("{}", err),
        };
    }
}

delegate_output!(State);
impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        let info = self.output_state.info(&output).expect("Get output info");
        let name = info.name.clone().context(concat![
            "Ok, this might sound stupid, but I hoped that every compositor would give each output a name...\n",
            "but it looks like as if your compositor isn't doing that.\n",
            "Please create an issue and tell which compositor you're using (and that you got this error (you can copy+paste this)).\n",
            "\n",
            "Sorry for the inconvenience."
        ]).unwrap();

        info!("Detected output: '{}'", &name);

        let config = match crate::output::config::load(&info) {
            Some(res) => match res {
                Ok(config) => {
                    info!("Reusing config of output '{}'.", name);
                    config
                }
                Err(err) => {
                    error!(
                        "Couldn't load config of output '{}'. Skipping output:{:?}",
                        name, err
                    );

                    return;
                }
            },
            None => match OutputConfig::new(&info) {
                Ok(config) => {
                    info!("Created new default config file for output: '{}'", name);
                    config
                }
                Err(err) => {
                    error!(
                        "Couldn't create new config for output '{}': {:?}. Skipping output...",
                        name, err
                    );
                    return;
                }
            },
        };

        if !config.enable {
            info!("Output is disabled. Skipping output '{}'", name);
            return;
        }

        let layer_surface = {
            let region = Region::new(&self.compositor_state).unwrap();
            let wl_surface = self.compositor_state.create_surface(qh);
            let layer_surface = self.layer_shell.create_layer_surface(
                qh,
                wl_surface,
                Layer::Background,
                Some(format!("{} background", crate::APP_NAME)),
                Some(&output),
            );
            layer_surface.set_input_region(Some(region.wl_region()));
            layer_surface
        };

        let surface: wgpu::Surface<'static> = {
            let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
            ));

            let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
                NonNull::new(layer_surface.wl_surface().id().as_ptr() as *mut _).unwrap(),
            ));

            unsafe {
                self.renderer
                    .instance()
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            }
        };

        let ctx = OutputCtx::new(
            info,
            surface,
            layer_surface,
            &self.renderer,
            &self.sample_processor,
            config,
            &self.global_resources,
        );

        self.outputs.insert(output, ctx);
    }

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        info!("An output was removed.");
        self.outputs.remove(&output);
    }
}

delegate_compositor!(State);
impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        _time: u32,
    ) {
        self.sample_processor.process_next_samples();

        let key = self
            .outputs
            .iter()
            .find(|(_out, ctx)| ctx.layer_surface().wl_surface() == surface)
            .map(|(out, _ctx)| out.clone())
            .unwrap();

        // update the shader resources first before rendering
        {
            let output = self.outputs.get_mut(&key).unwrap();
            for shader in output.shaders.iter_mut() {
                shader
                    .resources
                    .audio
                    .fetch_bar_values(&self.sample_processor);
            }
        }

        let output = self.outputs.get(&key).unwrap();

        self.render(output, qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }
}

delegate_layer!(State);
impl LayerShellHandler for State {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.run = false;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let new_size = Size::from(configure.new_size);
        debug!("Configure new size: {:?}", new_size);

        let (key, surface) = self
            .outputs
            .iter()
            .find(|(_out, ctx)| ctx.layer_surface() == layer)
            .map(|(out, ctx)| (out.clone(), ctx.layer_surface().wl_surface().clone()))
            .unwrap();

        {
            let output_mut = self.outputs.get_mut(&key).unwrap();
            output_mut.resize(&self.renderer, new_size);
        }

        self.frame(conn, qh, &surface, 0);
    }
}

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}
