use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

use crate::adb::AdbManager;
use crate::network::{self, NetworkInterfaceInfo};
use crate::overlay::renderer::OverlayRenderer;
use crate::pairing::PairingManager;
use crate::qr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Pairing,
    Overlay,
    Logs,
    Settings,
}

pub struct ClinkGuiApp {
    pub port: u16,
    pub lan_ip: String,
    pub available_interfaces: Vec<NetworkInterfaceInfo>,
    pub selected_interface_idx: usize,
    pub pairing_mgr: PairingManager,
    pub adb_mgr: AdbManager,
    pub renderer: Arc<Mutex<OverlayRenderer>>,
    pub redraw_tx: broadcast::Sender<()>,
    pub current_pin: String,
    pub pairing_uri: String,
    pub qr_texture: Option<egui::TextureHandle>,
    pub adb_installed: bool,
    pub adb_active: bool,
    pub adb_status_msg: String,
    pub overlay_active: bool,
    pub clickthrough_active: bool,
    pub fade_duration_secs: u64,
    pub screen_size: (u32, u32),
    pub status_notification: Option<(String, std::time::Instant)>,
    pub active_tab: ActiveTab,
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
        let available_interfaces = network::get_available_interfaces();
        let lan_ip = available_interfaces
            .first()
            .map(|i| i.ip.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let current_pin = pairing_mgr.get_pin();
        let pairing_uri = qr::generate_pairing_uri(&lan_ip, port, &current_pin);
        let adb_installed = adb_mgr.is_installed();
        let adb_status_msg = if !adb_installed {
            "ADB is not installed on this system".to_string()
        } else if adb_active {
            "Active (adb reverse tcp:8765 tcp:8765)".to_string()
        } else {
            "ADB ready. Plug in USB cable to connect".to_string()
        };

        crate::logging::log_activity("INIT", (0x00, 0xE5, 0xFF), format!("Clink companion started on port {}", port));

        Self {
            port,
            lan_ip,
            available_interfaces,
            selected_interface_idx: 0,
            pairing_mgr,
            adb_mgr,
            renderer,
            redraw_tx,
            current_pin,
            pairing_uri,
            qr_texture: None,
            adb_installed,
            adb_active,
            adb_status_msg,
            overlay_active: true,
            clickthrough_active: true,
            fade_duration_secs: 0,
            screen_size,
            status_notification: None,
            active_tab: ActiveTab::Pairing,
        }
    }

    pub fn set_interface_idx(&mut self, idx: usize) {
        if idx < self.available_interfaces.len() {
            self.selected_interface_idx = idx;
            self.lan_ip = self.available_interfaces[idx].ip.clone();
            self.pairing_uri = qr::generate_pairing_uri(&self.lan_ip, self.port, &self.current_pin);
            self.qr_texture = None;
            let label = self.available_interfaces[idx].label.clone();
            let msg = format!("Switched network interface: {}", label);
            crate::logging::log_activity("NET", (0x00, 0xE5, 0xFF), &msg);
            self.status_notification = Some((msg, std::time::Instant::now()));
        }
    }

    pub fn regenerate_pin(&mut self) {
        self.current_pin = self.pairing_mgr.regenerate_pin();
        self.pairing_uri = qr::generate_pairing_uri(&self.lan_ip, self.port, &self.current_pin);
        self.qr_texture = None;
        crate::logging::log_activity("PIN", (0xFF, 0x33, 0x66), "Generated new 6-digit security PIN");
        self.status_notification = Some((
            "Generated new 6-digit pairing PIN".to_string(),
            std::time::Instant::now(),
        ));
    }

    pub fn refresh_network(&mut self) {
        self.available_interfaces = network::get_available_interfaces();
        if self.selected_interface_idx >= self.available_interfaces.len() {
            self.selected_interface_idx = 0;
        }
        self.lan_ip = self.available_interfaces
            .get(self.selected_interface_idx)
            .map(|i| i.ip.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        self.pairing_uri = qr::generate_pairing_uri(&self.lan_ip, self.port, &self.current_pin);
        self.qr_texture = None;
        let msg = format!("Refreshed interfaces (found {})", self.available_interfaces.len());
        crate::logging::log_activity("NET", (0x00, 0xE5, 0xFF), &msg);
        self.status_notification = Some((msg, std::time::Instant::now()));
    }

    pub fn configure_adb(&mut self) {
        self.adb_installed = self.adb_mgr.is_installed();
        if !self.adb_installed {
            let msg = "ADB is missing. Please install adb first.".to_string();
            crate::logging::log_activity("WARN", (0xFF, 0x91, 0x00), &msg);
            self.status_notification = Some((msg, std::time::Instant::now()));
            return;
        }

        self.adb_active = self.adb_mgr.setup_reverse();
        self.adb_status_msg = if self.adb_active {
            "Active (adb reverse tcp:8765 tcp:8765)".to_string()
        } else {
            "USB connected, but reverse failed (check USB debugging)".to_string()
        };
        let msg = if self.adb_active {
            "ADB reverse port forwarding active!".to_string()
        } else {
            "No authorized USB device found.".to_string()
        };
        crate::logging::log_activity("USB", if self.adb_active { (0x00, 0xE6, 0x76) } else { (0xFF, 0x91, 0x00) }, &msg);
        self.status_notification = Some((msg, std::time::Instant::now()));
    }

    pub fn refresh_adb(&mut self) {
        self.adb_installed = self.adb_mgr.is_installed();
        if !self.adb_installed {
            self.adb_status_msg = "ADB tool is not installed on this system".to_string();
            return;
        }

        let devices = self.adb_mgr.get_devices();
        if devices.is_empty() {
            self.adb_active = false;
            self.adb_status_msg = "No USB devices detected (connect phone)".to_string();
        } else {
            self.adb_status_msg = format!("Found {} device(s): {}", devices.len(), devices.join(", "));
        }
        crate::logging::log_activity("USB", (0x00, 0xE5, 0xFF), &self.adb_status_msg);
    }

    pub fn install_adb_prompt(&mut self) {
        match AdbManager::try_install_adb() {
            Ok(msg) => {
                crate::logging::log_activity("USB", (0x00, 0xE6, 0x76), &msg);
                self.status_notification = Some((msg, std::time::Instant::now()));
            }
            Err(e) => {
                let err_msg = format!("Install hint: {}", e);
                crate::logging::log_activity("WARN", (0xFF, 0x91, 0x00), &err_msg);
                self.status_notification = Some((err_msg, std::time::Instant::now()));
            }
        }
        self.adb_installed = self.adb_mgr.is_installed();
    }

    pub fn clear_canvas(&mut self) {
        if let Ok(mut r) = self.renderer.lock() {
            r.clear();
        }
        let _ = self.redraw_tx.send(());
        crate::logging::log_activity("CANVAS", (0xFF, 0x33, 0x66), "Cleared all desktop canvas drawings");
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
            crate::logging::log_activity("CANVAS", (0x00, 0xE5, 0xFF), "Undo action performed");
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
            crate::logging::log_activity("CANVAS", (0x00, 0xE5, 0xFF), "Redo action performed");
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
                    color_pixels.extend_from_slice(&[0x0E, 0x0F, 0x16, 0xFF]);
                } else {
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
        let orange = egui::Color32::from_rgb(0xFF, 0x91, 0x00);
        let bg_color = egui::Color32::from_rgb(0x0C, 0x0D, 0x14);
        let card_bg = egui::Color32::from_rgb(0x13, 0x14, 0x20);
        let border_color = egui::Color32::from_rgb(0x23, 0x25, 0x38);
        let text_dim = egui::Color32::from_rgb(0x9E, 0x9E, 0xB8);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = bg_color;
        visuals.window_fill = bg_color;
        ui.ctx().set_visuals(visuals);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(10.0);

                // ==========================================
                // PROFESSIONAL CENTERED STUDIO CONTAINER
                // (Constrained to max width 540.0 px so expanding
                // or maximizing never stretches elements out of proportion)
                // ==========================================
                ui.vertical_centered(|ui| {
                    ui.set_max_width(540.0);

                    // ==========================================
                    // 1. HEADER BAR (Vector Brand Emblem & Live Status)
                    // ==========================================
                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::from_rgb(0x10, 0x11, 0x1D))
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .corner_radius(egui::CornerRadius::same(14))
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Vector Brand Emblem
                                let (logo_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                                ui.painter().rect_filled(logo_rect, 7.0, egui::Color32::from_rgb(0x28, 0x10, 0x22));
                                ui.painter().rect_stroke(logo_rect, 7.0, egui::Stroke::new(1.2, pink), egui::StrokeKind::Inside);

                                let c = logo_rect.center();
                                ui.painter().circle_filled(egui::pos2(c.x - 5.0, c.y), 3.0, pink);
                                ui.painter().circle_filled(egui::pos2(c.x + 5.0, c.y), 3.0, cyan);
                                ui.painter().line_segment(
                                    [egui::pos2(c.x - 5.0, c.y), egui::pos2(c.x + 5.0, c.y)],
                                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                                );

                                ui.add_space(4.0);

                                // Title & Studio Tag
                                ui.heading(
                                    egui::RichText::new("CLINK")
                                        .color(egui::Color32::WHITE)
                                        .strong()
                                        .size(19.0),
                                );
                                egui::Frame::canvas(ui.style())
                                    .fill(egui::Color32::from_rgb(0x28, 0x10, 0x20))
                                    .stroke(egui::Stroke::new(1.0, pink))
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                    .show(ui, |ui| {
                                        ui.colored_label(
                                            pink,
                                            egui::RichText::new("STUDIO").strong().size(10.0),
                                        );
                                    });

                                // Right-aligned live status pill
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(egui::Color32::from_rgb(0x0C, 0x24, 0x18))
                                        .stroke(egui::Stroke::new(1.0, green))
                                        .corner_radius(egui::CornerRadius::same(12))
                                        .inner_margin(egui::Margin::symmetric(10, 4))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                paint_status_dot(ui, green, 3.5);
                                                ui.colored_label(
                                                    green,
                                                    egui::RichText::new("ONLINE").strong().size(11.0),
                                                );
                                            });
                                        });
                                });
                            });
                        });

                    ui.add_space(10.0);

                    // ==========================================
                    // 2. MODERN NAVIGATION TABS (4 Clean Channels)
                    // ==========================================
                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::from_rgb(0x12, 0x13, 0x1E))
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(6, 6))
                        .show(ui, |ui| {
                            ui.columns(4, |columns| {
                                // Tab 1: Connect
                                let is_pairing = self.active_tab == ActiveTab::Pairing;
                                let tab1_bg = if is_pairing { egui::Color32::from_rgb(0x24, 0x14, 0x26) } else { egui::Color32::TRANSPARENT };
                                let tab1_stroke = if is_pairing { egui::Stroke::new(1.0, pink) } else { egui::Stroke::NONE };
                                let tab1_color = if is_pairing { pink } else { text_dim };

                                columns[0].vertical_centered(|ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(tab1_bg)
                                        .stroke(tab1_stroke)
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::symmetric(6, 7))
                                        .show(ui, |ui| {
                                            if ui.button(egui::RichText::new("Connect").color(tab1_color).strong().size(12.0)).clicked() {
                                                self.active_tab = ActiveTab::Pairing;
                                            }
                                        });
                                });

                                // Tab 2: Canvas & Overlay
                                let is_overlay = self.active_tab == ActiveTab::Overlay;
                                let tab2_bg = if is_overlay { egui::Color32::from_rgb(0x10, 0x24, 0x2C) } else { egui::Color32::TRANSPARENT };
                                let tab2_stroke = if is_overlay { egui::Stroke::new(1.0, cyan) } else { egui::Stroke::NONE };
                                let tab2_color = if is_overlay { cyan } else { text_dim };

                                columns[1].vertical_centered(|ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(tab2_bg)
                                        .stroke(tab2_stroke)
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::symmetric(6, 7))
                                        .show(ui, |ui| {
                                            if ui.button(egui::RichText::new("Overlay").color(tab2_color).strong().size(12.0)).clicked() {
                                                self.active_tab = ActiveTab::Overlay;
                                            }
                                        });
                                });

                                // Tab 3: Live Activity Logs
                                let is_logs = self.active_tab == ActiveTab::Logs;
                                let tab3_bg = if is_logs { egui::Color32::from_rgb(0x18, 0x26, 0x1A) } else { egui::Color32::TRANSPARENT };
                                let tab3_stroke = if is_logs { egui::Stroke::new(1.0, green) } else { egui::Stroke::NONE };
                                let tab3_color = if is_logs { green } else { text_dim };

                                columns[2].vertical_centered(|ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(tab3_bg)
                                        .stroke(tab3_stroke)
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::symmetric(6, 7))
                                        .show(ui, |ui| {
                                            if ui.button(egui::RichText::new("Activity Logs").color(tab3_color).strong().size(12.0)).clicked() {
                                                self.active_tab = ActiveTab::Logs;
                                            }
                                        });
                                });

                                // Tab 4: System & Settings
                                let is_settings = self.active_tab == ActiveTab::Settings;
                                let tab4_bg = if is_settings { egui::Color32::from_rgb(0x1A, 0x1A, 0x2A) } else { egui::Color32::TRANSPARENT };
                                let tab4_stroke = if is_settings { egui::Stroke::new(1.0, egui::Color32::from_rgb(0x50, 0x50, 0x75)) } else { egui::Stroke::NONE };
                                let tab4_color = if is_settings { egui::Color32::WHITE } else { text_dim };

                                columns[3].vertical_centered(|ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(tab4_bg)
                                        .stroke(tab4_stroke)
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::symmetric(6, 7))
                                        .show(ui, |ui| {
                                            if ui.button(egui::RichText::new("Settings").color(tab4_color).strong().size(12.0)).clicked() {
                                                self.active_tab = ActiveTab::Settings;
                                            }
                                        });
                                });
                            });
                        });

                    ui.add_space(8.0);

                    // ==========================================
                    // 3. TOAST NOTIFICATION PILL
                    // ==========================================
                    if let Some((msg, time)) = &self.status_notification {
                        if time.elapsed() < Duration::from_secs(4) {
                            egui::Frame::canvas(ui.style())
                                .fill(egui::Color32::from_rgb(0x1E, 0x14, 0x24))
                                .stroke(egui::Stroke::new(1.0, pink))
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::symmetric(14, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        paint_status_dot(ui, pink, 3.0);
                                        ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE).size(12.0));
                                    });
                                });
                            ui.add_space(8.0);
                        }
                    }

                    // ==========================================
                    // 4. TAB CONTENT ROUTING
                    // ==========================================
                    match self.active_tab {
                        ActiveTab::Pairing => self.render_pairing_tab(ui, pink, cyan, green, orange, card_bg, border_color, text_dim),
                        ActiveTab::Overlay => self.render_overlay_tab(ui, pink, cyan, green, card_bg, border_color, text_dim),
                        ActiveTab::Logs => self.render_logs_tab(ui, pink, cyan, green, card_bg, border_color, text_dim),
                        ActiveTab::Settings => self.render_settings_tab(ui, cyan, green, orange, card_bg, border_color, text_dim),
                    }
                });

                ui.add_space(16.0);
            });
    }
}

impl ClinkGuiApp {
    fn render_pairing_tab(
        &mut self,
        ui: &mut egui::Ui,
        pink: egui::Color32,
        cyan: egui::Color32,
        green: egui::Color32,
        orange: egui::Color32,
        card_bg: egui::Color32,
        border_color: egui::Color32,
        text_dim: egui::Color32,
    ) {
        // --- 1. HERO INSTANT PAIRING CARD (QR + PIN) ---
        render_card(ui, "INSTANT PAIRING (OPTICAL QR & PIN)", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                // Left Column: Bold PIN, Controls, & Instructions
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Pairing Security PIN:").color(text_dim).size(11.0));
                    let formatted_pin = if self.current_pin.len() == 6 {
                        format!("{} {}", &self.current_pin[..3], &self.current_pin[3..])
                    } else {
                        self.current_pin.clone()
                    };

                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::from_rgb(0x0C, 0x0D, 0x14))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x35, 0x18, 0x2A)))
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(formatted_pin).size(30.0).strong().color(pink));
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("New PIN").strong().size(11.5)).fill(pink)).clicked() {
                            self.regenerate_pin();
                        }
                        if ui.add(egui::Button::new(egui::RichText::new("Copy PIN").size(11.5)).fill(egui::Color32::from_rgb(0x24, 0x24, 0x38))).clicked() {
                            ui.ctx().copy_text(self.current_pin.clone());
                            self.status_notification = Some(("PIN copied to clipboard!".to_string(), std::time::Instant::now()));
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Scan with Clink on Android or connect via USB for sub-millisecond drawing response.")
                            .color(text_dim)
                            .size(11.0),
                    );
                });

                // Right Column: QR Code Texture with Glow Border
                if let Some(texture) = &self.qr_texture {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::canvas(ui.style())
                            .fill(egui::Color32::WHITE)
                            .stroke(egui::Stroke::new(2.0, pink))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.image((texture.id(), egui::vec2(130.0, 130.0)));
                            });
                    });
                }
            });
        });

        ui.add_space(10.0);

        // --- 2. WI-FI & PORTABLE HOTSPOT NETWORK SELECTOR ---
        render_card(ui, "WI-FI & PORTABLE HOTSPOT CONFIGURATION", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Network Interface:").color(text_dim));
                let current_label = self.available_interfaces
                    .get(self.selected_interface_idx)
                    .map(|i| i.label.clone())
                    .unwrap_or_else(|| self.lan_ip.clone());

                let mut selected_idx = self.selected_interface_idx;
                let mut switch_to_idx = None;
                egui::ComboBox::from_id_salt("net_interface_combo_v3")
                    .selected_text(egui::RichText::new(&current_label).size(12.0).strong())
                    .show_ui(ui, |ui| {
                        for (idx, iface) in self.available_interfaces.iter().enumerate() {
                            let label_text = if iface.is_hotspot {
                                format!("[Hotspot] {}", iface.label)
                            } else {
                                format!("[LAN] {}", iface.label)
                            };
                            if ui.selectable_value(&mut selected_idx, idx, label_text).clicked() {
                                switch_to_idx = Some(idx);
                            }
                        }
                    });
                if let Some(idx) = switch_to_idx {
                    self.set_interface_idx(idx);
                }

                if ui.add(egui::Button::new(egui::RichText::new("Refresh").size(11.0)).fill(egui::Color32::from_rgb(0x22, 0x22, 0x36))).clicked() {
                    self.refresh_network();
                }
            });

            // Hotspot Banner if active
            let is_hotspot = self.available_interfaces
                .get(self.selected_interface_idx)
                .map(|i| i.is_hotspot)
                .unwrap_or(false);

            if is_hotspot {
                ui.add_space(6.0);
                egui::Frame::canvas(ui.style())
                    .fill(egui::Color32::from_rgb(0x0E, 0x26, 0x1A))
                    .stroke(egui::Stroke::new(1.0, green))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            paint_status_dot(ui, green, 3.5);
                            ui.colored_label(green, egui::RichText::new("Direct Antenna Hotspot Active").strong().size(11.5));
                            ui.label(egui::RichText::new("(Zero router or internet data needed)").size(11.0).color(text_dim));
                        });
                    });
            }

            ui.add_space(8.0);
            egui::Frame::canvas(ui.style())
                .fill(egui::Color32::from_rgb(0x0B, 0x0C, 0x12))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x20, 0x22, 0x32)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("WebSocket URI:  ws://{}:{}", self.lan_ip, self.port)).monospace().size(11.5).color(egui::Color32::from_rgb(0xD6, 0xD6, 0xF0)));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("Copy URI").size(11.0)).fill(egui::Color32::from_rgb(0x24, 0x24, 0x38))).clicked() {
                                ui.ctx().copy_text(format!("ws://{}:{}", self.lan_ip, self.port));
                                self.status_notification = Some(("WebSocket URI copied to clipboard!".to_string(), std::time::Instant::now()));
                            }
                        });
                    });
                });
        });

        ui.add_space(10.0);

        // --- 3. USB WIRED CONNECTION (ADB REVERSE) ---
        render_card(ui, "USB CABLE CONNECTION (ADB REVERSE TUNNEL)", cyan, card_bg, border_color, |ui| {
            if !self.adb_installed {
                egui::Frame::canvas(ui.style())
                    .fill(egui::Color32::from_rgb(0x26, 0x1A, 0x12))
                    .stroke(egui::Stroke::new(1.0, orange))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.colored_label(orange, egui::RichText::new("[!] ADB Missing: Android Debug Bridge is required for USB mode").strong());
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                #[cfg(target_os = "linux")]
                                {
                                    if ui.add(egui::Button::new(egui::RichText::new("Auto-Install ADB").strong()).fill(orange)).clicked() {
                                        self.install_adb_prompt();
                                    }
                                }
                                if ui.add(egui::Button::new(egui::RichText::new("Check Again")).fill(egui::Color32::from_rgb(0x28, 0x28, 0x3C))).clicked() {
                                    self.refresh_adb();
                                }
                            });
                        });
                    });
            } else {
                ui.horizontal(|ui| {
                    let dot_color = if self.adb_active { green } else { text_dim };
                    paint_status_dot(ui, dot_color, 3.5);
                    ui.colored_label(dot_color, egui::RichText::new(&self.adb_status_msg).size(12.0).strong());
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Configure USB (adb reverse)").strong().size(11.5)).fill(pink)).clicked() {
                        self.configure_adb();
                    }
                    if ui.add(egui::Button::new(egui::RichText::new("Refresh USB").size(11.5)).fill(egui::Color32::from_rgb(0x22, 0x22, 0x36))).clicked() {
                        self.refresh_adb();
                    }
                });
            }
        });
    }

    fn render_overlay_tab(
        &mut self,
        ui: &mut egui::Ui,
        pink: egui::Color32,
        cyan: egui::Color32,
        _green: egui::Color32,
        card_bg: egui::Color32,
        border_color: egui::Color32,
        text_dim: egui::Color32,
    ) {
        // --- 1. METRICS DASHBOARD ---
        let (stroke_count, can_undo, can_redo) = if let Ok(r) = self.renderer.lock() {
            (r.stroke_count(), r.can_undo(), r.can_redo())
        } else {
            (0, false, false)
        };

        egui::Frame::canvas(ui.style())
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.0, border_color))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.columns(3, |columns| {
                    columns[0].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("CANVAS SIZE").size(10.0).color(text_dim));
                        ui.label(egui::RichText::new(format!("{}x{}", self.screen_size.0, self.screen_size.1)).size(14.0).strong().color(egui::Color32::WHITE));
                    });
                    columns[1].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("ACTIVE STROKES").size(10.0).color(text_dim));
                        ui.label(egui::RichText::new(format!("{}", stroke_count)).size(14.0).strong().color(pink));
                    });
                    columns[2].vertical_centered(|ui| {
                        ui.label(egui::RichText::new("HISTORY STACK").size(10.0).color(text_dim));
                        let hist_str = format!("Undo:{} Redo:{}", if can_undo { "Yes" } else { "No" }, if can_redo { "Yes" } else { "No" });
                        ui.label(egui::RichText::new(hist_str).size(14.0).strong().color(cyan));
                    });
                });
            });

        ui.add_space(10.0);

        // --- 2. OVERLAY CONTROLS ---
        render_card(ui, "TRANSPARENT OVERLAY BEHAVIOR", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Desktop Overlay Active:").color(text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut active = self.overlay_active;
                    if ui.checkbox(&mut active, "").changed() {
                        self.overlay_active = active;
                        let _ = self.redraw_tx.send(());
                    }
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Click-Through Mode (pass clicks to desktop):").color(text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.clickthrough_active, "");
                });
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Auto-Clear Fade Timer:").color(text_dim).size(11.5));
            ui.add_space(4.0);

            // Clean Segmented Duration Pills
            let durations: [(u64, &str); 6] = [
                (0, "Never (Off)"),
                (3, "3s"),
                (5, "5s"),
                (10, "10s"),
                (30, "30s"),
                (60, "60s"),
            ];

            ui.horizontal_wrapped(|ui| {
                for (sec, label) in durations {
                    let is_sel = self.fade_duration_secs == sec;
                    let fill = if is_sel { pink } else { egui::Color32::from_rgb(0x1F, 0x20, 0x30) };
                    let text_color = if is_sel { egui::Color32::WHITE } else { text_dim };

                    if ui.add(egui::Button::new(egui::RichText::new(label).color(text_color).strong().size(11.5)).fill(fill)).clicked() {
                        self.fade_duration_secs = sec;
                        if let Ok(mut r) = self.renderer.lock() {
                            r.set_fade_duration(sec);
                        }
                        crate::logging::log_activity("TIMER", (0xFF, 0x91, 0x00), format!("Fade timer set to {}s", sec));
                        self.status_notification = Some((
                            if sec == 0 { "Annotations will persist indefinitely".to_string() } else { format!("Auto-fade set to {} seconds", sec) },
                            std::time::Instant::now(),
                        ));
                    }
                }
            });
        });

        ui.add_space(10.0);

        // --- 3. CANVAS ACTIONS ---
        render_card(ui, "CANVAS ACTIONS", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                if ui.add_sized([120.0, 36.0], egui::Button::new(egui::RichText::new("Clear All").strong().size(12.5)).fill(pink)).clicked() {
                    self.clear_canvas();
                }
                if ui.add_sized([90.0, 36.0], egui::Button::new(egui::RichText::new("Undo").strong().size(12.5)).fill(egui::Color32::from_rgb(0x22, 0x24, 0x38))).clicked() {
                    self.undo();
                }
                if ui.add_sized([90.0, 36.0], egui::Button::new(egui::RichText::new("Redo").strong().size(12.5)).fill(egui::Color32::from_rgb(0x22, 0x24, 0x38))).clicked() {
                    self.redo();
                }
            });
        });
    }

    fn render_logs_tab(
        &mut self,
        ui: &mut egui::Ui,
        _pink: egui::Color32,
        cyan: egui::Color32,
        green: egui::Color32,
        card_bg: egui::Color32,
        border_color: egui::Color32,
        text_dim: egui::Color32,
    ) {
        render_card(ui, "LIVE ACTIVITY & WEBSOCKET LOGS", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                paint_status_dot(ui, green, 3.5);
                ui.colored_label(green, egui::RichText::new("LIVE CONSOLE").strong().size(11.0));
                ui.label(egui::RichText::new(format!("ws://{}:{}", self.lan_ip, self.port)).size(11.0).color(text_dim));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Clear Logs").size(11.0)).fill(egui::Color32::from_rgb(0x28, 0x1A, 0x22))).clicked() {
                        crate::logging::clear_activity_logs();
                    }
                    if ui.add(egui::Button::new(egui::RichText::new("Copy Logs").size(11.0)).fill(egui::Color32::from_rgb(0x22, 0x22, 0x36))).clicked() {
                        let logs = crate::logging::get_activity_logs();
                        let text = logs.iter().map(|l| format!("[{}] [{}] {}", l.timestamp, l.tag, l.message)).collect::<Vec<_>>().join("\n");
                        ui.ctx().copy_text(text);
                        self.status_notification = Some(("Copied activity logs to clipboard!".to_string(), std::time::Instant::now()));
                    }
                });
            });

            ui.add_space(8.0);

            egui::Frame::canvas(ui.style())
                .fill(egui::Color32::from_rgb(0x09, 0x0A, 0x12))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x1E, 0x20, 0x30)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    let logs = crate::logging::get_activity_logs();
                    if logs.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("No activity yet. Connect phone to view live log events.").color(text_dim).size(11.5));
                            ui.add_space(20.0);
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(320.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for entry in logs {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("[{}]", entry.timestamp)).monospace().size(10.5).color(text_dim));
                                        let tag_color = egui::Color32::from_rgb(entry.color_rgb.0, entry.color_rgb.1, entry.color_rgb.2);
                                        egui::Frame::canvas(ui.style())
                                            .fill(egui::Color32::from_rgba_unmultiplied(entry.color_rgb.0, entry.color_rgb.1, entry.color_rgb.2, 35))
                                            .stroke(egui::Stroke::new(1.0, tag_color))
                                            .corner_radius(egui::CornerRadius::same(4))
                                            .inner_margin(egui::Margin::symmetric(4, 1))
                                            .show(ui, |ui| {
                                                ui.colored_label(tag_color, egui::RichText::new(&entry.tag).monospace().size(9.5).strong());
                                            });
                                        ui.label(egui::RichText::new(&entry.message).size(11.0).color(egui::Color32::from_rgb(0xEE, 0xEE, 0xF6)));
                                    });
                                }
                            });
                    }
                });
        });
    }

    fn render_settings_tab(
        &mut self,
        ui: &mut egui::Ui,
        cyan: egui::Color32,
        green: egui::Color32,
        _orange: egui::Color32,
        card_bg: egui::Color32,
        border_color: egui::Color32,
        text_dim: egui::Color32,
    ) {
        // --- 1. SYSTEM & PLATFORM ---
        render_card(ui, "PLATFORM & HARDWARE ENGINE", cyan, card_bg, border_color, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Screen Resolution:").color(text_dim));
                ui.label(egui::RichText::new(format!("{} x {}", self.screen_size.0, self.screen_size.1)).strong());
            });
            ui.horizontal(|ui| {
                #[cfg(target_os = "linux")]
                let platform_name = "Linux (X11 XFixes & XShape Truecolor 32-bit)";
                #[cfg(target_os = "windows")]
                let platform_name = "Windows (Win32 Layered Transparency & SendInput)";
                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                let platform_name = "Generic Desktop";

                ui.label(egui::RichText::new("Input Subsystem:").color(text_dim));
                ui.label(egui::RichText::new(platform_name).strong());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("WebSocket Server:").color(text_dim));
                ui.colored_label(green, egui::RichText::new(format!("Listening on 0.0.0.0:{}", self.port)).strong());
            });
        });

        ui.add_space(10.0);

        // --- 2. ABOUT & CREDITS ---
        render_card(ui, "ABOUT CLINK", cyan, card_bg, border_color, |ui| {
            ui.label(egui::RichText::new("Clink converts your Android phone or tablet into an ultra-low-latency drawing tablet and live desktop annotation overlay.").color(text_dim).size(11.5));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Maintainer:").color(text_dim));
                ui.label(egui::RichText::new("chriz-3656").strong());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("License:").color(text_dim));
                ui.label(egui::RichText::new("MIT License").strong());
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(egui::RichText::new("Open GitHub Repository").size(11.5)).fill(egui::Color32::from_rgb(0x22, 0x24, 0x38))).clicked() {
                    ui.ctx().copy_text("https://github.com/chriz-3656/clink".to_string());
                    self.status_notification = Some(("Copied GitHub URL to clipboard!".to_string(), std::time::Instant::now()));
                }
            });
        });
    }
}

fn paint_status_dot(ui: &mut egui::Ui, color: egui::Color32, radius: f32) {
    let size = egui::vec2(radius * 2.0 + 4.0, radius * 2.0 + 4.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let center = rect.center();
    let halo_color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 45);
    ui.painter().circle_filled(center, radius + 2.0, halo_color);
    ui.painter().circle_filled(center, radius, color);
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
        .inner_margin(egui::Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, 14.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.5, accent);
                ui.colored_label(accent, egui::RichText::new(title).size(11.5).strong());
            });
            ui.add_space(8.0);
            content(ui)
        })
        .inner
}
