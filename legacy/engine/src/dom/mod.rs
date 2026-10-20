// DOM (Document Object Model) - Production-grade tree structure
// Implements W3C DOM Level 4 specification

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod event_system;
pub mod js_integration;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
}

static NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub parent_id: Option<usize>,
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        Node {
            id: NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            node_type,
            children: Vec::new(),
            parent_id: None,
        }
    }

    pub fn element(tag_name: String, attributes: HashMap<String, String>) -> Self {
        Node::new(NodeType::Element(ElementData {
            tag_name,
            attributes,
        }))
    }

    pub fn text(data: String) -> Self {
        Node::new(NodeType::Text(data))
    }

    pub fn comment(data: String) -> Self {
        Node::new(NodeType::Comment(data))
    }

    pub fn add_child(&mut self, mut child: Node) {
        child.parent_id = Some(self.id);
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child_id: usize) -> Option<Node> {
        if let Some(pos) = self.children.iter().position(|n| n.id == child_id) {
            let mut child = self.children.remove(pos);
            child.parent_id = None;
            Some(child)
        } else {
            None
        }
    }

    pub fn insert_before(&mut self, new_child: Node, reference_id: usize) -> Result<(), String> {
        if let Some(pos) = self.children.iter().position(|n| n.id == reference_id) {
            let mut child = new_child;
            child.parent_id = Some(self.id);
            self.children.insert(pos, child);
            Ok(())
        } else {
            Err("Reference node not found".to_string())
        }
    }

    pub fn replace_child(&mut self, new_child: Node, old_child_id: usize) -> Result<Node, String> {
        if let Some(pos) = self.children.iter().position(|n| n.id == old_child_id) {
            let mut child = new_child;
            child.parent_id = Some(self.id);
            let mut old = self.children.remove(pos);
            old.parent_id = None;
            self.children.insert(pos, child);
            Ok(old)
        } else {
            Err("Old child not found".to_string())
        }
    }

    pub fn set_attribute(&mut self, name: String, value: String) -> Result<(), String> {
        match &mut self.node_type {
            NodeType::Element(elem) => {
                elem.attributes.insert(name, value);
                Ok(())
            }
            _ => Err("Cannot set attribute on non-element node".to_string()),
        }
    }

    pub fn remove_attribute(&mut self, name: &str) -> Result<(), String> {
        match &mut self.node_type {
            NodeType::Element(elem) => {
                elem.attributes.remove(name);
                Ok(())
            }
            _ => Err("Cannot remove attribute on non-element node".to_string()),
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        match &self.node_type {
            NodeType::Element(elem) => elem.attributes.contains_key(name),
            _ => false,
        }
    }

    pub fn get_text_content(&self) -> String {
        match &self.node_type {
            NodeType::Text(text) => text.clone(),
            NodeType::Element(_) => {
                let mut content = String::new();
                for child in &self.children {
                    content.push_str(&child.get_text_content());
                }
                content
            }
            NodeType::Comment(_) => String::new(),
        }
    }

    pub fn set_text_content(&mut self, text: String) {
        match &mut self.node_type {
            NodeType::Text(content) => *content = text,
            NodeType::Element(_) => {
                self.children.clear();
                self.add_child(Node::text(text));
            }
            NodeType::Comment(_) => {}
        }
    }

    pub fn clone_node(&self, deep: bool) -> Node {
        let mut clone = Node::new(self.node_type.clone());
        if deep {
            for child in &self.children {
                clone.add_child(child.clone_node(true));
            }
        }
        clone
    }

    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        match &self.node_type {
            NodeType::Element(elem) => elem.attributes.get(name),
            _ => None,
        }
    }

    pub fn get_tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(elem) => Some(&elem.tag_name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dom {
    pub root: Node,
    element_cache: HashMap<String, Vec<usize>>, // Cache for ID and class lookups
}

impl Dom {
    pub fn new(root: Node) -> Self {
        Dom {
            root,
            element_cache: HashMap::new(),
        }
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<&Node> {
        self.find_element_by_id(&self.root, id)
    }

    fn find_element_by_id<'a>(&'a self, node: &'a Node, id: &str) -> Option<&'a Node> {
        if let Some(node_id) = node.get_attribute("id") {
            if node_id == id {
                return Some(node);
            }
        }

        for child in &node.children {
            if let Some(found) = self.find_element_by_id(child, id) {
                return Some(found);
            }
        }

        None
    }

    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<&Node> {
        self.find_elements_by_class(&self.root, class_name)
    }

    fn find_elements_by_class<'a>(&'a self, node: &'a Node, class_name: &str) -> Vec<&'a Node> {
        let mut results = Vec::new();

        if let Some(class) = node.get_attribute("class") {
            if class.split_whitespace().any(|c| c == class_name) {
                results.push(node);
            }
        }

        for child in &node.children {
            results.extend(self.find_elements_by_class(child, class_name));
        }

        results
    }

    pub fn query_selector(&self, selector: &str) -> Option<&Node> {
        self.query_selector_impl(&self.root, selector)
    }

    fn query_selector_impl<'a>(&'a self, node: &'a Node, selector: &str) -> Option<&'a Node> {
        if self.matches_selector(node, selector) {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = self.query_selector_impl(child, selector) {
                return Some(found);
            }
        }

        None
    }

    pub fn query_selector_all(&self, selector: &str) -> Vec<&Node> {
        self.query_selector_all_impl(&self.root, selector)
    }

    fn query_selector_all_impl<'a>(&'a self, node: &'a Node, selector: &str) -> Vec<&'a Node> {
        let mut results = Vec::new();

        if self.matches_selector(node, selector) {
            results.push(node);
        }

        for child in &node.children {
            results.extend(self.query_selector_all_impl(child, selector));
        }

        results
    }

    fn matches_selector(&self, node: &Node, selector: &str) -> bool {
        // Basic selector matching (ID, class, tag, attribute)
        let selector = selector.trim();

        if selector.starts_with('#') {
            // ID selector
            if let Some(id) = node.get_attribute("id") {
                return id == &selector[1..];
            }
        } else if selector.starts_with('.') {
            // Class selector
            if let Some(class) = node.get_attribute("class") {
                return class.split_whitespace().any(|c| c == &selector[1..]);
            }
        } else if selector.starts_with('[') && selector.ends_with(']') {
            // Attribute selector
            let attr_query = &selector[1..selector.len() - 1];
            if let Some(equals_pos) = attr_query.find('=') {
                let attr_name = &attr_query[..equals_pos];
                let attr_value = attr_query[equals_pos + 1..].trim_matches('"').trim_matches('\'');
                if let Some(value) = node.get_attribute(attr_name) {
                    return value == attr_value;
                }
            } else {
                // Just check attribute exists
                return node.has_attribute(attr_query);
            }
        } else {
            // Tag selector
            if let Some(tag) = node.get_tag_name() {
                return tag == selector;
            }
        }

        false
    }

    pub fn create_element(&self, tag_name: &str) -> Node {
        Node::element(tag_name.to_string(), HashMap::new())
    }

    pub fn create_text_node(&self, data: &str) -> Node {
        Node::text(data.to_string())
    }

    pub fn create_comment(&self, data: &str) -> Node {
        Node::comment(data.to_string())
    }

    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&Node> {
        self.find_elements(&self.root, tag_name)
    }

    fn find_elements<'a>(&'a self, node: &'a Node, tag_name: &str) -> Vec<&'a Node> {
        let mut results = Vec::new();
        
        if let Some(name) = node.get_tag_name() {
            if name == tag_name {
                results.push(node);
            }
        }

        for child in &node.children {
            results.extend(self.find_elements(child, tag_name));
        }

        results
    }

    pub fn pretty_print(&self) {
        self.print_node(&self.root, 0);
    }

    fn print_node(&self, node: &Node, indent: usize) {
        let indent_str = "  ".repeat(indent);
        match &node.node_type {
            NodeType::Element(elem) => {
                println!("{}<{}>", indent_str, elem.tag_name);
                for child in &node.children {
                    self.print_node(child, indent + 1);
                }
                println!("{}</{}>", indent_str, elem.tag_name);
            }
            NodeType::Text(text) => {
                if !text.trim().is_empty() {
                    println!("{}{}", indent_str, text.trim());
                }
            }
            NodeType::Comment(comment) => {
                println!("{}<!-- {} -->", indent_str, comment);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::text("Hello".to_string());
        assert!(matches!(node.node_type, NodeType::Text(_)));
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_add_child() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        let child = Node::text("Child".to_string());
        let child_id = child.id;
        
        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].id, child_id);
        assert_eq!(parent.children[0].parent_id, Some(parent.id));
    }

    #[test]
    fn test_remove_child() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        let child = Node::text("Child".to_string());
        let child_id = child.id;
        
        parent.add_child(child);
        let removed = parent.remove_child(child_id);
        
        assert!(removed.is_some());
        assert_eq!(parent.children.len(), 0);
        assert_eq!(removed.unwrap().parent_id, None);
    }

    #[test]
    fn test_attributes() {
        let mut node = Node::element("div".to_string(), HashMap::new());
        
        assert!(node.set_attribute("id".to_string(), "test".to_string()).is_ok());
        assert_eq!(node.get_attribute("id"), Some(&"test".to_string()));
        assert!(node.has_attribute("id"));
        
        assert!(node.remove_attribute("id").is_ok());
        assert!(!node.has_attribute("id"));
    }

    #[test]
    fn test_text_content() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        parent.add_child(Node::text("Hello ".to_string()));
        parent.add_child(Node::text("World".to_string()));
        
        assert_eq!(parent.get_text_content(), "Hello World");
        
        parent.set_text_content("New content".to_string());
        assert_eq!(parent.get_text_content(), "New content");
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_clone_node() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        parent.add_child(Node::text("Child".to_string()));
        
        let shallow = parent.clone_node(false);
        assert_eq!(shallow.children.len(), 0);
        
        let deep = parent.clone_node(true);
        assert_eq!(deep.children.len(), 1);
        assert_ne!(deep.id, parent.id);
    }

    #[test]
    fn test_get_element_by_id() {
        let mut root = Node::element("html".to_string(), HashMap::new());
        let mut div = Node::element("div".to_string(), HashMap::new());
        div.set_attribute("id".to_string(), "test-id".to_string()).unwrap();
        root.add_child(div);
        
        let dom = Dom::new(root);
        let found = dom.get_element_by_id("test-id");
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_tag_name(), Some("div"));
    }

    #[test]
    fn test_get_elements_by_class_name() {
        let mut root = Node::element("html".to_string(), HashMap::new());
        
        let mut div1 = Node::element("div".to_string(), HashMap::new());
        div1.set_attribute("class".to_string(), "test-class".to_string()).unwrap();
        
        let mut div2 = Node::element("div".to_string(), HashMap::new());
        div2.set_attribute("class".to_string(), "test-class other".to_string()).unwrap();
        
        root.add_child(div1);
        root.add_child(div2);
        
        let dom = Dom::new(root);
        let found = dom.get_elements_by_class_name("test-class");
        
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_query_selector() {
        let mut root = Node::element("html".to_string(), HashMap::new());
        
        let mut div = Node::element("div".to_string(), HashMap::new());
        div.set_attribute("id".to_string(), "test".to_string()).unwrap();
        div.set_attribute("class".to_string(), "container".to_string()).unwrap();
        
        root.add_child(div);
        let dom = Dom::new(root);
        
        assert!(dom.query_selector("#test").is_some());
        assert!(dom.query_selector(".container").is_some());
        assert!(dom.query_selector("div").is_some());
        assert!(dom.query_selector("[id]").is_some());
        assert!(dom.query_selector("[id=test]").is_some());
    }

    #[test]
    fn test_query_selector_all() {
        let mut root = Node::element("html".to_string(), HashMap::new());
        root.add_child(Node::element("div".to_string(), HashMap::new()));
        root.add_child(Node::element("div".to_string(), HashMap::new()));
        root.add_child(Node::element("span".to_string(), HashMap::new()));
        
        let dom = Dom::new(root);
        let divs = dom.query_selector_all("div");
        
        assert_eq!(divs.len(), 2);
    }

    #[test]
    fn test_insert_before() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        let child1 = Node::text("First".to_string());
        let child1_id = child1.id;
        let child2 = Node::text("Third".to_string());
        
        parent.add_child(child1);
        parent.add_child(child2);
        
        let new_child = Node::text("Second".to_string());
        assert!(parent.insert_before(new_child, child1_id).is_ok());
        assert_eq!(parent.children.len(), 3);
    }

    #[test]
    fn test_replace_child() {
        let mut parent = Node::element("div".to_string(), HashMap::new());
        let old_child = Node::text("Old".to_string());
        let old_id = old_child.id;
        
        parent.add_child(old_child);
        
        let new_child = Node::text("New".to_string());
        let replaced = parent.replace_child(new_child, old_id);
        
        assert!(replaced.is_ok());
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].get_text_content(), "New");
    }
}
