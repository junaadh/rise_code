#[derive(Debug, Clone)]
pub struct Code {
    pub tmux_session: String,
    pub language: String,
    pub file: String,
    pub github: String,
    pub attach_status: bool,
    pub detach_status: bool,
}

impl Default for Code {
    fn default() -> Self {
        Self {
            tmux_session: "".to_string(),
            language: "".to_string(),
            file: "".to_string(),
            github: "".to_string(),
            attach_status: false,
            detach_status: true,
        }
    }
}

impl Code {
    pub fn new(
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

    pub fn check_integ(&self) -> bool {
        if self.tmux_session.is_empty() {
            return false;
        }
        if self.language.is_empty() {
            return false;
        }
        true
    }

    pub fn detach_new(detach_status: bool) -> Self {
        Self {
            tmux_session: "".to_string(),
            language: "".to_string(),
            file: "".to_string(),
            github: "".to_string(),
            attach_status: false,
            detach_status,
        }
    }

    pub fn disconnect(mut self) -> Self {
        self.attach_status = false;
        self.detach_status = true;
        self
    }
}
