// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod style_tests {
    use feather::engine::{
        css::{CssColor, CssParser, Unit, Value},
        html::HtmlParser,
        style::{style_tree, StyledNode},
    };
    use feather::engine::dom::NodeType;

    /// Depth-first search: first node whose element tag matches `tag`.
    fn find_by_tag<'a>(node: &'a StyledNode, tag: &str) -> Option<&'a StyledNode> {
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag { return Some(node); }
        }
        for child in &node.children {
            if let Some(found) = find_by_tag(child, tag) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_color_inherited_by_child() {
        let dom = HtmlParser::parse("<div><span></span></div>");
        let sheet = CssParser::parse("div { color: red; }");
        let styled = style_tree(&dom.root, &sheet);
        let span = find_by_tag(&styled, "span").expect("span not found");
        assert_eq!(
            span.properties.get("color"),
            Some(&Value::Color(CssColor { r: 255, g: 0, b: 0, a: 255 })),
            "span should inherit color: red from its div parent"
        );
    }

    #[test]
    fn test_font_size_inherited_by_child() {
        let dom = HtmlParser::parse("<p><em></em></p>");
        let sheet = CssParser::parse("p { font-size: 20px; }");
        let styled = style_tree(&dom.root, &sheet);
        let em_node = find_by_tag(&styled, "em").expect("em not found");
        assert_eq!(
            em_node.properties.get("font-size"),
            Some(&Value::Length(20.0, Unit::Px)),
            "em element should inherit font-size: 20px from its p parent"
        );
    }

    #[test]
    fn test_explicit_child_overrides_inherited() {
        let dom = HtmlParser::parse("<div><span></span></div>");
        let sheet = CssParser::parse("div { color: red; } span { color: blue; }");
        let styled = style_tree(&dom.root, &sheet);
        let span = find_by_tag(&styled, "span").expect("span not found");
        assert_eq!(
            span.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "span's explicit color: blue should override inherited color: red"
        );
    }

    #[test]
    fn test_em_resolves_relative_to_parent() {
        // HtmlParser wraps content in html > head + body.
        // "body { font-size: 20px }" matches the implicit body element.
        // "p { font-size: 0.5em }" is specified for p; em resolution uses body's 20px.
        let dom = HtmlParser::parse("<p></p>");
        let sheet = CssParser::parse("body { font-size: 20px; } p { font-size: 0.5em; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("font-size"),
            Some(&Value::Length(10.0, Unit::Px)),
            "p font-size: 0.5em should resolve to 10px when parent body has font-size: 20px"
        );
    }

    #[test]
    fn test_deeply_nested_inherits_from_grandparent() {
        let dom = HtmlParser::parse("<div><p><span></span></p></div>");
        let sheet = CssParser::parse("div { color: blue; }");
        let styled = style_tree(&dom.root, &sheet);
        let span = find_by_tag(&styled, "span").expect("span not found");
        assert_eq!(
            span.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "span should inherit color: blue through p from its grandparent div"
        );
    }

    #[test]
    fn test_root_with_no_parent_does_not_panic() {
        let dom = HtmlParser::parse("<div></div>");
        let sheet = CssParser::parse("");
        // Must not panic; properties may be empty (no rules match).
        let styled = style_tree(&dom.root, &sheet);
        let div = find_by_tag(&styled, "div").expect("div not found");
        // No assertion on values — just verify it ran without panic.
        let _ = div.properties.get("color");
        let _ = div.properties.get("font-size");
    }
}
