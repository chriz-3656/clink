//! Network auto-detection for LAN IP, Portable Hotspot, and interfaces.

use std::net::UdpSocket;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterfaceInfo {
    pub ip: String,
    pub label: String,
    pub is_hotspot: bool,
}

/// Probes the network routing table and interface list to find all valid local IPv4 addresses.
pub fn get_available_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut ips = Vec::new();

    // 1. Probe common gateways and subnets
    let probe_targets = [
        ("192.168.43.1:80", true, "Android Hotspot Gateway"),
        ("10.42.0.1:80", true, "Linux Hotspot Gateway"),
        ("192.168.137.1:80", true, "Windows Mobile Hotspot"),
        ("192.168.1.1:80", false, "Wi-Fi / LAN Router"),
        ("192.168.0.1:80", false, "Wi-Fi / LAN Router"),
        ("1.1.1.1:80", false, "Internet Gateway"),
        ("8.8.8.8:80", false, "Internet Gateway"),
        ("10.255.255.255:1", false, "Subnet Broadcast"),
    ];

    for (target, is_hotspot, default_desc) in probe_targets {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(target).is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip().to_string();
                    if ip != "0.0.0.0" && ip != "127.0.0.1" && !ips.iter().any(|i: &NetworkInterfaceInfo| i.ip == ip) {
                        let desc = if ip.starts_with("192.168.43.") {
                            "Android Portable Hotspot"
                        } else if ip.starts_with("10.42.0.") {
                            "Linux PC Hotspot"
                        } else if ip.starts_with("192.168.137.") {
                            "Windows Mobile Hotspot"
                        } else {
                            default_desc
                        };
                        ips.push(NetworkInterfaceInfo {
                            ip: ip.clone(),
                            label: format!("{} ({})", ip, desc),
                            is_hotspot: is_hotspot || ip.starts_with("192.168.43.") || ip.starts_with("10.42.0.") || ip.starts_with("192.168.137."),
                        });
                    }
                }
            }
        }
    }

    // 2. Query OS interface addresses directly
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("hostname").arg("-I").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for word in stdout.split_whitespace() {
                let clean_ip = word.trim().to_string();
                if clean_ip.contains('.') && !clean_ip.starts_with("127.") && !ips.iter().any(|i| i.ip == clean_ip) {
                    let desc = if clean_ip.starts_with("192.168.43.") {
                        "Android Portable Hotspot"
                    } else if clean_ip.starts_with("10.42.0.") {
                        "Linux PC Hotspot"
                    } else if clean_ip.starts_with("192.168.") {
                        "Wi-Fi LAN"
                    } else if clean_ip.starts_with("10.") {
                        "Local Network"
                    } else {
                        "Network Adapter"
                    };
                    ips.push(NetworkInterfaceInfo {
                        ip: clean_ip.clone(),
                        label: format!("{} ({})", clean_ip, desc),
                        is_hotspot: clean_ip.starts_with("192.168.43.") || clean_ip.starts_with("10.42.0."),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("ipconfig").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IPv4") {
                    if let Some(pos) = line.rfind(':') {
                        let clean_ip = line[pos + 1..].trim().to_string();
                        if clean_ip.contains('.') && !clean_ip.starts_with("127.") && !ips.iter().any(|i| i.ip == clean_ip) {
                            let desc = if clean_ip.starts_with("192.168.43.") {
                                "Android Portable Hotspot"
                            } else if clean_ip.starts_with("192.168.137.") {
                                "Windows Mobile Hotspot"
                            } else if clean_ip.starts_with("192.168.") {
                                "Wi-Fi LAN"
                            } else {
                                "Network Adapter"
                            };
                            ips.push(NetworkInterfaceInfo {
                                ip: clean_ip.clone(),
                                label: format!("{} ({})", clean_ip, desc),
                                is_hotspot: clean_ip.starts_with("192.168.43.") || clean_ip.starts_with("192.168.137."),
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort to prioritize Hotspot interfaces first:
    ips.sort_by_key(|info| {
        if info.ip.starts_with("192.168.43.") {
            0
        } else if info.ip.starts_with("10.42.0.") || info.ip.starts_with("192.168.137.") {
            1
        } else if info.ip.starts_with("192.168.") {
            2
        } else if info.ip.starts_with("10.") {
            3
        } else if info.ip.starts_with("172.") {
            4
        } else {
            5
        }
    });

    if ips.is_empty() {
        ips.push(NetworkInterfaceInfo {
            ip: "127.0.0.1".to_string(),
            label: "127.0.0.1 (Localhost)".to_string(),
            is_hotspot: false,
        });
    }

    ips
}

pub fn get_lan_ip() -> String {
    let interfaces = get_available_interfaces();
    interfaces.first().map(|i| i.ip.clone()).unwrap_or_else(|| "127.0.0.1".to_string())
}
