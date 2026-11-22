//! Cyberpunk-styled terminal dashboard with guaranteed-rendered graphical icons and live indicators.

use colored::Colorize;

pub fn print_banner(version: &str) {
    println!("{}", "================================================================================".truecolor(40, 40, 62));
    println!(
        "{} {}",
        "  ⚡ CLINK DESKTOP COMPANION".truecolor(0, 229, 255).bold(),
        format!("v{} [Rust Native]", version).truecolor(255, 51, 102).bold()
    );
    println!("{}", "================================================================================".truecolor(40, 40, 62));
}

pub fn print_dashboard(
    port: u16,
    lan_ip: &str,
    pin: &str,
    ascii_qr: &str,
    overlay_active: bool,
    adb_active: bool,
) {
    let formatted_pin = if pin.len() == 6 {
        format!("{} {}", &pin[..3], &pin[3..])
    } else {
        pin.to_string()
    };

    println!();
    println!("  {}", "[STATUS DASHBOARD]".truecolor(0, 229, 255).bold());
    println!("  {} Server Status : {}", "●".truecolor(0, 230, 118), "ACTIVE & LISTENING".truecolor(0, 230, 118).bold());
    println!("  {} USB ADB       : {}", "●".truecolor(0, 229, 255), if adb_active { "CONNECTED / READY".truecolor(0, 230, 118) } else { "LISTENING (CONNECT USB)".truecolor(158, 158, 184) });
    println!("  {} Overlay Window: {}", "●".truecolor(255, 51, 102), if overlay_active { "ACTIVE (TRANSPARENT CLICK-THROUGH)".truecolor(0, 230, 118) } else { "HEADLESS / DISABLED".truecolor(158, 158, 184) });
    println!();

    println!("  {}", "[CONNECTION ENDPOINTS]".truecolor(0, 229, 255).bold());
    println!("  {} Local (USB)   : {}", "->".truecolor(255, 51, 102), format!("ws://127.0.0.1:{}", port).truecolor(240, 240, 250));
    println!("  {} Wi-Fi / Hotspot: {}", "->".truecolor(255, 51, 102), format!("ws://{}:{}", lan_ip, port).truecolor(240, 240, 250));
    println!();

    println!("  {}", "[INSTANT PAIRING PIN]".truecolor(0, 229, 255).bold());
    println!("  +--------------------+");
    println!("  |    {}   |", formatted_pin.truecolor(255, 51, 102).bold());
    println!("  +--------------------+");
    println!("  {}", "Enter this 6-digit PIN on the Clink Android app or scan the QR code below:".truecolor(158, 158, 184));
    println!();

    if !ascii_qr.is_empty() {
        println!("  {}", "[QUICK CONNECT QR CODE]".truecolor(0, 229, 255).bold());
        for line in ascii_qr.lines() {
            println!("  {}", line);
        }
        println!();
    }

    println!("  {}", "[SHORTCUTS & COMMANDS]".truecolor(0, 229, 255).bold());
    println!("  Press {} to shutdown companion gracefully.", "Ctrl + C".truecolor(255, 51, 102).bold());
    println!("{}", "================================================================================".truecolor(40, 40, 62));
}
