//! Transparent Linux X11 click-through overlay window using x11rb.

use super::OverlayRenderer;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use x11rb::connection::Connection;
use x11rb::protocol::shape::SK;
use x11rb::protocol::xfixes;
use x11rb::protocol::xproto::{
    ColormapAlloc, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, Gcontext,
    ImageFormat, VisualClass, Window, WindowClass,
};
use x11rb::rust_connection::RustConnection;

pub struct X11OverlayWindow {
    conn: Arc<RustConnection>,
    pub window: Window,
    pub gc: Gcontext,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    renderer: Arc<Mutex<OverlayRenderer>>,
}

impl X11OverlayWindow {
    pub fn new(renderer: Arc<Mutex<OverlayRenderer>>) -> Option<Self> {
        let (conn, screen_num) = match RustConnection::connect(None) {
            Ok(c) => c,
            Err(e) => {
                debug!("X11 not available for overlay window: {}", e);
                return None;
            }
        };

        let conn = Arc::new(conn);
        let setup = conn.setup();
        let screen = match setup.roots.get(screen_num) {
            Some(s) => s,
            None => return None,
        };

        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;

        // Find 32-bit ARGB visual for true alpha channel transparency
        let mut visual_id = screen.root_visual;
        let mut depth = screen.root_depth;

        for depth_type in &screen.allowed_depths {
            if depth_type.depth == 32 {
                for vis in &depth_type.visuals {
                    if vis.class == VisualClass::TRUE_COLOR {
                        visual_id = vis.visual_id;
                        depth = 32;
                        break;
                    }
                }
            }
        }

        let colormap = match conn.generate_id() {
            Ok(id) => id,
            Err(_) => return None,
        };

        if depth == 32 {
            let _ = conn.create_colormap(ColormapAlloc::NONE, colormap, screen.root, visual_id);
        }

        let window = match conn.generate_id() {
            Ok(id) => id,
            Err(_) => return None,
        };

        let win_aux = CreateWindowAux::new()
            .override_redirect(1) // Override window manager borders and minimization
            .background_pixel(0)
            .border_pixel(0)
            .colormap(if depth == 32 { colormap } else { 0 })
            .event_mask(EventMask::EXPOSURE);

        let _ = conn.create_window(
            depth,
            window,
            screen.root,
            0,
            0,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            visual_id,
            &win_aux,
        );

        // Apply XFixes empty input shape so all mouse clicks pass through
        if let Ok(_) = xfixes::query_version(&*conn, 5, 0) {
            if let Ok(empty_region) = conn.generate_id() {
                let _ = xfixes::create_region(&*conn, empty_region, &[]);
                let _ = xfixes::set_window_shape_region(
                    &*conn,
                    window,
                    SK::INPUT,
                    0,
                    0,
                    empty_region,
                );
                let _ = xfixes::destroy_region(&*conn, empty_region);
            }
        }

        let gc = match conn.generate_id() {
            Ok(id) => id,
            Err(_) => return None,
        };
        let gc_aux = CreateGCAux::new();
        let _ = conn.create_gc(gc, window, &gc_aux);

        let _ = conn.map_window(window);
        let _ = conn.flush();

        info!("Native X11 transparent overlay initialized: {}x{} (depth: {})", width, height, depth);

        Some(Self {
            conn,
            window,
            gc,
            width,
            height,
            depth,
            renderer,
        })
    }

    pub fn redraw(&self) {
        let guard = match self.renderer.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let pixmap = guard.get_pixmap();
        let data = pixmap.data();

        // Convert RGBA -> BGRA for X11 ZPixmap
        let mut bgra = Vec::with_capacity(data.len());
        for chunk in data.chunks_exact(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            bgra.push(b);
            bgra.push(g);
            bgra.push(r);
            bgra.push(a);
        }

        let _ = self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            self.width,
            self.height,
            0,
            0,
            0,
            self.depth,
            &bgra,
        );
        let _ = self.conn.flush();
    }
}
