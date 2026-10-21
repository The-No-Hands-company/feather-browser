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
        assert!(large.line_height > small.line_height,
            "32px line_height ({}) should exceed 12px ({})", large.line_height, small.line_height);
        assert!(large.advance_width('M') > small.advance_width('M'),
            "32px 'M' advance should exceed 12px");
    }

    #[test]
    fn test_glyph_advance_width_field_populated() {
        let fm = FontMetrics::load(16.0);
        let g = fm.rasterize('A');
        let from_method = fm.advance_width('A');
        assert!(
            (g.advance_width - from_method).abs() < 0.5,
            "GlyphBitmap.advance_width {:.2} should match advance_width('A') {:.2}",
            g.advance_width, from_method
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
            cached.advance_width, uncached.advance_width
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
            glyph.advance_width, from_metrics
        );
    }
}
