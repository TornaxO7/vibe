use std::{
    collections::HashSet,
    io::{self, BufRead, BufReader},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use tracing::{debug, warn};
use vibe_daemon::socket::Action;

use crate::state::State;

#[derive(Debug)]
pub struct Daemon {
    _listener: JoinHandle<()>,

    /// all actions which were coming from the clients
    pending_action: Arc<Mutex<HashSet<Action>>>,
}

impl Daemon {
    pub fn new() -> io::Result<Self> {
        let pending_action = Arc::new(Mutex::new(HashSet::new()));

        let listener = {
            let socket_path = vibe_daemon::socket::path();
            if socket_path.exists() {
                std::fs::remove_file(&socket_path)?;
            }

            let action_buffer = pending_action.clone();
            let listener = UnixListener::bind(&socket_path)?;
            debug!("Start listener at {}", socket_path.to_str().unwrap());

            thread::spawn(move || {
                for stream in listener.incoming() {
                    let action_buffer = action_buffer.clone();

                    match stream {
                        Ok(stream) => {
                            thread::spawn(move || handle_client(stream, action_buffer));
                        }
                        Err(err) => {
                            warn!("Couldn't connect to client: {}", err);
                        }
                    };
                }
            })
        };

        Ok(Self {
            _listener: listener,
            pending_action,
        })
    }

    pub fn apply_action(&self, state: &mut State) {
        let actions: Vec<Action> = { self.pending_action.lock().unwrap().drain().collect() };

        for action in actions {
            match action {
                Action::Redraw => state.action_redraw(),
                Action::Reload => state.action_reload(),
                Action::Close => {} // already managed by `handle_client`
                Action::Exit => state.action_exit(),
            };
        }
    }
}

fn handle_client(stream: UnixStream, action_buffer: Arc<Mutex<HashSet<Action>>>) {
    let buf_reader = BufReader::new(stream);

    for line in buf_reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                warn!("Couldn't read message from client: {}", err);
                warn!("Closing connection.");
                return;
            }
        };

        let action: Action = match toml::from_str(&line) {
            Ok(action) => action,
            Err(err) => {
                warn!("Couldn't parse action from client: {}", err);
                continue;
            }
        };

        debug!("Retrieved action: {:?}", action);
        {
            action_buffer.lock().unwrap().insert(action);
        };

        if action == Action::Close {
            return;
        }
    }
}
