// SPDX-License-Identifier:  GPL-2.0-only

use tracing::info;
use tracing_subscriber::EnvFilter;

mod app;
mod config;
mod i18n;

fn main() -> cosmic::iced::Result {
    init_logging();

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(360.0)
            .min_height(180.0),
    );

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("vibe=info")))
        .without_time()
        .pretty()
        .init();

    info!("Logger initialised");
}
