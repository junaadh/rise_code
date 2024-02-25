mod attach;
mod core;
mod detach;
use tokio::sync::mpsc::Sender;

use crate::interface::code::Code;

const PORT: &str = "/tmp/dev_rpc";

// pub async fn start(tx: Sender<Code>) {
//     let listener = core::attach().await;
//     let mut session = Code::default();

//     let listener = listener.into_std().unwrap();
//     // listener.set_nonblocking(false).unwrap();

//     for stream in listener.incoming() {
//         if stream.is_ok() {
//             let mut stream = stream.unwrap();
//             if !session.attach_status {
//                 log::info!("listening for attach requests on {PORT}");
//                 let res = attach::parse_result(&mut stream).await;
//                 match res {
//                     Ok(code) => {
//                         session.update(&code);
//                         let _ = tx.send(session.clone()).await;
//                     }
//                     Err(_) => {
//                         log::warn!("Invalid attach request");
//                     }
//                 }
//             } else {
//                 log::info!("listening for detach requests on {PORT}");
//                 let res = detach::parse_result(&mut stream).await;
//                 if let Ok(str) = res {
//                     if str == session.tmux_session {
//                         session.disconnect();
//                         let _ = tx.send(session.clone()).await;
//                     }
//                 }
//             }
//         }
//     }
// }

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

// loop {
//     match listener.accept().await {
//         Ok((mut msg, _)) => {
//             msg.readable().await.unwrap();
//             if !session.attach_status {
//                 log::info!("listening for attach requests on {PORT}");
//                 let res = attach::parse_result(&mut msg).await;
//                 match res {
//                     Ok(code) => {
//                         session.update(&code);
//                         let _ = tx.send(session.clone()).await;
//                     }
//                     Err(_) => {
//                         log::warn!("Invalid attach request");
//                     }
//                 }
//             } else {
//                 log::info!("listening for detach requests on {PORT}");
//                 let res = detach::parse_result(&mut msg).await;
//                 if let Ok(str) = res {
//                     if str == session.tmux_session {
//                         session.disconnect();
//                         let _ = tx.send(session.clone()).await;
//                     }
//                 }
//             }
//         }
//         Err(err) => {
//             log::warn!("error at socket {PORT}: {err}");
//             time::sleep(Duration::from_secs(2)).await;
//             continue;
//         }
//     }
//     time::sleep(Duration::from_secs(2)).await;
// }

// pub struct AsyncPipeListener(UnixListener);

// impl AsyncPipeListener {
//     pub async fn accept(&mut self) -> UnixStream {
//         self.0
//             .accept()
//             .await
//             .map_err(|err| warn!("Unable to get listener: {err:#?}"))
//             .map(|(s, _)| s)
//             .unwrap()
//     }
// }

// async fn handle(session: &mut Code, mut msg: tokio::net::UnixStream, tx: Sender<Code>) {
//     if !session.attach_status {
//         log::info!("listening for attach requests on {PORT}");
//         let res = attach::parse_result(&mut msg).await;
//         match res {
//             Ok(code) => {
//                 session.update(&code);
//                 let _ = tx.send(session.clone()).await;
//             }
//             Err(_) => {
//                 log::warn!("Invalid attach request");
//             }
//         }
//     } else {
//         log::info!("listening for detach requests on {PORT}");
//         let res = detach::parse_result(&mut msg).await;
//         if let Ok(str) = res {
//             if str == session.tmux_session {
//                 session.disconnect();
//                 let _ = tx.send(session.clone()).await;
//             }
//         }
//     }
// }
