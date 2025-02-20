mod state;
use state::State;

use smithay_client_toolkit::{
    reexports::client::{globals::registry_queue_init, Connection},
    registry::{RegistryHandler, RegistryState},
};

fn main() {
    let conn = Connection::connect_to_env().expect("Connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Init registry queue");

    let mut state = State::new(globals, event_queue.handle());

    while state.run {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
