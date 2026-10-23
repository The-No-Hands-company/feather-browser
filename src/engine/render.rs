// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{CssColor, Value};
#[cfg(feature = "feather-renderer")]
use crate::engine::dom::NodeType;
#[cfg(feature = "feather-renderer")]
use crate::engine::font::{FontMetrics, GlyphBitmap};
use crate::engine::layout::{BoxType, LayoutBox, Rect, TextDecoration};

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
    pub fn paint(layout_root: &LayoutBox, viewport: Rect, font_size: f32) -> Canvas {
        let width = viewport.width.ceil() as usize;
        let height = viewport.height.ceil() as usize;
        let mut canvas = Canvas::new(width.max(1), height.max(1));
        #[cfg(feature = "feather-renderer")]
        {
            let font = crate::engine::font::FontMetrics::load_cached(font_size);
            render_box(&mut canvas, layout_root, &font, viewport.y, 0, 1.0);
        }
        #[cfg(not(feature = "feather-renderer"))]
        {
            let _ = font_size;
            render_box_plain(&mut canvas, layout_root, 0);
        }
        canvas
    }
}

#[cfg(feature = "feather-renderer")]
fn with_opacity(c: CssColor, opacity: f32) -> CssColor {
    CssColor { a: (c.a as f32 * opacity).round() as u8, ..c }
}

#[cfg(feature = "feather-renderer")]
fn get_box_opacity(lb: &LayoutBox) -> f32 {
    match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) => match s.properties.get("opacity") {
            Some(Value::Number(n)) => n.clamp(0.0, 1.0),
            _ => 1.0,
        },
        BoxType::Anonymous => 1.0,
    }
}

#[cfg(feature = "feather-renderer")]
fn render_box(
    canvas: &mut Canvas,
    lb: &LayoutBox,
    font: &FontMetrics,
    scroll_y: f32,
    depth: usize,
    opacity: f32,
) {
    if depth >= 512 {
        return;
    }
    let eff_opacity = (opacity * get_box_opacity(lb)).clamp(0.0, 1.0);
    render_background_scrolled(canvas, lb, scroll_y, eff_opacity);
    render_borders_scrolled(canvas, lb, scroll_y, eff_opacity);
    // img placeholder: paint grey box + alt text, skip normal text render.
    if is_img_element(lb) {
        render_img_placeholder(canvas, lb, scroll_y, font);
        for child in &lb.children {
            render_box(canvas, child, font, scroll_y, depth + 1, eff_opacity);
        }
        return;
    }
    if !lb.line_boxes.is_empty() {
        let origin_x = lb.dimensions.content.x;
        let origin_y = lb.dimensions.content.y - scroll_y;
        for line in &lb.line_boxes {
            let baseline_y = (origin_y + line.y + line.baseline) as i32;
            let line_top_y = (origin_y + line.y) as i32;
            let line_bottom_y = (origin_y + line.y + line.height) as i32;
            if line_bottom_y < 0 || line_top_y > canvas.height as i32 {
                continue;
            }
            for item in &line.items {
                let item_font = crate::engine::font::FontMetrics::load_cached_variant(
                    item.font_size,
                    item.font_bold,
                    item.font_italic,
                );
                let mut pen_x_f: f32 = origin_x + item.x;
                let text_color = with_opacity(item.color, eff_opacity);
                for ch in item.text.chars() {
                    let glyph = item_font.rasterize_cached(ch);
                    blit_glyph(
                        canvas,
                        &glyph,
                        pen_x_f.round() as i32,
                        baseline_y,
                        text_color,
                    );
                    pen_x_f += glyph.advance_width;
                }
                // Draw text decoration after glyphs so it sits on top.
                let deco_x = (origin_x + item.x) as i32;
                let deco_w = item.width.ceil() as i32;
                match item.text_decoration {
                    TextDecoration::Underline => {
                        let y = baseline_y + (item_font.descent * 0.25).ceil() as i32 + 1;
                        for dx in 0..deco_w {
                            canvas.set_pixel(
                                (deco_x + dx).max(0) as usize,
                                y.max(0) as usize,
                                text_color,
                            );
                        }
                    }
                    TextDecoration::LineThrough => {
                        let y = baseline_y - (item_font.ascent * 0.3).ceil() as i32;
                        for dx in 0..deco_w {
                            canvas.set_pixel(
                                (deco_x + dx).max(0) as usize,
                                y.max(0) as usize,
                                text_color,
                            );
                        }
                    }
                    TextDecoration::None => {}
                }
            }
        }
    } else {
        render_text_scrolled(canvas, lb, scroll_y);
    }
    let overflow_hidden = has_overflow_hidden(lb);
    for child in &lb.children {
        if overflow_hidden {
            let pb = lb.dimensions.border_box();
            let cb = child.dimensions.border_box();
            let outside = cb.x + cb.width <= pb.x
                || cb.x >= pb.x + pb.width
                || cb.y + cb.height <= pb.y
                || cb.y >= pb.y + pb.height;
            if outside { continue; }
        }
        render_box(canvas, child, font, scroll_y, depth + 1, eff_opacity);
    }
}

#[cfg(not(feature = "feather-renderer"))]
fn render_box_plain(canvas: &mut Canvas, lb: &LayoutBox, depth: usize) {
    if depth >= 512 {
        return;
    }
    render_background(canvas, lb);
    render_borders(canvas, lb);
    render_text(canvas, lb);
    let overflow_hidden = has_overflow_hidden(lb);
    for child in &lb.children {
        if overflow_hidden {
            let pb = lb.dimensions.border_box();
            let cb = child.dimensions.border_box();
            let outside = cb.x + cb.width <= pb.x
                || cb.x >= pb.x + pb.width
                || cb.y + cb.height <= pb.y
                || cb.y >= pb.y + pb.height;
            if outside { continue; }
        }
        render_box_plain(canvas, child, depth + 1);
    }
}

fn render_background(canvas: &mut Canvas, lb: &LayoutBox) {
    let color = get_color(lb, "background-color").unwrap_or(CssColor::TRANSPARENT);
    if color.a == 0 {
        return;
    }
    canvas.fill_rect(lb.dimensions.border_box(), color);
}

fn render_borders(canvas: &mut Canvas, lb: &LayoutBox) {
    let color = match get_color(lb, "border-color") {
        Some(c) => c,
        None => return,
    };
    let d = &lb.dimensions;
    let bb = d.border_box();
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y,
            width: bb.width,
            height: d.border.top,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y + bb.height - d.border.bottom,
            width: bb.width,
            height: d.border.bottom,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y,
            width: d.border.left,
            height: bb.height,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x + bb.width - d.border.right,
            y: bb.y,
            width: d.border.right,
            height: bb.height,
        },
        color,
    );
}

fn render_text(canvas: &mut Canvas, lb: &LayoutBox) {
    let text = match &lb.text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };
    let font_color = CssColor::DARK;
    let rect = Rect {
        x: lb.dimensions.content.x,
        y: lb.dimensions.content.y,
        width: (text.len() as f32 * 7.0).min(lb.dimensions.content.width),
        height: 14.0,
    };
    canvas.fill_rect(rect, font_color);
}

#[cfg(feature = "feather-renderer")]
pub fn blit_glyph(
    canvas: &mut Canvas,
    glyph: &GlyphBitmap,
    x: i32,
    baseline_y: i32,
    color: CssColor,
) {
    for gy in 0..glyph.height {
        for gx in 0..glyph.width {
            let alpha = glyph.pixels[gy * glyph.width + gx];
            if alpha == 0 {
                continue;
            }
            let cx = x + glyph.bearing_x + gx as i32;
            let cy = baseline_y - glyph.bearing_y + gy as i32;
            if cx < 0 || cy < 0 {
                continue;
            }
            let alpha_f = alpha as f32 / 255.0;
            let blended = CssColor {
                r: (color.r as f32 * alpha_f + 255.0 * (1.0 - alpha_f)) as u8,
                g: (color.g as f32 * alpha_f + 255.0 * (1.0 - alpha_f)) as u8,
                b: (color.b as f32 * alpha_f + 255.0 * (1.0 - alpha_f)) as u8,
                a: 255,
            };
            canvas.set_pixel(cx as usize, cy as usize, blended);
        }
    }
}

#[cfg(feature = "feather-renderer")]
fn render_background_scrolled(canvas: &mut Canvas, lb: &LayoutBox, scroll_y: f32, opacity: f32) {
    let color = with_opacity(get_color(lb, "background-color").unwrap_or(CssColor::TRANSPARENT), opacity);
    if color.a == 0 {
        return;
    }
    let mut rect = lb.dimensions.border_box();
    rect.y -= scroll_y;
    canvas.fill_rect(rect, color);
}

#[cfg(feature = "feather-renderer")]
fn render_borders_scrolled(canvas: &mut Canvas, lb: &LayoutBox, scroll_y: f32, opacity: f32) {
    let color = match get_color(lb, "border-color").map(|c| with_opacity(c, opacity)) {
        Some(c) if c.a > 0 => c,
        _ => return,
    };
    let d = &lb.dimensions;
    let mut bb = d.border_box();
    bb.y -= scroll_y;
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y,
            width: bb.width,
            height: d.border.top,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y + bb.height - d.border.bottom,
            width: bb.width,
            height: d.border.bottom,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x,
            y: bb.y,
            width: d.border.left,
            height: bb.height,
        },
        color,
    );
    canvas.fill_rect(
        Rect {
            x: bb.x + bb.width - d.border.right,
            y: bb.y,
            width: d.border.right,
            height: bb.height,
        },
        color,
    );
}

#[cfg(feature = "feather-renderer")]
fn is_img_element(lb: &LayoutBox) -> bool {
    match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) => {
            matches!(&s.node.node_type, NodeType::Element(e) if e.tag_name == "img")
        }
        BoxType::Anonymous => false,
    }
}

#[cfg(feature = "feather-renderer")]
fn render_img_placeholder(canvas: &mut Canvas, lb: &LayoutBox, scroll_y: f32, font: &FontMetrics) {
    let c = lb.dimensions.content;
    let grey = CssColor {
        r: 180,
        g: 180,
        b: 180,
        a: 255,
    };
    let border_col = CssColor {
        r: 140,
        g: 140,
        b: 140,
        a: 255,
    };
    let y = c.y - scroll_y;

    // grey fill
    canvas.fill_rect(
        Rect {
            x: c.x,
            y,
            width: c.width,
            height: c.height,
        },
        grey,
    );
    // 1-px border
    canvas.fill_rect(
        Rect {
            x: c.x,
            y,
            width: c.width,
            height: 1.0,
        },
        border_col,
    );
    canvas.fill_rect(
        Rect {
            x: c.x,
            y: y + c.height - 1.0,
            width: c.width,
            height: 1.0,
        },
        border_col,
    );
    canvas.fill_rect(
        Rect {
            x: c.x,
            y,
            width: 1.0,
            height: c.height,
        },
        border_col,
    );
    canvas.fill_rect(
        Rect {
            x: c.x + c.width - 1.0,
            y,
            width: 1.0,
            height: c.height,
        },
        border_col,
    );

    // centered alt text
    if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
        if let NodeType::Element(e) = &s.node.node_type {
            if let Some(alt) = e.attributes.get("alt") {
                if !alt.is_empty() {
                    let fm = FontMetrics::load_cached(12.0);
                    let text_w = fm.measure_str(alt);
                    let pen_start_x = c.x + ((c.width - text_w) / 2.0).max(0.0);
                    let baseline_y =
                        (y + (c.height - fm.line_height) / 2.0 + fm.ascent).round() as i32;
                    let mut pen_x = pen_start_x;
                    for ch in alt.chars() {
                        let glyph = fm.rasterize_cached(ch);
                        blit_glyph(
                            canvas,
                            &glyph,
                            pen_x.round() as i32,
                            baseline_y,
                            CssColor::DARK,
                        );
                        pen_x += glyph.advance_width;
                    }
                }
            }
        }
    }
    let _ = font; // kept for signature parity with render_box
}

#[cfg(feature = "feather-renderer")]
fn render_text_scrolled(canvas: &mut Canvas, lb: &LayoutBox, scroll_y: f32) {
    let text = match &lb.text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };
    let font_color = CssColor::DARK;
    let rect = Rect {
        x: lb.dimensions.content.x,
        y: lb.dimensions.content.y - scroll_y,
        width: (text.len() as f32 * 7.0).min(lb.dimensions.content.width),
        height: 14.0,
    };
    canvas.fill_rect(rect, font_color);
}

fn has_overflow_hidden(lb: &LayoutBox) -> bool {
    match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) =>
            matches!(s.properties.get("overflow"), Some(Value::Keyword(k)) if k == "hidden"),
        BoxType::Anonymous => false,
    }
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
