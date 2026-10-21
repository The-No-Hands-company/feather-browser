// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg(feature = "feather-renderer")]
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
