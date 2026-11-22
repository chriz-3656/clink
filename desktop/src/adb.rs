//! Automated ADB reverse port forwarder.

use std::process::Command;
use tracing::{debug, info};

pub struct AdbManager {
    port: u16,
}

impl AdbManager {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn setup_reverse(&self) -> bool {
        let port_str = format!("tcp:{}", self.port);
        let output = Command::new("adb")
            .args(["reverse", &port_str, &port_str])
            .output();

        match output {
            Ok(res) if res.status.success() => {
                info!("ADB reverse port forwarding established on {}", port_str);
                true
            }
            Ok(res) => {
                let err = String::from_utf8_lossy(&res.stderr);
                debug!("ADB reverse returned: {}", err.trim());
                false
            }
            Err(e) => {
                debug!("ADB tool not found or failed to execute: {}", e);
                false
            }
        }
    }

    pub fn remove_reverse(&self) {
        let port_str = format!("tcp:{}", self.port);
        let _ = Command::new("adb")
            .args(["reverse", "--remove", &port_str])
            .output();
    }
}
