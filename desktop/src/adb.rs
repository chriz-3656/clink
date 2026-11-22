//! Automated ADB reverse port forwarder and dependency manager.

use std::path::Path;
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

    /// Finds the absolute path or command name for the ADB binary.
    pub fn get_adb_cmd() -> Option<String> {
        // 1. Check if `adb` is directly runnable from PATH
        if Command::new("adb").arg("version").output().is_ok() {
            return Some("adb".to_string());
        }

        // 2. Platform-specific known candidate paths
        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let candidates = [
                format!("{}\\Android\\Sdk\\platform-tools\\adb.exe", local_app_data),
                format!("{}\\platform-tools\\adb.exe", local_app_data),
                "C:\\adb\\adb.exe".to_string(),
                "C:\\platform-tools\\adb.exe".to_string(),
                "C:\\Program Files\\Android\\platform-tools\\adb.exe".to_string(),
                "C:\\Program Files\\platform-tools\\adb.exe".to_string(),
                ".\\adb.exe".to_string(),
                ".\\platform-tools\\adb.exe".to_string(),
            ];
            for candidate in candidates {
                if Path::new(&candidate).exists() {
                    return Some(candidate);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = [
                "/usr/bin/adb",
                "/usr/local/bin/adb",
                "/data/android-sdk/platform-tools/adb",
            ];
            for candidate in candidates {
                if Path::new(candidate).exists() {
                    return Some(candidate.to_string());
                }
            }
        }

        None
    }

    pub fn is_installed(&self) -> bool {
        Self::get_adb_cmd().is_some()
    }

    pub fn get_devices(&self) -> Vec<String> {
        let cmd = match Self::get_adb_cmd() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let output = Command::new(&cmd).arg("devices").output();
        if let Ok(res) = output {
            let stdout = String::from_utf8_lossy(&res.stdout);
            stdout
                .lines()
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
        let cmd = match Self::get_adb_cmd() {
            Some(c) => c,
            None => return false,
        };

        let port_str = format!("tcp:{}", self.port);
        let output = Command::new(&cmd)
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
                debug!("ADB tool failed to execute: {}", e);
                false
            }
        }
    }

    pub fn remove_reverse(&self) {
        if let Some(cmd) = Self::get_adb_cmd() {
            let port_str = format!("tcp:{}", self.port);
            let _ = Command::new(&cmd)
                .args(["reverse", "--remove", &port_str])
                .output();
        }
    }

    /// Triggers an interactive or automated installation prompt for ADB.
    pub fn try_install_adb() -> Result<String, String> {
        #[cfg(target_os = "linux")]
        {
            // Try pkexec graphical auth first
            if Command::new("which").arg("pkexec").output().map(|o| o.status.success()).unwrap_or(false) {
                let child = Command::new("pkexec")
                    .args(["apt-get", "install", "-y", "adb"])
                    .spawn();
                if let Ok(mut c) = child {
                    return match c.wait() {
                        Ok(st) if st.success() => Ok("Successfully installed ADB via package manager!".to_string()),
                        _ => Err("Installation cancelled or authentication failed.".to_string()),
                    };
                }
            }

            // Try launching a terminal window to run sudo apt install -y adb
            let terminals = [
                ("x-terminal-emulator", vec!["-e", "sudo apt-get update && sudo apt-get install -y adb"]),
                ("gnome-terminal", vec!["--", "sudo", "apt-get", "install", "-y", "adb"]),
                ("xterm", vec!["-e", "sudo apt-get install -y adb"]),
            ];

            for (term, args) in terminals {
                if Command::new("which").arg(term).output().map(|o| o.status.success()).unwrap_or(false) {
                    let _ = Command::new(term).args(&args).spawn();
                    return Ok(format!("Opened terminal ({}) to install adb with sudo.", term));
                }
            }

            Err("Please run: sudo apt install -y adb".to_string())
        }

        #[cfg(target_os = "windows")]
        {
            // Try launching winget in a separate cmd window to automatically install Google.PlatformTools
            let child = Command::new("cmd")
                .args([
                    "/C",
                    "start",
                    "cmd",
                    "/K",
                    "echo Installing Android Platform-Tools via Windows Package Manager (winget)... & winget install --id Google.PlatformTools -e --accept-source-agreements --accept-package-agreements || (echo. & echo Winget failed or not available. Opening official Google download page... & start https://developer.android.com/tools/releases/platform-tools)",
                ])
                .spawn();
            if child.is_ok() {
                return Ok("Launching Windows Package Manager (winget) to install Android Platform-Tools...".to_string());
            }

            // Fallback: Open official Google platform-tools download in default web browser
            let url = "https://developer.android.com/tools/releases/platform-tools";
            let _ = Command::new("cmd")
                .args(["/C", "start", url])
                .spawn();
            Ok("Opening official Android Platform-Tools download page in your browser...".to_string())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err("Please install 'adb' from your package manager.".to_string())
        }
    }
}
