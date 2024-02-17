use std::{
    io::Read,
    os::unix::net::UnixListener,
    process::{Command, Stdio},
    str,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

#[derive(Debug, Clone)]
struct Code {
    tmux_session: String,
    language: String,
    attach_status: bool,
    detach_status: bool,
}

impl Code {
    fn new(tmux_session: &str, language: &str, attach_status: bool, detach_status: bool) -> Self {
        Self {
            tmux_session: tmux_session.to_string(),
            language: language.to_string(),
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

    fn attach(mut self, cond: bool) -> Self {
        self.attach_status = cond;
        self
    }

    fn detach(mut self, cond: bool) -> Self {
        self.detach_status = cond;
        self
    }
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
                // println!("{}", &buffer);
                let mut parts = buffer.trim().split(':');
                let sess = parts.next().unwrap_or_default();
                let lang = parts.next().unwrap_or_default();
                thread::sleep(Duration::from_secs(5));
                return Code::new(sess, lang, true, false);
            }
            Err(_) => continue,
        }
    }
    Code::new("", "", false, false)
}

fn check_session_state(tmux_sess: &str) -> bool {
    let tmux = Command::new("tmux").arg("ls").output();
    match tmux {
        Ok(output) => {
            if output.status.success() {
                let output_str = str::from_utf8(&output.stdout).expect("Invalud utf");
                return !output_str
                    .split('\n')
                    .any(|line| line.contains(tmux_sess) && line.contains("(attached)"));
            } else {
                false
            }
        }
        Err(_) => true,
    }
}

fn listen_detach(tmux_sess: &str) -> Code {
    match check_session_state(tmux_sess) {
        true => Code::new("", "", false, true),
        false => Code::new("", "", false, false),
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
            false => continue,
        }
        thread::sleep(Duration::from_secs(5));
        'detach: loop {
            let code = listen_detach(&sess);
            match code.detach_status {
                true => {
                    let _ = tx.send(code);
                    thread::sleep(Duration::from_secs(5));
                    continue 'main;
                }
                false => continue 'detach,
            }
        }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel::<Code>();

    thread::spawn(|| {
        listen(tx);
    });

    for rcv in rx {
        println!("{:#?}", rcv);
    }
}
