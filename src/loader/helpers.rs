use std::{env, process::Command};

use regex::Regex;

use crate::commands;

pub fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

fn get_home() -> String {
    env::var("HOME").unwrap_or_default()
}

pub fn setup_log(name: &str) {
    let name = format!("{}/{name}", get_home());
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] @ {} -{:?}:{:?}- {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                message
            ))
        })
        .chain(
            fern::log_file(&name)
                .map_err(|err| println!("Failed to open logfile {}: {}", name, err))
                .unwrap(),
        )
        .level(log::LevelFilter::Debug)
        .apply()
        .map_err(|err| println!("Failed to initialize logger: {}", err))
        .unwrap();
    log::info!("Start logging @ {name}", name = &name);
}

pub fn get_open(name: &str) -> bool {
    Command::new("pgrep")
        .arg(name)
        .output()
        .unwrap()
        .status
        .success()
}

pub fn get_filenames(text: String) -> String {
    let ins_pos = commands::grep::grep(&text, "INS");
    let nor_pos = commands::grep::grep(&text, "NOR");
    let active = if ins_pos == -1 { nor_pos } else { ins_pos };

    let pattern = r#"^*(?:[a-zA-Z0-9_-]+\/)*[a-zA-Z0-9_-]+(?:\.[a-zA-Z]+)?*$"#;
    let re = Regex::new(pattern)
        .map_err(|err| println!("Cannot log: {err}"))
        .unwrap();
    let mut file = commands::awk::awk(&text, active + 1);
    // check active + 2
    if !re.is_match(&file) {
        file = commands::awk::awk(&text, active + 2);
    }

    // make sure active + 2 is filename if not return empty string
    if !re.is_match(&file) {
        file = "".to_string()
    }
    file
}
