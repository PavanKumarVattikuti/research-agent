use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String, // "INFO" | "SEARCH" | "RANK" | "LLM" | "WARN" | "ERROR" | "DONE"
    pub elapsed_ms: u64,
    pub message: String,
}

static LOG_BUFFER: Mutex<Vec<LogEntry>> = Mutex::new(Vec::new());

pub fn log_event(app: Option<&tauri::AppHandle>, level: &str, elapsed_ms: u64, message: &str) {
    let entry = LogEntry {
        timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
        level: level.to_string(),
        elapsed_ms,
        message: message.to_string(),
    };

    println!(
        "[{}] [{}] (+{}ms) {}",
        entry.timestamp, entry.level, entry.elapsed_ms, entry.message
    );

    if let Ok(mut buf) = LOG_BUFFER.lock() {
        if buf.len() >= 1000 {
            buf.remove(0);
        }
        buf.push(entry.clone());
    }

    if let Some(h) = app {
        h.emit("pipeline-log", entry).ok();
    }
}

pub fn get_all_logs() -> Vec<LogEntry> {
    LOG_BUFFER.lock().map(|b| b.clone()).unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut b) = LOG_BUFFER.lock() {
        b.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_buffer() {
        clear_logs();
        log_event(None, "INFO", 100, "Test log message");
        let logs = get_all_logs();
        assert!(!logs.is_empty());
        assert_eq!(logs.last().unwrap().message, "Test log message");
        clear_logs();
        assert!(get_all_logs().is_empty());
    }
}
