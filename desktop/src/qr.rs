//! Pure Rust QR code generator.

use qrcode::render::unicode;
use qrcode::QrCode;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_pairing_uri(ip: &str, port: u16, pin: &str) -> String {
    format!("clink://connect?host={}&port={}&pin={}", ip, port, pin)
}

pub fn render_terminal_qr(uri: &str) -> String {
    match QrCode::new(uri.as_bytes()) {
        Ok(code) => code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build(),
        Err(_) => String::new(),
    }
}

pub fn save_qr_png(uri: &str, output_path: &Path) -> Result<PathBuf, String> {
    let code = QrCode::new(uri.as_bytes()).map_err(|e| e.to_string())?;
    
    // Render to image buffer (black on white)
    let image = code.render::<image::Luma<u8>>().min_dimensions(200, 200).build();
    
    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    image
        .save(output_path)
        .map_err(|e| e.to_string())?;
        
    Ok(output_path.to_path_buf())
}
