use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
};

// use super::recieve_stream;

// pub fn run(listener: &UnixListener) -> String {
//     log::info!("Waiting for detach request:");
//     let stream = recieve_stream(listener);
//     parse_result(stream.unwrap()).unwrap()
// }

pub fn parse_result(stream: &mut UnixStream) -> Result<String, ()> {
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .map_err(|err| log::error!("Failed to read stream: {err}"))?;
    Ok(buf.trim().to_string())
}
