// Web APIs - DOM API, Fetch API, Web Workers, IndexedDB, etc.
use std::collections::HashMap;

pub mod dom_api;
pub mod fetch_api;
pub mod console_api;
pub mod fetch;
pub mod webgl;
pub mod media;
pub mod webrtc;
pub mod service_worker;
pub mod indexeddb;
pub mod observers;
pub mod crypto;
pub mod device_apis;
pub mod workers;
pub mod wasm;
pub mod audio;
pub mod device;
pub mod timers;
pub mod url_api;
pub mod blob;
pub mod history;
pub mod encoding;
pub mod performance;
pub mod formdata;
pub mod misc_apis;
pub mod clipboard;
pub mod geolocation;

#[derive(Debug, Clone)]
pub struct DOMNode {
    pub node_type: NodeType,
    pub tag_name: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub text_content: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Document,
}

pub struct DOM {
    nodes: HashMap<usize, DOMNode>,
    next_id: usize,
    root: Option<usize>,
}

impl DOM {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 0,
            root: None,
        }
    }
    
    pub fn create_element(&mut self, tag_name: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let node = DOMNode {
            node_type: NodeType::Element,
            tag_name: Some(tag_name.to_string()),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
        };
        
        self.nodes.insert(id, node);
        
        if self.root.is_none() {
            self.root = Some(id);
        }
        
        id
    }
    
    pub fn create_text_node(&mut self, text: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let node = DOMNode {
            node_type: NodeType::Text,
            tag_name: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: Some(text.to_string()),
        };
        
        self.nodes.insert(id, node);
        id
    }
    
    pub fn append_child(&mut self, parent_id: usize, child_id: usize) -> Result<(), String> {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
        
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = Some(parent_id);
        }
        
        Ok(())
    }
    
    pub fn get_element_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter()
            .find(|(_, node)| {
                node.attributes.get("id") == Some(&id.to_string())
            })
            .map(|(node_id, _)| *node_id)
    }
    
    pub fn query_selector(&self, selector: &str) -> Option<usize> {
        for (id, node) in &self.nodes {
            if let Some(tag_name) = &node.tag_name {
                if selector.starts_with('.') {
                    if let Some(class) = node.attributes.get("class") {
                        if class.contains(&selector[1..]) {
                            return Some(*id);
                        }
                    }
                } else if selector.starts_with('#') {
                    if let Some(elem_id) = node.attributes.get("id") {
                        if elem_id == &selector[1..] {
                            return Some(*id);
                        }
                    }
                } else if tag_name == selector {
                    return Some(*id);
                }
            }
        }
        None
    }
}
