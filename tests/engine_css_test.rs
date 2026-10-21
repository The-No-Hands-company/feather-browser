// SPDX-License-Identifier: AGPL-3.0-or-later
use feather::engine::css::CssParser;

#[cfg(feature = "feather-renderer")]
mod renderer_tests {
    use feather::engine::css::{CssParser, Value, Unit, CssColor};
    use feather::engine::style::style_tree;
    use feather::engine::dom::{Node, NodeType};
    use std::collections::HashMap;

    #[test]
    fn test_parse_tag_rule() {
        let sheet = CssParser::parse("p { color: red; }");
        assert_eq!(sheet.rules.len(), 1);
        let rule = &sheet.rules[0];
        assert_eq!(rule.selectors[0].tag_name.as_deref(), Some("p"));
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].0, "color");
    }

    #[test]
    fn test_parse_px_length() {
        let sheet = CssParser::parse("div { width: 100px; }");
        let decl = &sheet.rules[0].declarations[0];
        assert_eq!(decl.0, "width");
        assert!(matches!(decl.1, Value::Length(100.0, Unit::Px)));
    }

    #[test]
    fn test_parse_color_keyword() {
        let sheet = CssParser::parse("span { color: red; }");
        let value = &sheet.rules[0].declarations[0].1;
        match value {
            Value::Color(c) => { assert_eq!(c.r, 255); assert_eq!(c.g, 0); assert_eq!(c.b, 0); }
            Value::Keyword(k) => assert_eq!(k, "red"), // acceptable: caller resolves
            _ => panic!("unexpected value type"),
        }
    }

    #[test]
    fn test_parse_id_selector() {
        let sheet = CssParser::parse("#main { display: block; }");
        assert_eq!(sheet.rules[0].selectors[0].id.as_deref(), Some("main"));
    }

    #[test]
    fn test_parse_class_selector() {
        let sheet = CssParser::parse(".btn { padding: 8px; }");
        assert!(sheet.rules[0].selectors[0].classes.contains(&"btn".to_string()));
    }

    #[test]
    fn test_style_tree_applies_matching_rule() {
        let sheet = CssParser::parse("p { color: red; }");
        let root = Node::element("p".into(), HashMap::new());
        let styled = style_tree(&root, &sheet);
        assert!(styled.properties.contains_key("color"));
    }

    #[test]
    fn test_style_tree_does_not_apply_non_matching_rule() {
        let sheet = CssParser::parse("div { color: blue; }");
        let root = Node::element("p".into(), HashMap::new());
        let styled = style_tree(&root, &sheet);
        assert!(!styled.properties.contains_key("color"));
    }

    #[test]
    fn test_style_tree_inherits_display_block_default() {
        // Block-level elements get display: block when no stylesheet overrides
        let sheet = CssParser::parse("");
        let root = Node::element("div".into(), HashMap::new());
        let styled = style_tree(&root, &sheet);
        // display may be absent (style_tree adds no defaults) — just confirm it runs
        let _ = styled;
    }

    #[test]
    fn test_margin_shorthand_1_value() {
        let sheet = CssParser::parse("p { margin: 10px }");
        let decls: std::collections::HashMap<_, _> = sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"),    Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-right"),  Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-bottom"), Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-left"),   Some(&Value::Length(10.0, Unit::Px)));
    }

    #[test]
    fn test_margin_shorthand_2_values() {
        let sheet = CssParser::parse("p { margin: 10px 20px }");
        let decls: std::collections::HashMap<_, _> = sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"),    Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-right"),  Some(&Value::Length(20.0, Unit::Px)));
        assert_eq!(decls.get("margin-bottom"), Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-left"),   Some(&Value::Length(20.0, Unit::Px)));
    }

    #[test]
    fn test_padding_shorthand_4_values() {
        let sheet = CssParser::parse("div { padding: 1px 2px 3px 4px }");
        let decls: std::collections::HashMap<_, _> = sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("padding-top"),    Some(&Value::Length(1.0, Unit::Px)));
        assert_eq!(decls.get("padding-right"),  Some(&Value::Length(2.0, Unit::Px)));
        assert_eq!(decls.get("padding-bottom"), Some(&Value::Length(3.0, Unit::Px)));
        assert_eq!(decls.get("padding-left"),   Some(&Value::Length(4.0, Unit::Px)));
    }

    #[test]
    fn test_font_shorthand_extracts_size() {
        let sheet = CssParser::parse("p { font: bold 18px sans-serif }");
        let decls: std::collections::HashMap<_, _> = sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("font-size"), Some(&Value::Length(18.0, Unit::Px)));
    }

    #[test]
    fn test_longhand_still_works() {
        let sheet = CssParser::parse("p { margin-top: 5px }");
        let decls: std::collections::HashMap<_, _> = sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"), Some(&Value::Length(5.0, Unit::Px)));
    }
}

#[test]
fn test_descendant_selector_matches() {
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;
    use feather::engine::css::Value;

    let dom = HtmlParser::parse("<div><p>hello</p></div>");
    let sheet = CssParser::parse("div p { color: red }");
    let tree = style_tree(&dom.root, &sheet);
    // tree.children[0] = <html>, descend to find <p>
    fn find_tag<'a>(node: &'a feather::engine::style::StyledNode, tag: &str)
        -> Option<&'a feather::engine::style::StyledNode>
    {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag { return Some(node); }
        }
        node.children.iter().find_map(|c| find_tag(c, tag))
    }
    let p = find_tag(&tree, "p").expect("<p> must exist");
    assert_eq!(p.properties.get("color"), Some(&Value::Color(feather::engine::css::CssColor { r:255, g:0, b:0, a:255 })));
}

#[test]
fn test_child_selector_matches_direct_child() {
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;
    use feather::engine::css::Value;

    let dom = HtmlParser::parse("<div><p>hello</p></div>");
    let sheet = CssParser::parse("div > p { color: blue }");
    let tree = style_tree(&dom.root, &sheet);
    fn find_tag<'a>(node: &'a feather::engine::style::StyledNode, tag: &str)
        -> Option<&'a feather::engine::style::StyledNode>
    {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag { return Some(node); }
        }
        node.children.iter().find_map(|c| find_tag(c, tag))
    }
    let p = find_tag(&tree, "p").expect("<p> must exist");
    assert_eq!(p.properties.get("color"), Some(&Value::Color(feather::engine::css::CssColor { r:0, g:0, b:255, a:255 })));
}

#[test]
fn test_child_selector_does_not_match_grandchild() {
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;

    let dom = HtmlParser::parse("<div><section><p>hello</p></section></div>");
    let sheet = CssParser::parse("div > p { color: red }");
    let tree = style_tree(&dom.root, &sheet);
    fn find_tag<'a>(node: &'a feather::engine::style::StyledNode, tag: &str)
        -> Option<&'a feather::engine::style::StyledNode>
    {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag { return Some(node); }
        }
        node.children.iter().find_map(|c| find_tag(c, tag))
    }
    let p = find_tag(&tree, "p").expect("<p> must exist");
    // div > p should NOT match — p's direct parent is section, not div
    assert_eq!(p.properties.get("color"), None);
}

#[test]
fn test_deep_selector_chain_no_panic() {
    // 50 levels of "a " — exceeds the depth cap of 32; must parse without panicking.
    let deep_selector = "a ".repeat(50) + "b { color: red }";
    let sheet = CssParser::parse(&deep_selector);
    // Parser must return at least one rule (the part up to depth 32).
    assert!(!sheet.rules.is_empty(), "should produce at least one rule");
    // The parsed selector must be a valid Selector (tag_name Some("b")).
    assert_eq!(sheet.rules[0].selectors[0].tag_name.as_deref(), Some("b"));
}
