use tokio::{io::AsyncReadExt, net::UnixStream};

// use std::{io::Read, os::unix::net::UnixStream};

use crate::interface::{code::Code, languages::LanguageExt};

// pub fn run(listener: &UnixListener) -> Code {
//     log::info!("Listening to attach requests on: {PORT}");
//     let stream = recieve_stream(listener);
//     parse_result(stream.unwrap()).unwrap()
// }

pub async fn parse_result(stream: &mut UnixStream) -> Result<Code, ()> {
    // get steram and parse the data in the stream to a code struct
    // code struct because the channel can send messages of code struct
    // create buffer for reading stream to string
    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .await
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
    let mut def_lang = LanguageExt::default();
    def_lang.push(language);

    let file_name = parts.next().unwrap_or_else(|| {
        log::warn!("file name returned none");
        ""
    });
    let repo_name = parts
        .next()
        .map(|repo| format!("https://github.com/{repo}"))
        .unwrap_or_else(|| {
            log::warn!("repo name returned none");
            "https://".to_string()
        });
    Ok(Code::new(
        session_name,
        def_lang,
        file_name,
        repo_name.as_str(),
        true,
        false,
    ))
}
