//! Software 2D stroke rasterizer powered by tiny-skia.

use crate::protocol::{Point, Stroke};
use std::collections::HashMap;
use std::time::Instant;
use tiny_skia::{
    BlendMode, Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke as SkiaStroke,
    Transform,
};

#[derive(Debug, Clone)]
pub struct TimedStroke {
    pub stroke: Stroke,
    pub created_at: Instant,
}

pub struct OverlayRenderer {
    width: u32,
    height: u32,
    pub strokes: Vec<TimedStroke>,
    pub undo_stack: Vec<Vec<TimedStroke>>,
    pub redo_stack: Vec<Vec<TimedStroke>>,
    pub active_strokes: HashMap<String, Stroke>,
    pub laser_pos: Option<(f32, f32)>,
    pub laser_active: bool,
    pub fade_timeout: f32, // 0.0 = persistent
    pixmap: Pixmap,
}

impl OverlayRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let pixmap = Pixmap::new(width, height).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());

        Self {
            width,
            height,
            strokes: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_strokes: HashMap::new(),
            laser_pos: None,
            laser_active: false,
            fade_timeout: 0.0,
            pixmap,
        }
    }

    #[allow(dead_code)]
    pub fn get_pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn stroke_count(&self) -> usize {
        self.strokes.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.strokes.is_empty() || !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn set_fade_duration(&mut self, secs: u64) {
        self.fade_timeout = secs as f32;
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width.max(1);
        self.height = height.max(1);
        self.pixmap = Pixmap::new(self.width, self.height).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());
        self.render_all();
    }

    pub fn start_stroke(&mut self, stroke: Stroke) {
        self.redo_stack.clear();
        self.active_strokes.insert(stroke.stroke_id.clone(), stroke);
        self.render_all();
    }

    pub fn append_points(&mut self, stroke_id: &str, points: Vec<Point>) {
        if let Some(stroke) = self.active_strokes.get_mut(stroke_id) {
            stroke.points.extend(points);
            self.render_all();
        }
    }

    pub fn end_stroke(&mut self, stroke_id: &str, final_point: Option<Point>) -> Option<Stroke> {
        let mut stroke = self.active_strokes.remove(stroke_id)?;
        if let Some(pt) = final_point {
            stroke.points.push(pt);
        }
        stroke.is_completed = true;

        self.undo_stack.push(self.strokes.clone());
        self.redo_stack.clear();

        self.strokes.push(TimedStroke {
            stroke: stroke.clone(),
            created_at: Instant::now(),
        });

        self.render_all();
        Some(stroke)
    }

    pub fn undo(&mut self) -> bool {
        if self.strokes.is_empty() && self.undo_stack.is_empty() {
            return false;
        }

        self.redo_stack.push(self.strokes.clone());
        if let Some(prev) = self.undo_stack.pop() {
            self.strokes = prev;
        } else {
            self.strokes.pop();
        }

        self.render_all();
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.redo_stack.is_empty() {
            return false;
        }

        self.undo_stack.push(self.strokes.clone());
        if let Some(next) = self.redo_stack.pop() {
            self.strokes = next;
        }

        self.render_all();
        true
    }

    pub fn clear(&mut self) {
        if self.strokes.is_empty() && self.active_strokes.is_empty() {
            return;
        }

        self.undo_stack.push(self.strokes.clone());
        self.redo_stack.clear();
        self.strokes.clear();
        self.active_strokes.clear();
        self.pixmap.fill(Color::TRANSPARENT);
    }

    pub fn set_laser(&mut self, x: f32, y: f32, active: bool) {
        self.laser_pos = Some((x, y));
        self.laser_active = active;
        self.render_all();
    }

    pub fn tick_fade(&mut self) -> bool {
        if self.fade_timeout <= 0.0 || self.strokes.is_empty() {
            return false;
        }

        let now = Instant::now();
        let timeout_secs = self.fade_timeout;
        let initial_len = self.strokes.len();

        self.strokes
            .retain(|s| now.duration_since(s.created_at).as_secs_f32() < timeout_secs);

        if self.strokes.len() != initial_len {
            self.render_all();
            true
        } else {
            false
        }
    }

    pub fn render_all(&mut self) {
        self.pixmap.fill(Color::TRANSPARENT);

        // 1. Render completed strokes
        for timed_stroke in &self.strokes {
            Self::draw_stroke_to_pixmap(&mut self.pixmap, self.width, self.height, &timed_stroke.stroke);
        }

        // 2. Render active in-progress strokes
        for stroke in self.active_strokes.values() {
            Self::draw_stroke_to_pixmap(&mut self.pixmap, self.width, self.height, stroke);
        }

        // 3. Render laser pointer
        if self.laser_active {
            if let Some((lx, ly)) = self.laser_pos {
                let px = lx * self.width as f32;
                let py = ly * self.height as f32;

                let mut paint = Paint::default();
                paint.set_color_rgba8(255, 30, 80, 220);
                paint.anti_alias = true;

                let mut pb = PathBuilder::new();
                pb.push_circle(px, py, 9.0);
                if let Some(path) = pb.finish() {
                    self.pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
        }
    }

    fn draw_stroke_to_pixmap(pixmap: &mut Pixmap, canvas_w: u32, canvas_h: u32, stroke: &Stroke) {
        if stroke.points.is_empty() {
            return;
        }

        let mut paint = Paint::default();
        paint.anti_alias = true;

        let parsed_color = parse_hex_color(&stroke.color);
        let alpha = match stroke.tool.as_str() {
            "highlighter" => 0.35,
            "eraser" => {
                paint.blend_mode = BlendMode::Clear;
                1.0
            }
            _ => 1.0,
        };

        paint.set_color(Color::from_rgba(
            parsed_color.0,
            parsed_color.1,
            parsed_color.2,
            parsed_color.3 * alpha,
        ).unwrap_or(Color::BLACK));

        let width = (stroke.width * (canvas_w as f32 / 1920.0)).max(1.5);
        let skia_stroke = SkiaStroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };

        let mut pb = PathBuilder::new();

        match stroke.tool.as_str() {
            "line" if stroke.points.len() >= 2 => {
                let p0 = &stroke.points[0];
                let p1 = &stroke.points[stroke.points.len() - 1];
                pb.move_to(p0.x * canvas_w as f32, p0.y * canvas_h as f32);
                pb.line_to(p1.x * canvas_w as f32, p1.y * canvas_h as f32);
            }
            "rectangle" if stroke.points.len() >= 2 => {
                let p0 = &stroke.points[0];
                let p1 = &stroke.points[stroke.points.len() - 1];
                let x0 = p0.x * canvas_w as f32;
                let y0 = p0.y * canvas_h as f32;
                let x1 = p1.x * canvas_w as f32;
                let y1 = p1.y * canvas_h as f32;

                let rx = x0.min(x1);
                let ry = y0.min(y1);
                let rw = (x1 - x0).abs();
                let rh = (y1 - y0).abs();

                pb.move_to(rx, ry);
                pb.line_to(rx + rw, ry);
                pb.line_to(rx + rw, ry + rh);
                pb.line_to(rx, ry + rh);
                pb.close();
            }
            "circle" if stroke.points.len() >= 2 => {
                let p0 = &stroke.points[0];
                let p1 = &stroke.points[stroke.points.len() - 1];
                let x0 = p0.x * canvas_w as f32;
                let y0 = p0.y * canvas_h as f32;
                let x1 = p1.x * canvas_w as f32;
                let y1 = p1.y * canvas_h as f32;

                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = (((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() / 2.0).max(1.0);

                pb.push_circle(cx, cy, radius);
            }
            _ => {
                // Freehand pen/pencil/highlighter
                let first = &stroke.points[0];
                let fx = first.x * canvas_w as f32;
                let fy = first.y * canvas_h as f32;
                pb.move_to(fx, fy);

                if stroke.points.len() == 1 {
                    pb.line_to(fx + 0.1, fy + 0.1);
                } else {
                    for pt in &stroke.points[1..] {
                        pb.line_to(pt.x * canvas_w as f32, pt.y * canvas_h as f32);
                    }
                }
            }
        }

        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &skia_stroke, Transform::identity(), None);
        }
    }
}

fn parse_hex_color(hex: &str) -> (f32, f32, f32, f32) {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
        }
    } else if s.len() == 8 {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
            u8::from_str_radix(&s[6..8], 16),
        ) {
            return (
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            );
        }
    }
    // Default pink #FF3366
    (1.0, 0.2, 0.4, 1.0)
}
