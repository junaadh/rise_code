use std::{env, process::Command};

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
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
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
}

pub fn get_open(name: &str) -> bool {
    Command::new("pgrep")
        .arg(name)
        .output()
        .unwrap()
        .status
        .success()
}
