use std::io::Write;

use crate::traits::{DirCreator, DirPerms, UnwrapAndLogRes};

#[cfg(target_os = "macos")]
fn startup() -> String {
    r#"#!/bin/sh
        
    type=$1
    session_name=$2
        
    if [[ $1 == "attach" ]]; then
        path=$3
        language=$(echo $path | cut -d'/' -f5)
        repo=$(git --git-dir=$path/.git config --get remote.origin.url | cut -d':' -f2)

        hx_window=$(tmux list-windows -t $session_name -F '#{window_index} #{window_name}' | grep hx | cut -d' ' -f1)
        echo $session_name:$language:$file_name:$repo | nc -U /tmp/dev_rpc
        exit 0
    else
        echo $session_name | nc -U /tmp/dev_rpc
        exit 0
    fi"#
    .to_string()
    //TODO: if git is not a ssh authenticated
}

#[cfg(not(target_os = "macos"))]
fn startup() -> String {
    r#"#!/bin/sh
        
    type=$1
    session_name=$2
        
    if [[ $1 == "attach" ]]; then
        path=$3
        language=$(echo $path | cut -d'/' -f5)
        repo=$(git --git-dir=$path/.git config --get remote.origin.url | cut -d':' -f2)

        hx_window=$(tmux list-windows -t $session_name -F '#{window_index} #{window_name}' | grep hx | cut -d' ' -f1)
        echo $session_name:$language:$file_name:$repo | socat - UNIX-CONNECT:/tmp/dev_rpc
        exit 0
    else
        echo $session_name | socat - UNIX-CONNECT:/tmp/dev_rpc
        exit 0
    fi"#
    .to_string()
    //TODO: if git is not a ssh authenticated
}

pub async fn create(path: &str) -> Result<(), ()> {
    let mut responder = path.create();
    responder.write_all(startup().as_bytes()).unwrap_log();
    responder.execute();
    Ok(())
}
