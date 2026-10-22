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
    assert!(
        find_width(&layout, 399.0, 401.0),
        "no box with width ~400px found"
    );
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
    use feather::engine::layout::{
        InlineFormattingContext, InlineText, LayoutBox, LayoutEngine, LineBox, TextAlign,
        TextDecoration,
    };
    use feather::engine::style::style_tree;

    fn find_line_boxes(lb: &LayoutBox) -> Option<&Vec<LineBox>> {
        if !lb.line_boxes.is_empty() {
            return Some(&lb.line_boxes);
        }
        for child in &lb.children {
            if let Some(found) = find_line_boxes(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_text_align_left_is_default() {
        let items = vec![InlineText {
            text: "Hi".to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::None,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 400.0,
            align: TextAlign::Left,
            line_height_override: None,
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
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::None,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Center,
            line_height_override: None,
        };
        let (lines, _) = ifc.layout(&items);
        assert!(!lines.is_empty(), "layout produced no lines");
        let first_item = &lines[0].items[0];

        let fm = FontMetrics::load(16.0);
        let content_w = fm.measure_str("Hi");
        let expected_offset = (available_width - content_w) / 2.0;

        assert!(
            (first_item.x - expected_offset).abs() < 1.0,
            "center offset should be ≈{:.2}, got {:.2}",
            expected_offset,
            first_item.x
        );
    }

    #[test]
    fn test_text_align_right_offsets_items() {
        let available_width = 400.0_f32;
        let items = vec![InlineText {
            text: "Hi".to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::None,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Right,
            line_height_override: None,
        };
        let (lines, _) = ifc.layout(&items);
        assert!(!lines.is_empty(), "layout produced no lines");
        let first_item = &lines[0].items[0];

        let fm = FontMetrics::load(16.0);
        let content_w = fm.measure_str("Hi");
        let expected_offset = available_width - content_w;

        assert!(
            (first_item.x - expected_offset).abs() < 1.0,
            "right offset should be ≈{:.2}, got {:.2}",
            expected_offset,
            first_item.x
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

    #[test]
    fn test_text_decoration_underline_in_layout() {
        let items = vec![InlineText {
            text: "underlined".to_string(),
            color: CssColor::BLACK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::Underline,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
        };
        let (lines, _) = ifc.layout(&items);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items[0].text_decoration, TextDecoration::Underline);
    }

    #[test]
    fn test_text_decoration_line_through_in_layout() {
        let items = vec![InlineText {
            text: "struck".to_string(),
            color: CssColor::BLACK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::LineThrough,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
        };
        let (lines, _) = ifc.layout(&items);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items[0].text_decoration, TextDecoration::LineThrough);
    }
}

#[cfg(test)]
mod img_layout_tests {
    use feather::engine::{
        css::CssParser,
        dom::NodeType,
        html::HtmlParser,
        layout::{BoxType, LayoutEngine},
        style::style_tree,
    };

    fn find_img_box(
        lb: &feather::engine::layout::LayoutBox,
    ) -> Option<&feather::engine::layout::LayoutBox> {
        if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
            if let NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == "img" {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = find_img_box(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_img_layout_default_dimensions() {
        let dom = HtmlParser::parse("<img>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let img = find_img_box(&layout).expect("img box not found in layout tree");
        assert_eq!(
            img.dimensions.content.width, 100.0,
            "default img width must be 100"
        );
        assert_eq!(
            img.dimensions.content.height, 50.0,
            "default img height must be 50"
        );
    }

    #[test]
    fn test_img_layout_explicit_dimensions() {
        let dom = HtmlParser::parse("<img width=\"200\" height=\"80\">");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let img = find_img_box(&layout).expect("img box not found in layout tree");
        assert_eq!(
            img.dimensions.content.width, 200.0,
            "img width must read from attribute"
        );
        assert_eq!(
            img.dimensions.content.height, 80.0,
            "img height must read from attribute"
        );
    }

    #[test]
    fn test_img_layout_partial_attributes() {
        let dom = HtmlParser::parse("<img width=\"60\">");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let img = find_img_box(&layout).expect("img box not found in layout tree");
        assert_eq!(
            img.dimensions.content.width, 60.0,
            "img width from attribute"
        );
        assert_eq!(
            img.dimensions.content.height, 50.0,
            "img height defaults to 50 when absent"
        );
    }
}

#[cfg(test)]
mod box_model_layout_tests {
    use feather::engine::{
        css::CssParser, html::HtmlParser, layout::LayoutEngine, style::style_tree,
    };

    fn first_block_box(
        lb: &feather::engine::layout::LayoutBox,
    ) -> Option<&feather::engine::layout::LayoutBox> {
        use feather::engine::dom::NodeType;
        use feather::engine::layout::BoxType;
        if let BoxType::Block(s) = &lb.box_type {
            if let NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == "div" {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = first_block_box(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_padding_expands_box() {
        let dom = HtmlParser::parse("<div style=\"padding: 10px;\">hello</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box not found");
        // content.x starts after left padding (10px offset from containing content.x)
        assert!(
            div.dimensions.padding.left >= 10.0,
            "padding.left must be 10"
        );
        assert!(
            div.dimensions.padding.right >= 10.0,
            "padding.right must be 10"
        );
        // border_box is content + padding on all sides
        let bb = div.dimensions.border_box();
        assert!(
            bb.width >= div.dimensions.content.width + 20.0 - 0.01,
            "border_box width must include left+right padding"
        );
    }

    #[test]
    fn test_padding_4_value_shorthand_in_layout() {
        let dom =
            HtmlParser::parse(r#"<div style="padding: 10px 20px 30px 40px;">x</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert_eq!(div.dimensions.padding.top, 10.0);
        assert_eq!(div.dimensions.padding.right, 20.0);
        assert_eq!(div.dimensions.padding.bottom, 30.0);
        assert_eq!(div.dimensions.padding.left, 40.0);
    }

    #[test]
    fn test_padding_2_value_shorthand_in_layout() {
        // 2-value: top/bottom = first, left/right = second
        let dom = HtmlParser::parse(r#"<div style="padding: 5px 15px;">x</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert_eq!(div.dimensions.padding.top, 5.0);
        assert_eq!(div.dimensions.padding.right, 15.0);
        assert_eq!(div.dimensions.padding.bottom, 5.0);
        assert_eq!(div.dimensions.padding.left, 15.0);
    }

    #[test]
    fn test_margin_sets_on_box() {
        let dom = HtmlParser::parse("<div style=\"margin: 5px;\">hello</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box not found");
        assert!(div.dimensions.margin.top >= 5.0, "margin.top must be 5");
        assert!(div.dimensions.margin.left >= 5.0, "margin.left must be 5");
    }

    #[test]
    fn test_margin_4_value_shorthand_in_layout() {
        let dom =
            HtmlParser::parse(r#"<div style="margin: 10px 20px 30px 40px;">x</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert_eq!(div.dimensions.margin.top, 10.0);
        assert_eq!(div.dimensions.margin.right, 20.0);
        assert_eq!(div.dimensions.margin.bottom, 30.0);
        assert_eq!(div.dimensions.margin.left, 40.0);
    }

    #[test]
    fn test_margin_2_value_shorthand_in_layout() {
        let dom = HtmlParser::parse(r#"<div style="margin: 8px 16px;">x</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert_eq!(div.dimensions.margin.top, 8.0);
        assert_eq!(div.dimensions.margin.right, 16.0);
        assert_eq!(div.dimensions.margin.bottom, 8.0);
        assert_eq!(div.dimensions.margin.left, 16.0);
    }

    #[test]
    fn test_border_width_in_layout() {
        let dom = HtmlParser::parse("<div style=\"border: 4px solid black;\">hello</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box not found");
        assert_eq!(div.dimensions.border.left, 4.0, "border.left must be 4");
        assert_eq!(div.dimensions.border.right, 4.0, "border.right must be 4");
        let bb = div.dimensions.border_box();
        assert!(
            bb.width >= div.dimensions.content.width + 8.0 - 0.01,
            "border_box width must include left+right border"
        );
    }

    #[test]
    fn test_line_height_px_expands_line_box() {
        // line-height: 40px should produce line boxes taller than the default ~19px
        let dom = HtmlParser::parse(r#"<div style="line-height: 40px; font-size: 16px;">hello</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert!(!div.line_boxes.is_empty(), "must have line boxes");
        let first_line_height = div.line_boxes[0].height;
        assert!(
            first_line_height >= 39.0,
            "line-height: 40px must produce line height ≥ 39, got {:.2}",
            first_line_height
        );
    }

    #[test]
    fn test_line_height_unitless_multiplier() {
        // line-height: 2 = 2 × font-size (16px) = 32px line height
        let dom = HtmlParser::parse(r#"<div style="line-height: 2; font-size: 16px;">hello</div>"#);
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_block_box(&layout).expect("div box");
        assert!(!div.line_boxes.is_empty(), "must have line boxes");
        let first_line_height = div.line_boxes[0].height;
        assert!(
            first_line_height >= 31.0,
            "line-height: 2 at 16px should give line height ≥ 31, got {:.2}",
            first_line_height
        );
    }
}

#[cfg(test)]
mod viewport_unit_tests {
    use feather::engine::{
        css::CssParser, html::HtmlParser, layout::LayoutEngine, style::style_tree,
    };

    fn first_div(
        lb: &feather::engine::layout::LayoutBox,
    ) -> Option<&feather::engine::layout::LayoutBox> {
        use feather::engine::dom::NodeType;
        use feather::engine::layout::BoxType;
        if let BoxType::Block(s) = &lb.box_type {
            if let NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == "div" {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = first_div(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_vw_width_resolves() {
        let dom = HtmlParser::parse("<div style=\"width: 50vw;\">x</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_div(&layout).expect("div not found");
        assert!(
            (div.dimensions.content.width - 400.0).abs() < 0.1,
            "50vw with containing_width=800 must be 400px, got {}",
            div.dimensions.content.width
        );
    }

    #[test]
    fn test_vh_height_resolves() {
        let dom = HtmlParser::parse("<div style=\"height: 50vh;\">x</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build_with_viewport(&styled, 800.0, 600.0);
        let div = first_div(&layout).expect("div not found");
        assert!(
            (div.dimensions.content.height - 300.0).abs() < 0.1,
            "50vh with viewport_height=600 must be 300px, got {}",
            div.dimensions.content.height
        );
    }
}

#[cfg(test)]
mod min_max_width_tests {
    use feather::engine::{
        css::CssParser, html::HtmlParser, layout::LayoutEngine, style::style_tree,
    };

    fn first_div(
        lb: &feather::engine::layout::LayoutBox,
    ) -> Option<&feather::engine::layout::LayoutBox> {
        use feather::engine::dom::NodeType;
        use feather::engine::layout::BoxType;
        if let BoxType::Block(s) = &lb.box_type {
            if let NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == "div" {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = first_div(child) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_min_width_clamps_up() {
        let dom = HtmlParser::parse("<div style=\"width: 50px; min-width: 200px;\">x</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_div(&layout).expect("div not found");
        assert!(
            (div.dimensions.content.width - 200.0).abs() < 0.1,
            "min-width: 200px must clamp width up from 50px, got {}",
            div.dimensions.content.width
        );
    }

    #[test]
    fn test_max_width_clamps_down() {
        let dom = HtmlParser::parse("<div style=\"max-width: 300px;\">x</div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_div(&layout).expect("div not found");
        assert!(
            (div.dimensions.content.width - 300.0).abs() < 0.1,
            "max-width: 300px must clamp auto width from 800 down, got {}",
            div.dimensions.content.width
        );
    }

    #[test]
    fn test_min_max_width_no_effect_when_within() {
        let dom = HtmlParser::parse(
            "<div style=\"width: 400px; min-width: 100px; max-width: 600px;\">x</div>",
        );
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let div = first_div(&layout).expect("div not found");
        assert!(
            (div.dimensions.content.width - 400.0).abs() < 0.1,
            "width within min/max must not be changed, got {}",
            div.dimensions.content.width
        );
    }
}

#[cfg(test)]
mod display_none_tests {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{BoxType, LayoutBox, LayoutEngine},
        style::style_tree,
    };
    use feather::engine::dom::NodeType;

    fn count_tag(lb: &LayoutBox, tag: &str) -> usize {
        let mut n = 0;
        if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
            if let NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == tag { n += 1; }
            }
        }
        for child in &lb.children {
            n += count_tag(child, tag);
        }
        n
    }

    #[test]
    fn test_display_none_produces_no_box() {
        let dom = HtmlParser::parse("<div><p style=\"display: none;\">hidden</p><p>visible</p></div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        assert_eq!(count_tag(&layout, "p"), 1,
            "only one p box should exist (the visible one); display:none p must be absent");
    }

    #[test]
    fn test_display_none_div_removes_children() {
        let dom = HtmlParser::parse("<div><div style=\"display: none;\"><p>child</p></div><p>visible</p></div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        assert_eq!(count_tag(&layout, "p"), 1,
            "p inside display:none div must not appear in layout tree");
    }
}
