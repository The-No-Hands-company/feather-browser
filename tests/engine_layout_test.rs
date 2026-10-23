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
            force_break: false,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 400.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: false,
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
            force_break: false,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Center,
            line_height_override: None,
            nowrap: false,
            pre: false,
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
            force_break: false,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width,
            align: TextAlign::Right,
            line_height_override: None,
            nowrap: false,
            pre: false,
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
            force_break: false,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: false,
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
            force_break: false,
        }];
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: false,
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

#[cfg(test)]
mod position_relative_tests {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{BoxType, LayoutBox, LayoutEngine},
        style::style_tree,
    };

    fn find_box<'a>(lb: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
        if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
            if let feather::engine::dom::NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == tag {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = find_box(child, tag) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_position_relative_top_left_offsets_content() {
        let dom = HtmlParser::parse(
            "<div><p style=\"position: relative; top: 10px; left: 20px;\">hi</p></div>",
        );
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);

        let p_box = find_box(&layout, "p").expect("p box not found");
        assert!(
            p_box.dimensions.content.x >= 20.0,
            "left: 20px must shift x by ≥20, got {:.2}",
            p_box.dimensions.content.x
        );
        assert!(
            p_box.dimensions.content.y >= 10.0,
            "top: 10px must shift y by ≥10, got {:.2}",
            p_box.dimensions.content.y
        );
    }

    #[test]
    fn test_position_static_no_offset() {
        let dom = HtmlParser::parse("<div><p>hi</p></div>");
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);

        let p_box = find_box(&layout, "p").expect("p box not found");
        assert_eq!(
            p_box.dimensions.content.x, 0.0,
            "static p must have x=0, got {:.2}",
            p_box.dimensions.content.x
        );
    }

    #[test]
    fn test_position_relative_negative_top_shifts_up() {
        let dom = HtmlParser::parse(
            "<div><p style=\"position: relative; top: -5px;\">hi</p></div>",
        );
        let sheet = CssParser::parse("");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);

        let p_box = find_box(&layout, "p").expect("p box not found");
        // Normal flow y would be 0 (first child); top: -5px shifts it up to -5
        assert!(
            p_box.dimensions.content.y < 0.0,
            "top: -5px must yield y < 0, got {:.2}",
            p_box.dimensions.content.y
        );
    }
}

#[cfg(test)]
mod list_marker_tests {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{BoxType, LayoutBox, LayoutEngine},
        style::style_tree,
    };

    fn find_box<'a>(lb: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
        if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
            if let feather::engine::dom::NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == tag {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = find_box(child, tag) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_ul_li_disc_marker_in_line_box() {
        let dom = HtmlParser::parse("<ul><li>Item one</li></ul>");
        let sheet = CssParser::parse(
            "ul{display:block;padding-left:40px;list-style-type:disc}li{display:block}",
        );
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let li_box = find_box(&layout, "li").expect("li box not found");
        // The first line box item must start with the disc bullet character.
        assert!(
            !li_box.line_boxes.is_empty(),
            "li must have at least one line box"
        );
        let first_text = &li_box.line_boxes[0].items[0].text;
        assert!(
            first_text.starts_with('•'),
            "first item text must start with disc bullet '•', got {:?}",
            first_text
        );
    }

    #[test]
    fn test_ol_li_decimal_marker_in_line_box() {
        let dom = HtmlParser::parse("<ol><li>First</li><li>Second</li></ol>");
        let sheet = CssParser::parse(
            "ol{display:block;padding-left:40px;list-style-type:decimal}li{display:block}",
        );
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        // Collect all li boxes in document order.
        fn collect_li<'a>(lb: &'a LayoutBox, out: &mut Vec<&'a LayoutBox>) {
            if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
                if let feather::engine::dom::NodeType::Element(e) = &s.node.node_type {
                    if e.tag_name == "li" {
                        out.push(lb);
                    }
                }
            }
            for child in &lb.children {
                collect_li(child, out);
            }
        }
        let mut lis = Vec::new();
        collect_li(&layout, &mut lis);
        assert_eq!(lis.len(), 2, "must find two li boxes");
        // First li: marker "1. "
        assert!(
            !lis[0].line_boxes.is_empty(),
            "first li must have line boxes"
        );
        let first_marker = &lis[0].line_boxes[0].items[0].text;
        assert!(
            first_marker.starts_with("1."),
            "first li marker must start with '1.', got {:?}",
            first_marker
        );
        // Second li: marker "2. "
        let second_marker = &lis[1].line_boxes[0].items[0].text;
        assert!(
            second_marker.starts_with("2."),
            "second li marker must start with '2.', got {:?}",
            second_marker
        );
    }

    #[test]
    fn test_ul_li_no_marker_without_list_style() {
        let dom = HtmlParser::parse("<ul><li>Item</li></ul>");
        // No list-style-type set, so no marker should be injected.
        let sheet = CssParser::parse("ul{display:block}li{display:block}");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let li_box = find_box(&layout, "li").expect("li box not found");
        if !li_box.line_boxes.is_empty() {
            let first_text = &li_box.line_boxes[0].items[0].text;
            assert!(
                !first_text.starts_with('•'),
                "without list-style-type:disc no bullet must appear, got {:?}",
                first_text
            );
        }
    }
}

#[cfg(test)]
mod br_hr_tests {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{BoxType, LayoutBox, LayoutEngine},
        style::style_tree,
    };

    fn find_line_boxes_in<'a>(lb: &'a LayoutBox) -> Option<&'a Vec<feather::engine::layout::LineBox>> {
        if !lb.line_boxes.is_empty() {
            return Some(&lb.line_boxes);
        }
        for child in &lb.children {
            if let Some(found) = find_line_boxes_in(child) {
                return Some(found);
            }
        }
        None
    }

    fn find_box_by_tag<'a>(lb: &'a LayoutBox, tag: &str) -> Option<&'a LayoutBox> {
        if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
            if let feather::engine::dom::NodeType::Element(e) = &s.node.node_type {
                if e.tag_name == tag {
                    return Some(lb);
                }
            }
        }
        for child in &lb.children {
            if let Some(found) = find_box_by_tag(child, tag) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_br_forces_second_line_box() {
        // Two text runs separated by <br> must produce two line boxes.
        let dom = HtmlParser::parse("<p>First line<br>Second line</p>");
        let sheet = CssParser::parse("p{display:block}br{display:inline}");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let lines = find_line_boxes_in(&layout).expect("no line boxes in layout");
        assert!(
            lines.len() >= 2,
            "<br> must produce at least 2 line boxes, got {}",
            lines.len()
        );
    }

    #[test]
    fn test_br_only_produces_empty_line() {
        // <br> at the start: second paragraph should start on a new line.
        let dom = HtmlParser::parse("<p>line1<br>line2</p>");
        let sheet = CssParser::parse("p{display:block}br{display:inline}");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let lines = find_line_boxes_in(&layout).expect("no line boxes");
        // line 0 has "line1", line 1 has "line2"
        assert!(
            lines.len() >= 2,
            "expected ≥2 line boxes after <br>, got {}",
            lines.len()
        );
        assert!(
            lines[0].y < lines[1].y,
            "second line box must be lower than the first (y={} vs y={})",
            lines[0].y,
            lines[1].y
        );
    }

    #[test]
    fn test_hr_ua_rules_apply() {
        let dom = HtmlParser::parse("<hr>");
        let ua = "hr{display:block;border-top-width:2px;border-color:#888888;margin-top:8px;margin-bottom:8px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 800.0);
        let hr_box = find_box_by_tag(&layout, "hr").expect("hr box not found");
        // border-top-width must be stored in dimensions.border.top
        assert_eq!(
            hr_box.dimensions.border.top,
            2.0,
            "hr border-top must be 2px, got {}",
            hr_box.dimensions.border.top
        );
        // margin-top must be 8px
        assert_eq!(
            hr_box.dimensions.margin.top,
            8.0,
            "hr margin-top must be 8px, got {}",
            hr_box.dimensions.margin.top
        );
    }
}

#[cfg(feature = "feather-renderer")]
mod white_space_tests {
    use feather::engine::{
        css::{CssColor, CssParser},
        html::HtmlParser,
        layout::{InlineFormattingContext, InlineText, LayoutEngine, TextAlign, TextDecoration},
        style::style_tree,
    };

    fn text_item(s: &str) -> InlineText {
        InlineText {
            text: s.to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::None,
            force_break: false,
        }
    }

    #[test]
    fn test_nowrap_ifc_produces_single_line() {
        // With nowrap:true even a long sentence on a narrow available_width must stay on one line.
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 80.0, // too narrow for the sentence without wrap
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: true,
            pre: false,
        };
        let (lines, _) = ifc.layout(&[text_item("The quick brown fox jumps over the lazy dog")]);
        assert_eq!(lines.len(), 1, "nowrap must keep all text on one line regardless of width; got {} lines", lines.len());
    }

    #[test]
    fn test_normal_wrap_ifc_produces_multiple_lines() {
        // Sanity-check: same sentence with nowrap:false wraps on the same narrow width.
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 80.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: false,
        };
        let (lines, _) = ifc.layout(&[text_item("The quick brown fox jumps over the lazy dog")]);
        assert!(lines.len() > 1, "normal wrap must wrap text; got {} line(s)", lines.len());
    }

    #[test]
    fn test_white_space_nowrap_via_css_pipeline() {
        // Full pipeline: CSS white-space:nowrap on a narrow container must yield one line box.
        let dom = HtmlParser::parse("<p>The quick brown fox jumps over the lazy dog</p>");
        let sheet = CssParser::parse("p { display: block; white-space: nowrap; font-size: 16px; }");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 80.0); // 80px — far too narrow to fit
        // Walk the tree looking for a layout box with line_boxes.
        fn count_lines(lb: &feather::engine::layout::LayoutBox) -> usize {
            let here = lb.line_boxes.len();
            lb.children.iter().map(count_lines).fold(here, |a, b| a.max(b))
        }
        let max_lines = count_lines(&layout);
        assert_eq!(max_lines, 1, "white-space:nowrap must produce 1 line box; got {max_lines}");
    }
}

#[cfg(feature = "feather-renderer")]
mod white_space_pre_tests {
    use feather::engine::{
        css::{CssColor, CssParser},
        html::HtmlParser,
        layout::{InlineFormattingContext, InlineText, LayoutEngine, TextAlign, TextDecoration},
        style::style_tree,
    };

    fn text_item(s: &str) -> InlineText {
        InlineText {
            text: s.to_string(),
            color: CssColor::DARK,
            font_size: 16.0,
            font_bold: false,
            font_italic: false,
            text_decoration: TextDecoration::None,
            force_break: false,
        }
    }

    #[test]
    fn test_pre_ifc_newline_produces_two_lines() {
        // An InlineText whose text contains "\n" must produce 2 line boxes when pre:true.
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: true,
        };
        let (lines, _) = ifc.layout(&[text_item("line one\nline two")]);
        assert_eq!(lines.len(), 2, "pre IFC must split on \\n; got {} lines", lines.len());
        assert_eq!(lines[0].items[0].text, "line one");
        assert_eq!(lines[1].items[0].text, "line two");
    }

    #[test]
    fn test_pre_ifc_no_wrap_on_narrow_width() {
        // pre:true on a narrow IFC must not wrap — the single logical line stays as one line box.
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 60.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: true,
        };
        let (lines, _) = ifc.layout(&[text_item("The quick brown fox")]);
        assert_eq!(lines.len(), 1, "pre must not auto-wrap; got {} lines", lines.len());
    }

    #[test]
    fn test_pre_ifc_multiple_newlines_produce_correct_line_count() {
        let ifc = InlineFormattingContext {
            font_size: 16.0,
            available_width: 800.0,
            align: TextAlign::Left,
            line_height_override: None,
            nowrap: false,
            pre: true,
        };
        let (lines, _) = ifc.layout(&[text_item("a\nb\nc")]);
        assert_eq!(lines.len(), 3, "3 segments from 2 newlines must yield 3 lines; got {}", lines.len());
    }

    #[test]
    fn test_white_space_pre_via_css_pipeline() {
        // CSS white-space:pre on a narrow container must not wrap.
        let dom = HtmlParser::parse("<p>The quick brown fox jumps over the lazy dog</p>");
        let sheet = CssParser::parse("p { display: block; white-space: pre; font-size: 16px; }");
        let styled = style_tree(&dom.root, &sheet);
        let layout = LayoutEngine::build(&styled, 80.0);
        fn count_lines(lb: &feather::engine::layout::LayoutBox) -> usize {
            let here = lb.line_boxes.len();
            lb.children.iter().map(count_lines).fold(here, |a, b| a.max(b))
        }
        let max_lines = count_lines(&layout);
        assert_eq!(max_lines, 1, "white-space:pre must not auto-wrap; got {max_lines} lines");
    }
}
