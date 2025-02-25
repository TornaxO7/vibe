mod config;
mod daemon;
mod gpu;
mod output;
pub mod state;

use std::num::NonZeroUsize;

use state::State;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use wayland_client::{globals::registry_queue_init, Connection};

const DEFAULT_AMOUNT_BARS: NonZeroUsize = NonZeroUsize::new(60).unwrap();

fn main() -> anyhow::Result<()> {
    init_logging();

    let daemon = daemon::Daemon::new()?;
    let (mut state, mut event_loop) = {
        let conn = Connection::connect_to_env()?;
        let (globals, event_loop) = registry_queue_init(&conn)?;
        let qh = event_loop.handle();
        let state = State::new(&globals, qh);

        (state, event_loop)
    };

    while state.run {
        event_loop.flush()?;
        if let Some(read_guard) = event_loop.prepare_read() {
            read_guard.read()?;
            event_loop.dispatch_pending(&mut state)?;
        };

        // == Daemon ==
        daemon.apply_action(&mut state);
    }

    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::builder().parse("vibe=info").unwrap()),
        )
        .without_time()
        .pretty()
        .init();

    debug!("Debug logger initialised");
    info!("Info logger initialised");
}
