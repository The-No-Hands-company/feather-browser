// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod inner {
    use fontdue::{Font, FontSettings};

    const FONT_BYTES: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");

    pub struct GlyphBitmap {
        pub pixels: Vec<u8>,
        pub width: usize,
        pub height: usize,
        pub bearing_x: i32,
        pub bearing_y: i32,
    }

    pub struct FontMetrics {
        font: Font,
        pub size: f32,
        pub ascent: f32,
        pub descent: f32,
        pub line_height: f32,
        pub space_advance: f32,
    }

    impl FontMetrics {
        pub fn load(size: f32) -> Self {
            let font = Font::from_bytes(FONT_BYTES, FontSettings::default())
                .expect("embedded NotoSans font bytes are valid");
            let lm = font.horizontal_line_metrics(size)
                .unwrap_or(fontdue::LineMetrics {
                    ascent: size * 0.8,
                    descent: -(size * 0.2),
                    line_gap: 0.0,
                    new_line_size: size,
                });
            let ascent = lm.ascent;
            let descent = (-lm.descent).max(0.0);
            let line_height = (ascent + descent) * 1.2;
            let space_advance = font.metrics(' ', size).advance_width;
            FontMetrics { font, size, ascent, descent, line_height, space_advance }
        }

        pub fn advance_width(&self, ch: char) -> f32 {
            self.font.metrics(ch, self.size).advance_width
        }

        pub fn measure_str(&self, s: &str) -> f32 {
            s.chars().map(|c| self.advance_width(c)).sum()
        }

        pub fn rasterize(&self, ch: char) -> GlyphBitmap {
            let (metrics, pixels) = self.font.rasterize(ch, self.size);
            GlyphBitmap {
                pixels,
                width: metrics.width,
                height: metrics.height,
                bearing_x: metrics.xmin,
                // bearing_y: distance from baseline to top of glyph (positive = above baseline)
                bearing_y: metrics.ymin + metrics.height as i32,
            }
        }
    }
}

#[cfg(feature = "feather-renderer")]
pub use inner::{FontMetrics, GlyphBitmap};
