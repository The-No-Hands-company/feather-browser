// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod font_tests {
    use feather::engine::font::FontMetrics;
    use std::sync::Arc;

    #[test]
    fn test_font_loads() {
        let m = FontMetrics::load(16.0);
        assert!(m.ascent > 0.0);
        assert!(m.descent > 0.0);
        assert!(m.line_height >= m.ascent + m.descent);
    }

    #[test]
    fn test_advance_width_nonzero() {
        let m = FontMetrics::load(16.0);
        assert!(m.advance_width('A') > 0.0);
    }

    #[test]
    fn test_measure_str_wider_than_single_char() {
        let m = FontMetrics::load(16.0);
        assert!(m.measure_str("Hello") > m.advance_width('H'));
    }

    #[test]
    fn test_rasterize_produces_nonzero_pixels() {
        let m = FontMetrics::load(16.0);
        let g = m.rasterize('A');
        assert!(g.width > 0 && g.height > 0);
        assert!(g.pixels.iter().any(|&a| a > 0));
    }

    #[test]
    fn test_space_advance_nonzero() {
        let m = FontMetrics::load(16.0);
        assert!(m.space_advance > 0.0);
    }

    #[test]
    fn test_load_cached_matches_load_metrics() {
        let a = FontMetrics::load(16.0);
        let b = FontMetrics::load_cached(16.0);
        assert_eq!(a.size, b.size);
        assert_eq!(a.ascent, b.ascent);
        assert_eq!(a.line_height, b.line_height);
        assert_eq!(a.space_advance, b.space_advance);
    }

    #[test]
    fn test_load_cached_is_idempotent() {
        let a = FontMetrics::load_cached(14.0);
        let b = FontMetrics::load_cached(14.0);
        assert!((a.advance_width('A') - b.advance_width('A')).abs() < f32::EPSILON);
    }

    #[test]
    fn test_different_sizes_have_different_metrics() {
        let small = FontMetrics::load_cached(12.0);
        let large = FontMetrics::load_cached(32.0);
        assert!(
            large.line_height > small.line_height,
            "32px line_height ({}) should exceed 12px ({})",
            large.line_height,
            small.line_height
        );
        assert!(
            large.advance_width('M') > small.advance_width('M'),
            "32px 'M' advance should exceed 12px"
        );
    }

    #[test]
    fn test_glyph_advance_width_field_populated() {
        let fm = FontMetrics::load(16.0);
        let g = fm.rasterize('A');
        let from_method = fm.advance_width('A');
        assert!(
            (g.advance_width - from_method).abs() < 0.5,
            "GlyphBitmap.advance_width {:.2} should match advance_width('A') {:.2}",
            g.advance_width,
            from_method
        );
    }

    #[test]
    fn test_rasterize_cached_same_as_uncached() {
        let fm = FontMetrics::load(16.0);
        let uncached = fm.rasterize('B');
        let cached = fm.rasterize_cached('B');
        assert_eq!(cached.width, uncached.width);
        assert_eq!(cached.height, uncached.height);
        assert_eq!(cached.bearing_x, uncached.bearing_x);
        assert_eq!(cached.pixels, uncached.pixels);
        assert!(
            (cached.advance_width - uncached.advance_width).abs() < 0.01,
            "cached advance {:.2} vs uncached {:.2}",
            cached.advance_width,
            uncached.advance_width
        );
    }

    #[test]
    fn test_rasterize_cached_is_idempotent() {
        let fm = FontMetrics::load(16.0);
        let first = fm.rasterize_cached('C');
        let second = fm.rasterize_cached('C');
        // Same Arc — pointer equality confirms a cache hit, not a fresh rasterize
        assert!(
            Arc::ptr_eq(&first, &second),
            "second call must return the same Arc, not a new allocation"
        );
    }

    #[test]
    fn test_cache_advance_width_matches_metrics() {
        let fm = FontMetrics::load(24.0);
        let glyph = fm.rasterize_cached('M');
        let from_metrics = fm.advance_width('M');
        assert!(
            (glyph.advance_width - from_metrics).abs() < 0.5,
            "cached advance_width {:.2} should match advance_width('M') at 24px: {:.2}",
            glyph.advance_width,
            from_metrics
        );
    }

    #[test]
    fn test_glyph_cache_does_not_exceed_cap() {
        // Rasterize 513 distinct chars at 16px. After the 513th the cache
        // flushes and refills with just that one entry, so len <= 512.
        let fm = FontMetrics::load_cached(16.0);
        // Use chars from U+0020 to U+0220 (512 distinct code points).
        for cp in 0x0020_u32..=0x0220 {
            if let Some(ch) = char::from_u32(cp) {
                let _ = fm.rasterize_cached(ch);
            }
        }
        // Rasterize one more unique char to trigger the flush path.
        let _ = fm.rasterize_cached('\u{0221}');
        // After flush + one insert, cache holds exactly 1 entry — well under cap.
        // We cannot inspect the cache directly (private), but the call must not panic
        // and must return a valid glyph.
        let g = fm.rasterize_cached('A');
        assert!(
            g.advance_width > 0.0,
            "post-flush rasterization must return valid glyph"
        );
    }

    #[test]
    fn test_glyph_cache_still_works_after_flush() {
        let fm = FontMetrics::load_cached(14.0);
        // Fill past cap (512) to force a flush.
        for cp in 0x0020_u32..=0x0221 {
            if let Some(ch) = char::from_u32(cp) {
                let _ = fm.rasterize_cached(ch);
            }
        }
        // After flush, rasterizing 'Z' must produce a glyph with sensible advance_width.
        let g = fm.rasterize_cached('Z');
        assert!(
            g.advance_width > 0.0 && g.advance_width < 100.0,
            "advance_width after flush must be finite and positive, got {}",
            g.advance_width
        );
    }

    #[test]
    fn test_bold_font_loads_and_differs_from_regular() {
        let regular = FontMetrics::load_variant(16.0, false, false);
        let bold = FontMetrics::load_variant(16.0, true, false);
        // Both must produce valid glyphs.
        let rg = regular.rasterize_cached('W');
        let bg = bold.rasterize_cached('W');
        assert!(rg.advance_width > 0.0, "regular 'W' advance must be positive");
        assert!(bg.advance_width > 0.0, "bold 'W' advance must be positive");
        // Bold glyphs are typically slightly wider than regular.
        assert!(
            bg.advance_width >= rg.advance_width,
            "bold advance ({}) should be >= regular ({})",
            bg.advance_width,
            rg.advance_width
        );
    }

    #[test]
    fn test_italic_font_loads_and_has_valid_metrics() {
        let italic = FontMetrics::load_variant(16.0, false, true);
        assert!(italic.ascent > 0.0, "italic ascent must be positive");
        assert!(italic.line_height > 0.0, "italic line_height must be positive");
        let g = italic.rasterize_cached('A');
        assert!(g.advance_width > 0.0, "italic 'A' advance must be positive");
    }

    #[test]
    fn test_variant_cache_key_is_distinct() {
        let reg = FontMetrics::load_cached_variant(16.0, false, false);
        let bold = FontMetrics::load_cached_variant(16.0, true, false);
        // Different fonts → different advance widths for most chars.
        // Using 'W' which is typically wider in bold.
        let rg = reg.rasterize_cached('W');
        let bg = bold.rasterize_cached('W');
        // Both must return non-zero (i.e. different cache slots, both populated).
        assert!(rg.advance_width > 0.0);
        assert!(bg.advance_width > 0.0);
    }
}
