mod attach;
mod core;
mod detach;
use tokio::sync::mpsc::Sender;

use crate::interface::code::Code;

const PORT: &str = "/tmp/dev_rpc";

pub async fn start(tx: Sender<Code>) {
    let listener = core::attach().await;
    let mut session = Code::default();

    // Main loop to listen for incoming requests
    loop {
        // Wait for either an incoming connection or a message on the channel
        tokio::select! {
        // Accept incoming connection
            Ok((mut stream, _)) = listener.accept() => {
                log::info!("Connection accepted");
                // Handle the request based on session state
                if !session.attach_status {
                    log::info!("Listening for attach requests on {}", PORT);
                    if let Ok(code) = attach::parse_result(&mut stream).await {
                        session.update(&code);
                        let _ = tx.send(session.clone()).await;
                    } else {
                        log::warn!("Invalid attach request");
                    }
                } else {
                    log::info!("Listening for detach requests on {}", PORT);
                    if let Ok(str) = detach::parse_result(&mut stream).await {
                        if str == session.tmux_session {
                            session.disconnect();
                            let _ = tx.send(session.clone()).await;
                        }
                    }
                }
            }
        }
        // Wait for a message on the channel
    }
}
