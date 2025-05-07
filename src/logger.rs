use chrono;
use std::fs::OpenOptions;
use std::io::Write;

const LOG_FILE_NAME: &str = "project_fetcher.log";

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Success,
    Error,
    Warning,
}

impl LogLevel {
    fn to_prefix(&self) -> &'static str {
        match self {
            LogLevel::Info => "[INFO]",
            LogLevel::Success => "[SUCCESS]",
            LogLevel::Error => "[ERROR]",
            LogLevel::Warning => "[WARN]",
        }
    }
}

pub fn log_to_file(level: LogLevel, message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE_NAME)
    {
        let _ = writeln!(
            file,
            "[{}] {} {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level.to_prefix(),
            message
        );
    } else {
        eprintln!(
            "Failed to open or create log file: {}. Message: [{}] {} {}",
            LOG_FILE_NAME,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            level.to_prefix(),
            message
        );
    }
}