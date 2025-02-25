use std::{path::PathBuf, sync::OnceLock};

use xdg::BaseDirectories;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

const OUTPUT_CONFIG_DIR_NAME: &str = "output_configs";
const CONFIG_FILE_NAME: &str = "config.toml";

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

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
