//! Desktop overlay management.

pub mod renderer;

#[cfg(target_os = "linux")]
pub mod x11_window;

#[allow(unused_imports)]
pub use renderer::OverlayRenderer;
