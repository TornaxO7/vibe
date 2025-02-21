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

    pub wgpu_states: HashMap<WlOutput, VibeOutputState>,
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
            wgpu_states: HashMap::new(),
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
        self.wgpu_states.insert(output, wgpu_state);
    }

    #[instrument(skip_all)]
    fn update_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        debug!("Updating new output to vibe.");

        // just overwrite the output
        self.new_output(conn, qh, output);
    }

    #[instrument(skip_all)]
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        debug!("Removing new output to vibe.");
        self.wgpu_states.remove(&output);
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

    fn frame(&mut self, conn: &Connection, qh: &QueueHandle<Self>, surface: &WlSurface, time: u32) {
        todo!("The stuff with render pass ")
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
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        serial: u32,
    ) {
        todo!("Surface size changed")
    }
}
