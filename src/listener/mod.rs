mod attach;
mod detach;

use std::{
    os::unix::net::{UnixListener, UnixStream},
    process::{Command, Stdio},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

use crate::interface::code::Code;

const PORT: &str = "/tmp/dev_rpc";

pub fn start(tx: Sender<Code>) {
    let listener = attach();

    'main: loop {
        let code = attach::run(&listener);
        let session_name = code.tmux_session.clone();
        match code.check_integ() && code.attach_status {
            true => {
                let _ = tx.send(code);
            }
            false => continue 'main,
        }

        thread::sleep(Duration::from_secs(5));

        let mut bool = true;
        while bool {
            if session_name == detach::run(&listener) {
                bool = false;
            }
        }
    }
}

fn attach() -> UnixListener {
    // removes existing /tmp/dev_rpc i exists
    // if not returns error
    // result is ignored no matter
    let _ = Command::new("rm")
        .arg(PORT)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // bidn port and return the listener on the port
    UnixListener::bind(PORT)
        .map_err(|err| log::error!("failed to bind to socket {PORT}: {err}"))
        .unwrap()
}

fn recieve_stream(listener: &UnixListener) -> Option<UnixStream> {
    // get a stream from the port
    for stream in listener.incoming() {
        match stream {
            // return stream
            Ok(next) => {
                return Some(next);
            }
            // log error and continue loop
            Err(err) => {
                log::warn!("revieving stream error: {err}");
                continue;
            }
        }
    }
    // here to handle edge cases and make compiler happy
    None
}
