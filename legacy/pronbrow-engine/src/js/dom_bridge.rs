// JavaScript DOM Bridge - Connects JS engine to DOM
use crate::dom::{Node, NodeType};
use crate::js::{JSEngine, JSValue};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Bridge between JavaScript and DOM
#[derive(Debug)]
pub struct DOMBridge {
    dom_nodes: HashMap<usize, Rc<RefCell<Node>>>,
    next_node_id: usize,
}

impl DOMBridge {
    pub fn new() -> Self {
        Self {
            dom_nodes: HashMap::new(),
            next_node_id: 1,
        }
    }

    /// Register a DOM node and return its ID
    pub fn register_node(&mut self, node: Rc<RefCell<Node>>) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.dom_nodes.insert(id, node);
        id
    }

    /// Get a DOM node by ID
    pub fn get_node(&self, id: usize) -> Option<Rc<RefCell<Node>>> {
        self.dom_nodes.get(&id).cloned()
    }

    /// Setup document object in JS engine
    pub fn setup_document(&self, js_engine: &mut JSEngine, _root: Rc<RefCell<Node>>) {
        // Create document object with methods
        js_engine.set_global("document", JSValue::Object(0));
        
        // TODO: Use root node to build complete document object tree
        // Add getElementById, querySelector, etc. as native functions
        // These would be implemented as callbacks in the real engine
    }

    /// Execute inline script tags from HTML
    pub fn execute_scripts(&mut self, js_engine: &mut JSEngine, root: &Node) -> Result<(), String> {
        self.find_and_execute_scripts(js_engine, root)
    }

    fn find_and_execute_scripts(&mut self, js_engine: &mut JSEngine, node: &Node) -> Result<(), String> {
        // If this is a <script> tag, execute its content
        if let NodeType::Element(ref data) = node.node_type {
            if data.tag_name == "script" {
                // Get script content from children
                for child in &node.children {
                    if let NodeType::Text(ref text) = child.node_type {
                        println!("Executing script: {}", text);
                        js_engine.execute(text)?;
                    }
                }
            }
        }

        // Recursively process children
        for child in &node.children {
            self.find_and_execute_scripts(js_engine, child)?;
        }

        Ok(())
    }
}

/// DOM API exposed to JavaScript
pub struct DocumentAPI {
    bridge: Rc<RefCell<DOMBridge>>,
}

impl DocumentAPI {
    pub fn new(bridge: Rc<RefCell<DOMBridge>>) -> Self {
        Self { bridge }
    }

    /// document.getElementById(id)
    pub fn get_element_by_id(&self, _id: &str) -> JSValue {
        // In real implementation, would search DOM tree
        JSValue::Null
    }

    /// document.querySelector(selector)
    pub fn query_selector(&self, _selector: &str) -> JSValue {
        // Would use CSS selector matching
        JSValue::Null
    }

    /// document.createElement(tagName)
    pub fn create_element(&mut self, tag_name: &str) -> JSValue {
        let node = Node::element(tag_name.to_string(), HashMap::new());
        let id = self.bridge.borrow_mut().register_node(Rc::new(RefCell::new(node)));
        JSValue::Object(id)
    }

    /// document.createTextNode(text)
    pub fn create_text_node(&mut self, text: &str) -> JSValue {
        let node = Node::text(text.to_string());
        let id = self.bridge.borrow_mut().register_node(Rc::new(RefCell::new(node)));
        JSValue::Object(id)
    }
}

/// Element API exposed to JavaScript
pub struct ElementAPI {
    bridge: Rc<RefCell<DOMBridge>>,
    element_id: usize,
}

impl ElementAPI {
    pub fn new(bridge: Rc<RefCell<DOMBridge>>, element_id: usize) -> Self {
        Self { bridge, element_id }
    }

    /// element.getAttribute(name)
    pub fn get_attribute(&self, name: &str) -> JSValue {
        if let Some(node_rc) = self.bridge.borrow().get_node(self.element_id) {
            let node = node_rc.borrow();
            if let NodeType::Element(ref data) = node.node_type {
                if let Some(value) = data.attributes.get(name) {
                    return JSValue::String(value.clone());
                }
            }
        }
        JSValue::Null
    }

    /// element.setAttribute(name, value)
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        if let Some(node_rc) = self.bridge.borrow().get_node(self.element_id) {
            let mut node = node_rc.borrow_mut();
            if let NodeType::Element(ref mut data) = node.node_type {
                data.attributes.insert(name.to_string(), value.to_string());
            }
        }
    }

    /// element.appendChild(child)
    pub fn append_child(&mut self, child_id: usize) {
        if let Some(parent_rc) = self.bridge.borrow().get_node(self.element_id) {
            if let Some(child_rc) = self.bridge.borrow().get_node(child_id) {
                let mut parent = parent_rc.borrow_mut();
                let child = child_rc.borrow().clone();
                parent.children.push(child);
            }
        }
    }

    /// element.removeChild(child)
    pub fn remove_child(&mut self, _child_id: usize) {
        // Would remove child from children vec
    }

    /// element.innerHTML (getter)
    pub fn get_inner_html(&self) -> JSValue {
        // Would serialize children to HTML
        JSValue::String(String::new())
    }

    /// element.innerHTML (setter)
    pub fn set_inner_html(&mut self, _html: &str) {
        // Would parse HTML and replace children
    }

    /// element.textContent
    pub fn get_text_content(&self) -> JSValue {
        if let Some(node_rc) = self.bridge.borrow().get_node(self.element_id) {
            let node = node_rc.borrow();
            return JSValue::String(self.extract_text(&node));
        }
        JSValue::String(String::new())
    }

    fn extract_text(&self, node: &Node) -> String {
        let mut text = String::new();
        if let NodeType::Text(ref t) = node.node_type {
            text.push_str(t);
        }
        for child in &node.children {
            text.push_str(&self.extract_text(child));
        }
        text
    }

    /// element.style
    pub fn get_style(&self) -> JSValue {
        // Would return CSSStyleDeclaration object
        JSValue::Object(0)
    }

    /// element.classList
    pub fn get_class_list(&self) -> JSValue {
        // Would return DOMTokenList
        JSValue::Object(0)
    }
}

/// Window API exposed to JavaScript
pub struct WindowAPI {
    pub inner_width: u32,
    pub inner_height: u32,
    pub location: LocationAPI,
}

impl WindowAPI {
    pub fn new() -> Self {
        Self {
            inner_width: 1920,
            inner_height: 1080,
            location: LocationAPI::new("about:blank"),
        }
    }

    /// window.alert(message)
    pub fn alert(&self, message: &str) {
        println!("[ALERT] {}", message);
    }

    /// window.setTimeout(callback, delay)
    pub fn set_timeout(&self, _callback: JSValue, _delay: u32) -> u32 {
        // Would schedule callback execution
        1
    }

    /// window.setInterval(callback, delay)
    pub fn set_interval(&self, _callback: JSValue, _delay: u32) -> u32 {
        // Would schedule repeated callback execution
        1
    }

    /// window.requestAnimationFrame(callback)
    pub fn request_animation_frame(&self, _callback: JSValue) -> u32 {
        // Would schedule callback for next frame
        1
    }
}

/// Location API (window.location)
pub struct LocationAPI {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
}

impl LocationAPI {
    pub fn new(href: &str) -> Self {
        Self {
            href: href.to_string(),
            protocol: "http:".to_string(),
            host: "localhost".to_string(),
            pathname: "/".to_string(),
            search: String::new(),
            hash: String::new(),
        }
    }

    /// location.reload()
    pub fn reload(&self) {
        println!("Reloading page...");
    }

    /// location.assign(url)
    pub fn assign(&mut self, url: &str) {
        self.href = url.to_string();
        println!("Navigating to: {}", url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_bridge() {
        let mut bridge = DOMBridge::new();
        let node = Node::element("div".to_string(), HashMap::new());
        let id = bridge.register_node(Rc::new(RefCell::new(node)));
        
        assert!(bridge.get_node(id).is_some());
    }

    #[test]
    fn test_element_api() {
        let bridge = Rc::new(RefCell::new(DOMBridge::new()));
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "test".to_string());
        
        let node = Node::element("div".to_string(), attrs);
        let id = bridge.borrow_mut().register_node(Rc::new(RefCell::new(node)));
        
        let element_api = ElementAPI::new(bridge, id);
        let attr_value = element_api.get_attribute("id");
        
        match attr_value {
            JSValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
}
