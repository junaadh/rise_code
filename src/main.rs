use std::{
    io::Read,
    os::unix::net::UnixListener,
    process::{Command, Stdio},
    str,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};

enum Language {
    C,
    Cpp,
    Css,
    Go,
    Html,
    Java,
    Javascript,
    Lua,
    Python,
    R,
    Rust,
    Typescript,
}

impl Language {
    fn match_lang(&self) -> &str {
        match self {
            Self::C => "c-logo",
            Self::Cpp => "cpp-logo",
            Self::Css => "css-logo",
            Self::Go => "go-logo",
            Self::Html => "html-logo",
            Self::Java => "java-logo",
            Self::Javascript => "javascript-logo",
            Self::Lua => "lua-logo",
            Self::Python => "python-logo",
            Self::R => "r-logo",
            Self::Rust => "rust-logo",
            Self::Typescript => "typescript-logo",
        }
    }

    fn get_logo(lang: &str) -> &str {
        match lang {
            "c" => Self::C.match_lang(),
            "cpp" => Self::Cpp.match_lang(),
            "css" => Self::Css.match_lang(),
            "go" => Self::Go.match_lang(),
            "html" => Self::Html.match_lang(),
            "java" => Self::Java.match_lang(),
            "javascript" => Self::Javascript.match_lang(),
            "lua" => Self::Lua.match_lang(),
            "python" => Self::Python.match_lang(),
            "r" => Self::R.match_lang(),
            "rust" => Self::Rust.match_lang(),
            "typescript" => Self::Typescript.match_lang(),
            _ => "wild-card",
        }
    }
}

#[derive(Debug, Clone)]
struct Code {
    tmux_session: String,
    language: String,
    file: String,
    github: String,
    attach_status: bool,
    detach_status: bool,
}

impl Code {
    fn new(
        tmux_session: &str,
        language: &str,
        file: &str,
        github: &str,
        attach_status: bool,
        detach_status: bool,
    ) -> Self {
        Self {
            tmux_session: tmux_session.to_string(),
            language: language.to_string(),
            file: file.to_string(),
            github: github.to_string(),
            attach_status,
            detach_status,
        }
    }

    fn check_integ(&self) -> bool {
        if self.tmux_session.is_empty() {
            return false;
        }
        if self.language.is_empty() {
            return false;
        }
        true
    }

    // fn attach(mut self, cond: bool) -> Self {
    //     self.attach_status = cond;
    //     self
    // }

    // fn detach(mut self, cond: bool) -> Self {
    //     self.detach_status = cond;
    //     self
    // }
}

fn listen_attach(port: &str, buffer: &mut String) -> Code {
    let _ = Command::new("rm")
        .arg(port)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let listener = UnixListener::bind(port).expect("Failed to bind socket");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                buffer.clear();
                stream
                    .read_to_string(buffer)
                    .expect("Failed to read stream");
                let mut parts = buffer.trim().split(':');
                let sess = parts.next().unwrap_or_default();
                let lang = parts.next().unwrap_or_default();
                let file_name = parts.next().unwrap_or("");
                let repo_name = parts
                    .next()
                    .unwrap_or("https://google.com")
                    .replace(';', ":");
                return Code::new(sess, lang, file_name, repo_name.as_str(), true, false);
            }
            Err(_) => continue,
        }
    }
    Code::new("", "", "", "", false, false)
}

fn check_session_state(tmux_sess: &str) -> bool {
    let tmux = Command::new("tmux").arg("ls").output();
    match tmux {
        Ok(output) => {
            if output.status.success() {
                let output_str = str::from_utf8(&output.stdout).expect("Invalud utf");
                let res = !output_str
                    .split('\n')
                    .any(|line| line.contains(tmux_sess) && line.contains("(attached)"));
                res
            } else {
                true
            }
        }
        Err(_) => {
            println!("checking 123");
            true
        }
    }
}

fn listen_detach(tmux_sess: &str) -> Code {
    match check_session_state(tmux_sess) {
        true => Code::new("", "", "", "", false, true),
        false => Code::new("", "", "", "", false, false),
    }
}

fn listen(tx: Sender<Code>) {
    let mut buffer = String::new();
    let port = "/tmp/dev_rpc";
    println!("Listening on port {}", port);
    'main: loop {
        let code = listen_attach(port, &mut buffer);
        let sess = code.tmux_session.clone();
        match code.attach_status && code.check_integ() {
            true => {
                let _ = tx.send(code);
            }
            false => continue 'main,
        }
        thread::sleep(Duration::from_secs(5));
        'detach: loop {
            let code = listen_detach(&sess);
            match code.detach_status {
                true => {
                    let _ = tx.send(code);
                    continue 'main;
                }
                false => continue 'detach,
            }
        }
    }
}

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

async fn load_client(code: &Code, client: &mut DiscordIpcClient) -> Result<(), ()> {
    let big_text = format!("Programming Language: {}", code.language);
    let small_text = format!("Helix Editor opened in Tmux: {}", code.tmux_session);
    let code_str = if code.file.is_empty() {
        format!("Coding in {}", code.language)
    } else {
        format!("Editing: {}", code.file)
    };
    let tmux = format!("#Tmux: {}", &code.tmux_session);
    let assets = Assets::new()
        // .large_image("helix-logo");
        .large_image(Language::get_logo(&code.language))
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
    // TODO: buttons
    client.set_activity(activity).map_err(|_| {
        println!("Failed to load activity: trying again");
    })?;
    Ok(())
}

fn fetch_info(code: &Code) -> Code {
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
                .expect("Invalid format")
                .replace(['▍', '│'], "")
                .trim()
                .to_string();
        }
    }
    Code::new(
        &code.tmux_session,
        &code.language,
        &pane_content,
        &code.github,
        code.attach_status,
        code.detach_status,
    )
}

async fn run(rx: &Receiver<Code>) {
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
                println!(
                    "Session {} connected: {}",
                    &code.tmux_session, &code.attach_status
                );
                println!("Coding in {}", &code.language);
                'update: loop {
                    code = fetch_info(&code);
                    // if !get_open("Discord") {
                    // continue 'run;
                    // }
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
    println!("Session {} disconnected: {}", sess, &code.detach_status);
}

fn get_open(name: &str) -> bool {
    let discord = Command::new("pgrep").arg(name).output().unwrap();
    discord.status.success()
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<Code>();

    thread::spawn(|| {
        listen(tx);
    });

    loop {
        run(&rx).await;
    }
}
