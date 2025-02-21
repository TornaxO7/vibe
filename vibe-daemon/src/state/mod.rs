mod vibe_output_state;
mod wayland_handle;

use std::collections::HashMap;

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
use wayland_handle::WaylandHandle;

pub struct State {
    pub run: bool,

    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,

    pub globals: GlobalList,

    pub wgpu_states: Vec<(WlOutput, WlSurface, VibeOutputState)>,
}

impl State {
    pub fn new(globals: GlobalList, event_queue_handle: QueueHandle<Self>) -> Self {
        Self {
            run: true,

            registry_state: RegistryState::new(&globals),
            output_state: OutputState::new(&globals, &event_queue_handle),
            compositor_state: CompositorState::bind(&globals, &event_queue_handle)
                .expect("Retrieve compositor state"),

            globals,
            wgpu_states: Vec::new(),
        }
    }
}

delegate_output!(State);

impl OutputHandler for State {
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

        let wl_surface = self.compositor_state.create_surface(qh);
        let wl_handle = WaylandHandle::new(conn.clone(), wl_surface.clone());

        let layer = {
            let layer_shell = LayerShell::bind(&self.globals, qh).expect("Compositor does not implement the wlr_layer_shell protocol. (https://wayland.app/protocols/wlr-layer-shell-unstable-v1)");

            let layer = layer_shell.create_layer_surface(
                qh,
                wl_surface.clone(),
                Layer::Background,
                Some("Music visualizer background"),
                Some(&output),
            );

            layer.set_anchor(Anchor::BOTTOM);
            layer.set_keyboard_interactivity(KeyboardInteractivity::None);
            layer.set_size(width, height);

            // > After creating a layer_surface object and setting it up, the client must perform an initial commit without any buffer attached.
            //
            // https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:get_layer_surface
            layer.commit();
            layer
        };

        debug!("Adding new output to vibe.");

        let wgpu_state = VibeOutputState::new(wl_handle, layer, width, height);
        self.wgpu_states.push((output, wl_surface, wgpu_state));
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
        debug!("Removing new output to vibe.");

        let mut i = 0;
        for (out, _surface, _state) in self.wgpu_states.iter() {
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
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &WlSurface,
        _time: u32,
    ) {
        for (_out, sur, state) in self.wgpu_states.iter_mut() {
            if sur == surface {
                match state.render() {
                    Ok(_) => {
                        state.prepare_next_frame();
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        unreachable!("Out of memory");
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        tracing::error!("A frame took too long to be present");
                    }
                    Err(err) => tracing::warn!("{}", err),
                }
                if let Err(err) = state.render() {
                    tracing::error!("{}", err);
                }
            }
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
        for (out, sur, _state) in self.wgpu_states.iter_mut() {
            if sur == surface {
                *out = output.clone();
            }
        }
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

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    fn runtime_add_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
        _version: u32,
    ) {
    }

    fn runtime_remove_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
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
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(state) = self
            .wgpu_states
            .iter_mut()
            .map(|(_out, _surface, state)| state)
            .find(|state| &state.layer == layer)
        {
            let new_width = configure.new_size.0;
            let new_height = configure.new_size.1;

            if new_width > 0 && new_height > 0 {
                state.resize(new_width, new_height);
            }
        }
    }
}
