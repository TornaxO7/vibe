mod cli;

use clap::Parser;
use tracing::{error, info};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vibe::{window, State};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> anyhow::Result<()> {
    init_logging();

    let args = cli::Args::parse();
    if args.show_output_devices {
        let device_ids = vibe_audio::util::get_device_ids(vibe_audio::util::DeviceType::Output)?
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>();

        info!("Available output devices:\n\n{:#?}\n", device_ids);
        return Ok(());
    }

    let result = if let Some(output_name) = args.output_name {
        window::run(output_name)
    } else {
        run_daemon()
    };

    if let Err(err) = result {
        error!("{:?}", err);
        anyhow::bail!("Fatal error");
    }

    Ok(())
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
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::builder().parse("vibe=info").unwrap());

    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::fmt()
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_env_filter(env_filter)
        .without_time()
        .pretty()
        .finish()
        .with(indicatif_layer)
        .init();

    tracing::debug!("Debug logging enabled");
}
