// Text Rendering Module - Font loading and text rasterization

use fontdue::{Font, FontSettings};
use crate::css::Color;
use crate::layout::Rect;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

impl Default for TextDecoration {
    fn default() -> Self {
        TextDecoration::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

impl Default for TextTransform {
    fn default() -> Self {
        TextTransform::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontVariant {
    Normal,
    SmallCaps,
}

impl Default for FontVariant {
    fn default() -> Self {
        FontVariant::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
}

impl Default for WhiteSpace {
    fn default() -> Self {
        WhiteSpace::Normal
    }
}

#[derive(Debug, Clone)]
pub struct FontCache {
    fonts: HashMap<String, Font>,
    default_font: Font,
    bold_font: Font,
    italic_font: Font,
    bold_italic_font: Font,
}

impl FontCache {
    pub fn new() -> Self {
        let default_font_data = include_bytes!("../assets/Roboto-Regular.ttf");
        let bold_font_data = include_bytes!("../assets/Roboto-Bold.ttf");
        let italic_font_data = include_bytes!("../assets/Roboto-Italic.ttf");
        let bold_italic_font_data = include_bytes!("../assets/Roboto-BoldItalic.ttf");
        
        let default_font = Font::from_bytes(
            default_font_data as &[u8],
            FontSettings::default()
        ).expect("Failed to load default font");

        let bold_font = Font::from_bytes(
            bold_font_data as &[u8],
            FontSettings::default()
        ).expect("Failed to load bold font");

        let italic_font = Font::from_bytes(
            italic_font_data as &[u8],
            FontSettings::default()
        ).expect("Failed to load italic font");

        let bold_italic_font = Font::from_bytes(
            bold_italic_font_data as &[u8],
            FontSettings::default()
        ).expect("Failed to load bold-italic font");

        FontCache {
            fonts: HashMap::new(),
            default_font,
            bold_font,
            italic_font,
            bold_italic_font,
        }
    }

    pub fn load_font(&mut self, name: String, data: &[u8]) -> Result<(), String> {
        let font = Font::from_bytes(data, FontSettings::default())
            .map_err(|e| format!("Failed to parse font: {}", e))?;
        self.fonts.insert(name, font);
        Ok(())
    }

    pub fn get_font(&self, name: &str) -> &Font {
        self.fonts.get(name).unwrap_or(&self.default_font)
    }

    pub fn get_styled_font(&self, weight: u16, italic: bool) -> &Font {
        match (weight >= 700, italic) {
            (true, true) => &self.bold_italic_font,
            (true, false) => &self.bold_font,
            (false, true) => &self.italic_font,
            (false, false) => &self.default_font,
        }
    }

    pub fn get_default_font(&self) -> &Font {
        &self.default_font
    }
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub font_italic: bool,
    pub color: Color,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub text_transform: TextTransform,
    pub text_shadow: Option<TextShadow>,
    pub font_variant: FontVariant,
    pub white_space: WhiteSpace,
}

#[derive(Debug, Clone, Copy)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub color: Color,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font_family: "default".to_string(),
            font_size: 16.0,
            font_weight: 400,
            font_italic: false,
            color: Color { r: 0, g: 0, b: 0, a: 255 },
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Left,
            text_decoration: TextDecoration::None,
            text_transform: TextTransform::None,
            text_shadow: None,
            font_variant: FontVariant::Normal,
            white_space: WhiteSpace::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlyphInstance {
    pub x: f32,
    pub y: f32,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub glyphs: Vec<GlyphInstance>,
    pub bounds: Rect,
}

#[derive(Debug, Clone)]
pub struct TextRenderer {
    font_cache: FontCache,
}

impl TextRenderer {
    pub fn new() -> Self {
        TextRenderer {
            font_cache: FontCache::new(),
        }
    }

    pub fn layout_text(
        &self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> TextLayout {
        // Preprocess text based on white-space property
        let preprocessed_text = self.preprocess_whitespace(text, style.white_space);
        
        // Apply text transform
        let transformed_text = self.apply_text_transform(&preprocessed_text, style.text_transform);
        
        let font = self.font_cache.get_styled_font(style.font_weight, style.font_italic);
        let scale = style.font_size;
        
        let mut glyphs = Vec::new();
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut max_x: f32 = 0.0;
        let line_height = style.font_size * style.line_height;

        // First pass: layout glyphs
        let mut line_glyphs = Vec::new();
        let mut line_width: f32 = 0.0;
        let mut prev_was_space = false;

        for ch in transformed_text.chars() {
            if ch == '\n' {
                // Apply alignment to completed line
                self.apply_alignment(&mut line_glyphs, line_width, max_width, style.text_align);
                glyphs.append(&mut line_glyphs);
                
                x = 0.0;
                y += line_height;
                line_width = 0.0;
                prev_was_space = false;
                continue;
            }

            if ch.is_control() {
                continue;
            }

            // Handle small-caps for lowercase letters
            let (actual_ch, actual_scale) = if style.font_variant == FontVariant::SmallCaps && ch.is_lowercase() {
                (ch.to_uppercase().next().unwrap_or(ch), scale * 0.8)
            } else {
                (ch, scale)
            };

            let (metrics, bitmap) = font.rasterize(actual_ch, actual_scale);

            // Check word wrapping (only for Normal and PreWrap modes)
            if let Some(max_w) = max_width {
                if matches!(style.white_space, WhiteSpace::Normal | WhiteSpace::PreWrap) {
                    if x + metrics.advance_width > max_w && x > 0.0 {
                        // Apply alignment to completed line
                        let final_line_width = line_width;
                        self.apply_alignment(&mut line_glyphs, final_line_width, max_width, style.text_align);
                        glyphs.append(&mut line_glyphs);
                        
                        x = 0.0;
                        y += line_height;
                        // line_width will be reset to x (0.0) automatically in the update below
                        max_x = max_x.max(final_line_width);
                        prev_was_space = false;
                    }
                }
            }

            let glyph = GlyphInstance {
                x: x + metrics.xmin as f32,
                y: y + metrics.ymin as f32,
                width: metrics.width,
                height: metrics.height,
                pixels: bitmap,
            };

            line_glyphs.push(glyph);
            
            // Add word spacing after space characters
            let spacing = if ch.is_whitespace() && !prev_was_space {
                style.word_spacing
            } else {
                0.0
            };
            
            x += metrics.advance_width + style.letter_spacing + spacing;
            line_width = x;
            max_x = max_x.max(x);
            prev_was_space = ch.is_whitespace();
        }

        // Apply alignment to last line
        if !line_glyphs.is_empty() {
            self.apply_alignment(&mut line_glyphs, line_width, max_width, style.text_align);
            glyphs.append(&mut line_glyphs);
        }

        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: max_width.unwrap_or(max_x),
            height: y + line_height,
        };

        TextLayout { glyphs, bounds }
    }

    fn preprocess_whitespace(&self, text: &str, white_space: WhiteSpace) -> String {
        match white_space {
            WhiteSpace::Normal => {
                // Collapse whitespace sequences to single space
                let mut result = String::new();
                let mut prev_was_space = false;
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        if !prev_was_space {
                            result.push(' ');
                            prev_was_space = true;
                        }
                    } else {
                        result.push(ch);
                        prev_was_space = false;
                    }
                }
                result.trim().to_string()
            }
            WhiteSpace::Nowrap => {
                // Same as Normal but prevent wrapping (handled in layout)
                let mut result = String::new();
                let mut prev_was_space = false;
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        if !prev_was_space {
                            result.push(' ');
                            prev_was_space = true;
                        }
                    } else {
                        result.push(ch);
                        prev_was_space = false;
                    }
                }
                result.trim().to_string()
            }
            WhiteSpace::Pre => {
                // Preserve all whitespace and newlines
                text.to_string()
            }
            WhiteSpace::PreWrap => {
                // Preserve whitespace but allow wrapping
                text.to_string()
            }
            WhiteSpace::PreLine => {
                // Collapse whitespace sequences but preserve newlines
                let mut result = String::new();
                let mut prev_was_space = false;
                for ch in text.chars() {
                    if ch == '\n' {
                        result.push('\n');
                        prev_was_space = false;
                    } else if ch.is_whitespace() {
                        if !prev_was_space {
                            result.push(' ');
                            prev_was_space = true;
                        }
                    } else {
                        result.push(ch);
                        prev_was_space = false;
                    }
                }
                result
            }
        }
    }

    fn apply_alignment(
        &self,
        line_glyphs: &mut Vec<GlyphInstance>,
        line_width: f32,
        max_width: Option<f32>,
        align: TextAlign,
    ) {
        if line_glyphs.is_empty() {
            return;
        }

        let container_width = max_width.unwrap_or(line_width);
        let offset = match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (container_width - line_width) / 2.0,
            TextAlign::Right => container_width - line_width,
            TextAlign::Justify => {
                // For justify, we don't offset but space out words
                // (simplified: just left align for now)
                0.0
            }
        };

        // Apply offset to all glyphs in the line
        for glyph in line_glyphs.iter_mut() {
            glyph.x += offset;
        }
    }

    fn apply_text_transform(&self, text: &str, transform: TextTransform) -> String {
        match transform {
            TextTransform::None => text.to_string(),
            TextTransform::Uppercase => text.to_uppercase(),
            TextTransform::Lowercase => text.to_lowercase(),
            TextTransform::Capitalize => {
                let mut result = String::new();
                let mut capitalize_next = true;
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        capitalize_next = true;
                        result.push(ch);
                    } else if capitalize_next {
                        result.push_str(&ch.to_uppercase().to_string());
                        capitalize_next = false;
                    } else {
                        result.push(ch);
                    }
                }
                result
            }
        }
    }

    pub fn measure_text(&self, text: &str, style: &TextStyle) -> Rect {
        let layout = self.layout_text(text, style, None);
        layout.bounds
    }
}

// Render text decorations (underline, overline, line-through)
pub fn render_text_decoration(
    pixels: &mut [Color],
    canvas_width: usize,
    canvas_height: usize,
    layout: &TextLayout,
    style: &TextStyle,
    offset_x: f32,
    offset_y: f32,
) {
    if style.text_decoration == TextDecoration::None {
        return;
    }

    let thickness = (style.font_size * 0.08).max(1.0) as usize;
    let bounds = &layout.bounds;
    
    let y_pos = match style.text_decoration {
        TextDecoration::Underline => {
            offset_y + bounds.height - thickness as f32
        }
        TextDecoration::Overline => {
            offset_y
        }
        TextDecoration::LineThrough => {
            offset_y + bounds.height * 0.5
        }
        TextDecoration::None => return,
    };

    let x0 = offset_x.max(0.0) as usize;
    let x1 = (offset_x + bounds.width).min(canvas_width as f32) as usize;
    let y0 = y_pos.max(0.0) as usize;
    let y1 = (y_pos + thickness as f32).min(canvas_height as f32) as usize;

    for y in y0..y1 {
        for x in x0..x1 {
            if y * canvas_width + x < pixels.len() {
                pixels[y * canvas_width + x] = style.color;
            }
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn blend_glyph(
    canvas_pixels: &mut [Color],
    canvas_width: usize,
    canvas_height: usize,
    glyph: &GlyphInstance,
    color: Color,
) {
    let glyph_x = glyph.x as i32;
    let glyph_y = glyph.y as i32;

    for dy in 0..glyph.height {
        for dx in 0..glyph.width {
            let canvas_x = glyph_x + dx as i32;
            let canvas_y = glyph_y + dy as i32;

            if canvas_x < 0 || canvas_y < 0 
                || canvas_x >= canvas_width as i32 
                || canvas_y >= canvas_height as i32 {
                continue;
            }

            let canvas_idx = canvas_y as usize * canvas_width + canvas_x as usize;
            let glyph_idx = dy * glyph.width + dx;

            if canvas_idx >= canvas_pixels.len() || glyph_idx >= glyph.pixels.len() {
                continue;
            }

            let alpha = glyph.pixels[glyph_idx] as f32 / 255.0;
            
            if alpha > 0.0 {
                let bg = &canvas_pixels[canvas_idx];
                let inv_alpha = 1.0 - alpha;
                canvas_pixels[canvas_idx] = Color {
                    r: ((color.r as f32 * alpha + bg.r as f32 * inv_alpha) as u8),
                    g: ((color.g as f32 * alpha + bg.g as f32 * inv_alpha) as u8),
                    b: ((color.b as f32 * alpha + bg.b as f32 * inv_alpha) as u8),
                    a: 255,
                };
            }
        }
    }
}
