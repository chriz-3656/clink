use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

use crate::adb::AdbManager;
use crate::network;
use crate::overlay::renderer::OverlayRenderer;
use crate::pairing::PairingManager;
use crate::qr;

pub struct ClinkGuiApp {
    pub port: u16,
    pub lan_ip: String,
    pub pairing_mgr: PairingManager,
    pub adb_mgr: AdbManager,
    pub renderer: Arc<Mutex<OverlayRenderer>>,
    pub redraw_tx: broadcast::Sender<()>,
    pub current_pin: String,
    pub pairing_uri: String,
    pub qr_texture: Option<egui::TextureHandle>,
    pub adb_active: bool,
    pub adb_status_msg: String,
    pub overlay_active: bool,
    pub clickthrough_active: bool,
    pub fade_duration_secs: u64,
    pub screen_size: (u32, u32),
    pub status_notification: Option<(String, std::time::Instant)>,
}

impl ClinkGuiApp {
    pub fn new(
        port: u16,
        pairing_mgr: PairingManager,
        adb_mgr: AdbManager,
        renderer: Arc<Mutex<OverlayRenderer>>,
        redraw_tx: broadcast::Sender<()>,
        screen_size: (u32, u32),
        adb_active: bool,
    ) -> Self {
        let lan_ip = network::get_lan_ip();
        let current_pin = pairing_mgr.get_pin();
        let pairing_uri = qr::generate_pairing_uri(&lan_ip, port, &current_pin);
        let adb_status_msg = if adb_active {
            "Active (adb reverse tcp:8765 tcp:8765)".to_string()
        } else {
            "No USB device found (Check adb devices)".to_string()
        };

        Self {
            port,
            lan_ip,
            pairing_mgr,
            adb_mgr,
            renderer,
            redraw_tx,
            current_pin,
            pairing_uri,
            qr_texture: None,
            adb_active,
            adb_status_msg,
            overlay_active: true,
            clickthrough_active: true,
            fade_duration_secs: 0,
            screen_size,
            status_notification: None,
        }
    }

    pub fn regenerate_pin(&mut self) {
        self.current_pin = self.pairing_mgr.regenerate_pin();
        self.pairing_uri = qr::generate_pairing_uri(&self.lan_ip, self.port, &self.current_pin);
        self.qr_texture = None; // Invalidate texture to recreate
        self.status_notification = Some((
            "Generated fresh 6-digit security PIN!".to_string(),
            std::time::Instant::now(),
        ));
    }

    pub fn refresh_network_ip(&mut self) {
        self.lan_ip = network::get_lan_ip();
        self.pairing_uri = qr::generate_pairing_uri(&self.lan_ip, self.port, &self.current_pin);
        self.qr_texture = None;
        self.status_notification = Some((
            format!("Refreshed LAN IP: {}", self.lan_ip),
            std::time::Instant::now(),
        ));
    }

    pub fn configure_adb(&mut self) {
        self.adb_active = self.adb_mgr.setup_reverse();
        self.adb_status_msg = if self.adb_active {
            "Active (adb reverse tcp:8765 tcp:8765)".to_string()
        } else {
            "Failed to reverse port (Check USB debugging)".to_string()
        };
        self.status_notification = Some((
            if self.adb_active {
                "USB reverse port forwarding configured!".to_string()
            } else {
                "No authorized USB device detected.".to_string()
            },
            std::time::Instant::now(),
        ));
    }

    pub fn refresh_adb(&mut self) {
        let devices = self.adb_mgr.get_devices();
        if devices.is_empty() {
            self.adb_active = false;
            self.adb_status_msg = "No USB devices detected".to_string();
        } else {
            self.adb_status_msg = format!("Found {} device(s): {}", devices.len(), devices.join(", "));
        }
    }

    pub fn clear_canvas(&mut self) {
        if let Ok(mut r) = self.renderer.lock() {
            r.clear();
        }
        let _ = self.redraw_tx.send(());
        self.status_notification = Some((
            "Cleared all desktop canvas drawings.".to_string(),
            std::time::Instant::now(),
        ));
    }

    pub fn undo(&mut self) {
        let changed = if let Ok(mut r) = self.renderer.lock() {
            r.undo()
        } else {
            false
        };
        if changed {
            let _ = self.redraw_tx.send(());
            self.status_notification = Some(("Undo performed.".to_string(), std::time::Instant::now()));
        }
    }

    pub fn redo(&mut self) {
        let changed = if let Ok(mut r) = self.renderer.lock() {
            r.redo()
        } else {
            false
        };
        if changed {
            let _ = self.redraw_tx.send(());
            self.status_notification = Some(("Redo performed.".to_string(), std::time::Instant::now()));
        }
    }

    fn ensure_qr_texture(&mut self, ctx: &egui::Context) {
        if self.qr_texture.is_some() {
            return;
        }

        use qrcode::QrCode;
        if let Ok(code) = QrCode::new(self.pairing_uri.as_bytes()) {
            let image = code.render::<image::Luma<u8>>().min_dimensions(200, 200).build();
            let width = image.width() as usize;
            let height = image.height() as usize;
            let raw_pixels = image.into_raw();
            let mut color_pixels = Vec::with_capacity(width * height * 4);
            for lum in raw_pixels {
                if lum < 128 {
                    // Dark module: Dark deep navy/black
                    color_pixels.extend_from_slice(&[0x12, 0x12, 0x18, 0xFF]);
                } else {
                    // Light module: Crisp white
                    color_pixels.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
                }
            }
            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &color_pixels);
            self.qr_texture = Some(ctx.load_texture(
                "clink-qr-code",
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }
    }
}

impl eframe::App for ClinkGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.ensure_qr_texture(&ctx);
        ctx.request_repaint_after(Duration::from_millis(500));

        let pink = egui::Color32::from_rgb(0xFF, 0x33, 0x66);
        let cyan = egui::Color32::from_rgb(0x00, 0xE5, 0xFF);
        let green = egui::Color32::from_rgb(0x00, 0xE6, 0x76);
        let bg_color = egui::Color32::from_rgb(0x12, 0x12, 0x18);
        let card_bg = egui::Color32::from_rgb(0x19, 0x19, 0x26);
        let border_color = egui::Color32::from_rgb(0x28, 0x28, 0x3E);
        let text_dim = egui::Color32::from_rgb(0x9E, 0x9E, 0xB8);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = bg_color;
        visuals.window_fill = bg_color;
        ui.ctx().set_visuals(visuals);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);

            // --- TOP HEADER ---
            egui::Frame::canvas(ui.style())
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x28))
                .stroke(egui::Stroke::new(1.0, border_color))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading(egui::RichText::new("CLINK").color(egui::Color32::WHITE).strong());
                                ui.colored_label(pink, egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).strong());
                            });
                            ui.label(egui::RichText::new("Native Companion • Tablet & Desktop Annotation").color(text_dim).size(11.0));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(green, egui::RichText::new("● ONLINE").strong().size(12.0));
                        });
                    });
                });

            ui.add_space(10.0);

            // Notification Toast Pill
            if let Some((msg, time)) = &self.status_notification {
                if time.elapsed() < Duration::from_secs(3) {
                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::from_rgb(0x20, 0x18, 0x2C))
                        .stroke(egui::Stroke::new(1.0, pink))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(12, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(pink, egui::RichText::new("ℹ").strong());
                                ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE).size(11.5));
                            });
                        });
                    ui.add_space(8.0);
                }
            }

            // --- CARD 1: SERVICE & CONNECTION URLS ---
            render_card(ui, "COMPANION SERVICE", cyan, card_bg, border_color, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Service Status:").color(text_dim));
                    ui.colored_label(green, egui::RichText::new(format!("Active (Port {})", self.port)).strong());
                });

                ui.add_space(4.0);
                egui::Frame::canvas(ui.style())
                    .fill(egui::Color32::from_rgb(0x0F, 0x0F, 0x16))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x22, 0x22, 0x34)))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("Local (USB):      ws://127.0.0.1:{}", self.port)).monospace().size(11.0).color(egui::Color32::from_rgb(0xD6, 0xD6, 0xF0)));
                            ui.label(egui::RichText::new(format!("Wi-Fi / Hotspot:  ws://{}:{}", self.lan_ip, self.port)).monospace().size(11.0).color(egui::Color32::from_rgb(0xD6, 0xD6, 0xF0)));
                        });
                    });
            });

            ui.add_space(10.0);

            // --- CARD 2: INSTANT PAIRING (PIN & QR CODE) ---
            render_card(ui, "INSTANT PAIRING (PIN & QR CODE)", cyan, card_bg, border_color, |ui| {
                ui.horizontal(|ui| {
                    // Left Column: PIN & Buttons
                    ui.vertical(|ui| {
                        let formatted_pin = if self.current_pin.len() == 6 {
                            format!("{} {}", &self.current_pin[..3], &self.current_pin[3..])
                        } else {
                            self.current_pin.clone()
                        };

                        ui.label(egui::RichText::new(formatted_pin).size(30.0).strong().color(pink));
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("New PIN").strong()).fill(pink)).clicked() {
                                self.regenerate_pin();
                            }
                            if ui.add(egui::Button::new(egui::RichText::new("Refresh IP")).fill(egui::Color32::from_rgb(0x2A, 0x2A, 0x3E))).clicked() {
                                self.refresh_network_ip();
                            }
                        });

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Scan QR with Clink on Android or type the PIN to pair instantly over Wi-Fi or Hotspot.")
                                .color(text_dim)
                                .size(11.0),
                        );
                    });

                    // Right Column: QR Code Image in white border frame
                    if let Some(texture) = &self.qr_texture {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            egui::Frame::canvas(ui.style())
                                .fill(egui::Color32::WHITE)
                                .stroke(egui::Stroke::new(2.0, pink))
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    ui.image((texture.id(), egui::vec2(120.0, 120.0)));
                                });
                        });
                    }
                });
            });

            ui.add_space(10.0);

            // --- CARD 3: USB CONNECTION (ADB REVERSE) ---
            render_card(ui, "USB CONNECTION (ADB REVERSE)", cyan, card_bg, border_color, |ui| {
                ui.horizontal(|ui| {
                    let status_color = if self.adb_active { green } else { text_dim };
                    let dot = if self.adb_active { "●" } else { "○" };
                    ui.colored_label(status_color, egui::RichText::new(format!("{} {}", dot, self.adb_status_msg)).size(11.5));
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Configure USB (adb reverse)").strong()).fill(pink)).clicked() {
                        self.configure_adb();
                    }
                    if ui.add(egui::Button::new(egui::RichText::new("Refresh USB")).fill(egui::Color32::from_rgb(0x2A, 0x2A, 0x3E))).clicked() {
                        self.refresh_adb();
                    }
                });
            });

            ui.add_space(10.0);

            // --- CARD 4: DESKTOP ANNOTATION OVERLAY ---
            render_card(ui, "DESKTOP ANNOTATION OVERLAY", cyan, card_bg, border_color, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Overlay Active:").color(text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut active = self.overlay_active;
                        if ui.checkbox(&mut active, "").changed() {
                            self.overlay_active = active;
                            let _ = self.redraw_tx.send(());
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Click-Through Mode (Pass input to desktop):").color(text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.clickthrough_active, "");
                    });
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Auto-Clear Duration:").color(text_dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut selected = self.fade_duration_secs;
                        egui::ComboBox::from_id_salt("fade_duration_combo")
                            .selected_text(match selected {
                                0 => "Never (Persistent)",
                                3 => "3 seconds",
                                5 => "5 seconds",
                                10 => "10 seconds",
                                30 => "30 seconds",
                                60 => "60 seconds",
                                _ => "Custom",
                            })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut selected, 0, "Never (Persistent)").clicked() {
                                    self.fade_duration_secs = 0;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(0); }
                                }
                                if ui.selectable_value(&mut selected, 3, "3 seconds").clicked() {
                                    self.fade_duration_secs = 3;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(3); }
                                }
                                if ui.selectable_value(&mut selected, 5, "5 seconds").clicked() {
                                    self.fade_duration_secs = 5;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(5); }
                                }
                                if ui.selectable_value(&mut selected, 10, "10 seconds").clicked() {
                                    self.fade_duration_secs = 10;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(10); }
                                }
                                if ui.selectable_value(&mut selected, 30, "30 seconds").clicked() {
                                    self.fade_duration_secs = 30;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(30); }
                                }
                                if ui.selectable_value(&mut selected, 60, "60 seconds").clicked() {
                                    self.fade_duration_secs = 60;
                                    if let Ok(mut r) = self.renderer.lock() { r.set_fade_duration(60); }
                                }
                            });
                    });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Clear All").strong()).fill(pink)).clicked() {
                        self.clear_canvas();
                    }
                    if ui.add(egui::Button::new(egui::RichText::new("Undo").strong()).fill(egui::Color32::from_rgb(0x2A, 0x2A, 0x3E))).clicked() {
                        self.undo();
                    }
                    if ui.add(egui::Button::new(egui::RichText::new("Redo").strong()).fill(egui::Color32::from_rgb(0x2A, 0x2A, 0x3E))).clicked() {
                        self.redo();
                    }
                });
            });

            ui.add_space(10.0);

            // --- CARD 5: HARDWARE & SYSTEM INFO ---
            render_card(ui, "HARDWARE & SYSTEM", cyan, card_bg, border_color, |ui| {
                let (w, h) = self.screen_size;
                let (stroke_count, can_undo, can_redo) = if let Ok(r) = self.renderer.lock() {
                    (r.stroke_count(), r.can_undo(), r.can_redo())
                } else {
                    (0, false, false)
                };

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Screen Resolution:").color(text_dim));
                    ui.label(egui::RichText::new(format!("{} × {}", w, h)).strong());
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Active Strokes:").color(text_dim));
                    ui.label(egui::RichText::new(format!("{} (Undo: {}, Redo: {})", stroke_count, can_undo, can_redo)).color(pink));
                });
                ui.horizontal(|ui| {
                    #[cfg(target_os = "linux")]
                    let platform_name = "Linux (X11 / Wayland)";
                    #[cfg(target_os = "windows")]
                    let platform_name = "Windows (Win32 SendInput)";
                    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                    let platform_name = "Generic Desktop";

                    ui.label(egui::RichText::new("Platform Backend:").color(text_dim));
                    ui.label(egui::RichText::new(platform_name).strong());
                });
            });

            ui.add_space(16.0);
        });
    }
}

fn render_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    accent: egui::Color32,
    bg: egui::Color32,
    border: egui::Color32,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::canvas(ui.style())
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, border))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.colored_label(accent, egui::RichText::new(title).size(12.0).strong());
            ui.add_space(6.0);
            content(ui)
        })
        .inner
}
