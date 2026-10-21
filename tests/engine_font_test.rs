// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod font_tests {
    use feather::engine::font::FontMetrics;

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
}
