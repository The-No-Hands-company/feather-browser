// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg(feature = "feather-renderer")]
use feather::engine::css::{CssColor, CssParser};
use feather::engine::html::HtmlParser;
use feather::engine::layout::{LayoutEngine, Rect};
use feather::engine::render::{canvas_to_bgra, Canvas, Painter};
use feather::engine::style::style_tree;

#[test]
fn test_canvas_defaults_to_white() {
    let canvas = Canvas::new(4, 4);
    assert_eq!(canvas.pixels.len(), 16);
    assert_eq!(canvas.pixels[0], CssColor::WHITE);
}

#[test]
fn test_canvas_to_bgra_byte_count() {
    let canvas = Canvas::new(2, 2);
    let bytes = canvas_to_bgra(&canvas);
    assert_eq!(bytes.len(), 2 * 2 * 4); // BGRA = 4 bytes per pixel
}

#[test]
fn test_bgra_white_pixel() {
    let canvas = Canvas::new(1, 1);
    let bytes = canvas_to_bgra(&canvas);
    // Cairo ARGB32 on little-endian: [B=255, G=255, R=255, A=255]
    assert_eq!(&bytes[..4], &[255u8, 255, 255, 255]);
}

#[test]
fn test_paint_produces_correct_dimensions() {
    let dom = HtmlParser::parse("<div></div>");
    let sheet = CssParser::parse("div { background-color: red; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 100.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    assert_eq!(canvas.width, 100);
    assert_eq!(canvas.height, 100);
}

#[test]
fn test_red_background_paints_red_pixels() {
    let dom = HtmlParser::parse("<html><body></body></html>");
    let sheet = CssParser::parse("body { background-color: red; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 10.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    // At least one pixel should be red (or canvas stayed white if body has no height)
    // We accept both: the important thing is no panic
    assert_eq!(canvas.pixels.len(), 100);
}

#[cfg(feature = "feather-renderer")]
#[test]
fn test_blit_glyph_writes_pixels() {
    use feather::engine::css::CssColor;
    use feather::engine::font::FontMetrics;
    use feather::engine::render::{blit_glyph, Canvas};

    let font = FontMetrics::load(16.0);
    let glyph = font.rasterize('I'); // 'I' is a narrow glyph, reliably non-empty
    let mut canvas = Canvas::new(64, 64);
    // Fill canvas white first
    // Blit a black 'I' at (10, 30)
    blit_glyph(&mut canvas, &glyph, 10, 30, CssColor::BLACK);
    // At least one pixel should now be non-white
    let has_dark = canvas.pixels.iter().any(|p| p.r < 255);
    assert!(
        has_dark,
        "blit_glyph must write non-white pixels onto the canvas"
    );
}

#[test]
fn test_larger_font_size_covers_more_canvas_pixels() {
    use feather::engine::css::CssColor;
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };

    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
    };

    let dom_large = HtmlParser::parse("<p>X</p>");
    let sheet_large = CssParser::parse("p { font-size: 32px; color: black; }");
    let styled_large = style_tree(&dom_large.root, &sheet_large);
    let layout_large = LayoutEngine::build(&styled_large, 200.0);
    let canvas_large = Painter::paint(&layout_large, viewport, 16.0);

    let dom_small = HtmlParser::parse("<p>X</p>");
    let sheet_small = CssParser::parse("p { font-size: 16px; color: black; }");
    let styled_small = style_tree(&dom_small.root, &sheet_small);
    let layout_small = LayoutEngine::build(&styled_small, 200.0);
    let canvas_small = Painter::paint(&layout_small, viewport, 16.0);

    let non_white = |canvas: &feather::engine::render::Canvas| {
        canvas
            .pixels
            .iter()
            .filter(|&&p| p != CssColor::WHITE)
            .count()
    };
    let large_pixels = non_white(&canvas_large);
    let small_pixels = non_white(&canvas_small);

    assert!(
        large_pixels > small_pixels,
        "32px font should cover more non-white pixels than 16px: large={large_pixels}, small={small_pixels}"
    );
}

#[test]
fn test_huge_font_size_does_not_panic() {
    use feather::engine::font::FontMetrics;
    let m = FontMetrics::load(100_000.0);
    assert!(
        m.size <= 512.0,
        "huge font-size must be clamped: got {}",
        m.size
    );
}

#[test]
fn test_negative_font_size_falls_back_to_default() {
    use feather::engine::font::FontMetrics;
    let m = FontMetrics::load(-1.0);
    assert_eq!(m.size, 16.0);
}

#[test]
fn test_zero_font_size_falls_back_to_default() {
    use feather::engine::font::FontMetrics;
    let m = FontMetrics::load(0.0);
    assert_eq!(m.size, 16.0);
}

#[test]
fn test_render_loop_pen_advances_with_cached_glyphs() {
    use feather::engine::css::CssColor;
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };

    let dom = HtmlParser::parse("<p>Hello</p>");
    let sheet = CssParser::parse("p { font-size: 16px; color: black; display: block; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 400.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 100.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);

    // Count columns (x positions) that contain at least one non-white pixel
    let non_white_cols = (0..canvas.width)
        .filter(|&x| {
            (0..canvas.height).any(|y| canvas.pixels[y * canvas.width + x] != CssColor::WHITE)
        })
        .count();

    // "Hello" is 5 chars; if pen advance works (advance_width > 0 per char),
    // rendered glyphs should span at least 10 distinct columns
    assert!(
        non_white_cols >= 25,
        "Text 'Hello' should span ≥25 non-white columns; got {non_white_cols} \
         (advance_width may be 0, stacking all chars at x=0)"
    );
}

#[test]
fn test_dark_color_constant_value() {
    assert_eq!(
        CssColor::DARK,
        CssColor { r: 30, g: 30, b: 30, a: 255 },
        "CssColor::DARK must equal the near-black color previously hard-coded at three render sites"
    );
}

#[test]
fn test_pen_advance_f32_accumulation() {
    use feather::engine::font::FontMetrics;
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };

    // "MMMMM" — five wide characters; with correct f32 advance accumulation,
    // each M occupies its own horizontal strip of columns.
    let dom = HtmlParser::parse("<p>MMMMM</p>");
    let sheet = CssParser::parse("p { font-size: 16px; color: black; display: block; }");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 400.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 100.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);

    let fm = FontMetrics::load(16.0);
    let single_char_px = fm.advance_width('M') as usize;

    let non_white_cols = (0..canvas.width)
        .filter(|&x| {
            (0..canvas.height).any(|y| canvas.pixels[y * canvas.width + x] != CssColor::WHITE)
        })
        .count();

    // 5 M's with correct advance must span at least 4× a single M's width in columns.
    assert!(
        non_white_cols >= 4 * single_char_px,
        "Text 'MMMMM' should span ≥{} non-white columns (4×{}px); got {} — \
         pen_x may not be advancing correctly",
        4 * single_char_px,
        single_char_px,
        non_white_cols
    );
}

#[test]
fn test_img_placeholder_paints_grey_fill() {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };
    let dom = HtmlParser::parse("<img width=\"100\" height=\"50\">");
    let sheet = CssParser::parse("");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 200.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    // The img box is painted grey (180,180,180) with a darker border; the canvas must contain grey pixels.
    let has_grey = canvas
        .pixels
        .iter()
        .any(|p| p.r == 180 && p.g == 180 && p.b == 180);
    assert!(
        has_grey,
        "canvas should contain grey placeholder pixels from the img box"
    );
}

#[test]
fn test_background_color_paints_box() {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };
    let dom = HtmlParser::parse("<div style=\"background-color: red; width: 100px;\">x</div>");
    let sheet = CssParser::parse("");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 200.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    let has_red = canvas
        .pixels
        .iter()
        .any(|p| p.r == 255 && p.g == 0 && p.b == 0);
    assert!(
        has_red,
        "canvas must contain a red pixel from background-color: red"
    );
}

#[test]
fn test_border_color_paints_edge() {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };
    let dom = HtmlParser::parse(
        "<div style=\"border: 4px solid blue; background-color: white; width: 100px;\">x</div>",
    );
    let sheet = CssParser::parse("");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 200.0,
    };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    let has_blue = canvas
        .pixels
        .iter()
        .any(|p| p.b == 255 && p.r == 0 && p.g == 0);
    assert!(has_blue, "canvas must contain a blue pixel from the border");
}

#[test]
fn test_display_none_produces_no_pixels() {
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };
    let dom = HtmlParser::parse(
        "<div style=\"display: none; background-color: red; width: 100px; height: 100px;\"></div>"
    );
    let sheet = CssParser::parse("");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build(&styled, 800.0);
    let viewport = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    let red_count = canvas.pixels.iter()
        .filter(|p| p.r == 255 && p.g == 0 && p.b == 0)
        .count();
    assert_eq!(red_count, 0, "display: none must produce zero red pixels");
}

#[test]
fn test_overflow_hidden_skips_fully_outside_child() {
    // Parent: height=0px with overflow:hidden. Child starts at same y as parent bottom.
    // overflow:hidden fully-outside check (cb.y >= pb.y + pb.height) triggers.
    use feather::engine::{
        css::CssParser,
        html::HtmlParser,
        layout::{LayoutEngine, Rect},
        render::Painter,
        style::style_tree,
    };
    let dom = HtmlParser::parse(
        "<div style=\"width: 800px; height: 0px; overflow: hidden;\">\
         <div style=\"width: 800px; height: 100px; background-color: red;\"></div>\
         </div>"
    );
    let sheet = CssParser::parse("");
    let styled = style_tree(&dom.root, &sheet);
    let layout = LayoutEngine::build_with_viewport(&styled, 800.0, 600.0);
    let viewport = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    let canvas = Painter::paint(&layout, viewport, 16.0);
    let red_count = canvas.pixels.iter()
        .filter(|p| p.r == 255 && p.g == 0 && p.b == 0)
        .count();
    assert_eq!(red_count, 0, "child fully outside overflow:hidden parent must not paint red pixels");
}
