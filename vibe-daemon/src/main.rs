mod cli;
mod config;
mod output;
mod state;
mod types;
mod window;

use std::{path::PathBuf, sync::OnceLock};

use clap::Parser;
use state::State;
use tracing_subscriber::EnvFilter;
use wayland_client::{globals::registry_queue_init, Connection};
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const OUTPUT_CONFIG_DIR_NAME: &str = "output_configs";
const CONFIG_FILE_NAME: &str = "config.toml";

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    init_logging();

    let args = cli::Args::parse();
    if let Some(output_name) = args.output_name {
        window::run(output_name)
    } else {
        run_daemon()
    }
}

fn run_daemon() -> anyhow::Result<()> {
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

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME).unwrap())
}

/// Returns the path to the directory where the config files of each output lies.
/// Each config file has the form `<output-name>.toml`.
pub fn get_output_config_dir() -> PathBuf {
    get_xdg()
        .create_config_directory(OUTPUT_CONFIG_DIR_NAME)
        .unwrap()
}

/// Returns the path to the config file of `vibe`.
pub fn get_config_path() -> PathBuf {
    get_xdg().place_config_file(CONFIG_FILE_NAME).unwrap()
}
