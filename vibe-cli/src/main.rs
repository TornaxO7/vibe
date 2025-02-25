use std::{
    io::{self, Write},
    os::unix::net::UnixStream,
};

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// Redraw every output again.
    Redraw,

    /// Reload the config file of each output and of the daemon.
    Reload,

    /// Stop the daemon.
    Exit,
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let mut stream =
        UnixStream::connect(vibe_daemon::socket::path()).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => io::Error::new(
                err.kind(),
                format!(
                    "The socket file at '{}' of the daemon does not exist. Please check if the daemon is running.",
                    vibe_daemon::socket::path().to_string_lossy()
                ),
            ),
            _other => err,
        })?;
    let args = Args::parse();

    let command = match args.command {
        Command::Redraw => vibe_daemon::socket::Command::Redraw,
        Command::Reload => vibe_daemon::socket::Command::Reload,
        Command::Exit => vibe_daemon::socket::Command::Exit,
    };

    stream
        .write(toml::to_string(&command).unwrap().as_bytes())
        .expect("Send command");

    let should_close_connection = args.command != Command::Exit;
    if should_close_connection {
        let close_command = toml::to_string(&vibe_daemon::socket::Command::Close).unwrap();
        stream.write(close_command.as_bytes())?;
    }

    Ok(())
}
