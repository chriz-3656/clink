//! Input injection abstraction.

pub trait InputBackend: Send + Sync {
    fn get_screen_size(&self) -> (u32, u32);
    fn move_absolute(&self, norm_x: f32, norm_y: f32, pressure: f32);
    fn move_relative(&self, dx: f32, dy: f32, pressure: f32);
    fn button_down(&self, button: u32, pressure: f32);
    fn button_up(&self, button: u32);
    fn cleanup(&self);
}

pub mod dummy;

#[cfg(target_os = "linux")]
pub mod x11;
