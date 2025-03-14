mod config;
mod output;
mod state;

use state::State;
use tracing_subscriber::EnvFilter;
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> anyhow::Result<()> {
    init_logging();

    let (mut state, mut event_loop) = {
        let conn = Connection::connect_to_env()?;
        let (globals, event_loop) = registry_queue_init(&conn)?;
        let qh = event_loop.handle();
        let state = State::new(&globals, &qh)?;

        (state, event_loop)
    };

    while state.run {
        event_loop.blocking_dispatch(&mut state)?;
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

    tracing::debug!("Debug logging enabled");
}
