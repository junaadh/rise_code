use regex::Regex;

pub trait RegexMatcher {
    fn regex(&self, pattern: &str) -> bool;
}

impl RegexMatcher for String {
    fn regex(&self, pattern: &str) -> bool {
        let regex_pattern = Regex::new(pattern)
            .map_err(|err| log::debug!("{err}"))
            .unwrap();
        regex_pattern.is_match(self)
    }
}

pub trait UnwrapAndLog<T> {
    fn unwrap_log(self, msg: &str) -> T;
}

impl<T> UnwrapAndLog<T> for Option<T>
where
    T: Default,
{
    fn unwrap_log(self, msg: &str) -> T {
        match self {
            Some(res) => res,
            None => {
                log::warn!("{}", msg);
                T::default()
            }
        }
    }
}

pub trait UnwrapAndLogOr<T> {
    fn unwrap_log_or(self, msg: &str, value: T) -> T;
}

impl<T> UnwrapAndLogOr<T> for Option<T> {
    fn unwrap_log_or(self, msg: &str, value: T) -> T {
        match self {
            Some(res) => res,
            None => {
                log::warn!("{}", msg);
                value
            }
        }
    }
}

pub trait RiseFormat {
    fn truncate(&self, max_length: usize) -> &str;
}

impl RiseFormat for String {
    fn truncate(&self, max_length: usize) -> &str {
        match self.char_indices().nth(max_length) {
            Some((idx, _)) => &self[..idx],
            None => self,
        }
    }
}

impl RiseFormat for &str {
    fn truncate(&self, max_length: usize) -> &str {
        match self.char_indices().nth(max_length) {
            Some((idx, _)) => &self[..idx],
            None => self,
        }
    }
}

pub trait TimeKeeper {
    fn elapsed(&self) -> String;
}

pub trait EventHandler {
    fn write_msg(&self);
    fn ewrite_msg(&self);
}
