// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{CssColor, Value};
use crate::engine::layout::{BoxType, LayoutBox, Rect};

/// A software-rendered pixel buffer. Each pixel is RGBA (not pre-multiplied).
#[derive(Debug, Clone)]
pub struct Canvas {
    pub pixels: Vec<CssColor>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    /// Create a white-filled canvas.
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            pixels: vec![CssColor::WHITE; width * height],
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: CssColor) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if color.a == 255 {
                self.pixels[idx] = color;
            } else {
                let alpha = color.a as f32 / 255.0;
                let bg = self.pixels[idx];
                self.pixels[idx] = CssColor {
                    r: (color.r as f32 * alpha + bg.r as f32 * (1.0 - alpha)) as u8,
                    g: (color.g as f32 * alpha + bg.g as f32 * (1.0 - alpha)) as u8,
                    b: (color.b as f32 * alpha + bg.b as f32 * (1.0 - alpha)) as u8,
                    a: 255,
                };
            }
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: CssColor) {
        let x0 = rect.x.max(0.0) as usize;
        let y0 = rect.y.max(0.0) as usize;
        let x1 = ((rect.x + rect.width) as usize).min(self.width);
        let y1 = ((rect.y + rect.height) as usize).min(self.height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.set_pixel(x, y, color);
            }
        }
    }
}

/// Convert a Canvas pixel buffer to Cairo ARGB32 byte order (BGRA on little-endian).
pub fn canvas_to_bgra(canvas: &Canvas) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(canvas.width * canvas.height * 4);
    for pixel in &canvas.pixels {
        bytes.push(pixel.b);
        bytes.push(pixel.g);
        bytes.push(pixel.r);
        bytes.push(pixel.a);
    }
    bytes
}

/// Walks the layout tree and paints every box onto a Canvas.
pub struct Painter;

impl Painter {
    pub fn paint(layout_root: &LayoutBox, viewport: Rect) -> Canvas {
        let width = viewport.width.ceil() as usize;
        let height = viewport.height.ceil() as usize;
        let mut canvas = Canvas::new(width.max(1), height.max(1));
        render_box(&mut canvas, layout_root, 0);
        canvas
    }
}

fn render_box(canvas: &mut Canvas, lb: &LayoutBox, depth: usize) {
    // Depth cap: prevent stack overflow on pathological layout tree (Fix 1).
    if depth >= 512 { return; }
    render_background(canvas, lb);
    render_borders(canvas, lb);
    render_text(canvas, lb);
    for child in &lb.children {
        render_box(canvas, child, depth + 1);
    }
}

fn render_background(canvas: &mut Canvas, lb: &LayoutBox) {
    let color = get_color(lb, "background-color").unwrap_or(CssColor::TRANSPARENT);
    if color.a == 0 { return; }
    canvas.fill_rect(lb.dimensions.border_box(), color);
}

fn render_borders(canvas: &mut Canvas, lb: &LayoutBox) {
    let color = match get_color(lb, "border-color") {
        Some(c) => c,
        None => return,
    };
    let d = &lb.dimensions;
    let bb = d.border_box();
    canvas.fill_rect(Rect { x: bb.x, y: bb.y, width: bb.width, height: d.border.top }, color);
    canvas.fill_rect(Rect { x: bb.x, y: bb.y + bb.height - d.border.bottom, width: bb.width, height: d.border.bottom }, color);
    canvas.fill_rect(Rect { x: bb.x, y: bb.y, width: d.border.left, height: bb.height }, color);
    canvas.fill_rect(Rect { x: bb.x + bb.width - d.border.right, y: bb.y, width: d.border.right, height: bb.height }, color);
}

fn render_text(canvas: &mut Canvas, lb: &LayoutBox) {
    let text = match &lb.text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };
    let font_color = CssColor { r: 30, g: 30, b: 30, a: 255 };
    let rect = Rect {
        x: lb.dimensions.content.x,
        y: lb.dimensions.content.y,
        width: (text.len() as f32 * 7.0).min(lb.dimensions.content.width),
        height: 14.0,
    };
    canvas.fill_rect(rect, font_color);
}

fn get_color(lb: &LayoutBox, prop: &str) -> Option<CssColor> {
    let styled = match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) => s,
        BoxType::Anonymous => return None,
    };
    match styled.properties.get(prop) {
        Some(Value::Color(c)) => Some(*c),
        _ => None,
    }
}
