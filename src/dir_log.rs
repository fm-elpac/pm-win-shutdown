//! 基础路径 / 时间 / 日志
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use chrono::{Local, Utc};

/// 程序数据根目录: %LOCALAPPDATA%\pm-win-shutdown
pub fn base_dir() -> PathBuf {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    local.join("pm-win-shutdown")
}

/// 当前 UTC 时间戳, 格式与 (js) Date.toISOString() 相同: 2026-09-15T13:28:45.123Z
pub fn iso_utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// 日志文件名用的日期 (本地日期, 格式 YYYYMMDD, 如 20260915)
fn local_date_compact() -> String {
    Local::now().format("%Y%m%d").to_string()
}

/// 日志文件路径
pub fn log_file() -> Option<PathBuf> {
    let dir = base_dir().join("log");
    if fs::create_dir_all(&dir).is_err() {
        return None;
    }
    Some(dir.join(format!("{}.txt", local_date_compact())))
}

/// 追加一行日志到 %LOCALAPPDATA%\pm-win-shutdown\log\YYYYMMDD.txt (UTF-8, 仅追加)
pub fn log_line(msg: &str) {
    if let Some(p) = log_file() {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&p) {
            let _ = writeln!(f, "{}  {}", iso_utc_now(), msg);
        }
    }
}
