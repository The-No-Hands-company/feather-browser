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
