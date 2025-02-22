mod socket_listener;
mod state;

use socket_listener::SocketListener;
use state::State;

use smithay_client_toolkit::reexports::client::{globals::registry_queue_init, Connection};
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
enum Action {
    Exit,
}

fn main() -> anyhow::Result<()> {
    init_logging();

    let (mut state, mut event_queue) = {
        let conn = Connection::connect_to_env().expect("Connect to wayland server");
        let (globals, event_queue) = registry_queue_init(&conn).expect("Init registry queue");
        let state = State::new(globals, event_queue.handle())?;

        (state, event_queue)
    };

    let mut listener = SocketListener::new()?;

    while state.run {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if let Some(action) = listener.get_next_action() {
            match action {
                Action::Exit => state.run = false,
            };
        }
    }

    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .without_time()
        .pretty()
        .init();

    debug!("Logger initialised");
}
