//! Dummy/mock input backend for tests and headless execution.

use super::InputBackend;
use tracing::debug;

pub struct DummyInputBackend {
    width: u32,
    height: u32,
}

impl DummyInputBackend {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for DummyInputBackend {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

impl InputBackend for DummyInputBackend {
    fn get_screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn move_absolute(&self, norm_x: f32, norm_y: f32, pressure: f32) {
        debug!("Dummy move_absolute: ({:.3}, {:.3}), p={:.2}", norm_x, norm_y, pressure);
    }

    fn move_relative(&self, dx: f32, dy: f32, pressure: f32) {
        debug!("Dummy move_relative: ({:.1}, {:.1}), p={:.2}", dx, dy, pressure);
    }

    fn button_down(&self, button: u32, pressure: f32) {
        debug!("Dummy button_down: {}, p={:.2}", button, pressure);
    }

    fn button_up(&self, button: u32) {
        debug!("Dummy button_up: {}", button);
    }

    fn cleanup(&self) {
        debug!("Dummy cleanup");
    }
}
