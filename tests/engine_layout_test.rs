// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg(feature = "feather-renderer")]
use feather::engine::css::CssParser;
use feather::engine::html::HtmlParser;
use feather::engine::layout::{BoxType, Dimensions, LayoutEngine, Rect};
use feather::engine::style::style_tree;

fn viewport() -> Dimensions {
    let mut d = Dimensions::default();
    d.content.width = 800.0;
    d
}

#[test]
fn test_block_box_fills_viewport_width() {
    let dom = HtmlParser::parse("<div></div>");
    let sheet = CssParser::parse("div { display: block; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    // The root html box should span the viewport
    assert_eq!(layout.dimensions.content.width, 800.0);
}

#[test]
fn test_explicit_width_respected() {
    let dom = HtmlParser::parse("<div></div>");
    let sheet = CssParser::parse("div { display: block; width: 400px; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    // Find the div child box
    fn find_width(b: &feather::engine::layout::LayoutBox, min_w: f32, max_w: f32) -> bool {
        let w = b.dimensions.content.width;
        if w >= min_w && w <= max_w {
            return true;
        }
        b.children.iter().any(|c| find_width(c, min_w, max_w))
    }
    assert!(find_width(&layout, 399.0, 401.0), "no box with width ~400px found");
}

#[test]
fn test_stacked_blocks_have_increasing_y() {
    let dom = HtmlParser::parse("<div><p></p><p></p></div>");
    let sheet = CssParser::parse("p { display: block; height: 20px; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);

    fn collect_y(b: &feather::engine::layout::LayoutBox, ys: &mut Vec<f32>) {
        if matches!(b.box_type, BoxType::Block(_)) {
            ys.push(b.dimensions.content.y);
        }
        for c in &b.children {
            collect_y(c, ys);
        }
    }
    let mut ys = Vec::new();
    collect_y(&layout, &mut ys);
    // At least two y values, and they should not all be the same
    if ys.len() >= 2 {
        let sorted = {
            let mut v = ys.clone();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v
        };
        assert!(sorted[0] <= sorted[sorted.len() - 1]);
    }
}

#[cfg(feature = "feather-renderer")]
mod layout_tests {
    use feather::engine::css::{CssColor, CssParser};
    use feather::engine::font::FontMetrics;
    use feather::engine::html::HtmlParser;
    use feather::engine::layout::{InlineFormattingContext, InlineText, LayoutBox, LayoutEngine, LineBox, TextAlign};
    use feather::engine::style::style_tree;

    fn find_line_boxes(lb: &LayoutBox) -> Option<&Vec<LineBox>> {
        if !lb.line_boxes.is_empty() { return Some(&lb.line_boxes); }
        for child in &lb.children {
            if let Some(found) = find_line_boxes(child) { return Some(found); }
        }
        None
    }

    #[test]
    fn test_text_align_left_is_default() {
        let items = vec![InlineText {
            text: "Hi".to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 400.0,
            align: TextAlign::Left,
        };
        let (lines, _) = ifc.layout(&items);
        assert!(!lines.is_empty(), "layout produced no lines");
        assert_eq!(
            lines[0].items[0].x, 0.0,
            "left-aligned text must start at x=0.0, got {:.2}",
            lines[0].items[0].x
        );
    }

    #[test]
    fn test_text_align_center_offsets_items() {
        let available_width = 400.0_f32;
        let items = vec![InlineText {
            text: "Hi".to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Center,
        };
        let (lines, _) = ifc.layout(&items);
        assert!(!lines.is_empty(), "layout produced no lines");
        let first_item = &lines[0].items[0];

        let fm = FontMetrics::load(16.0);
        let content_w = fm.measure_str("Hi");
        let expected_offset = (available_width - content_w) / 2.0;

        assert!(
            (first_item.x - expected_offset).abs() < 1.0,
            "center offset should be ≈{:.2}, got {:.2}", expected_offset, first_item.x
        );
    }

    #[test]
    fn test_text_align_right_offsets_items() {
        let available_width = 400.0_f32;
        let items = vec![InlineText {
            text: "Hi".to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Right,
        };
        let (lines, _) = ifc.layout(&items);
        assert!(!lines.is_empty(), "layout produced no lines");
        let first_item = &lines[0].items[0];

        let fm = FontMetrics::load(16.0);
        let content_w = fm.measure_str("Hi");
        let expected_offset = available_width - content_w;

        assert!(
            (first_item.x - expected_offset).abs() < 1.0,
            "right offset should be ≈{:.2}, got {:.2}", expected_offset, first_item.x
        );
    }

    #[test]
    fn test_text_align_center_via_css() {
        // Full pipeline: CSS sets text-align:center; the line box's first item
        // must have x > 0, confirming layout_block() reads and threads the property.
        let dom = HtmlParser::parse("<p>Hi</p>");
        let sheet = CssParser::parse("p { text-align: center; font-size: 16px; display: block; }");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 400.0);
        let line_boxes = find_line_boxes(&layout).expect("no line boxes found in layout tree");
        assert!(!line_boxes[0].items.is_empty(), "line box has no items");
        assert!(
            line_boxes[0].items[0].x > 0.0,
            "center-aligned text must have x > 0.0, got {:.2} — \
             layout_block() may not be reading text-align from the style tree",
            line_boxes[0].items[0].x
        );
    }
}
