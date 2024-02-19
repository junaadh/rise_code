pub mod interface;
pub mod listener;

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

fn fetch_info(code: &interface::code::Code) -> interface::code::Code {
    let window_info = Command::new("tmux")
        .args([
            "list-windows",
            "-t",
            &code.tmux_session,
            "-F",
            "#{window_index} #{window_name}",
        ])
        .output()
        .map_err(|_| println!("Cannot get windowindex & windowname"))
        .unwrap();
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
                "-p",
                "-t",
                &format!("{}:{}", &code.tmux_session, info),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let grep = Command::new("grep")
            .arg("sel")
            .stdin(Stdio::from(pane_capture.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let awk = Command::new("awk")
            .arg("{ print $2 }")
            .stdin(Stdio::from(grep.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let output = awk.wait_with_output().unwrap();
        if output.status.success() {
            pane_content = str::from_utf8(&output.stdout)
                .map_err(|err| log::error!("error getting file path: {}", err))
                .unwrap()
                .replace(['▍', '│'], "")
                .trim()
                .to_string();
        }
    }
    interface::code::Code::new(
        &code.tmux_session,
        &code.language,
        &pane_content,
        &code.github,
        code.attach_status,
        code.detach_status,
    )
}

async fn run(rx: &Receiver<interface::code::Code>) {
    let mut code = rx.recv().unwrap();
    let sess = code.tmux_session.clone();
    let mut client;
    'run: loop {
        if !get_open("Discord") {
            continue 'run;
        }
        match get_client().await {
            Ok(disc) => {
                client = disc;
                log::info!(
                    "Session {} connected: {}",
                    &code.tmux_session,
                    &code.attach_status
                );
                log::info!("Coding in {}", &code.language);
                'update: loop {
                    code = fetch_info(&code);
                    if !get_open("Discord") || load_client(&code, &mut client).await.is_err() {
                        continue 'run;
                    };
                    match rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(cli) => code = cli,
                        Err(_) => continue 'update,
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
