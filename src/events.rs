use crate::traits::EventHandler;

pub struct REventData {
    msg: String,
}

impl REventData {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}

macro_rules! REventDataNew {
    ($($args: tt)*) => {{
        REventData::new(format!($($args)*))
    }};
}

impl EventHandler for REventData {
    fn write_msg(&self) {
        log::info!("{}", self.msg);
        println!("{}", self.msg);
    }

    fn ewrite_msg(&self) {
        log::error!("{}", self.msg);
        eprintln!("{}", self.msg);
    }
}

pub enum REvents {
    Startlog,
    SocketEstablished,
    ListeningAttach,
    UnixStreamRecieved,
    AttachReceived,
    AttachAcknowledged,
    AttachDropped,
    FetchClientId,
    CheckDiscordStatus,
    DiscordOpen,
    DiscordClosed,
    ConnectingIpcClient,
    IpcConnectionSuccess,
    IpcConnectionError,
    DiscordUnexpextedExit,
    ListeningDetach,
    DetachRecieved,
    DetachAcknowledged,
    DetachDropped,
    IpcClearActivity,
    IpcCloseClient,
    SessionStatus,
    SessionLanguage,
}

impl REvents {
    fn data(&self, val: Option<String>) -> REventData {
        match self {
            Self::Startlog => REventDataNew!("Start log {}...", val.unwrap_or_default()),
            Self::SocketEstablished => {
                REventDataNew!("UnixSocket {} validated", val.unwrap_or_default())
            }
            Self::ListeningAttach => REventDataNew!("Listening for attach requests..."),
            Self::UnixStreamRecieved => REventDataNew!("Stream recieved successfully..."),
            Self::AttachReceived => REventDataNew!("Attach request recieved..."),
            Self::AttachAcknowledged => REventDataNew!("Attach request acknowledged..."),
            Self::AttachDropped => REventDataNew!(
                "Attach request invalid... Dropped request: {}",
                val.unwrap_or_default()
            ),
            Self::FetchClientId => REventDataNew!("Successfully fetched client_id"),
            Self::CheckDiscordStatus => REventDataNew!("Checking discord status..."),
            Self::DiscordOpen => REventDataNew!("Discord open... [OK]"),
            Self::DiscordClosed => REventDataNew!("Discord closed... Retrying..."),
            Self::ConnectingIpcClient => REventDataNew!("Connecting to discord ipc client..."),
            Self::IpcConnectionSuccess => REventDataNew!("Ipc client connected successfully..."),
            Self::IpcConnectionError => {
                REventDataNew!(
                    "Ipc client failed to connect: {} Retrying...",
                    val.unwrap_or_default()
                )
            }
            Self::DiscordUnexpextedExit => REventDataNew!("Discord exited unexpectedly..."),
            Self::ListeningDetach => REventDataNew!("Listening for detach requests..."),
            Self::DetachRecieved => REventDataNew!("Detach request received..."),
            Self::DetachAcknowledged => REventDataNew!("Detach request acknowldged..."),
            Self::DetachDropped => REventDataNew!(
                "Detach request invalid... Dropped request: {}",
                val.unwrap_or_default()
            ),
            Self::IpcClearActivity => REventDataNew!("Clear rich presence activity"),
            Self::IpcCloseClient => REventDataNew!("Close Ipc client..."),
            Self::SessionStatus => REventDataNew!("Session {}", val.unwrap_or_default()),
            Self::SessionLanguage => REventDataNew!("Coding in {}", val.unwrap_or_default()),
        }
    }

    pub fn flush(&self, val: Option<String>) {
        self.data(val).write_msg()
    }

    pub fn eflush(&self, val: Option<String>) {
        self.data(val).ewrite_msg()
    }
}
