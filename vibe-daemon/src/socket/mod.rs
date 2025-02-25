use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SOCKET_FILE_NAME: &str = "daemon.sock";

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    /// Redraw every output again
    Redraw,

    /// Reload the config file of each output and of the daemon.
    Reload,

    /// Close the connection
    Close,

    /// Stop the daemon
    Exit,
}

/// Returns the path to the socket
pub fn path() -> PathBuf {
    super::get_xdg()
        .place_runtime_file(SOCKET_FILE_NAME)
        .unwrap()
}
