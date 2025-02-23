use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

fn main() {
    init_logging();
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
