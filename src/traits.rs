use std::{
    fmt::Debug,
    fs::{self, metadata, File, OpenOptions},
    os::unix::fs::PermissionsExt,
};

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

pub trait UnwrapAndLogRes<T> {
    fn unwrap_log(self) -> T;
}

impl<T, U> UnwrapAndLogRes<T> for Result<T, U>
where
    U: Debug,
{
    fn unwrap_log(self) -> T {
        self.map_err(|err| log::error!("{err:?}")).unwrap()
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

pub trait DirCreator {
    fn create(&self) -> File;
    fn check(&self) -> bool;
    #[allow(clippy::result_unit_err)]
    fn mkdir(&self) -> Result<(), ()>;
}

impl DirCreator for &str {
    fn create(&self) -> File {
        OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(self)
            .map_err(|err| log::error!("{err}"))
            .unwrap()
    }

    fn check(&self) -> bool {
        metadata(self).is_ok()
    }

    fn mkdir(&self) -> Result<(), ()> {
        match fs::create_dir_all(self) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("{err}");
                Err(())
            }
        }
    }
}

impl DirCreator for String {
    fn create(&self) -> File {
        OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(self)
            .map_err(|err| log::error!("{err}"))
            .unwrap()
    }

    fn check(&self) -> bool {
        metadata(self.as_str()).is_ok()
    }

    fn mkdir(&self) -> Result<(), ()> {
        match fs::create_dir_all(self) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("{err}");
                Err(())
            }
        }
    }
}

pub trait DirPerms {
    fn execute(self);
}

impl DirPerms for File {
    fn execute(self) {
        let mut permissions = self.metadata().unwrap_log().permissions();
        permissions.set_mode(0o755);
        self.set_permissions(permissions).unwrap_log();
    }
}
