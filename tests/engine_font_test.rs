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
}
