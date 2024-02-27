#[macro_export]
macro_rules! sleep {
    ($time: tt) => {
        tokio::time::sleep(Duration::from_secs($time)).await
    };
}

#[macro_export]
macro_rules! sleep_ms {
    ($time: tt) => {
        tokio::time::sleep(Duration::from_millis($time)).await
    };
}

#[macro_export]
macro_rules! grep {
    ($str: expr, $search_term: expr) => {
        if let Some(index) = $str
            .split_whitespace()
            .position(|word| word == $search_term)
        {
            (index + 1) as i32
        } else {
            -1
        }
    };
}

#[macro_export]
macro_rules! awk {
    ($str: expr, $no: expr) => {
        $str.split_whitespace()
            .collect::<Vec<&str>>()
            .get(($no - 1) as usize)
            .unwrap_or(&" ")
            .to_string()
    };
}

#[macro_export]
macro_rules! open {
    ($name: expr) => {
        std::process::Command::new("pgrep")
            .arg($name)
            .output()
            .unwrap()
            .status
            .success()
    };
}

#[macro_export]
macro_rules! home {
    () => {
        std::env::var("HOME").unwrap_or_default()
    };
}
