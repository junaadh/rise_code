use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    process::{Command, Stdio},
};

use crate::interface::code::Code;

use super::PORT;

pub fn run() -> Code {
    let listener = attach();
    log::info!("Listening for requests on: {PORT}");
    let stream = recieve_stream(listener);
    parse_result(stream.unwrap()).unwrap()
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

fn recieve_stream(listener: UnixListener) -> Option<UnixStream> {
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

fn parse_result(mut stream: UnixStream) -> Result<Code, ()> {
    // get steram and parse the data in the stream to a code struct
    // code struct because the channel can send messages of code struct
    // create buffer for reading stream to string
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .map_err(|err| log::error!("failed to read stream: {err}"))?;

    // split the stream string by : delimeter into an iter()
    let mut parts = buf.trim().split(':');

    // TODO: try to check if can match with serde
    let session_name = parts.next().unwrap_or_else(|| {
        log::warn!("session name returned none");
        ""
    });
    let language = parts.next().unwrap_or_else(|| {
        log::warn!("language returned none");
        ""
    });
    let file_name = parts.next().unwrap_or_else(|| {
        log::warn!("file name returned none");
        ""
    });
    let repo_name = parts
        .next()
        .unwrap_or_else(|| {
            log::warn!("repo name returned none");
            "https://"
        })
        .replace(';', ":");
    Ok(Code::new(
        session_name,
        language,
        file_name,
        repo_name.as_str(),
        true,
        false,
    ))
}
