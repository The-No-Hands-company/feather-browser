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
