use core::str;
use std::process::Command;

use crate::interface;

pub fn run(session_name: &str) -> interface::code::Code {
    interface::code::Code::detach_new(check_session_state(session_name))
}

// return true if session attached and vice versa
fn check_session_state(session_name: &str) -> bool {
    let tmux_ls = Command::new("tmux").arg("ls").output();

    match tmux_ls {
        Ok(ls_buffer) => match ls_buffer.status.success() {
            true => {
                let ls_str = str::from_utf8(&ls_buffer.stdout)
                    .map_err(|err| log::error!("Invalid utf8 format: {err}"))
                    .unwrap();
                !ls_str
                    .split('\n')
                    .any(|line| line.contains(session_name) && line.contains("(attached)"))
            }
            false => true,
        },
        Err(err) => {
            log::error!("failed to get tmux ls state: {err}");
            true
        }
    }
}
