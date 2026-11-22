//! Native Rust desktop companion entry point for Clink.

mod adb;
mod config;
mod dashboard;
mod gui;
mod input;
mod network;
mod overlay;
mod pairing;
mod protocol;
mod qr;
mod server;

use adb::AdbManager;
use clap::Parser;
use config::ClinkConfig;
use dashboard::{print_banner, print_dashboard};
use gui::ClinkGuiApp;
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
#[command(
    name = "clink",
    author = "chriz-3656",
    version = VERSION,
    about = "Clink Native Desktop Companion (Linux & Windows)"
)]
pub struct CliArgs {
    /// Port for WebSocket server
    #[arg(short, long, default_value_t = 8765)]
    pub port: u16,

    /// Bind host address
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Run in headless / CLI mode (no graphical GUI window)
    #[arg(long)]
    pub headless: bool,

    /// Disable authentication requirement
    #[arg(long)]
    pub no_auth: bool,

    /// Display QR code in terminal and exit
    #[arg(long)]
    pub qr: bool,

    /// Display current pairing PIN and exit
    #[arg(long)]
    pub pin: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        #[cfg(target_os = "windows")]
        {
            if let Some(win_backend) = input::windows::WindowsInputBackend::new() {
                let (w, h) = win_backend.get_screen_size();
                (Arc::new(win_backend), w, h)
            } else {
                (Arc::new(DummyInputBackend::default()), 1920, 1080)
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            (Arc::new(DummyInputBackend::default()), 1920, 1080)
        }
    };

    let renderer = Arc::new(Mutex::new(OverlayRenderer::new(scr_w, scr_h)));
    let (redraw_tx, _) = broadcast::channel::<()>(128);

    // Initialize native transparent overlay window if on Linux
    let _overlay_active = {
        #[cfg(target_os = "linux")]
        {
            if let Some(x11_win) = overlay::x11_window::X11OverlayWindow::new(renderer.clone()) {
                let mut rx = redraw_tx.subscribe();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    rt.block_on(async move {
                        while let Ok(_) = rx.recv().await {
                            x11_win.redraw();
                        }
                    });
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
    };

    // Auto-fade timer loop in background thread
    let renderer_clone = renderer.clone();
    let redraw_tx_clone = redraw_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
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
    });

    let config = ClinkConfig {
        host: args.host,
        port: args.port,
        require_auth: !args.no_auth,
        headless: args.headless,
        ..Default::default()
    };

    let server = ClinkServer::new(
        config,
        pairing_mgr.clone(),
        renderer.clone(),
        input_backend,
        redraw_tx.clone(),
    );

    // Launch background Tokio runtime for the WebSocket server
    let server_rt = tokio::runtime::Runtime::new()?;
    server_rt.spawn(async move {
        if let Err(e) = server.run().await {
            tracing::error!("WebSocket server error: {}", e);
        }
    });

    if args.headless {
        print_banner(VERSION);
        let ascii_qr = qr::render_terminal_qr(&pairing_uri);
        print_dashboard(args.port, &lan_ip, &current_pin, &ascii_qr, _overlay_active, adb_active);

        // Block on Ctrl+C in headless mode
        server_rt.block_on(async {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n  Shutting down Clink companion gracefully...");
        });
        adb_mgr.remove_reverse();
        return Ok(());
    }

    // Launch full native GUI Application Window
    let gui_app = ClinkGuiApp::new(
        args.port,
        pairing_mgr,
        adb_mgr.clone(),
        renderer,
        redraw_tx,
        (scr_w, scr_h),
        adb_active,
    );

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 780.0])
            .with_min_inner_size([460.0, 580.0])
            .with_title("Clink Desktop Companion"),
        ..Default::default()
    };

    let res = eframe::run_native(
        "Clink Desktop Companion",
        native_options,
        Box::new(move |_cc| Ok(Box::new(gui_app))),
    );

    // Cleanup when window closes
    adb_mgr.remove_reverse();
    server_rt.shutdown_timeout(Duration::from_millis(500));

    if let Err(e) = res {
        eprintln!("GUI Error: {}", e);
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
