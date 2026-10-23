// SPDX-License-Identifier: AGPL-3.0-or-later
#[cfg(feature = "feather-renderer")]
mod style_tests {
    use feather::engine::dom::NodeType;
    use feather::engine::{
        css::{CssColor, CssParser, Unit, Value},
        html::HtmlParser,
        style::{style_tree, StyledNode},
    };

    /// Depth-first search: first node whose element tag matches `tag`.
    fn find_by_tag<'a>(node: &'a StyledNode, tag: &str) -> Option<&'a StyledNode> {
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag {
                return Some(node);
            }
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
            Some(&Value::Color(CssColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            })),
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
            Some(&Value::Color(CssColor {
                r: 0,
                g: 0,
                b: 255,
                a: 255
            })),
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
            Some(&Value::Color(CssColor {
                r: 0,
                g: 0,
                b: 255,
                a: 255
            })),
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
        css::{CssColor, CssParser, Unit, Value},
        html::HtmlParser,
        style::{style_tree, StyledNode},
    };

    fn find_by_tag<'a>(node: &'a StyledNode, tag: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag {
                return Some(node);
            }
        }
        for child in &node.children {
            if let Some(found) = find_by_tag(child, tag) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_class<'a>(node: &'a StyledNode, class: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            let classes = e.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
            if classes.split_whitespace().any(|c| c == class) {
                return Some(node);
            }
        }
        for child in &node.children {
            if let Some(found) = find_by_class(child, class) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_id<'a>(node: &'a StyledNode, id: &str) -> Option<&'a StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.attributes.get("id").map(|s| s.as_str()) == Some(id) {
                return Some(node);
            }
        }
        for child in &node.children {
            if let Some(found) = find_by_id(child, id) {
                return Some(found);
            }
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
            Some(&Value::Color(CssColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            })),
            "p.foo must have color: red"
        );
        fn find_plain_p(node: &StyledNode) -> Option<&StyledNode> {
            use feather::engine::dom::NodeType;
            if let NodeType::Element(e) = &node.node.node_type {
                if e.tag_name == "p" {
                    let classes = e.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
                    if !classes.split_whitespace().any(|c| c == "foo") {
                        return Some(node);
                    }
                }
            }
            for child in &node.children {
                if let Some(found) = find_plain_p(child) {
                    return Some(found);
                }
            }
            None
        }
        let plain_p = find_plain_p(&styled).expect("plain p not found");
        assert_ne!(
            plain_p.properties.get("color"),
            Some(&Value::Color(CssColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            })),
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
            Some(&Value::Color(CssColor {
                r: 0,
                g: 0,
                b: 255,
                a: 255
            })),
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
            Some(&Value::Color(CssColor {
                r: 0,
                g: 128,
                b: 0,
                a: 255
            })),
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
            Some(&Value::Color(CssColor {
                r: 255,
                g: 165,
                b: 0,
                a: 255
            })),
            "direct child p must have background-color: orange"
        );
        let span_node = find_by_tag(&styled, "span").expect("span not found");
        assert_ne!(
            span_node.properties.get("background-color"),
            Some(&Value::Color(CssColor {
                r: 255,
                g: 165,
                b: 0,
                a: 255
            })),
            "grandchild span must NOT have background-color: orange from div > p"
        );
    }

    #[test]
    fn test_anchor_ua_color_applied() {
        let dom = HtmlParser::parse("<p><a href=\"#\">link</a></p>");
        // UA rule mirrors what feather_renderer injects
        let sheet = CssParser::parse("a{color:#0000EE;text-decoration:underline}");
        let styled = style_tree(&dom.root, &sheet);
        let a_node = find_by_tag(&styled, "a").expect("a node not found");
        assert_eq!(
            a_node.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 238, a: 255 })),
            "<a> must have UA blue color #0000EE"
        );
    }

    #[test]
    fn test_anchor_ua_text_decoration_applied() {
        let dom = HtmlParser::parse("<p><a href=\"#\">link</a></p>");
        let sheet = CssParser::parse("a{color:#0000EE;text-decoration:underline}");
        let styled = style_tree(&dom.root, &sheet);
        let a_node = find_by_tag(&styled, "a").expect("a node not found");
        assert_eq!(
            a_node.properties.get("text-decoration"),
            Some(&Value::Keyword("underline".to_string())),
            "<a> must have UA text-decoration: underline"
        );
    }

    #[test]
    fn test_anchor_color_overridable_by_author() {
        let dom = HtmlParser::parse("<a href=\"#\">link</a>");
        // Author rule after UA rule — author wins (cascade order in the sheet string)
        let sheet = CssParser::parse("a{color:#0000EE;text-decoration:underline} a{color:red}");
        let styled = style_tree(&dom.root, &sheet);
        let a_node = find_by_tag(&styled, "a").expect("a node not found");
        assert_eq!(
            a_node.properties.get("color"),
            Some(&Value::Color(CssColor { r: 255, g: 0, b: 0, a: 255 })),
            "author color:red must override UA blue"
        );
    }

    #[test]
    fn test_h1_ua_font_size() {
        let dom = HtmlParser::parse("<h1>Title</h1>");
        let ua = "h1,h2,h3,h4,h5,h6{display:block;font-weight:bold}\
            h1{font-size:32px;margin-top:21px;margin-bottom:21px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let h1 = find_by_tag(&styled, "h1").expect("h1 not found");
        assert_eq!(
            h1.properties.get("font-size"),
            Some(&Value::Length(32.0, Unit::Px)),
            "h1 must have UA font-size: 32px"
        );
        assert_eq!(
            h1.properties.get("font-weight"),
            Some(&Value::Keyword("bold".to_string())),
            "h1 must have UA font-weight: bold"
        );
    }

    #[test]
    fn test_h2_ua_font_size() {
        let dom = HtmlParser::parse("<h2>Sub</h2>");
        let ua = "h1,h2,h3,h4,h5,h6{display:block;font-weight:bold}\
            h2{font-size:24px;margin-top:19px;margin-bottom:19px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let h2 = find_by_tag(&styled, "h2").expect("h2 not found");
        assert_eq!(
            h2.properties.get("font-size"),
            Some(&Value::Length(24.0, Unit::Px)),
            "h2 must have UA font-size: 24px"
        );
    }

    #[test]
    fn test_h3_ua_font_size() {
        let dom = HtmlParser::parse("<h3>Section</h3>");
        let ua = "h1,h2,h3,h4,h5,h6{display:block;font-weight:bold}\
            h3{font-size:18px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let h3 = find_by_tag(&styled, "h3").expect("h3 not found");
        assert_eq!(
            h3.properties.get("font-size"),
            Some(&Value::Length(18.0, Unit::Px)),
        );
    }

    #[test]
    fn test_h1_margin_top_ua() {
        let dom = HtmlParser::parse("<h1>Title</h1>");
        let ua = "h1{font-size:32px;margin-top:21px;margin-bottom:21px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let h1 = find_by_tag(&styled, "h1").expect("h1 not found");
        assert_eq!(
            h1.properties.get("margin-top"),
            Some(&Value::Length(21.0, Unit::Px)),
            "h1 must have UA margin-top: 21px"
        );
    }

    #[test]
    fn test_p_ua_display_block() {
        let dom = HtmlParser::parse("<p>text</p>");
        let ua = "p{display:block;margin-top:16px;margin-bottom:16px}";
        let sheet = CssParser::parse(ua);
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        assert_eq!(
            p.properties.get("display"),
            Some(&Value::Keyword("block".to_string())),
            "p must have UA display: block"
        );
        assert_eq!(
            p.properties.get("margin-top"),
            Some(&Value::Length(16.0, Unit::Px)),
            "p must have UA margin-top: 16px"
        );
    }

    #[test]
    fn test_h1_author_font_size_overrides_ua() {
        let dom = HtmlParser::parse("<h1>Title</h1>");
        // UA first, author second — author wins
        let ua_then_author = "h1{font-size:32px} h1{font-size:48px}";
        let sheet = CssParser::parse(ua_then_author);
        let styled = style_tree(&dom.root, &sheet);
        let h1 = find_by_tag(&styled, "h1").expect("h1 not found");
        assert_eq!(
            h1.properties.get("font-size"),
            Some(&Value::Length(48.0, Unit::Px)),
            "author font-size:48px must override UA 32px"
        );
    }

    #[test]
    fn test_link_pseudo_applies_to_anchor_with_href() {
        // a:link rule must apply its color to an <a href="..."> element.
        let dom = HtmlParser::parse("<a href=\"https://example.com\">click</a>");
        let sheet = CssParser::parse("a:link { color: #0000ff; }");
        let styled = style_tree(&dom.root, &sheet);
        let a = find_by_tag(&styled, "a").expect("a not found");
        assert_eq!(
            a.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "a:link must apply blue color to <a href=...>"
        );
    }

    #[test]
    fn test_link_pseudo_does_not_apply_without_href() {
        // a:link must NOT match an <a> with no href attribute.
        let dom = HtmlParser::parse("<a>not a hyperlink</a>");
        let sheet = CssParser::parse("a:link { color: #0000ff; }");
        let styled = style_tree(&dom.root, &sheet);
        let a = find_by_tag(&styled, "a").expect("a not found");
        assert_ne!(
            a.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "a:link must not apply to <a> without href"
        );
    }

    #[test]
    fn test_visited_pseudo_never_matches() {
        // :visited matches nothing in this engine version (no history tracking).
        let dom = HtmlParser::parse("<a href=\"https://example.com\">visited?</a>");
        let sheet = CssParser::parse("a:visited { color: #551a8b; }");
        let styled = style_tree(&dom.root, &sheet);
        let a = find_by_tag(&styled, "a").expect("a not found");
        assert_ne!(
            a.properties.get("color"),
            Some(&Value::Color(CssColor { r: 85, g: 26, b: 139, a: 255 })),
            "a:visited must not match any link (no history)"
        );
    }

    #[test]
    fn test_css_custom_property_resolves_color() {
        // :root defines --brand: blue; body uses color: var(--brand) — must resolve to blue.
        let dom = HtmlParser::parse("<body><p>text</p></body>");
        let sheet = CssParser::parse(":root { --brand: #0000ff; } p { color: var(--brand); }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        // After var resolution, color must be #0000ff (blue).
        assert_eq!(
            p.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "var(--brand) must resolve to the custom property value"
        );
    }

    #[test]
    fn test_css_custom_property_inherited_by_children() {
        // Custom property defined on parent is available in child via var().
        let dom = HtmlParser::parse("<div><span>text</span></div>");
        let sheet = CssParser::parse("div { --gap: 8px; } span { font-size: var(--gap); }");
        let styled = style_tree(&dom.root, &sheet);
        let span = find_by_tag(&styled, "span").expect("span not found");
        assert_eq!(
            span.properties.get("font-size"),
            Some(&Value::Length(8.0, feather::engine::css::Unit::Px)),
            "var(--gap) inherited from parent must resolve to 8px"
        );
    }

    #[test]
    fn test_link_pseudo_specificity_beats_plain_tag() {
        // a:link color must override a color when both rules are present.
        let dom = HtmlParser::parse("<a href=\"https://example.com\">click</a>");
        let sheet = CssParser::parse("a { color: red; } a:link { color: #0000ff; }");
        let styled = style_tree(&dom.root, &sheet);
        let a = find_by_tag(&styled, "a").expect("a not found");
        assert_eq!(
            a.properties.get("color"),
            Some(&Value::Color(CssColor { r: 0, g: 0, b: 255, a: 255 })),
            "a:link (higher specificity) must override plain a color"
        );
    }
}

#[cfg(feature = "feather-renderer")]
mod structural_pseudo_class_tests {
    use feather::engine::{
        css::{CssColor, CssParser, Value},
        html::HtmlParser,
        style::style_tree,
    };

    fn find_by_tag<'a>(
        node: &'a feather::engine::style::StyledNode,
        tag: &str,
    ) -> Option<&'a feather::engine::style::StyledNode> {
        if node.node.tag_name() == Some(tag) {
            return Some(node);
        }
        node.children.iter().find_map(|c| find_by_tag(c, tag))
    }

    fn find_all_by_tag<'a>(
        node: &'a feather::engine::style::StyledNode,
        tag: &str,
        out: &mut Vec<&'a feather::engine::style::StyledNode>,
    ) {
        if node.node.tag_name() == Some(tag) {
            out.push(node);
        }
        for c in &node.children {
            find_all_by_tag(c, tag, out);
        }
    }

    #[test]
    fn test_first_child_matches_first_element() {
        let dom = HtmlParser::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let sheet = CssParser::parse("li:first-child { color: red; }");
        let styled = style_tree(&dom.root, &sheet);
        let mut lis = Vec::new();
        find_all_by_tag(&styled, "li", &mut lis);
        assert_eq!(lis.len(), 3, "expected 3 li elements");
        let red = CssColor { r: 255, g: 0, b: 0, a: 255 };
        assert_eq!(lis[0].properties.get("color"), Some(&Value::Color(red)),
            "first <li> should have red color from :first-child");
        assert_ne!(lis[1].properties.get("color"), Some(&Value::Color(red)),
            "second <li> should NOT match :first-child");
        assert_ne!(lis[2].properties.get("color"), Some(&Value::Color(red)),
            "third <li> should NOT match :first-child");
    }

    #[test]
    fn test_last_child_matches_last_element() {
        let dom = HtmlParser::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let sheet = CssParser::parse("li:last-child { color: blue; }");
        let styled = style_tree(&dom.root, &sheet);
        let mut lis = Vec::new();
        find_all_by_tag(&styled, "li", &mut lis);
        assert_eq!(lis.len(), 3, "expected 3 li elements");
        let blue = CssColor { r: 0, g: 0, b: 255, a: 255 };
        assert_ne!(lis[0].properties.get("color"), Some(&Value::Color(blue)),
            "first <li> should NOT match :last-child");
        assert_ne!(lis[1].properties.get("color"), Some(&Value::Color(blue)),
            "second <li> should NOT match :last-child");
        assert_eq!(lis[2].properties.get("color"), Some(&Value::Color(blue)),
            "last <li> should have blue color from :last-child");
    }

    #[test]
    fn test_only_child_matches_sole_element() {
        let dom = HtmlParser::parse("<div><p>solo</p></div>");
        let sheet = CssParser::parse("p:only-child { color: green; }");
        let styled = style_tree(&dom.root, &sheet);
        let p = find_by_tag(&styled, "p").expect("p not found");
        let green = CssColor { r: 0, g: 128, b: 0, a: 255 };
        assert_eq!(p.properties.get("color"), Some(&Value::Color(green)),
            "sole <p> should match :only-child");
    }

    #[test]
    fn test_only_child_does_not_match_with_siblings() {
        let dom = HtmlParser::parse("<div><p>A</p><p>B</p></div>");
        let sheet = CssParser::parse("p:only-child { color: green; }");
        let styled = style_tree(&dom.root, &sheet);
        let mut ps = Vec::new();
        find_all_by_tag(&styled, "p", &mut ps);
        assert_eq!(ps.len(), 2);
        let green = CssColor { r: 0, g: 128, b: 0, a: 255 };
        assert_ne!(ps[0].properties.get("color"), Some(&Value::Color(green)),
            "first <p> should NOT match :only-child when sibling exists");
        assert_ne!(ps[1].properties.get("color"), Some(&Value::Color(green)),
            "second <p> should NOT match :only-child when sibling exists");
    }

    #[test]
    fn test_first_child_specificity_beats_plain_tag() {
        let dom = HtmlParser::parse("<ul><li>A</li><li>B</li></ul>");
        // li:first-child has higher specificity than li, so it wins for the first item.
        let sheet = CssParser::parse("li { color: black; } li:first-child { color: red; }");
        let styled = style_tree(&dom.root, &sheet);
        let mut lis = Vec::new();
        find_all_by_tag(&styled, "li", &mut lis);
        let red = CssColor { r: 255, g: 0, b: 0, a: 255 };
        let black = CssColor { r: 0, g: 0, b: 0, a: 255 };
        assert_eq!(lis[0].properties.get("color"), Some(&Value::Color(red)),
            "first <li> should get :first-child (higher specificity) color");
        assert_eq!(lis[1].properties.get("color"), Some(&Value::Color(black)),
            "second <li> should get plain li color");
    }
}
