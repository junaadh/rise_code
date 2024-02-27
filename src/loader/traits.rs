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
    fn unwrap_log_or(self, msg: &str, value: T) -> T;
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
