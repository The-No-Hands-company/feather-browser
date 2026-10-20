// Render Engine - Paints the layout tree to pixels

pub mod gpu;

use crate::layout::{LayoutBox, Rect};
use crate::css::{Color, Value};
use crate::text::{TextRenderer, TextStyle, blend_glyph, TextDecoration, TextTransform, render_text_decoration};
use crate::dom::NodeType;
use crate::image::{ImageLoader, DecodedImage, blend_image_onto_canvas};

// Apply opacity to a color
fn apply_opacity(color: Color, opacity: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a as f32 * opacity.max(0.0).min(1.0)) as u8,
    }
}

// Blend a color with opacity onto existing pixel
fn blend_with_opacity(pixels: &mut [Color], index: usize, color: Color) {
    let existing = pixels[index];
    let alpha = color.a as f32 / 255.0;
    pixels[index] = Color {
        r: ((color.r as f32 * alpha + existing.r as f32 * (1.0 - alpha)) as u8),
        g: ((color.g as f32 * alpha + existing.g as f32 * (1.0 - alpha)) as u8),
        b: ((color.b as f32 * alpha + existing.b as f32 * (1.0 - alpha)) as u8),
        a: 255,
    };
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }

    pub fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            DisplayCommand::SolidColor(color, rect) => {
                let x0 = rect.x.max(0.0) as usize;
                let y0 = rect.y.max(0.0) as usize;
                let x1 = (rect.x + rect.width).min(self.width as f32) as usize;
                let y1 = (rect.y + rect.height).min(self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        let idx = y * self.width + x;
                        if color.a < 255 {
                            blend_with_opacity(&mut self.pixels, idx, *color);
                        } else {
                            self.pixels[idx] = *color;
                        }
                    }
                }
            }
            DisplayCommand::RoundedRect(color, rect, radius) => {
                let x0 = rect.x.max(0.0) as usize;
                let y0 = rect.y.max(0.0) as usize;
                let x1 = (rect.x + rect.width).min(self.width as f32) as usize;
                let y1 = (rect.y + rect.height).min(self.height as f32) as usize;
                let r = *radius;

                for y in y0..y1 {
                    for x in x0..x1 {
                        let px = x as f32 - rect.x;
                        let py = y as f32 - rect.y;
                        
                        // Check if pixel is within rounded corners
                        let in_corner = if px < r && py < r {
                            // Top-left corner
                            ((px - r).powi(2) + (py - r).powi(2)).sqrt() <= r
                        } else if px > rect.width - r && py < r {
                            // Top-right corner
                            ((px - (rect.width - r)).powi(2) + (py - r).powi(2)).sqrt() <= r
                        } else if px < r && py > rect.height - r {
                            // Bottom-left corner
                            ((px - r).powi(2) + (py - (rect.height - r)).powi(2)).sqrt() <= r
                        } else if px > rect.width - r && py > rect.height - r {
                            // Bottom-right corner
                            ((px - (rect.width - r)).powi(2) + (py - (rect.height - r)).powi(2)).sqrt() <= r
                        } else {
                            true
                        };
                        
                        if in_corner {
                            let idx = y * self.width + x;
                            if color.a < 255 {
                                blend_with_opacity(&mut self.pixels, idx, *color);
                            } else {
                                self.pixels[idx] = *color;
                            }
                        }
                    }
                }
            }
            DisplayCommand::BoxShadow(rect, offset_x, offset_y, blur, color) => {
                let shadow_rect = Rect {
                    x: rect.x + offset_x,
                    y: rect.y + offset_y,
                    width: rect.width,
                    height: rect.height,
                };
                
                let x0 = (shadow_rect.x - blur).max(0.0) as usize;
                let y0 = (shadow_rect.y - blur).max(0.0) as usize;
                let x1 = (shadow_rect.x + shadow_rect.width + blur).min(self.width as f32) as usize;
                let y1 = (shadow_rect.y + shadow_rect.height + blur).min(self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        let px = x as f32;
                        let py = y as f32;
                        
                        // Calculate distance from shadow rect
                        let dx = if px < shadow_rect.x {
                            shadow_rect.x - px
                        } else if px > shadow_rect.x + shadow_rect.width {
                            px - (shadow_rect.x + shadow_rect.width)
                        } else {
                            0.0
                        };
                        
                        let dy = if py < shadow_rect.y {
                            shadow_rect.y - py
                        } else if py > shadow_rect.y + shadow_rect.height {
                            py - (shadow_rect.y + shadow_rect.height)
                        } else {
                            0.0
                        };
                        
                        let dist = (dx.powi(2) + dy.powi(2)).sqrt();
                        
                        if dist < *blur {
                            let alpha = ((*blur - dist) / *blur * color.a as f32) as u8;
                            let existing = self.pixels[y * self.width + x];
                            let a = alpha as f32 / 255.0;
                            
                            self.pixels[y * self.width + x] = Color {
                                r: ((color.r as f32 * a + existing.r as f32 * (1.0 - a)) as u8),
                                g: ((color.g as f32 * a + existing.g as f32 * (1.0 - a)) as u8),
                                b: ((color.b as f32 * a + existing.b as f32 * (1.0 - a)) as u8),
                                a: 255,
                            };
                        }
                    }
                }
            }
            DisplayCommand::Text(layout, style) => {
                for glyph in &layout.glyphs {
                    blend_glyph(&mut self.pixels, self.width, self.height, glyph, style.color);
                }
                // Render text decorations (underline, overline, line-through)
                render_text_decoration(&mut self.pixels, self.width, self.height, layout, style, 0.0, 0.0);
            }
            DisplayCommand::DashedBorder(color, rect, dash_length) => {
                let x0 = rect.x.max(0.0) as usize;
                let y0 = rect.y.max(0.0) as usize;
                let x1 = (rect.x + rect.width).min(self.width as f32) as usize;
                let y1 = (rect.y + rect.height).min(self.height as f32) as usize;
                let dash = *dash_length as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        // Create dashed pattern (horizontal or vertical)
                        let is_horizontal = rect.height < rect.width;
                        let pos = if is_horizontal { x - x0 } else { y - y0 };
                        if (pos / dash) % 2 == 0 {
                            let idx = y * self.width + x;
                            if color.a < 255 {
                                blend_with_opacity(&mut self.pixels, idx, *color);
                            } else {
                                self.pixels[idx] = *color;
                            }
                        }
                    }
                }
            }
            DisplayCommand::DottedBorder(color, rect, dot_size) => {
                let x0 = rect.x.max(0.0) as usize;
                let y0 = rect.y.max(0.0) as usize;
                let x1 = (rect.x + rect.width).min(self.width as f32) as usize;
                let y1 = (rect.y + rect.height).min(self.height as f32) as usize;
                let dot = *dot_size as usize;
                let spacing = dot * 2;

                for y in y0..y1 {
                    for x in x0..x1 {
                        // Create dotted pattern
                        let is_horizontal = rect.height < rect.width;
                        let pos = if is_horizontal { x - x0 } else { y - y0 };
                        let in_dot = (pos % spacing) < dot;
                        if in_dot {
                            let idx = y * self.width + x;
                            if color.a < 255 {
                                blend_with_opacity(&mut self.pixels, idx, *color);
                            } else {
                                self.pixels[idx] = *color;
                            }
                        }
                    }
                }
            }
            DisplayCommand::Image(img, rect) => {
                blend_image_onto_canvas(
                    &mut self.pixels,
                    self.width,
                    self.height,
                    img,
                    rect.x as i32,
                    rect.y as i32,
                );
            }
        }
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::BufWriter;

        let file = File::create(filename)?;
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;

        let mut data = Vec::with_capacity(self.width * self.height * 4);
        for pixel in &self.pixels {
            data.push(pixel.r);
            data.push(pixel.g);
            data.push(pixel.b);
            data.push(pixel.a);
        }

        writer.write_image_data(&data)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    SolidColor(Color, Rect),
    RoundedRect(Color, Rect, f32), // color, rect, border-radius
    BoxShadow(Rect, f32, f32, f32, Color), // rect, offset-x, offset-y, blur, color
    DashedBorder(Color, Rect, f32), // color, rect, dash-length
    DottedBorder(Color, Rect, f32), // color, rect, dot-size
    Text(crate::text::TextLayout, TextStyle),
    Image(DecodedImage, Rect),
}

type DisplayList = Vec<DisplayCommand>;

#[derive(Debug, Clone)]
pub struct RenderEngine {
    text_renderer: TextRenderer,
    image_loader: ImageLoader,
}

impl RenderEngine {
    pub fn new() -> Self {
        RenderEngine {
            text_renderer: TextRenderer::new(),
            image_loader: ImageLoader::new(),
        }
    }

    pub fn get_image_loader_mut(&mut self) -> &mut ImageLoader {
        &mut self.image_loader
    }

    pub fn build_display_list(&mut self, layout_root: &LayoutBox) -> DisplayList {
        let mut list = Vec::new();
        self.render_layout_box(&mut list, layout_root);
        list
    }

    fn is_visible(&self, layout_box: &LayoutBox) -> bool {
        if let Some(styled_node) = layout_box.get_styled_node() {
            if let Some(Value::Keyword(vis)) = styled_node.value("visibility") {
                return vis != "hidden";
            }
        }
        true // Default: visible
    }

    fn render_layout_box(&mut self, list: &mut DisplayList, layout_box: &LayoutBox) {
        // Skip rendering if visibility: hidden
        if !self.is_visible(layout_box) {
            return;
        }
        
        self.render_background(list, layout_box);
        self.render_borders(list, layout_box);
        self.render_text(list, layout_box);
        self.render_image(list, layout_box);

        for child in &layout_box.children {
            self.render_layout_box(list, child);
        }
    }

    fn render_background(&self, list: &mut DisplayList, layout_box: &LayoutBox) {
        if let Some(mut color) = self.get_color(layout_box, "background-color") {
            let rect = layout_box.dimensions.border_box();
            
            // Apply opacity
            let opacity = self.get_opacity(layout_box);
            if opacity < 1.0 {
                color = apply_opacity(color, opacity);
            }
            
            // Check for box-shadow first (render before background)
            if let Some(styled_node) = layout_box.get_styled_node() {
                // Simple box-shadow parsing (x y blur color)
                if let Some(Value::Keyword(_shadow)) = styled_node.value("box-shadow") {
                    // For now, use default shadow
                    list.push(DisplayCommand::BoxShadow(
                        rect,
                        2.0,  // offset-x
                        2.0,  // offset-y
                        4.0,  // blur
                        Color { r: 0, g: 0, b: 0, a: 50 },
                    ));
                }
            }
            
            // Check for border-radius
            let border_radius = if let Some(styled_node) = layout_box.get_styled_node() {
                match styled_node.value("border-radius") {
                    Some(Value::Length(radius, _)) => *radius,
                    _ => 0.0,
                }
            } else {
                0.0
            };
            
            if border_radius > 0.0 {
                list.push(DisplayCommand::RoundedRect(color, rect, border_radius));
            } else {
                list.push(DisplayCommand::SolidColor(color, rect));
            }
        }
    }

    fn render_borders(&self, list: &mut DisplayList, layout_box: &LayoutBox) {
        if let Some(color) = self.get_color(layout_box, "border-color") {
            let d = &layout_box.dimensions;
            let border_box = d.border_box();
            
            // Get border style (solid, dashed, dotted)
            let border_style = if let Some(styled_node) = layout_box.get_styled_node() {
                match styled_node.value("border-style") {
                    Some(Value::Keyword(style)) => style.as_str(),
                    _ => "solid",
                }
            } else {
                "solid"
            };
            
            // Top border
            if d.border.top > 0.0 {
                let rect = Rect {
                    x: border_box.x,
                    y: border_box.y,
                    width: border_box.width,
                    height: d.border.top,
                };
                
                match border_style {
                    "dashed" => list.push(DisplayCommand::DashedBorder(color, rect, 8.0)),
                    "dotted" => list.push(DisplayCommand::DottedBorder(color, rect, 2.0)),
                    _ => list.push(DisplayCommand::SolidColor(color, rect)),
                }
            }
            
            // Right border
            if d.border.right > 0.0 {
                let rect = Rect {
                    x: border_box.x + border_box.width - d.border.right,
                    y: border_box.y,
                    width: d.border.right,
                    height: border_box.height,
                };
                
                match border_style {
                    "dashed" => list.push(DisplayCommand::DashedBorder(color, rect, 8.0)),
                    "dotted" => list.push(DisplayCommand::DottedBorder(color, rect, 2.0)),
                    _ => list.push(DisplayCommand::SolidColor(color, rect)),
                }
            }
            
            // Bottom border
            if d.border.bottom > 0.0 {
                let rect = Rect {
                    x: border_box.x,
                    y: border_box.y + border_box.height - d.border.bottom,
                    width: border_box.width,
                    height: d.border.bottom,
                };
                
                match border_style {
                    "dashed" => list.push(DisplayCommand::DashedBorder(color, rect, 8.0)),
                    "dotted" => list.push(DisplayCommand::DottedBorder(color, rect, 2.0)),
                    _ => list.push(DisplayCommand::SolidColor(color, rect)),
                }
            }
            
            // Left border
            if d.border.left > 0.0 {
                let rect = Rect {
                    x: border_box.x,
                    y: border_box.y,
                    width: d.border.left,
                    height: border_box.height,
                };
                
                match border_style {
                    "dashed" => list.push(DisplayCommand::DashedBorder(color, rect, 8.0)),
                    "dotted" => list.push(DisplayCommand::DottedBorder(color, rect, 2.0)),
                    _ => list.push(DisplayCommand::SolidColor(color, rect)),
                }
            }
        }
    }

    fn get_color(&self, layout_box: &LayoutBox, name: &str) -> Option<Color> {
        if let Some(styled_node) = layout_box.get_styled_node() {
            if let Some(Value::Color(color)) = styled_node.value(name) {
                return Some(*color);
            }
        }
        None
    }

    fn get_opacity(&self, layout_box: &LayoutBox) -> f32 {
        if let Some(styled_node) = layout_box.get_styled_node() {
            if let Some(Value::Length(opacity, _)) = styled_node.value("opacity") {
                return opacity.max(0.0).min(1.0);
            }
        }
        1.0 // Default: fully opaque
    }

    fn render_text(&self, list: &mut DisplayList, layout_box: &LayoutBox) {
        if let Some(styled_node) = layout_box.get_styled_node() {
            if let NodeType::Text(text) = &styled_node.node.node_type {
                let mut color = self.get_color(layout_box, "color")
                    .unwrap_or(Color { r: 0, g: 0, b: 0, a: 255 });
                
                // Apply opacity
                let opacity = self.get_opacity(layout_box);
                if opacity < 1.0 {
                    color = apply_opacity(color, opacity);
                }
                
                let font_size = match styled_node.value("font-size") {
                    Some(Value::Length(size, _)) => *size,
                    _ => 16.0,
                };

                let font_weight = match styled_node.value("font-weight") {
                    Some(Value::Keyword(w)) => match w.as_str() {
                        "bold" => 700,
                        "normal" => 400,
                        "lighter" => 300,
                        "bolder" => 900,
                        _ => 400,
                    },
                    Some(Value::Length(w, _)) => *w as u16,
                    _ => 400,
                };

                let font_italic = match styled_node.value("font-style") {
                    Some(Value::Keyword(s)) => s == "italic" || s == "oblique",
                    _ => false,
                };

                let text_align = match styled_node.value("text-align") {
                    Some(Value::Keyword(a)) => match a.as_str() {
                        "center" => crate::text::TextAlign::Center,
                        "right" => crate::text::TextAlign::Right,
                        "justify" => crate::text::TextAlign::Justify,
                        _ => crate::text::TextAlign::Left,
                    },
                    _ => crate::text::TextAlign::Left,
                };

                let text_decoration = match styled_node.value("text-decoration") {
                    Some(Value::Keyword(d)) => match d.as_str() {
                        "underline" => TextDecoration::Underline,
                        "overline" => TextDecoration::Overline,
                        "line-through" => TextDecoration::LineThrough,
                        _ => TextDecoration::None,
                    },
                    _ => TextDecoration::None,
                };

                let text_transform = match styled_node.value("text-transform") {
                    Some(Value::Keyword(t)) => match t.as_str() {
                        "uppercase" => TextTransform::Uppercase,
                        "lowercase" => TextTransform::Lowercase,
                        "capitalize" => TextTransform::Capitalize,
                        _ => TextTransform::None,
                    },
                    _ => TextTransform::None,
                };

                let font_variant = match styled_node.value("font-variant") {
                    Some(Value::Keyword(v)) => match v.as_str() {
                        "small-caps" => crate::text::FontVariant::SmallCaps,
                        _ => crate::text::FontVariant::Normal,
                    },
                    _ => crate::text::FontVariant::Normal,
                };

                let white_space = match styled_node.value("white-space") {
                    Some(Value::Keyword(ws)) => match ws.as_str() {
                        "nowrap" => crate::text::WhiteSpace::Nowrap,
                        "pre" => crate::text::WhiteSpace::Pre,
                        "pre-wrap" => crate::text::WhiteSpace::PreWrap,
                        "pre-line" => crate::text::WhiteSpace::PreLine,
                        _ => crate::text::WhiteSpace::Normal,
                    },
                    _ => crate::text::WhiteSpace::Normal,
                };

                let word_spacing = match styled_node.value("word-spacing") {
                    Some(Value::Length(spacing, _)) => *spacing,
                    _ => 0.0,
                };

                let letter_spacing = match styled_node.value("letter-spacing") {
                    Some(Value::Length(spacing, _)) => *spacing,
                    _ => 0.0,
                };

                let line_height = match styled_node.value("line-height") {
                    Some(Value::Length(lh, _)) => *lh,
                    _ => 1.2,
                };

                let style = TextStyle {
                    font_family: "default".to_string(),
                    font_size,
                    font_weight,
                    font_italic,
                    color,
                    text_align,
                    text_decoration,
                    text_transform,
                    font_variant,
                    white_space,
                    word_spacing,
                    letter_spacing,
                    line_height,
                    ..Default::default()
                };

                let layout = self.text_renderer.layout_text(text, &style, 
                    Some(layout_box.dimensions.content.width));

                list.push(DisplayCommand::Text(layout, style));
            }
        }
    }

    fn render_image(&mut self, list: &mut DisplayList, layout_box: &LayoutBox) {
        if let Some(styled_node) = layout_box.get_styled_node() {
            if let NodeType::Element(ref data) = &styled_node.node.node_type {
                if data.tag_name == "img" {
                    if let Some(_src) = data.attributes.get("src") {
                        let rect = layout_box.dimensions.content;
                        
                        // Try to load from src/assets/test-image.png for demo
                        let img = self.image_loader
                            .load_from_path("src/assets/test-image.png")
                            .unwrap_or_else(|_| {
                                // Fallback to placeholder
                                DecodedImage {
                                    width: rect.width as u32,
                                    height: rect.height as u32,
                                    pixels: vec![Color { r: 200, g: 200, b: 220, a: 255 }; 
                                        (rect.width * rect.height) as usize],
                                }
                            });
                        
                        list.push(DisplayCommand::Image(img, rect));
                    }
                }
            }
        }
    }

    pub fn paint(&mut self, layout_root: &LayoutBox, bounds: Rect) -> Canvas {
        let display_list = self.build_display_list(layout_root);
        
        // Note: z-index sorting would be implemented here when we track z-index per command
        // For now, document order is preserved
        
        let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
        
        for item in display_list {
            canvas.paint_item(&item);
        }
        
        canvas
    }
}

impl Default for RenderEngine {
    fn default() -> Self {
        Self::new()
    }
}
