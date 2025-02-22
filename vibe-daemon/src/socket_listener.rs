use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixListener,
    sync::mpsc::Sender,
};

use tracing::{debug, error};
use vibe_daemon::Message;

use crate::Action;

pub struct SocketListener {
    listener: UnixListener,
    tx: Sender<Action>,
}

impl SocketListener {
    pub fn new(tx: Sender<Action>) -> Result<Self, std::io::Error> {
        let listener = {
            let socket_path = vibe_daemon::get_socket_path();
            debug!("Daemon socket path: {}", socket_path.to_string_lossy());

            UnixListener::bind(socket_path.clone()).map_err(|err| {
                std::io::Error::new(
                    err.kind(),
                    format!("Socket {}\n{}\nYou can remove the file if you are sure that the vibe daemon isn't already running.", socket_path.to_string_lossy(), err),
                )
            })?
        };

        Ok(Self { listener, tx })
    }

    pub fn run(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let buf_reader = BufReader::new(stream);

                    for line in buf_reader.lines() {
                        let line = match line {
                            Ok(line) => line,
                            Err(err) => {
                                error!("{}", err);
                                continue;
                            }
                        };

                        let msg: Message = match ron::from_str(&line) {
                            Ok(msg) => msg,
                            Err(err) => {
                                error!("{}", err);
                                continue;
                            }
                        };

                        debug!("Received message: {:?}", msg);

                        match msg {
                            Message::Exit => {
                                self.tx.send(Action::Exit).unwrap();
                                return;
                            }
                            Message::Close => return,
                        }
                    }
                }
                Err(err) => tracing::error!("{}", err),
            }
        }
    }
}

impl Drop for SocketListener {
    fn drop(&mut self) {
        let socket_path = vibe_daemon::get_socket_path();
        std::fs::remove_file(socket_path).expect("Remove daemon socket");
    }
}
