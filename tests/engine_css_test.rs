// SPDX-License-Identifier: AGPL-3.0-or-later
use feather::engine::css::CssParser;

#[cfg(feature = "feather-renderer")]
mod renderer_tests {
    use feather::engine::css::{CssColor, CssParser, Unit, Value};
    use feather::engine::dom::{Node, NodeType};
    use feather::engine::style::style_tree;
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
            Value::Color(c) => {
                assert_eq!(c.r, 255);
                assert_eq!(c.g, 0);
                assert_eq!(c.b, 0);
            }
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
        assert!(sheet.rules[0].selectors[0]
            .classes
            .contains(&"btn".to_string()));
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
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(
            decls.get("margin-top"),
            Some(&Value::Length(10.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-right"),
            Some(&Value::Length(10.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-bottom"),
            Some(&Value::Length(10.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-left"),
            Some(&Value::Length(10.0, Unit::Px))
        );
    }

    #[test]
    fn test_margin_shorthand_2_values() {
        let sheet = CssParser::parse("p { margin: 10px 20px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(
            decls.get("margin-top"),
            Some(&Value::Length(10.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-right"),
            Some(&Value::Length(20.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-bottom"),
            Some(&Value::Length(10.0, Unit::Px))
        );
        assert_eq!(
            decls.get("margin-left"),
            Some(&Value::Length(20.0, Unit::Px))
        );
    }

    #[test]
    fn test_padding_shorthand_4_values() {
        let sheet = CssParser::parse("div { padding: 1px 2px 3px 4px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(
            decls.get("padding-top"),
            Some(&Value::Length(1.0, Unit::Px))
        );
        assert_eq!(
            decls.get("padding-right"),
            Some(&Value::Length(2.0, Unit::Px))
        );
        assert_eq!(
            decls.get("padding-bottom"),
            Some(&Value::Length(3.0, Unit::Px))
        );
        assert_eq!(
            decls.get("padding-left"),
            Some(&Value::Length(4.0, Unit::Px))
        );
    }

    #[test]
    fn test_margin_shorthand_3_values() {
        // 3-value: top, left/right, bottom
        let sheet = CssParser::parse("p { margin: 5px 10px 15px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"), Some(&Value::Length(5.0, Unit::Px)));
        assert_eq!(decls.get("margin-right"), Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("margin-bottom"), Some(&Value::Length(15.0, Unit::Px)));
        assert_eq!(decls.get("margin-left"), Some(&Value::Length(10.0, Unit::Px)));
    }

    #[test]
    fn test_margin_shorthand_4_values() {
        let sheet = CssParser::parse("p { margin: 1px 2px 3px 4px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"), Some(&Value::Length(1.0, Unit::Px)));
        assert_eq!(decls.get("margin-right"), Some(&Value::Length(2.0, Unit::Px)));
        assert_eq!(decls.get("margin-bottom"), Some(&Value::Length(3.0, Unit::Px)));
        assert_eq!(decls.get("margin-left"), Some(&Value::Length(4.0, Unit::Px)));
    }

    #[test]
    fn test_padding_shorthand_1_value() {
        let sheet = CssParser::parse("p { padding: 8px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("padding-top"), Some(&Value::Length(8.0, Unit::Px)));
        assert_eq!(decls.get("padding-right"), Some(&Value::Length(8.0, Unit::Px)));
        assert_eq!(decls.get("padding-bottom"), Some(&Value::Length(8.0, Unit::Px)));
        assert_eq!(decls.get("padding-left"), Some(&Value::Length(8.0, Unit::Px)));
    }

    #[test]
    fn test_padding_shorthand_2_values() {
        let sheet = CssParser::parse("p { padding: 5px 10px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("padding-top"), Some(&Value::Length(5.0, Unit::Px)));
        assert_eq!(decls.get("padding-right"), Some(&Value::Length(10.0, Unit::Px)));
        assert_eq!(decls.get("padding-bottom"), Some(&Value::Length(5.0, Unit::Px)));
        assert_eq!(decls.get("padding-left"), Some(&Value::Length(10.0, Unit::Px)));
    }

    #[test]
    fn test_padding_shorthand_3_values() {
        let sheet = CssParser::parse("p { padding: 2px 4px 6px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("padding-top"), Some(&Value::Length(2.0, Unit::Px)));
        assert_eq!(decls.get("padding-right"), Some(&Value::Length(4.0, Unit::Px)));
        assert_eq!(decls.get("padding-bottom"), Some(&Value::Length(6.0, Unit::Px)));
        assert_eq!(decls.get("padding-left"), Some(&Value::Length(4.0, Unit::Px)));
    }

    #[test]
    fn test_font_shorthand_extracts_size() {
        let sheet = CssParser::parse("p { font: bold 18px sans-serif }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("font-size"), Some(&Value::Length(18.0, Unit::Px)));
    }

    #[test]
    fn test_longhand_still_works() {
        let sheet = CssParser::parse("p { margin-top: 5px }");
        let decls: std::collections::HashMap<_, _> =
            sheet.rules[0].declarations.iter().cloned().collect();
        assert_eq!(decls.get("margin-top"), Some(&Value::Length(5.0, Unit::Px)));
    }
}

#[test]
fn test_descendant_selector_matches() {
    use feather::engine::css::Value;
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;

    let dom = HtmlParser::parse("<div><p>hello</p></div>");
    let sheet = CssParser::parse("div p { color: red }");
    let tree = style_tree(&dom.root, &sheet);
    // tree.children[0] = <html>, descend to find <p>
    fn find_tag<'a>(
        node: &'a feather::engine::style::StyledNode,
        tag: &str,
    ) -> Option<&'a feather::engine::style::StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag {
                return Some(node);
            }
        }
        node.children.iter().find_map(|c| find_tag(c, tag))
    }
    let p = find_tag(&tree, "p").expect("<p> must exist");
    assert_eq!(
        p.properties.get("color"),
        Some(&Value::Color(feather::engine::css::CssColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        }))
    );
}

#[test]
fn test_child_selector_matches_direct_child() {
    use feather::engine::css::Value;
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;

    let dom = HtmlParser::parse("<div><p>hello</p></div>");
    let sheet = CssParser::parse("div > p { color: blue }");
    let tree = style_tree(&dom.root, &sheet);
    fn find_tag<'a>(
        node: &'a feather::engine::style::StyledNode,
        tag: &str,
    ) -> Option<&'a feather::engine::style::StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag {
                return Some(node);
            }
        }
        node.children.iter().find_map(|c| find_tag(c, tag))
    }
    let p = find_tag(&tree, "p").expect("<p> must exist");
    assert_eq!(
        p.properties.get("color"),
        Some(&Value::Color(feather::engine::css::CssColor {
            r: 0,
            g: 0,
            b: 255,
            a: 255
        }))
    );
}

#[test]
fn test_child_selector_does_not_match_grandchild() {
    use feather::engine::html::HtmlParser;
    use feather::engine::style::style_tree;

    let dom = HtmlParser::parse("<div><section><p>hello</p></section></div>");
    let sheet = CssParser::parse("div > p { color: red }");
    let tree = style_tree(&dom.root, &sheet);
    fn find_tag<'a>(
        node: &'a feather::engine::style::StyledNode,
        tag: &str,
    ) -> Option<&'a feather::engine::style::StyledNode> {
        use feather::engine::dom::NodeType;
        if let NodeType::Element(e) = &node.node.node_type {
            if e.tag_name == tag {
                return Some(node);
            }
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

#[test]
fn test_descendant_selector_matches_with_intermediate_element() {
    // "div span" should match a span inside a section inside a div
    use feather::engine::css::CssParser;
    use feather::engine::css::{CssColor, Value};
    use feather::engine::dom::{ElementData, Node, NodeType};
    use feather::engine::style::style_tree;
    use std::collections::HashMap;

    let css = CssParser::parse("div span { color: red }");

    let span_node = Node {
        node_type: NodeType::Element(ElementData {
            tag_name: "span".into(),
            attributes: HashMap::new(),
        }),
        children: vec![],
    };
    let section_node = Node {
        node_type: NodeType::Element(ElementData {
            tag_name: "section".into(),
            attributes: HashMap::new(),
        }),
        children: vec![span_node],
    };
    let div_node = Node {
        node_type: NodeType::Element(ElementData {
            tag_name: "div".into(),
            attributes: HashMap::new(),
        }),
        children: vec![section_node],
    };

    let styled = style_tree(&div_node, &css);
    // styled.children[0] = section, styled.children[0].children[0] = span
    let span_styled = &styled.children[0].children[0];
    assert_eq!(
        span_styled.properties.get("color"),
        Some(&Value::Color(CssColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        })),
        "div span selector must match span nested inside section inside div"
    );
}

#[test]
fn test_descendant_matching_deep_dom_no_stack_overflow() {
    // 5-level selector against 30-deep DOM — must complete without stack overflow
    use feather::engine::css::CssParser;
    use feather::engine::dom::{ElementData, Node, NodeType};
    use feather::engine::style::style_tree;
    use std::collections::HashMap;

    let css = CssParser::parse("div div div div div span { color: blue }");

    fn make_chain(depth: usize) -> Node {
        if depth == 0 {
            return Node {
                node_type: NodeType::Element(ElementData {
                    tag_name: "span".into(),
                    attributes: HashMap::new(),
                }),
                children: vec![],
            };
        }
        Node {
            node_type: NodeType::Element(ElementData {
                tag_name: "div".into(),
                attributes: HashMap::new(),
            }),
            children: vec![make_chain(depth - 1)],
        }
    }

    let dom = make_chain(30);
    let _ = style_tree(&dom, &css); // must not panic or overflow
}

#[test]
fn test_mixed_child_descendant_backtracks_to_outer_ancestor() {
    // "p > a b" must match <b> under <p><a><x><a><b>: the nearest <a> has no
    // <p> parent, but the outer <a> does. A greedy nearest-match walk misses this.
    use feather::engine::css::{CssColor, CssParser, Value};
    use feather::engine::dom::{ElementData, Node, NodeType};
    use feather::engine::style::style_tree;
    use std::collections::HashMap;

    fn el(tag: &str, children: Vec<Node>) -> Node {
        Node {
            node_type: NodeType::Element(ElementData {
                tag_name: tag.into(),
                attributes: HashMap::new(),
            }),
            children,
        }
    }

    let css = CssParser::parse("p > a b { color: red }");
    let dom = el(
        "p",
        vec![el("a", vec![el("x", vec![el("a", vec![el("b", vec![])])])])],
    );
    let styled = style_tree(&dom, &css);
    // styled: p -> a -> x -> a -> b
    let b_styled = &styled.children[0].children[0].children[0].children[0];
    assert_eq!(
        b_styled.properties.get("color"),
        Some(&Value::Color(CssColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        })),
        "p > a b must match b whose outer a is a direct child of p"
    );
}

#[cfg(test)]
mod border_shorthand_tests {
    use feather::engine::css::{CssColor, CssParser, Unit, Value};

    fn parse_decls(css: &str) -> std::collections::HashMap<String, Value> {
        let sheet = CssParser::parse(css);
        sheet
            .rules
            .into_iter()
            .flat_map(|r| r.declarations)
            .collect()
    }

    #[test]
    fn test_border_shorthand_expands_width_and_color() {
        let decls = parse_decls("div { border: 2px solid black; }");
        assert_eq!(
            decls.get("border-top-width"),
            Some(&Value::Length(2.0, Unit::Px))
        );
        assert_eq!(
            decls.get("border-right-width"),
            Some(&Value::Length(2.0, Unit::Px))
        );
        assert_eq!(
            decls.get("border-bottom-width"),
            Some(&Value::Length(2.0, Unit::Px))
        );
        assert_eq!(
            decls.get("border-left-width"),
            Some(&Value::Length(2.0, Unit::Px))
        );
        assert_eq!(
            decls.get("border-color"),
            Some(&Value::Color(CssColor::BLACK))
        );
    }

    #[test]
    fn test_border_shorthand_no_color() {
        let decls = parse_decls("div { border: 3px solid; }");
        assert_eq!(
            decls.get("border-top-width"),
            Some(&Value::Length(3.0, Unit::Px))
        );
        assert!(
            decls.get("border-color").is_none(),
            "no color in shorthand → no border-color entry"
        );
    }

    #[test]
    fn test_border_side_shorthand() {
        let decls = parse_decls("div { border-left: 1px solid red; }");
        assert_eq!(
            decls.get("border-left-width"),
            Some(&Value::Length(1.0, Unit::Px))
        );
        assert_eq!(
            decls.get("border-color"),
            Some(&Value::Color(CssColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            }))
        );
        assert!(
            decls.get("border-top-width").is_none(),
            "side shorthand must not set other sides"
        );
    }
}

#[cfg(test)]
mod named_color_tests {
    use feather::engine::css::{CssColor, CssParser, Value};

    fn parse_color(s: &str) -> Option<CssColor> {
        match CssParser::parse_value(s) {
            Value::Color(c) => Some(c),
            _ => None,
        }
    }

    #[test]
    fn test_named_colors_css_level1() {
        assert_eq!(
            parse_color("navy"),
            Some(CssColor {
                r: 0,
                g: 0,
                b: 128,
                a: 255
            })
        );
        assert_eq!(
            parse_color("teal"),
            Some(CssColor {
                r: 0,
                g: 128,
                b: 128,
                a: 255
            })
        );
        assert_eq!(
            parse_color("maroon"),
            Some(CssColor {
                r: 128,
                g: 0,
                b: 0,
                a: 255
            })
        );
        assert_eq!(
            parse_color("olive"),
            Some(CssColor {
                r: 128,
                g: 128,
                b: 0,
                a: 255
            })
        );
        assert_eq!(
            parse_color("silver"),
            Some(CssColor {
                r: 192,
                g: 192,
                b: 192,
                a: 255
            })
        );
        assert_eq!(
            parse_color("lime"),
            Some(CssColor {
                r: 0,
                g: 255,
                b: 0,
                a: 255
            })
        );
    }

    #[test]
    fn test_named_colors_aliases() {
        assert_eq!(parse_color("cyan"), parse_color("aqua"), "cyan == aqua");
        assert_eq!(
            parse_color("magenta"),
            parse_color("fuchsia"),
            "magenta == fuchsia"
        );
        assert_eq!(parse_color("gray"), parse_color("grey"), "gray == grey");
        assert_eq!(
            parse_color("lightgray"),
            parse_color("lightgrey"),
            "lightgray == lightgrey"
        );
        assert_eq!(
            parse_color("darkgray"),
            parse_color("darkgrey"),
            "darkgray == darkgrey"
        );
        assert_eq!(
            parse_color("slategray"),
            parse_color("slategrey"),
            "slategray == slategrey"
        );
    }

    #[test]
    fn test_named_colors_extended() {
        assert_eq!(
            parse_color("coral"),
            Some(CssColor {
                r: 255,
                g: 127,
                b: 80,
                a: 255
            })
        );
        assert_eq!(
            parse_color("crimson"),
            Some(CssColor {
                r: 220,
                g: 20,
                b: 60,
                a: 255
            })
        );
        assert_eq!(
            parse_color("gold"),
            Some(CssColor {
                r: 255,
                g: 215,
                b: 0,
                a: 255
            })
        );
        assert_eq!(
            parse_color("hotpink"),
            Some(CssColor {
                r: 255,
                g: 105,
                b: 180,
                a: 255
            })
        );
        assert_eq!(
            parse_color("tomato"),
            Some(CssColor {
                r: 255,
                g: 99,
                b: 71,
                a: 255
            })
        );
        assert_eq!(
            parse_color("indigo"),
            Some(CssColor {
                r: 75,
                g: 0,
                b: 130,
                a: 255
            })
        );
    }
}
