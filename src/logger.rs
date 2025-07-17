use chrono::{DateTime, Local};
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::data::SAVE_DIR;
use crate::external::send_log;

static LOG_PATH: LazyLock<PathBuf> = LazyLock::new(|| SAVE_DIR.join("run.log"));

pub async fn log_message(msg: &str) {
    let now: DateTime<Local> = Local::now();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*LOG_PATH)
        .expect("Failed to open log file");
    let mut log = format!("{} {}\n", now.to_rfc3339(), msg);
    file.write_all(log.as_bytes()).expect("Failed to write log");
    log.pop();
    send_log(&log).await.ok();
}

pub fn read_logs() -> Vec<(DateTime<Local>, String)> {
    if let Ok(content) = read_to_string(&*LOG_PATH) {
        content
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ' ');
                let ts = parts.next()?;
                let msg = parts.next()?.to_string();
                DateTime::parse_from_rfc3339(ts)
                    .ok()
                    .map(|dt| (dt.with_timezone(&Local), msg))
            })
            .collect()
    } else {
        Vec::new()
    }
}
