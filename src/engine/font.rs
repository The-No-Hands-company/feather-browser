// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod inner {
    use fontdue::{Font, FontSettings};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::{Arc, OnceLock};

    const FONT_BYTES: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");

    static PARSED_FONT: OnceLock<Arc<Font>> = OnceLock::new();

    /// Maximum glyph bitmaps held per thread; the cache is cleared when full.
    const GLYPH_CACHE_CAP: usize = 512;

    fn shared_font() -> Arc<Font> {
        PARSED_FONT
            .get_or_init(|| {
                Arc::new(
                    Font::from_bytes(FONT_BYTES, FontSettings::default())
                        .expect("embedded NotoSans font bytes are valid"),
                )
            })
            .clone()
    }

    thread_local! {
        static METRICS_CACHE: RefCell<HashMap<u32, FontMetrics>> =
            RefCell::new(HashMap::new());
    }

    thread_local! {
        static GLYPH_CACHE: RefCell<HashMap<u64, Arc<GlyphBitmap>>> =
            RefCell::new(HashMap::new());
    }

    pub struct GlyphBitmap {
        pub pixels: Vec<u8>,
        pub width: usize,
        pub height: usize,
        pub bearing_x: i32,
        pub bearing_y: i32,
        pub advance_width: f32,
    }

    #[derive(Clone)]
    pub struct FontMetrics {
        pub size: f32,
        pub ascent: f32,
        pub descent: f32,
        pub line_height: f32,
        pub space_advance: f32,
        font: Arc<Font>,
    }

    impl FontMetrics {
        pub fn load(size: f32) -> Self {
            let size = if !size.is_finite() || size <= 0.0 {
                16.0
            } else {
                size.clamp(1.0, 512.0)
            };
            let font = shared_font();
            let lm = font
                .horizontal_line_metrics(size)
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
            FontMetrics {
                font,
                size,
                ascent,
                descent,
                line_height,
                space_advance,
            }
        }

        pub fn load_cached(size: f32) -> Self {
            let key = size.to_bits();
            METRICS_CACHE.with(|cache| {
                let mut map = cache.borrow_mut();
                if let Some(fm) = map.get(&key) {
                    return fm.clone();
                }
                let fm = FontMetrics::load(size);
                map.insert(key, fm.clone());
                fm
            })
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
                advance_width: metrics.advance_width,
            }
        }

        pub fn rasterize_cached(&self, ch: char) -> Arc<GlyphBitmap> {
            let key = (ch as u64) << 32 | self.size.to_bits() as u64;
            GLYPH_CACHE.with(|cache| {
                let mut map = cache.borrow_mut();
                if let Some(g) = map.get(&key) {
                    return g.clone();
                }
                if map.len() >= GLYPH_CACHE_CAP {
                    map.clear();
                }
                let g = Arc::new(self.rasterize(ch));
                map.insert(key, g.clone());
                g
            })
        }
    }
}

#[cfg(feature = "feather-renderer")]
pub use inner::{FontMetrics, GlyphBitmap};
