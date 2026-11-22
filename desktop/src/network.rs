//! Network auto-detection for LAN IP and Hotspot.

use std::net::UdpSocket;

pub fn get_lan_ip() -> String {
    // Attempt to probe outbound routing without actually sending data
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return "127.0.0.1".to_string(),
    };
    
    // Connect to external IP (does not send packets, purely queries routing table)
    if socket.connect("10.255.255.255:1").is_ok() {
        if let Ok(local_addr) = socket.local_addr() {
            let ip = local_addr.ip().to_string();
            if ip != "0.0.0.0" {
                return ip;
            }
        }
    }
    
    // Fallback: Check standard Android hotspot gateway
    "127.0.0.1".to_string()
}
