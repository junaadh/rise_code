pub mod interface;
pub mod listener;
pub mod loader;

use std::{env, time::Duration};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use tokio::{
    sync::mpsc::{self, error::TryRecvError, Receiver},
    time,
};

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
    let mut activity = Activity::new()
        .state(loader::helpers::truncate(&tmux, 128))
        .details(loader::helpers::truncate(&code_str, 128));
    let buttons = vec![Button::new("View Git Repo", &code.github)];
    if !code.github.trim().ends_with(".com/") {
        activity = activity.buttons(buttons);
    }
    activity = activity.assets(assets);
    client.set_activity(activity).map_err(|err| {
        log::error!("Failed to load activity: trying again {err}");
    })?;
    Ok(())
}

async fn fetch_info(code: &mut interface::code::Code) -> Result<(), ()> {
    time::sleep(Duration::from_secs(3)).await;
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
    let client_id = env::var("clientid")
        .map_err(|err| log::error!("Unable to fetch client_id: {err}"))
        .unwrap_or_default();
    if !client_id.is_empty() {
        log::info!("Successfully fetched client_id");
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
        let sess = code.tmux_session.clone();
        let mut client;
        let disc_status = loader::helpers::get_open("Discord");
        log::info!("Checking if Discord is open...");
        if !disc_status {
            log::warn!("Discord is closed... Waiting for connection...");
            sleep!(4);
            continue 'run;
        }
        log::info!("Discord open. Proceeding..,");
        log::info!("Connecting to Discord IPC client...");
        match loader::client::get_client(&client_id).await {
            Ok(disc) => {
                log::info!("Succesfully connected to client");
                client = disc;
                log::info!("Session {} connected", &code.tmux_session,);
                log::info!("Coding in {}", &code.language.name.to_string());
                'update: loop {
                    sleep_ms!(300);
                    let code_res = fetch_info(&mut code).await;
                    if code_res.is_ok() {
                        if !loader::helpers::get_open("Discord")
                            || load_client(&code, &mut client).await.is_err()
                        {
                            log::warn!("Discord exited or ipc client exited unexpectedly... Trying again...");
                            continue 'run;
                        };
                        match rx.try_recv() {
                            Ok(cli) => {
                                log::info!("Recieved another request...");
                                code = cli;
                            }
                            Err(_) => continue 'update,
                        }
                    }
                    if code.detach_status {
                        let _ = client.clear_activity();
                        break 'update;
                    }
                }
            }
            Err(err) => {
                log::error!("{err}");
                sleep!(5);
                continue 'run;
            }
        }
        let _ = client.close();
        log::info!("Session {} disconnected", sess);
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
