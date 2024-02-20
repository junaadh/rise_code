mod attach;
mod core;
mod detach;

use std::sync::mpsc::Sender;

use crate::interface::code::Code;

use self::core::attach;

const PORT: &str = "/tmp/dev_rpc";

pub fn start(tx: Sender<Code>) {
    let listener = attach();
    let mut session: Code = Code::default();

    for stream in listener.incoming() {
        match stream {
            Ok(mut msg) => {
                if !session.attach_status {
                    log::info!("listening for attach requests on {PORT}");
                    let res = attach::parse_result(&mut msg);
                    match res {
                        Ok(code) => {
                            session = code.clone();
                            let _ = tx.send(code);
                        }
                        Err(_) => continue,
                    }
                } else {
                    log::info!("listening for detach requests on {PORT}");
                    let res = detach::parse_result(&mut msg);
                    if let Ok(str) = res {
                        if str == session.tmux_session {
                            session = session.disconnect();
                            let _ = tx.send(session.clone());
                        }
                    }
                }
            }
            Err(err) => {
                log::warn!("error at socket {PORT}: {err}");
                continue;
            }
        }
    }
}
