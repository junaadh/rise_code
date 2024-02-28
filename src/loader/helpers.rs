use std::fs;

use super::traits::RegexMatcher;

use crate::{awk, envvar, grep};

pub fn setup_log(name: &str) {
    let name = format!("{}/{name}", envvar!());

    if let Ok(file) = fs::metadata(&name) {
        println!("Checking {name} integrity...");
        if file.len() > 20 * 1024 * 1024 {
            eprintln!(
                "log file exceeds 20mb... Removing file {name}...",
                name = &name
            );
            fs::remove_file(&name)
                .map_err(|err| eprintln!("[failed]\n{err}"))
                .unwrap();
        }
        println!("[OK]");
    }

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

pub fn get_filenames(text: String) -> String {
    let nor_pos = grep!(&text, "NOR");
    let ins_pos = grep!(&text, "INS");
    let active = if ins_pos == -1 { nor_pos } else { ins_pos };

    let pattern = r#"^*(?:[a-zA-Z0-9_-]+\/)*[a-zA-Z0-9_-]+(?:\.[a-zA-Z]+)?*$"#;
    let mut file = awk!(&text, active + 1);
    // check active + 2
    if !file.regex(pattern) {
        file = awk!(&text, active + 2);
    }

    // make sure active + 2 is filename if not return empty string
    if !file.regex(pattern) {
        file = "".to_string()
    }
    file
}
