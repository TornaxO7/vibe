mod socket_listener;
mod state;

use std::{sync::mpsc, thread, time::Duration};

use socket_listener::SocketListener;
use state::State;

use smithay_client_toolkit::reexports::client::{globals::registry_queue_init, Connection};
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
enum Action {
    Exit,
}

fn main() -> anyhow::Result<()> {
    init_logging();

    let (tx, rx) = mpsc::channel();

    let (mut state, mut event_queue) = {
        let conn = Connection::connect_to_env().expect("Connect to wayland server");
        let (globals, event_queue) = registry_queue_init(&conn).expect("Init registry queue");
        let state = State::new(globals, event_queue.handle());

        (state, event_queue)
    };

    let socket_listener_handle = {
        let mut listener = SocketListener::new(tx)?;
        thread::spawn(move || listener.run())
    };

    let mut run = true;
    while run {
        event_queue.dispatch_pending(&mut state).unwrap();

        match rx.try_recv() {
            Ok(action) => match action {
                Action::Exit => run = false,
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => unreachable!("Communication shouldn't be closed between the socket listener and the state/renderer"),
        };
    }

    socket_listener_handle.join().unwrap();

    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .pretty()
        .init();

    debug!("Logger initialised");
}
