mod vibe_output_state;

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
use tracing::{debug, instrument};
use vibe_output_state::VibeOutputState;

pub struct State {
    pub run: bool,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,

    pub wgpu_states: Vec<(WlOutput, VibeOutputState)>,
}

impl State {
    #[instrument(skip_all)]
    pub fn new(globals: GlobalList, event_queue_handle: QueueHandle<Self>) -> Self {
        Self {
            run: true,
            layer_shell: LayerShell::bind(&globals, &event_queue_handle).expect("Compositor does not implement the wlr_layer_shell protocol. (https://wayland.app/protocols/wlr-layer-shell-unstable-v1)"),
            registry_state: RegistryState::new(&globals),
            output_state: OutputState::new(&globals, &event_queue_handle),
            compositor_state: CompositorState::bind(&globals, &event_queue_handle)
                .expect("Retrieve compositor state"),

            wgpu_states: Vec::new(),
        }
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
        let (width, height) = self
            .output_state
            .info(&output)
            .expect("Retrieve output size (width and height)")
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

        debug!("Adding new output to {}.", vibe_daemon::APP_NAME);

        let wgpu_state = VibeOutputState::new(conn, layer, width, height);
        self.wgpu_states.push((output, wgpu_state));
    }

    #[instrument(skip_all)]
    fn update_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        debug!("Updating new output to vibe.");

        // remove the old entry
        self.output_destroyed(conn, qh, output.clone());

        // just append the output again
        self.new_output(conn, qh, output);
    }

    #[instrument(skip_all)]
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        debug!("Removing output to vibe.");

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
