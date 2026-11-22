//! Native Rust desktop companion entry point for Clink.

mod adb;
mod config;
mod dashboard;
mod input;
mod network;
mod overlay;
mod pairing;
mod qr;
mod protocol;
mod server;

use adb::AdbManager;
use clap::Parser;
use config::ClinkConfig;
use dashboard::{print_banner, print_dashboard};
use input::dummy::DummyInputBackend;
use input::InputBackend;
use overlay::renderer::OverlayRenderer;
use pairing::PairingManager;
use server::ClinkServer;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "clink", author = "chriz-3656", version = VERSION, about = "Clink Native Desktop Companion (Linux & Windows)")]
struct CliArgs {
    /// Port for WebSocket server
    #[arg(short, long, default_value_t = 8765)]
    port: u16,

    /// Bind host address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Run in headless mode (no transparent overlay window)
    #[arg(long)]
    headless: bool,

    /// Disable authentication requirement
    #[arg(long)]
    no_auth: bool,

    /// Display QR code only and exit
    #[arg(long)]
    qr: bool,

    /// Display current PIN only and exit
    #[arg(long)]
    pin: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = CliArgs::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let pairing_mgr = PairingManager::new();
    let lan_ip = network::get_lan_ip();
    let current_pin = pairing_mgr.get_pin();
    let pairing_uri = qr::generate_pairing_uri(&lan_ip, args.port, &current_pin);

    // Save QR code PNG to cache directory
    let cache_dir = dirs_fallback().join("clink");
    let qr_png_path = cache_dir.join("clink-pair.png");
    let _ = qr::save_qr_png(&pairing_uri, &qr_png_path);

    if args.pin {
        println!("{}", current_pin);
        return Ok(());
    }

    if args.qr {
        let ascii_qr = qr::render_terminal_qr(&pairing_uri);
        println!("{}", ascii_qr);
        return Ok(());
    }

    print_banner(VERSION);

    let adb_mgr = AdbManager::new(args.port);
    let adb_active = adb_mgr.setup_reverse();

    // Initialize input backend
    let (input_backend, scr_w, scr_h): (Arc<dyn InputBackend>, u32, u32) = {
        #[cfg(target_os = "linux")]
        {
            if let Some(x11_backend) = input::x11::X11InputBackend::new() {
                let (w, h) = x11_backend.get_screen_size();
                (Arc::new(x11_backend), w, h)
            } else {
                (Arc::new(DummyInputBackend::default()), 1920, 1080)
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            (Arc::new(DummyInputBackend::default()), 1920, 1080)
        }
    };

    let renderer = Arc::new(Mutex::new(OverlayRenderer::new(scr_w, scr_h)));
    let (redraw_tx, _) = broadcast::channel::<()>(128);

    // Initialize native transparent overlay window if not headless
    let overlay_active = if !args.headless {
        #[cfg(target_os = "linux")]
        {
            if let Some(x11_win) = overlay::x11_window::X11OverlayWindow::new(renderer.clone()) {
                let mut rx = redraw_tx.subscribe();
                tokio::spawn(async move {
                    while let Ok(_) = rx.recv().await {
                        x11_win.redraw();
                    }
                });
                true
            } else {
                false
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    } else {
        false
    };

    let ascii_qr = qr::render_terminal_qr(&pairing_uri);
    print_dashboard(args.port, &lan_ip, &current_pin, &ascii_qr, overlay_active, adb_active);

    // Auto-fade timer loop
    let renderer_clone = renderer.clone();
    let redraw_tx_clone = redraw_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let needs_redraw = {
                if let Ok(mut r) = renderer_clone.lock() {
                    r.tick_fade()
                } else {
                    false
                }
            };
            if needs_redraw {
                let _ = redraw_tx_clone.send(());
            }
        }
    });

    let config = ClinkConfig {
        host: args.host,
        port: args.port,
        require_auth: !args.no_auth,
        headless: args.headless,
        ..Default::default()
    };

    let server = ClinkServer::new(config, pairing_mgr, renderer, input_backend, redraw_tx);

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                eprintln!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n  Shutting down Clink companion gracefully...");
            adb_mgr.remove_reverse();
        }
    }

    Ok(())
}

fn dirs_fallback() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        std::env::temp_dir()
    }
}
