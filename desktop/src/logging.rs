//! In-memory activity log tracker for Clink Desktop.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub struct ActivityLog {
    pub timestamp: String,
    pub tag: String,
    pub color_rgb: (u8, u8, u8),
    pub message: String,
}

static ACTIVITY_LOGS: OnceLock<Mutex<VecDeque<ActivityLog>>> = OnceLock::new();

pub fn log_activity(tag: &str, color_rgb: (u8, u8, u8), message: impl Into<String>) {
    let logs = ACTIVITY_LOGS.get_or_init(|| {
        let mut initial = VecDeque::with_capacity(120);
        initial.push_back(ActivityLog {
            timestamp: current_time_str(),
            tag: "SYSTEM".to_string(),
            color_rgb: (0x00, 0xE5, 0xFF),
            message: "Clink Desktop logging engine initialized".to_string(),
        });
        Mutex::new(initial)
    });
    let now = current_time_str();
    let mut guard = logs.lock().unwrap_or_else(|e| e.into_inner());
    if guard.len() >= 100 {
        guard.pop_front();
    }
    guard.push_back(ActivityLog {
        timestamp: now,
        tag: tag.to_string(),
        color_rgb,
        message: message.into(),
    });
}

pub fn get_activity_logs() -> Vec<ActivityLog> {
    let logs = ACTIVITY_LOGS.get_or_init(|| Mutex::new(VecDeque::with_capacity(120)));
    let guard = logs.lock().unwrap_or_else(|e| e.into_inner());
    guard.iter().cloned().collect()
}

pub fn clear_activity_logs() {
    let logs = ACTIVITY_LOGS.get_or_init(|| Mutex::new(VecDeque::with_capacity(120)));
    let mut guard = logs.lock().unwrap_or_else(|e| e.into_inner());
    guard.clear();
}

fn current_time_str() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sec = now % 60;
    let min = (now / 60) % 60;
    let hour = (now / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hour, min, sec)
}
