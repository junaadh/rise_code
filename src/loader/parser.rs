use core::str;
use std::process::Command;

use crate::interface::code::Code;

#[allow(clippy::result_unit_err)]
pub fn get_window_id(session: &str) -> Result<String, ()> {
    let window_info = Command::new("tmux")
        .args([
            "list-windows",
            "-t",
            session,
            "-F",
            "#{window_index} #{window_name}",
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

pub fn get_pane_content(text: String, code: &Code) -> String {
    let tmux_pane = Command::new("tmux")
        .args([
            "capture-pane",
            "-pC",
            "-t",
            &format!("{}:{text}", &code.tmux_session),
        ])
        .output()
        .map_err(|err| {
            log::warn!(
                "error getting pane_capture on {}:{text}: {err}",
                &code.tmux_session
            )
        })
        .unwrap();

    str::from_utf8(&tmux_pane.stdout)
        .map_err(|err| log::error!("error getting file path: {}", err))
        .unwrap()
        .to_string()
}

pub fn parse_pane(text: String) -> String {
    let line = text.split('\n').collect::<Vec<&str>>();
    let middle = line.len() / 2 - 2;
    let mut line = line.into_iter();

    let middle_status = line.nth(middle).unwrap_or_default();
    let bottom_status = line.rev().skip(1).nth(1).unwrap_or_default();

    let mut combined_status = middle_status.to_owned();
    combined_status.push_str(bottom_status);
    super::helpers::get_filenames(combined_status)
}

// pub fn parse_vertical_content(pane_content: String) -> String {
//     pane_content
//         .split('\n')
//         .collect::<Vec<&str>>()
//         .into_iter()
//         .rev()
//         .skip(1)
//         .nth(1)
//         .unwrap_or("")
//         .to_string()
// }

// pub fn handle_vertical_split_name(line: String) -> String {
//     let vec = super::helpers::get_filenames(line.clone());

//     let pipe_pos = commands::grep::grep(line.as_str(), "│");
//     let nor_pos = commands::grep::grep(line.as_str(), "NOR");
//     let ins_pos = commands::grep::grep(line.as_str(), "INS");
//     let active_label = if nor_pos != -1 { nor_pos } else { ins_pos };

//     let vec_len = vec.len();

//     if vec_len == 1 || pipe_pos > active_label {
//         #[allow(clippy::needless_borrowed_reference)]
//         vec.first()
//             .map(|&(ref file, _)| file)
//             .unwrap_or(&"".to_string())
//             .to_owned()
//     } else {
//         #[allow(clippy::needless_borrowed_reference)]
//         vec.get(1)
//             .map(|&(ref file, _)| file)
//             .unwrap_or(&"".to_string())
//             .to_owned()
//     }
// }

// pub fn handle_horizontal_split_name(line: String) -> String {
//     let vec
// }
