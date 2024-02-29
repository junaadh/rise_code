use std::{
    fs::{self, OpenOptions},
    io::{Seek, Write},
    path::{Path, PathBuf},
};

use crate::envvar;

pub async fn walk(path: String) {
    let mut dirs = Vec::<&str>::new();
    let xdg_config_dirs = envvar!("XDG_CONFIG_DIRS");
    if !xdg_config_dirs.is_empty() {
        dirs.push(xdg_config_dirs.as_str());
    }
    let config = format!("{}/.config", path.as_str());
    dirs.push(config.as_str());
    dirs.push(&path);

    let tmux_configs = [".tmux.conf", "tmux/tmux.conf"];
    let tmux_config = find_file(&tmux_configs, &dirs).unwrap();
    let hooks = [
        r#"set-hook -g client-attached 'run-shell "~/.cargo/rise_code/responder.sh attach #{session_name} #{pane_current_path}"'"#,
        r#"set-hook -g client-detached 'run-shell "~/.cargo/rise_code/responder.sh detach #{session_name}"'"#,
    ];
    append_file(&hooks, &tmux_config).unwrap();

    let shell = envvar!("SHELL");
    let shell = shell.split('/').last().unwrap_or_default();
    println!("{shell}");
    let shell_configs = match shell {
        "bash" => [".bashrc", "bash/bashrc"],
        "zsh" => [".zshrc", "zsh/zshrc"],
        "fish" => ["fish/config.fish", ""],
        _ => ["", ""],
    };
    let shell_config = find_file(&shell_configs, &dirs).unwrap();
    let command = ["~/.cargo/rise_code/launcher.sh"];
    append_file(&command, &shell_config).unwrap();
}

fn find_file(config_names: &[&str], paths: &Vec<&str>) -> Result<PathBuf, ()> {
    for path in paths {
        let dir = Path::new(path);
        for config in config_names {
            let file = dir.join(config);
            if file.exists() {
                println!("{:?}", file);
                return Ok(file);
            }
        }
    }
    Err(())
}

fn append_file(lines: &[&str], file: &PathBuf) -> Result<(), ()> {
    let existing = fs::read_to_string(file)
        .map_err(|err| println!("{err}"))
        .unwrap();

    let lines_missing = lines
        .iter()
        .filter(|row| !existing.contains(*row))
        .copied()
        .collect::<Vec<&str>>();

    if lines_missing.is_empty() {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(file)
        .map_err(|er| println!("{er}"))
        .unwrap();

    file.seek(std::io::SeekFrom::End(0))
        .map_err(|err| println!("{err}"))?;
    let buf = lines_missing.join("\n");

    file.write_all(buf.as_bytes())
        .map_err(|err| println!("{err}"))?;
    Ok(())
}
