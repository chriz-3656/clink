//! Automated ADB reverse port forwarder.

use std::process::Command;
use tracing::{debug, info};

#[derive(Clone)]
pub struct AdbManager {
    port: u16,
}

impl AdbManager {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn get_devices(&self) -> Vec<String> {
        let output = Command::new("adb").arg("devices").output();
        if let Ok(res) = output {
            let stdout = String::from_utf8_lossy(&res.stdout);
            stdout.lines()
                .skip(1)
                .filter(|line| line.contains("device") && !line.contains("offline"))
                .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
                .filter(|id| !id.is_empty())
                .collect()
        } else {
            Vec::new()
        }
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
