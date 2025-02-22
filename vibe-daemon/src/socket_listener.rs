use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use tracing::{debug, error};
use vibe_daemon::Message;

use crate::Action;

pub struct SocketListener {
    _listener: JoinHandle<()>,

    actions: Arc<Mutex<VecDeque<Action>>>,
}

impl SocketListener {
    pub fn new() -> Result<Self, std::io::Error> {
        let actions = Arc::new(Mutex::new(VecDeque::new()));

        let listener = {
            let socket_path = vibe_daemon::get_socket_path();
            debug!("Daemon socket path: {}", socket_path.to_string_lossy());

            let listener = UnixListener::bind(socket_path.clone()).map_err(|err| {
                std::io::Error::new(
                    err.kind(),
                    format!("Socket {}\n{}\nYou can remove the file if you are sure that the vibe daemon isn't already running.", socket_path.to_string_lossy(), err),
                )
            })?;

            let actions_clone = actions.clone();

            thread::spawn(move || {
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => handle_client(stream, actions_clone.clone()),
                        Err(err) => {
                            error!("{}", err);
                            break;
                        }
                    }
                }
            })
        };

        Ok(Self {
            _listener: listener,
            actions,
        })
    }

    pub fn get_next_action(&mut self) -> Option<Action> {
        self.actions.lock().unwrap().pop_front()
    }
}

impl Drop for SocketListener {
    fn drop(&mut self) {
        let socket_path = vibe_daemon::get_socket_path();
        std::fs::remove_file(socket_path).expect("Remove daemon socket");
    }
}

fn handle_client(stream: UnixStream, actions: Arc<Mutex<VecDeque<Action>>>) {
    let buf_reader = BufReader::new(stream);

    for line in buf_reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                error!("{}\nClosing connection", err);
                break;
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
                actions.lock().unwrap().push_back(Action::Exit);
                return;
            }
            Message::Close => return,
        };
    }
}
