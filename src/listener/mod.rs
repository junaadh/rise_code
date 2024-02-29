mod attach;
mod core;
mod detach;
use tokio::sync::mpsc::Sender;

use crate::{events::REvents, interface::code::Code, loader::traits::TimeKeeper};

const PORT: &str = "/tmp/dev_rpc";

pub async fn start(tx: Sender<Code>) {
    let listener = core::attach().await;
    REvents::SocketEstablished.flush(Some(PORT.to_string()));
    let mut session = Code::default();

    // Main loop to listen for incoming requests
    loop {
        if session.attach_status {
            REvents::ListeningDetach.flush(None);
        } else {
            REvents::ListeningAttach.flush(None);
        }
        // Wait for either an incoming connection or a message on the channel
        tokio::select! {
        // Accept incoming connection
            Ok((mut stream, _)) = listener.accept() => {
                REvents::UnixStreamRecieved.flush(None);
                // Handle the request based on session state
                if !session.attach_status {
                    REvents::AttachReceived.flush(None);
                    if let Ok(code) = attach::parse_result(&mut stream).await {
                        REvents::AttachAcknowledged.flush(None);
                        session.update(&code);
                        let _ = tx.send(session.clone()).await;
                    } else {
                        REvents::AttachDropped.eflush(Some("Invalid payload".to_string()));
                    }
                } else {
                    REvents::DetachRecieved.flush(None);
                    if let Ok(str) = detach::parse_result(&mut stream).await {
                        if str == session.tmux_session {
                            REvents::DetachAcknowledged.flush(None);
                            session.disconnect();
                            log::info!("{}",session.elapsed());
                            let _ = tx.send(session.clone()).await;
                        } else {
                            REvents::DetachDropped.eflush(Some("Invalid payload".to_string()))
                        }
                    }
                }
            }
        }
        // Wait for a message on the channel
    }
}
