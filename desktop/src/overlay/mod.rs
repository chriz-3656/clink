//! Desktop overlay management.

pub mod renderer;

#[cfg(target_os = "linux")]
pub mod x11_window;

pub use renderer::OverlayRenderer;
