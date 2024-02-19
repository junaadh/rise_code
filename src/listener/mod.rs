mod attach;
mod detach;

use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::interface::code::Code;

const PORT: &str = "/tmp/dev_rpc";

pub fn start(tx: Sender<Code>) {
    'main: loop {
        let code = attach::run();
        let session_name = code.tmux_session.clone();
        match code.check_integ() && code.attach_status {
            true => {
                let _ = tx.send(code);
            }
            false => continue 'main,
        }

        thread::sleep(Duration::from_secs(5));

        'detach: loop {
            let code = detach::run(&session_name);
            match code.detach_status {
                true => {
                    let _ = tx.send(code);
                    continue 'main;
                }
                false => continue 'detach,
            }
        }
    }
}
