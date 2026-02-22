use crate::{
    config::ConfigError,
    output::{
        config::{component::Config, OutputConfig},
        OutputCtx,
    },
    types::size::Size,
};
use anyhow::Context;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
};
use std::{collections::HashMap, ptr::NonNull, time::Instant};
use tracing::{debug, error, info, warn};
use vibe_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use vibe_renderer::Renderer;
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output::WlOutput, wl_pointer::WlPointer, wl_surface::WlSurface},
    Connection, Proxy, QueueHandle,
};

pub struct State {
    pub run: bool,

    default_component: Config,

    output_state: OutputState,
    registry_state: RegistryState,
    seat_state: SeatState,
    layer_shell: LayerShell,
    compositor_state: CompositorState,

    renderer: Renderer,
    sample_processor: SampleProcessor<SystemAudioFetcher>,

    time: Instant,
    pointer: Option<WlPointer>,

    outputs: HashMap<WlOutput, OutputCtx>,
}

impl State {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> anyhow::Result<Self> {
        if is_on_kde() {
            message_to_kde();
        }

        let Ok(layer_shell) = LayerShell::bind(globals, qh) else {
            error!(concat![
                "Your compositor doesn't seem to implement the wlr_layer_shell protocol but this is required for this program to run. ",
                "Here's a list of compositors which implements this protocol: <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support>\n"
            ]);

            panic!("wlr_layer_shell protocol is not supported by compositor.");
        };

        let vibe_config = crate::config::load().unwrap_or_else(|err| {
            let config_path = crate::get_config_path();
            let default_config = crate::config::Config::default();

            match err {
                ConfigError::IO(io_err) =>
                {
                    match io_err.kind() {
                        std::io::ErrorKind::NotFound => {
                            warn!(concat![
                                "Looks like you are starting `vibe` for the first time.\n",
                                "\tPlease see the 5th point here: <https://github.com/TornaxO7/vibe/blob/main/USAGE.md>\n",
                                "\tto check if `vibe` is listenting to the correct source."
                            ]);

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

        let sample_processor = vibe_config.sample_processor()?;

        let renderer = Renderer::new(&vibe_renderer::RendererDescriptor::from(
            &vibe_config.graphics_config,
        ));

        Ok(Self {
            run: true,
            compositor_state: CompositorState::bind(globals, qh).unwrap(),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            layer_shell,
            renderer,

            time: Instant::now(),
            pointer: None,

            sample_processor,

            outputs: HashMap::new(),

            default_component: vibe_config.default_component.unwrap_or_default(),
        })
    }

    pub fn render(&mut self, output_key: WlOutput, qh: &QueueHandle<Self>) {
        let output = self.outputs.get_mut(&output_key).unwrap();

        // update the buffers for the next frame
        {
            let queue = self.renderer.queue();
            let curr_time = self.time.elapsed().as_secs_f32();

            for component in output.components.iter_mut() {
                component.update_audio(queue, &self.sample_processor);
                component.update_time(queue, curr_time);
            }
        }

        match output.surface().get_current_texture() {
            Ok(surface_texture) => {
                self.renderer.render(
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    &output.components,
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

        let config = match crate::output::config::load(&name) {
            Some((path, res)) => match res {
                Ok(config) => {
                    info!("Reusing '{}'.", path.to_string_lossy());
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
            None => match OutputConfig::new(&info, self.default_component.clone()) {
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
            let wl_surface = self.compositor_state.create_surface(qh);
            let layer_surface = self.layer_shell.create_layer_surface(
                qh,
                wl_surface,
                Layer::Background,
                Some(format!("{} background", crate::APP_NAME)),
                Some(&output),
            );
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

        self.render(key, qh);
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

delegate_seat!(State);
impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            debug!("Mouse found");
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_some() {
            debug!("Mouse removed");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
}

delegate_pointer!(State);
impl PointerHandler for State {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            if let Some(output) = self
                .outputs
                .values_mut()
                .find(|output| &event.surface == output.layer_surface().wl_surface())
            {
                if let PointerEventKind::Motion { .. } = event.kind {
                    let queue = self.renderer.queue();
                    output.update_mouse_position(queue, event.position);
                }
            }
        }
    }
}

fn is_on_kde() -> bool {
    match std::env::var("XDG_CURRENT_DESKTOP") {
        Ok(env) => return env == "KDE",
        Err(err) => {
            warn!("Couldn't read variable '$XDG_CURRENT_DESKTOP`:\n{}", err);
        }
    }

    false
}

fn message_to_kde() {
    warn!(concat![
        "It looks like you are running `vibe` on `KDE` (at least that's what `$XDG_CURRENT_DESKTOP` is telling me).\n",
        "Please note that you won't be able to interact with your desktop if `vibe` is running.\n",
        "It's sadly an unsolveable problem :(\n",
        "But if you still want a visualizer for your kde desktop, take a look at `kurve`: https://store.kde.org/p/2299506"
    ]);
}
