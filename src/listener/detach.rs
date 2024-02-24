// use super::recieve_stream;

// pub fn run(listener: &UnixListener) -> String {
//     log::info!("Waiting for detach request:");
//     let stream = recieve_stream(listener);
//     parse_result(stream.unwrap()).unwrap()
// }

use tokio::{io::AsyncReadExt, net::UnixStream};

// use std::{io::Read, os::unix::net::UnixStream};

pub async fn parse_result(stream: &mut UnixStream) -> Result<String, ()> {
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .await
        .map_err(|err| log::error!("Failed to read stream: {err}"))?;
    Ok(buf.trim().to_string())
}
