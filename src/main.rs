pub mod events;
pub mod interface;
pub mod listener;
pub mod loader;

use discord_rich_presence::{
    activity::{Activity, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use events::REvents;
use loader::traits::RiseFormat;
use tokio::sync::mpsc::{self, error::TryRecvError, Receiver};

async fn load_client(
    code: &interface::code::Code,
    client: &mut DiscordIpcClient,
) -> Result<(), ()> {
    let big_text = format!("Programming Language: {}", code.language.name.to_string());
    let small_text = format!("Helix Editor opened in Tmux: {}", code.tmux_session);
    let code_str = if code.file.is_empty() {
        format!("Coding in {}", code.language.name.to_string())
    } else {
        format!("Editing: {}", code.file)
    };
    let tmux = format!("#Tmux: {}", &code.tmux_session);
    let img = &code.language.name.get_logo();
    let assets = Assets::new()
        .large_image(img)
        .large_text(&big_text)
        .small_image("helix-logo-nice")
        .small_text(&small_text);
    let stamps = Timestamps::new().start(code.duration);
    let mut activity = Activity::new()
        .state(tmux.truncate(128))
        .details(code_str.truncate(128));
    let buttons = vec![Button::new("View Git Repo", &code.github)];
    if !code.github.trim().ends_with(".com/") {
        activity = activity.buttons(buttons);
    }
    activity = activity.assets(assets).timestamps(stamps);
    client.set_activity(activity).map_err(|err| {
        log::error!("Failed to load activity: trying again {err}");
    })?;
    Ok(())
}

async fn fetch_info(code: &mut interface::code::Code) -> Result<(), ()> {
    sleep!(5);
    let info = loader::parser::get_window_id(&code.tmux_session).unwrap_or_default();
    let mut language = code.language.clone();
    let pane_content = if !info.is_empty() {
        loader::parser::get_pane_content(info, code)
    } else {
        "".to_string()
    };
    let mut active_file = String::new();

    if !pane_content.is_empty() {
        active_file = loader::parser::parse_pane(pane_content);
        let (_body, ext) = active_file.rsplit_once('.').unwrap_or_default();
        if !ext.is_empty() {
            language.push_ext(ext);
        } else {
            language.get_max_ext();
        }
    }
    code.language(language);
    code.file(&active_file);
    Ok(())
}

async fn run(mut rx: Receiver<interface::code::Code>) {
    let mut code = interface::code::Code::default();
    let client_id = envvar!("clientid");
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
        let disc_status = open!("Discord");
        REvents::CheckDiscordStatus.flush(None);
        if !disc_status {
            REvents::DiscordClosed.eflush(None);
            sleep!(4);
            continue 'run;
        }
        REvents::DiscordOpen.flush(None);
        REvents::ConnectingIpcClient.flush(None);
        match loader::client::get_client(&client_id).await {
            Ok(disc) => {
                REvents::IpcConnectionSuccess.flush(None);
                client = disc;
                REvents::SessionStatus.flush(Some(format!("{} entered", code.tmux_session)));
                REvents::SessionLanguage.flush(Some(code.language.get_max().to_string()));
                'update: loop {
                    sleep_ms!(300);
                    let code_res = fetch_info(&mut code).await;
                    if code_res.is_ok() {
                        if !open!("Discord") || load_client(&code, &mut client).await.is_err() {
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

#[cfg(unix)]
#[tokio::main]
async fn main() {
    loader::helpers::setup_log(".cache/rise_code.log");
    let (tx, rx) = mpsc::channel::<interface::code::Code>(1);

    tokio::spawn(async move {
        listener::start(tx).await;
    });

    run(rx).await;
}
