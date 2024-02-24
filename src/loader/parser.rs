use std::process::Command;

fn get_window_id(session: &str) -> Result<String, ()> {
    let window_info = Command::new("tmux")
        .args([
            "list-windows",
            "-t",
            session,
            "-F",
            "#{window_index} #{windo_name}",
        ])
        .output()
        .map_err(|err| {
            log::debug!("cannot get window_name and index of window with hx running: {err}")
        })?;
    let mut window_id = String::new();
    if window_info.status.success() {
        window_id = core::str::from_utf8(&window_info.stdout)
            .map_err(|err| log::error!("invalid utf8 format: {err}"))?
            .split('\n')
            .find_map(|line| {
                if line.contains("hx") {
                    line.split_whitespace().next().map(|str| str.to_owned())
                } else {
                    None
                }
            })
            .unwrap_or("".to_string());
    }
    Ok(window_id)
}
