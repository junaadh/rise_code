use discord_rich_presence::DiscordIpc;
use tokio::sync::mpsc::{error::TryRecvError, Receiver};

use crate::{events::REvents, open, sleep, sleep_ms};

pub async fn run(mut rx: Receiver<crate::interface::code::Code>) {
    let mut code = crate::interface::code::Code::default();
    let client_id = "1208484529510154260";
    if !client_id.is_empty() {
        REvents::FetchClientId.flush(None);
    }
    'run: loop {
        match rx.try_recv() {
            Ok(rcv) => code = rcv,
            Err(err) => match err {
                TryRecvError::Empty => {
                    if !code.attach_status {
                        sleep!(3);
                        continue;
                    }
                }
                _ => {
                    sleep!(4);
                    continue;
                }
            },
        }
        let mut client;
        REvents::CheckDiscordStatus.flush(None);
        if !open!("Discord") {
            REvents::DiscordClosed.eflush(None);
            sleep!(4);
            continue 'run;
        }
        REvents::DiscordOpen.flush(None);
        REvents::ConnectingIpcClient.flush(None);
        match super::connect::get_client(client_id).await {
            Ok(disc) => {
                REvents::IpcConnectionSuccess.flush(None);
                client = disc;
                REvents::SessionStatus.flush(Some(format!("{} entered", code.tmux_session)));
                REvents::SessionLanguage.flush(Some(code.language.get_max().to_string()));
                'update: loop {
                    sleep_ms!(300);
                    let code_res = super::fetch::fetch_info(&mut code).await;
                    if code_res.is_ok() {
                        if !open!("Discord")
                            || super::load::load_client(&code, &mut client).await.is_err()
                        {
                            REvents::DiscordUnexpextedExit.flush(None);
                            continue 'run;
                        };
                        match rx.try_recv() {
                            Ok(cli) => {
                                code = cli;
                            }
                            Err(_) => continue 'update,
                        }
                    }
                    if code.detach_status {
                        let _ = client.clear_activity();
                        REvents::IpcClearActivity.flush(None);
                        break 'update;
                    }
                }
            }
            Err(err) => {
                REvents::IpcConnectionError.eflush(Some(err));
                sleep!(5);
                continue 'run;
            }
        }
        let _ = client.close();
        REvents::IpcCloseClient.flush(None);
        REvents::SessionStatus.flush(Some(format!("{} exited", code.tmux_session)));
    }
}
