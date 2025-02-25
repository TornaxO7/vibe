use std::collections::HashMap;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
};
use tracing::{error, info, warn};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    Connection, QueueHandle,
};

use crate::{
    gpu_context::GpuCtx,
    output_config::OutputConfig,
    output_context::{OutputCtx, Size},
};

pub struct State {
    pub run: bool,

    output_state: OutputState,
    registry_state: RegistryState,
    layer_shell: LayerShell,
    compositor_state: CompositorState,

    gpu: GpuCtx,

    outputs: HashMap<WlOutput, OutputCtx>,
}

impl State {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Self {
        let gpu = GpuCtx::new();

        Self {
            run: true,
            compositor_state: CompositorState::bind(globals, qh).unwrap(),
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            layer_shell: LayerShell::bind(globals, qh).expect("Your compositor doesn't seem to implement the wlr_layer_shell protocol but this is required."),
            gpu,

            outputs: HashMap::new(),
        }
    }
}

delegate_output!(State);
impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {
        let info = self.output_state.info(&output).expect("Get output info");
        let name = info.name.clone().unwrap();
        info!("Detected output: '{}'", &name);

        let config = match crate::output_config::load(&info) {
            Some(config) => {
                info!("Reusing config of output '{}'.", name);
                config
            }
            None => match OutputConfig::new(&info) {
                Ok(config) => {
                    info!("Created new default config file for output: '{}'", name);
                    config
                }
                Err(err) => {
                    error!(
                        "Couldn't create new config for output '{}': {}. Skipping output...",
                        name, err
                    );
                    return;
                }
            },
        };

        if !config.enable {
            info!("Output is disabled. Skipping output.");
            return;
        }

        let layer_surface = {
            let wl_surface = self.compositor_state.create_surface(qh);
            self.layer_shell.create_layer_surface(
                qh,
                wl_surface,
                Layer::Background,
                Some(format!("{} background", crate::APP_NAME)),
                Some(&output),
            )
        };

        let ctx = match OutputCtx::new(conn, info, layer_surface, &self.gpu, config) {
            Ok(ctx) => ctx,
            Err(err) => {
                error!("Couldn't prepare new registered output: {}", err);
                return;
            }
        };
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
        let output = self
            .outputs
            .values_mut()
            .find(|ctx| ctx.layer_surface().wl_surface() == surface)
            .unwrap();

        match self.gpu.render(output) {
            Ok(_) => output.request_redraw(qh),
            Err(wgpu::SurfaceError::OutOfMemory) => unreachable!("Out of memory"),
            Err(wgpu::SurfaceError::Timeout) => {
                error!("A frame took too long to be present")
            }
            Err(err) => warn!("{}", err),
        }
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
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let new_size = Size::from(configure.new_size);

        let ctx = self
            .outputs
            .values_mut()
            .find(|ctx| ctx.layer_surface() == layer)
            .unwrap();

        ctx.resize(&self.gpu, new_size);
        ctx.request_redraw(qh);
    }
}

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}
