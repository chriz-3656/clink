//! Configuration settings for Clink companion.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinkConfig {
    pub host: String,
    pub port: u16,
    pub require_auth: bool,
    pub relative_sensitivity: f32,
    pub relative_acceleration: f32,
    pub default_timer_seconds: u32,
    pub headless: bool,
}

impl Default for ClinkConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8765,
            require_auth: true,
            relative_sensitivity: 1.0,
            relative_acceleration: 1.0,
            default_timer_seconds: 0,
            headless: false,
        }
    }
}
