pub mod config;

use std::{path::PathBuf, sync::OnceLock};

use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const SOCKET_NAME: &str = "daemon.sock";

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Tell the daemon process to exit
    Exit,
    /// Close the socket connection
    Close,
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME).unwrap())
}

pub fn get_socket_path() -> PathBuf {
    get_xdg()
        .place_runtime_file(SOCKET_NAME)
        .expect("Get path to daemon socket")
}

pub fn config_directory() -> PathBuf {
    get_xdg()
        .create_config_directory(config::DIR_NAME)
        .expect("Get output config directory")
}
