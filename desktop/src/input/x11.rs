//! Linux X11 XTest input injector using pure Rust x11rb.

use super::InputBackend;
use std::sync::Mutex;
use tracing::{debug, info};
use x11rb::connection::Connection;
use x11rb::protocol::xtest;
use x11rb::rust_connection::RustConnection;

pub struct X11InputBackend {
    conn: Mutex<Option<RustConnection>>,
    #[allow(dead_code)]
    screen_num: usize,
    width: u32,
    height: u32,
    pressed_buttons: Mutex<Vec<u32>>,
    accum_x: Mutex<f32>,
    accum_y: Mutex<f32>,
}

impl X11InputBackend {
    pub fn new() -> Option<Self> {
        let (conn, screen_num) = match RustConnection::connect(None) {
            Ok(c) => c,
            Err(e) => {
                debug!("Could not connect to X11 display: {}", e);
                return None;
            }
        };

        let setup = conn.setup();
        let screen = match setup.roots.get(screen_num) {
            Some(s) => s,
            None => return None,
        };

        let width = screen.width_in_pixels as u32;
        let height = screen.height_in_pixels as u32;

        info!("X11 display connected: {}x{} on screen {}", width, height, screen_num);

        Some(Self {
            conn: Mutex::new(Some(conn)),
            screen_num,
            width,
            height,
            pressed_buttons: Mutex::new(Vec::new()),
            accum_x: Mutex::new(0.0),
            accum_y: Mutex::new(0.0),
        })
    }
}

impl InputBackend for X11InputBackend {
    fn get_screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn move_absolute(&self, norm_x: f32, norm_y: f32, _pressure: f32) {
        let target_x = (norm_x.clamp(0.0, 1.0) * self.width as f32) as i16;
        let target_y = (norm_y.clamp(0.0, 1.0) * self.height as f32) as i16;

        if let Ok(guard) = self.conn.lock() {
            if let Some(conn) = guard.as_ref() {
                let setup = conn.setup();
                if let Some(screen) = setup.roots.get(self.screen_num) {
                    let root = screen.root;
                    // MotionNotify event type = 6, detail = 0 (absolute motion)
                    let _ = xtest::fake_input(conn, 6, 0, 0, root, target_x, target_y, 0);
                    let _ = conn.flush();
                }
            }
        }
    }

    fn move_relative(&self, dx: f32, dy: f32, _pressure: f32) {
        if let Ok(guard) = self.conn.lock() {
            if let Some(conn) = guard.as_ref() {
                let (step_x, step_y) = {
                    let mut ax = self.accum_x.lock().unwrap_or_else(|e| e.into_inner());
                    let mut ay = self.accum_y.lock().unwrap_or_else(|e| e.into_inner());
                    let total_x = *ax + dx;
                    let total_y = *ay + dy;
                    let sx = total_x.trunc() as i16;
                    let sy = total_y.trunc() as i16;
                    *ax = total_x.fract();
                    *ay = total_y.fract();
                    (sx, sy)
                };

                if step_x != 0 || step_y != 0 {
                    // MotionNotify event type = 6, detail = 1 (relative motion), root = 0
                    let _ = xtest::fake_input(conn, 6, 1, 0, 0, step_x, step_y, 0);
                    let _ = conn.flush();
                }
            }
        }
    }

    fn button_down(&self, button: u32, _pressure: f32) {
        if let Ok(guard) = self.conn.lock() {
            if let Some(conn) = guard.as_ref() {
                // ButtonPress event type = 4
                let _ = xtest::fake_input(conn, 4, button as u8, 0, 0, 0, 0, 0);
                let _ = conn.flush();
            }
        }
        if let Ok(mut pressed) = self.pressed_buttons.lock() {
            if !pressed.contains(&button) {
                pressed.push(button);
            }
        }
    }

    fn button_up(&self, button: u32) {
        if let Ok(guard) = self.conn.lock() {
            if let Some(conn) = guard.as_ref() {
                // ButtonRelease event type = 5
                let _ = xtest::fake_input(conn, 5, button as u8, 0, 0, 0, 0, 0);
                let _ = conn.flush();
            }
        }
        if let Ok(mut pressed) = self.pressed_buttons.lock() {
            pressed.retain(|&b| b != button);
        }
    }

    fn cleanup(&self) {
        let buttons_to_release: Vec<u32> = {
            if let Ok(mut pressed) = self.pressed_buttons.lock() {
                pressed.drain(..).collect()
            } else {
                Vec::new()
            }
        };

        if let Ok(guard) = self.conn.lock() {
            if let Some(conn) = guard.as_ref() {
                for btn in buttons_to_release {
                    let _ = xtest::fake_input(conn, 5, btn as u8, 0, 0, 0, 0, 0);
                }
                let _ = conn.flush();
            }
        }
    }
}
