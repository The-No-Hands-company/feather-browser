// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod inline_tests {
    use feather::engine::css::CssColor;
    use feather::engine::layout::{InlineFormattingContext, InlineText};

    fn text_item(s: &str) -> InlineText {
        InlineText { text: s.to_string(), color: CssColor::BLACK, font_size: 16.0 }
    }

    #[test]
    fn test_single_word_fits_one_line() {
        let ifc = InlineFormattingContext { font_size: 16.0, available_width: 800.0 };
        let (lines, _) = ifc.layout(&[text_item("Hello")]);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items[0].text, "Hello");
    }

    #[test]
    fn test_long_sentence_wraps_to_multiple_lines() {
        let ifc = InlineFormattingContext { font_size: 16.0, available_width: 80.0 };
        let (lines, height) = ifc.layout(&[text_item("The quick brown fox jumps over the lazy dog")]);
        assert!(lines.len() > 1, "expected multiple lines, got {}", lines.len());
        assert!(height > 16.0 * 1.2, "expected height > one line");
    }

    #[test]
    fn test_empty_text_produces_no_lines() {
        let ifc = InlineFormattingContext { font_size: 16.0, available_width: 800.0 };
        let (lines, height) = ifc.layout(&[text_item("")]);
        assert_eq!(lines.len(), 0);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_items_x_positions_increase_left_to_right() {
        let ifc = InlineFormattingContext { font_size: 16.0, available_width: 800.0 };
        let (lines, _) = ifc.layout(&[text_item("one two three")]);
        assert_eq!(lines.len(), 1);
        let xs: Vec<f32> = lines[0].items.iter().map(|i| i.x).collect();
        for w in xs.windows(2) {
            assert!(w[1] > w[0], "x positions must increase: {:?}", xs);
        }
    }

    #[test]
    fn test_line_height_matches_font() {
        let ifc = InlineFormattingContext { font_size: 16.0, available_width: 800.0 };
        let (lines, _) = ifc.layout(&[text_item("X")]);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].height > 0.0);
    }
}
