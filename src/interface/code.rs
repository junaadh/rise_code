use std::time::{SystemTime, UNIX_EPOCH};

use crate::traits::TimeKeeper;

use super::languages::LanguageExt;

#[derive(Debug, Clone)]
pub struct Code {
    pub tmux_session: String,
    pub language: LanguageExt,
    pub file: String,
    pub github: String,
    pub attach_status: bool,
    pub detach_status: bool,
    pub duration: i64,
}

impl Default for Code {
    fn default() -> Self {
        Self {
            tmux_session: "".to_string(),
            language: LanguageExt::default(),
            file: "".to_string(),
            github: "".to_string(),
            attach_status: false,
            detach_status: true,
            duration: 0,
        }
    }
}

impl Code {
    pub fn new(
        tmux_session: &str,
        language: LanguageExt,
        file: &str,
        github: &str,
        attach_status: bool,
        detach_status: bool,
    ) -> Self {
        Self {
            tmux_session: tmux_session.to_string(),
            language,
            file: file.to_string(),
            github: github.to_string(),
            attach_status,
            detach_status,
            duration: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    pub fn check_integ(&self) -> bool {
        if self.tmux_session.is_empty() {
            return false;
        }
        true
    }

    pub fn detach_new(detach_status: bool) -> Self {
        Self {
            tmux_session: "".to_string(),
            language: LanguageExt::default(),
            file: "".to_string(),
            github: "".to_string(),
            attach_status: false,
            detach_status,
            duration: 0,
        }
    }

    pub fn disconnect(&mut self) {
        self.attach_status = false;
        self.detach_status = true;
        let duration = self.duration;
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.duration = time_now - duration;
    }

    pub fn language(&mut self, language: LanguageExt) {
        self.language = language;
    }

    pub fn file(&mut self, file: &str) {
        self.file = file.to_string()
    }

    pub fn update(&mut self, code: &Self) {
        self.tmux_session = code.tmux_session.clone();
        self.language = code.language.clone();
        self.file = code.file.clone();
        self.github = code.github.clone();
        self.attach_status = code.attach_status;
        self.detach_status = code.detach_status;
        self.duration = code.duration;
    }
}

impl TimeKeeper for Code {
    fn elapsed(&self) -> String {
        match self.duration {
            0..=59 => format!("Coded for {} secs", self.duration),
            60..=3599 => format!(
                "Coded for {mins}:{secs:02}",
                mins = &self.duration / 60,
                secs = &self.duration % 60
            ),
            _ => format!(
                "Coded for {hrs}:{mins:02}:{secs:02}",
                hrs = &self.duration / 3600,
                mins = (&self.duration % 3600) / 60,
                secs = &self.duration % 60
            ),
        }
    }
}
