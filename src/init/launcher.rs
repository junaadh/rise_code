use std::io::Write;

use crate::traits::{DirCreator, DirPerms, UnwrapAndLogRes};

fn startup() -> String {
    let name = "rise_code";
    let script_data = r#"#!/bin/sh
    
    program={name}
    
    if ! pgrep "$program" >/dev/null; then
        "$program" > /tmp/{name}.stdout 2> /tmp/{name}.stderr & disown $!
    fi"#;
    script_data
        .replace("{name}", name)
        .replace("{name}", name)
        .replace("{name}", name)
}

pub async fn create(path: &str) -> Result<(), ()> {
    let mut launcher = path.create();
    launcher.write_all(startup().as_bytes()).unwrap_log();
    launcher.execute();
    Ok(())
}
