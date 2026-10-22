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

    #[test]
    fn test_text_align_inherited_from_parent() {
        let dom = HtmlParser::parse("<div><p></p></div>");
        let sheet = CssParser::parse("div { text-align: center; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("text-align"),
            Some(&Value::Keyword("center".into())),
            "p should inherit text-align: center from parent div"
        );
    }

    #[test]
    fn test_text_align_child_override_beats_inherited() {
        let dom = HtmlParser::parse("<div><p></p></div>");
        let sheet = CssParser::parse("div { text-align: center; } p { text-align: left; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("text-align"),
            Some(&Value::Keyword("left".into())),
            "p's explicit text-align: left must override inherited center"
        );
    }

    #[test]
    fn test_rem_font_size_resolves_to_px() {
        let dom = HtmlParser::parse("<p></p>");
        let sheet = CssParser::parse("p { font-size: 2rem; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("font-size"),
            Some(&Value::Length(32.0, Unit::Px)),
            "font-size: 2rem must resolve to 32px (2 × 16px default rem base)"
        );
    }

    #[test]
    fn test_percent_font_size_resolves_relative_to_parent() {
        let dom = HtmlParser::parse("<p></p>");
        let sheet = CssParser::parse("body { font-size: 20px; } p { font-size: 80%; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("font-size"),
            Some(&Value::Length(16.0, Unit::Px)),
            "font-size: 80% with parent body 20px must resolve to 16px"
        );
    }

    #[test]
    fn test_percent_font_size_fallback_when_no_parent_px() {
        let dom = HtmlParser::parse("<p></p>");
        let sheet = CssParser::parse("p { font-size: 50%; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("font-size"),
            Some(&Value::Length(8.0, Unit::Px)),
            "font-size: 50% with no parent px must resolve to 8px (50% of 16px base)"
        );
    }
}

#[cfg(test)]
mod selector_tests {
    use feather::engine::{
        css::{CssColor, CssParser, Value},
        html::HtmlParser,
        style::{style_tree, StyledNode},
    };

    fn find_by_tag<'a>(node: &'a StyledNode, tag: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag { return Some(node); }
        }
        for child in &node.children {
            if let Some(found) = find_by_tag(child, tag) { return Some(found); }
        }
        None
    }

    fn find_by_class<'a>(node: &'a StyledNode, class: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            let classes = e.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
            if classes.split_whitespace().any(|c| c == class) { return Some(node); }
        }
        for child in &node.children {
            if let Some(found) = find_by_class(child, class) { return Some(found); }
        }
        None
    }

    fn find_by_id<'a>(node: &'a StyledNode, id: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.attributes.get("id").map(|s| s.as_str()) == Some(id) { return Some(node); }
        }
        for child in &node.children {
            if let Some(found) = find_by_id(child, id) { return Some(found); }
        }
        None
    }

    #[test]
    fn test_class_selector_matches() {
        let dom = HtmlParser::parse("<div><p class=\"foo\">x</p><p>y</p></div>");
        let sheet = CssParser::parse(".foo { color: red; }");
        let styled = style_tree(&dom.root, &sheet);
        let foo_p = find_by_class(&styled, "foo").expect("p.foo not found");
        assert_eq!(
            foo_p.properties.get("color"),
            Some(&Value::Color(CssColor { r: 255, g: 0, b: 0, a: 255 })),
            "p.foo must have color: red"
        );
        fn find_plain_p(node: &StyledNode) -> Option<&StyledNode> {
            use feather::engine::dom::NodeType;
            if let NodeType::Element(e) = &node.node.node_type {
                if e.tag_name == "p" {
                    let classes = e.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
                    if !classes.split_whitespace().any(|c| c == "foo") { return Some(node); }
                }
            }
            for child in &node.children {
                if let Some(found) = find_plain_p(child) { return Some(found); }
            }
            None
        }
        let plain_p = find_plain_p(&styled).expect("plain p not found");
        assert_ne!(
            plain_p.properties.get("color"),
            Some(&Value::Color(CssColor { r: 255, g: 0, b: 0, a: 255 })),
            "plain p must NOT have color: red"
        );
    }

    #[test]
    fn test_id_selector_matches() {
        let dom = HtmlParser::parse("<div><p id=\"bar\">x</p><p>y</p></div>");
        let sheet = CssParser::parse("#bar { color: blue; }");
        let styled = style_tree(&dom.root, &sheet);
        let bar_p = find_by_id(&styled, "bar").expect("#bar not found");
        assert_eq!(
            bar_p.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "#bar must have color: blue"
        );
    }

    #[test]
    fn test_descendant_combinator() {
        let dom = HtmlParser::parse("<div><p>inside</p></div><p>outside</p>");
        let sheet = CssParser::parse("div p { color: green; }");
        let styled = style_tree(&dom.root, &sheet);
        let div_node = find_by_tag(&styled, "div").expect("div not found");
        let inside_p = find_by_tag(div_node, "p").expect("p inside div not found");
        assert_eq!(
            inside_p.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 128, b: 0, a: 255 })),
            "p inside div must have color: green"
        );
    }

    #[test]
    fn test_child_combinator() {
        // Use background-color (non-inherited) so span doesn't get it through inheritance.
        let dom = HtmlParser::parse("<div><p><span>deep</span></p></div>");
        let sheet = CssParser::parse("div > p { background-color: orange; }");
        let styled = style_tree(&dom.root, &sheet);
        let div_node = find_by_tag(&styled, "div").expect("div not found");
        let p_node = find_by_tag(div_node, "p").expect("p not found");
        assert_eq!(
            p_node.properties.get("background-color"),
            Some(&Value::Color(CssColor { r: 255, g: 165, b: 0, a: 255 })),
            "direct child p must have background-color: orange"
        );
        let span_node = find_by_tag(&styled, "span").expect("span not found");
        assert_ne!(
            span_node.properties.get("background-color"),
            Some(&Value::Color(CssColor { r: 255, g: 165, b: 0, a: 255 })),
            "grandchild span must NOT have background-color: orange from div > p"
        );
    }
}
