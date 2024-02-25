pub mod commands;
pub mod interface;
pub mod listener;
pub mod loader;

use std::{
    env,
    process::Command,
    str,
    // sync::mpsc::{self, Receiver, TryRecvError},
    // thread,
    time::Duration,
};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};
use tokio::{
    sync::mpsc::{self, error::TryRecvError, Receiver},
    time,
};

fn truncate(text: &str, max_length: usize) -> &str {
    match text.char_indices().nth(max_length) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

async fn get_client() -> Result<DiscordIpcClient, String> {
    let mut client = DiscordIpcClient::new("1208484529510154260").expect("invalid client id");
    client
        .connect()
        .map_err(|_| "Failed at connecting to discord client".to_string())?;
    Ok(client)
}

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
        .state(truncate(&tmux, 128))
        .details(truncate(&code_str, 128));
    let buttons = vec![Button::new("View Git Repo", &code.github)];
    if !code.github.trim().ends_with(".com/") {
        activity = activity.buttons(buttons);
    }
    activity = activity.assets(assets);
    client.set_activity(activity).map_err(|_| {
        log::error!("Failed to load activity: trying again");
    })?;
    Ok(())
}

async fn fetch_info(code: &mut interface::code::Code) -> Result<(), ()> {
    // thread::sleep(Duration::from_secs(3));
    time::sleep(Duration::from_secs(3)).await;
    let info = loader::parser::get_window_id(&code.tmux_session).unwrap();
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
    'run: loop {
        match rx.try_recv() {
            Ok(rcv) => code = rcv,
            Err(err) => match err {
                TryRecvError::Empty => {
                    if !code.attach_status {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        continue;
                    }
                }
                _ => {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            },
        }
        let sess = code.tmux_session.clone();
        let mut client;
        let disc_status = get_open("Discord");
        log::info!("Check if discord is open: {disc_status}");
        if !disc_status {
            log::warn!("discord is closed... Waiting for connection...");
            // thread::sleep(Duration::from_secs(4));
            time::sleep(Duration::from_secs(4)).await;
            continue 'run;
        }
        match get_client().await {
            Ok(disc) => {
                log::info!("Succesfully connected to client");
                client = disc;
                log::info!("Session {} connected", &code.tmux_session,);
                log::info!("Coding in {}", &code.language.name.to_string());
                'update: loop {
                    let code_res = fetch_info(&mut code).await;
                    if code_res.is_ok() {
                        if !get_open("Discord") || load_client(&code, &mut client).await.is_err() {
                            continue 'run;
                        };
                        match rx.try_recv() {
                            Ok(cli) => code = cli,
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
                // thread::sleep(Duration::from_secs(5));
                time::sleep(Duration::from_secs(5)).await;
                continue 'run;
            }
        }
        let _ = client.close();
        log::info!("Session {} disconnected: {}", sess, &code.detach_status);
    }
}

fn get_open(name: &str) -> bool {
    let discord = Command::new("pgrep").arg(name).output().unwrap();
    discord.status.success()
}

fn get_home() -> String {
    env::var("HOME").unwrap_or_default()
}

fn init_log(name: &str) {
    let name = format!("{}/{}", get_home(), name);
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}]@{} : {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(
            fern::log_file(&name)
                .map_err(|err| println!("Failed to open logfile {}: {}", name, err))
                .unwrap(),
        )
        .level(log::LevelFilter::Debug)
        .apply()
        .map_err(|err| println!("Failed to initialize logger: {}", err))
        .unwrap();
}

#[tokio::main]
async fn main() {
    init_log(".cache/rise_code.log");
    log::info!("Start log");
    let (tx, rx) = mpsc::channel::<interface::code::Code>(1);

    tokio::spawn(async move {
        listener::start(tx).await;
    });

    run(rx).await;
}
