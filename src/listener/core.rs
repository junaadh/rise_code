use std::{
    os::unix::net::{UnixListener, UnixStream},
    process::{Command, Stdio},
};

use super::PORT;

pub fn attach() -> UnixListener {
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

// pub fn recieve_stream(listener: &UnixListener) -> Option<UnixStream> {
//     // get a stream from the port
//     for stream in listener.incoming() {
//         match stream {
//             // return stream
//             Ok(next) => {
//                 return Some(next);
//             }
//             // log error and continue loop
//             Err(err) => {
//                 log::warn!("revieving stream error: {err}");
//                 continue;
//             }
//         }
//     }
//     // here to handle edge cases and make compiler happy
//     None
// }
