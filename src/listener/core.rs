use std::process::{Command, Stdio};

use tokio::net::UnixListener;

use super::PORT;

pub async fn attach() -> UnixListener {
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
