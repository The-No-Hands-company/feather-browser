// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg(feature = "feather-renderer")]
use feather::engine::dom::{Dom, Node, NodeType};
use std::collections::HashMap;

#[test]
fn test_element_node_has_tag_name() {
    let node = Node::element("div".into(), HashMap::new());
    match &node.node_type {
        NodeType::Element(e) => assert_eq!(e.tag_name, "div"),
        _ => panic!("expected Element"),
    }
}

#[test]
fn test_text_node_stores_data() {
    let node = Node::text("hello".into());
    match &node.node_type {
        NodeType::Text(s) => assert_eq!(s, "hello"),
        _ => panic!("expected Text"),
    }
}

#[test]
fn test_add_child_to_element() {
    let child = Node::text("world".into());
    let mut parent = Node::element("p".into(), HashMap::new());
    parent.children.push(child);
    assert_eq!(parent.children.len(), 1);
}

#[test]
fn test_dom_root_accessible() {
    let root = Node::element("html".into(), HashMap::new());
    let dom = Dom { root };
    assert!(matches!(dom.root.node_type, NodeType::Element(_)));
}

// ---------------------------------------------------------------------------
// HTML parser tests
// ---------------------------------------------------------------------------
use feather::engine::html::HtmlParser;

#[test]
fn test_parse_simple_element() {
    let dom = HtmlParser::parse("<div></div>");
    assert_eq!(dom.root.tag_name(), Some("html"));
    let body = dom
        .root
        .children
        .iter()
        .find(|n| n.tag_name() == Some("body"))
        .expect("body element");
    let div = body
        .children
        .iter()
        .find(|n| n.tag_name() == Some("div"))
        .expect("div element");
    assert!(div.children.is_empty());
}

#[test]
fn test_parse_text_node() {
    let dom = HtmlParser::parse("<p>hello</p>");
    let body = dom
        .root
        .children
        .iter()
        .find(|n| n.tag_name() == Some("body"))
        .unwrap();
    let p = body
        .children
        .iter()
        .find(|n| n.tag_name() == Some("p"))
        .unwrap();
    assert_eq!(p.children.len(), 1);
    match &p.children[0].node_type {
        NodeType::Text(s) => assert_eq!(s.trim(), "hello"),
        _ => panic!("expected text"),
    }
}

#[test]
fn test_parse_attribute() {
    let dom = HtmlParser::parse(r#"<a href="https://example.com">link</a>"#);
    let body = dom
        .root
        .children
        .iter()
        .find(|n| n.tag_name() == Some("body"))
        .unwrap();
    let a = body
        .children
        .iter()
        .find(|n| n.tag_name() == Some("a"))
        .unwrap();
    let data = a.element_data().unwrap();
    assert_eq!(
        data.attributes.get("href").map(|s| s.as_str()),
        Some("https://example.com")
    );
}

#[test]
fn test_void_element_self_closes() {
    let dom = HtmlParser::parse("<br>");
    let body = dom
        .root
        .children
        .iter()
        .find(|n| n.tag_name() == Some("body"))
        .unwrap();
    let br = body
        .children
        .iter()
        .find(|n| n.tag_name() == Some("br"))
        .unwrap();
    assert!(br.children.is_empty());
}

#[test]
fn test_nested_elements() {
    let dom = HtmlParser::parse("<ul><li>a</li><li>b</li></ul>");
    let body = dom
        .root
        .children
        .iter()
        .find(|n| n.tag_name() == Some("body"))
        .unwrap();
    let ul = body
        .children
        .iter()
        .find(|n| n.tag_name() == Some("ul"))
        .unwrap();
    let lis: Vec<_> = ul
        .children
        .iter()
        .filter(|n| n.tag_name() == Some("li"))
        .collect();
    assert_eq!(lis.len(), 2);
}
