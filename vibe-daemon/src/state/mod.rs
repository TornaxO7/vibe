mod vibe_output_state;

use std::{collections::HashMap, io};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::client::{
        globals::GlobalList,
        protocol::{
            wl_output::{Transform, WlOutput},
            wl_surface::WlSurface,
        },
        Connection, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
};
use tracing::{debug, error, instrument};
use vibe_daemon::config::OutputConfig;
use vibe_output_state::VibeOutputState;
use wgpu::naga::{self, front::glsl::Options, ShaderStage};

pub struct State {
    pub run: bool,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,

    output_configs: HashMap<String, OutputConfig>,
    wgpu_states: Vec<(WlOutput, VibeOutputState)>,
}

impl State {
    #[instrument(skip_all)]
    pub fn new(globals: GlobalList, event_queue_handle: QueueHandle<Self>) -> io::Result<Self> {
        let layer_shell = LayerShell::bind(&globals, &event_queue_handle).expect(concat![
            "Your compositor doesn't support the wlr_layer_shell protocol :(\n",
            "A list of compositors which support wlr_layer_shell can be seen here: <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support>"
        ]);

        Ok(Self {
            run: true,

            layer_shell,
            registry_state: RegistryState::new(&globals),
            output_state: OutputState::new(&globals, &event_queue_handle),
            compositor_state: CompositorState::bind(&globals, &event_queue_handle)
                .expect("Retrieve compositor state"),

            wgpu_states: Vec::new(),
            output_configs: HashMap::new(),
        })
    }
}

delegate_output!(State);
impl OutputHandler for State {
    #[instrument(skip_all)]
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    #[instrument(skip_all)]
    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        let output_info = self
            .output_state
            .info(&output)
            .expect("Retrieve information of new output");

        match output_info.name {
            Some(name) => match vibe_daemon::config::load(&name) {
                Ok(Some(config)) => {
                    let (width, height) = output_info
                        .logical_size
                        .map(|(width, height)| (width as u32, height as u32))
                        .expect("Output has logical size");

                    let layer = {
                        let surface = self.compositor_state.create_surface(qh);

                        let layer = self.layer_shell.create_layer_surface(
                            qh,
                            surface,
                            Layer::Background,
                            Some("Music visualizer background"),
                            Some(&output),
                        );

                        layer.set_exclusive_zone(0);
                        layer.set_anchor(Anchor::BOTTOM);
                        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
                        debug!("Surface size: {} x {}", width, height);
                        layer.set_size(width, height);

                        // > After creating a layer_surface object and setting it up, the client must perform an initial commit without any buffer attached.
                        //
                        // https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:get_layer_surface
                        layer.commit();
                        layer
                    };

                    let shader = match &config.shader_code {
                        vibe_daemon::config::ShaderCode::Wgsl(code) => {
                            match naga::front::wgsl::parse_str(code) {
                                Ok(module) => module,
                                Err(err) => {
                                    error!("Couldn't parse wgsl shader: {}\nSkip shader...", err);
                                    return;
                                }
                            }
                        }
                        vibe_daemon::config::ShaderCode::Glsl(code) => {
                            let mut parser = naga::front::glsl::Frontend::default();

                            match parser.parse(&Options::from(ShaderStage::Fragment), code) {
                                Ok(module) => module,
                                Err(err) => {
                                    error!("Couldn't parse glsl shader: {}\nSkip shader...", err);
                                    return;
                                }
                            }
                        }
                    };

                    let wgpu_state = VibeOutputState::new(conn, layer, width, height, shader);

                    self.output_configs.insert(name, config);
                    self.wgpu_states.push((output, wgpu_state));
                }
                Ok(None) => tracing::warn!("The output '{}' doesn't have a config.", name),
                Err(err) => tracing::error!(
                    "Couldn't try to load the config of the output '{}': {}",
                    err,
                    name
                ),
            },
            None => unreachable!(concat![
                "Ooooooooof... your compositor doesn't provide a name for an output.\n",
                "",
                "Please create an issue about this one. I hoped that this wouldn't happen ;-;"
            ]),
        };
    }

    #[instrument(skip_all)]
    fn update_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        // remove the old entry
        self.output_destroyed(conn, qh, output.clone());

        // just append the output again
        self.new_output(conn, qh, output);
    }

    #[instrument(skip_all)]
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        let mut i = 0;
        for (out, _state) in self.wgpu_states.iter() {
            if *out == output {
                break;
            }
            i += 1;
        }

        self.wgpu_states.remove(i);
    }
}

delegate_compositor!(State);
impl CompositorHandler for State {
    #[instrument(skip_all)]
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_factor: i32,
    ) {
    }

    #[instrument(skip_all)]
    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: Transform,
    ) {
    }

    #[instrument(skip_all)]
    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        _time: u32,
    ) {
        for (_out, state) in self.wgpu_states.iter_mut() {
            let sur = state.layer.wl_surface();
            if sur == surface {
                state.prepare_next_frame();
                match state.render() {
                    Ok(_) => {
                        state.request_redraw(qh);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        unreachable!("Out of memory");
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        tracing::error!("A frame took too long to be present");
                    }
                    Err(err) => tracing::warn!("{}", err),
                }

                break;
            }
        }
    }

    #[instrument(skip_all)]
    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
        for (out, state) in self.wgpu_states.iter_mut() {
            let sur = state.layer.wl_surface();
            if sur == surface {
                *out = output.clone();
                break;
            }
        }
    }

    #[instrument(skip_all)]
    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }
}

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}

delegate_layer!(State);
impl LayerShellHandler for State {
    #[instrument(skip_all)]
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.run = false;
    }

    #[instrument(skip_all)]
    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(state) = self
            .wgpu_states
            .iter_mut()
            .map(|(_out, state)| state)
            .find(|state| &state.layer == layer)
        {
            let new_width = configure.new_size.0;
            let new_height = configure.new_size.1;

            if new_width > 0 && new_height > 0 {
                state.resize(new_width, new_height);
            }

            // do the initial rendering step
            state.request_redraw(qh);
        }
    }
}
