use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SOCKET_FILE_NAME: &str = "daemon.sock";

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    /// Redraw every output again
    Redraw,

    /// Reload the config file of each output and of the daemon.
    Reload,

    Close,
    Exit,
}

/// Returns the path to the socket
pub fn path() -> PathBuf {
    super::get_xdg()
        .place_runtime_file(SOCKET_FILE_NAME)
        .unwrap()
}
