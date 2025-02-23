use cosmic::cctk::{
    sctk::{
        delegate_output, delegate_registry,
        output::{OutputHandler, OutputState},
        registry::{ProvidesRegistryState, RegistryState},
        registry_handlers,
    },
    wayland_client::{globals::GlobalList, protocol::wl_output::WlOutput, Connection, QueueHandle},
};
use tracing::warn;

type OutputName = String;

pub struct State {
    output_state: OutputState,
    registry_state: RegistryState,

    pub added_outputs: Vec<OutputName>,
    pub removed_outputs: Vec<OutputName>,
}

impl State {
    pub fn new(globals: &GlobalList, qh: QueueHandle<Self>) -> Self {
        let output_state = OutputState::new(&globals, &qh);
        let registry_state = RegistryState::new(&globals);

        Self {
            output_state,
            registry_state,
            added_outputs: Vec::new(),
            removed_outputs: Vec::new(),
        }
    }
}

delegate_output!(State);
impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        let info = self
            .output_state
            .info(&output)
            .expect("Retrieve output info of output.");

        match info.name {
            Some(name) => {
                self.added_outputs.push(name);
            }
            None => {
                warn!("Couldn't retrieve name of an output :( Please create an issue.")
            }
        }
    }

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: WlOutput) {
        let output_info = self.output_state.info(&output).unwrap();

        if let Some(name) = output_info.name {
            self.removed_outputs.push(name);
        }
    }
}

delegate_registry!(State);
impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}
