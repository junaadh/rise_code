pub mod interface;
pub mod listener;
pub mod loader;

use std::{
    env,
    process::{Command, Stdio},
    str,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
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
    let big_text = format!("Programming Language: {}", code.language);
    let small_text = format!("Helix Editor opened in Tmux: {}", code.tmux_session);
    let code_str = if code.file.is_empty() {
        format!("Coding in {}", code.language)
    } else {
        format!("Editing: {}", code.file)
    };
    let tmux = format!("#Tmux: {}", &code.tmux_session);
    let assets = Assets::new()
        .large_image(interface::languages::Language::get_logo(&code.language))
        .large_text(&big_text)
        .small_image("helix-logo-nice")
        .small_text(&small_text);
    let mut activity = Activity::new()
        .state(truncate(&tmux, 128))
        .details(truncate(&code_str, 128));
    let buttons = vec![Button::new("View Git Repo", &code.github)];
    if code.github.trim().contains("github") {
        activity = activity.buttons(buttons);
    }
    activity = activity.assets(assets);
    client.set_activity(activity).map_err(|_| {
        log::error!("Failed to load activity: trying again");
    })?;
    Ok(())
}

async fn fetch_info(code: &interface::code::Code) -> Result<interface::code::Code, ()> {
    thread::sleep(Duration::from_secs(3));
    let window_info = Command::new("tmux")
        .args([
            "list-windows",
            "-t",
            &code.tmux_session,
            "-F",
            "#{window_index} #{window_name}",
        ])
        .output()
        .map_err(|err| log::debug!("Cannot get window_index & window_name: {err}"))?;
    let mut info = String::new();
    if window_info.status.success() {
        info = str::from_utf8(&window_info.stdout)
            .expect("Invalid format")
            .split('\n')
            .find_map(|line| {
                if line.contains("hx") {
                    line.split_whitespace().next().map(|str| str.to_owned())
                } else {
                    None
                }
            })
            .unwrap_or("".to_string());
    }
    let mut pane_content = String::new();
    if !info.is_empty() {
        let pane_capture = Command::new("tmux")
            .args([
                "capture-pane",
                "-peC",
                "-t",
                &format!("{}:{}", &code.tmux_session, info),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| {
                log::warn!(
                    "error running pane-capture on {}:{info}: {err}",
                    &code.tmux_session
                )
            })?;
        let grep = Command::new("grep")
            .arg("sel")
            .stdin(Stdio::from(pane_capture.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| {
                log::warn!("error running grep on {}:{info}: {err}", &code.tmux_session)
            })?;
        let awk = Command::new("awk")
            .arg("{ print $2 }")
            .stdin(Stdio::from(grep.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| {
                log::warn!("error running awk on {}:{info}: {err}", &code.tmux_session)
            })?;

        let output = awk
            .wait_with_output()
            .map_err(|err| log::warn!("error getiing pane-capture | grep | awk: {err}"))?;
        if output.status.success() {
            pane_content = str::from_utf8(&output.stdout)
                .map_err(|err| log::error!("error getting file path: {}", err))
                .unwrap()
                .replace(['▍', '│'], "")
                .trim()
                .to_string();
        }
    }
    Ok(interface::code::Code::new(
        &code.tmux_session,
        &code.language,
        &pane_content,
        &code.github,
        code.attach_status,
        code.detach_status,
    ))
}

async fn run(rx: &Receiver<interface::code::Code>) {
    let mut code = rx.recv().map_err(|err| println!("{err}")).unwrap();
    let sess = code.tmux_session.clone();
    let mut client;
    'run: loop {
        let disc_status = get_open("Discord");
        log::info!("Check if discord is open: {disc_status}");
        if !disc_status {
            log::warn!("discord is closed... Waiting for connection...");
            continue 'run;
        }
        match get_client().await {
            Ok(disc) => {
                log::info!("Succesfullu connected to client");
                client = disc;
                log::info!(
                    "Session {} connected: {}",
                    &code.tmux_session,
                    &code.attach_status
                );
                log::info!("Coding in {}", &code.language);
                'update: loop {
                    let code_res = fetch_info(&code).await;
                    if code_res.is_ok() {
                        code = code_res.unwrap();
                        if !get_open("Discord") || load_client(&code, &mut client).await.is_err() {
                            continue 'run;
                        };
                        match rx.recv_timeout(Duration::from_secs(1)) {
                            Ok(cli) => code = cli,
                            Err(_) => continue 'update,
                        }
                    }
                    if code.detach_status {
                        let _ = client.clear_activity();
                        break 'run;
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(5));
                continue 'run;
            }
        }
    }
    let _ = client.close();
    log::info!("Session {} disconnected: {}", sess, &code.detach_status);
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
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.target(),
                record.level(),
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
    let (tx, rx) = mpsc::channel::<interface::code::Code>();

    thread::spawn(|| {
        // listen(tx);
        listener::start(tx);
    });

    loop {
        run(&rx).await;
    }
}
