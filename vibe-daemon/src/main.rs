mod state;
use std::sync::OnceLock;

use state::State;

use smithay_client_toolkit::reexports::client::{globals::registry_queue_init, Connection};
use tracing::debug;
use tracing_subscriber::EnvFilter;
use xdg::BaseDirectories;

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    init_logging();

    let conn = Connection::connect_to_env().expect("Connect to wayland server");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Init registry queue");

    let mut state = State::new(globals, event_queue.handle());

    while state.run {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME).unwrap())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .pretty()
        .init();

    debug!("Logger initialised");
}
