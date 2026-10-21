// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

/// Data carried by an element node.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
}

/// The payload of a DOM node.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
    Comment(String),
}

/// An owned DOM tree node.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

impl Node {
    pub fn element(tag_name: String, attributes: HashMap<String, String>) -> Self {
        Self {
            node_type: NodeType::Element(ElementData { tag_name, attributes }),
            children: Vec::new(),
        }
    }

    pub fn text(data: String) -> Self {
        Self {
            node_type: NodeType::Text(data),
            children: Vec::new(),
        }
    }

    pub fn comment(data: String) -> Self {
        Self {
            node_type: NodeType::Comment(data),
            children: Vec::new(),
        }
    }
}

/// A parsed document.
#[derive(Debug, Clone, PartialEq)]
pub struct Dom {
    pub root: Node,
}
