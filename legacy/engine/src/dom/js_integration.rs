// DOM-JavaScript Integration Layer
// Production-grade implementation connecting Rust DOM with JS runtime
// Implements WHATWG DOM Standard: https://dom.spec.whatwg.org/

use crate::dom::{Node, NodeType, ElementData};
use crate::web_api::observers::{MutationObserver, MutationRecord, MutationType, MutationObserverInit};
use crate::js::engine::{JSEngine, JSValue};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// DOM Manager with JavaScript integration and mutation tracking
pub struct DOMManager {
    root: Arc<RwLock<Node>>,
    observers: Arc<RwLock<Vec<MutationObserver>>>,
    node_registry: Arc<RwLock<HashMap<usize, Arc<RwLock<Node>>>>>,
    js_engine: Option<Arc<RwLock<JSEngine>>>,
}

impl DOMManager {
    pub fn new(root: Node) -> Self {
        let root_id = root.id;
        let root_arc = Arc::new(RwLock::new(root));
        let mut registry = HashMap::new();
        registry.insert(root_id, Arc::clone(&root_arc));

        Self {
            root: root_arc,
            observers: Arc::new(RwLock::new(Vec::new())),
            node_registry: Arc::new(RwLock::new(registry)),
            js_engine: None,
        }
    }

    pub fn attach_js_engine(&mut self, engine: Arc<RwLock<JSEngine>>) {
        self.js_engine = Some(engine);
    }

    /// Register a mutation observer
    pub fn register_observer(&self, observer: MutationObserver) {
        let mut observers = self.observers.write().unwrap();
        observers.push(observer);
    }

    /// Notify all observers of a mutation
    fn notify_mutation(&self, record: MutationRecord) {
        let observers = self.observers.read().unwrap();
        for observer in observers.iter() {
            observer.record_mutation(record.clone());
        }
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: usize) -> Option<Arc<RwLock<Node>>> {
        let registry = self.node_registry.read().unwrap();
        registry.get(&node_id).cloned()
    }

    /// Create element (corresponds to document.createElement)
    pub fn create_element(&self, tag_name: String) -> Node {
        let node = Node::element(tag_name, HashMap::new());
        self.register_node(node.clone());
        node
    }

    /// Create text node (corresponds to document.createTextNode)
    pub fn create_text_node(&self, data: String) -> Node {
        let node = Node::text(data);
        self.register_node(node.clone());
        node
    }

    /// Append child to a node with mutation tracking
    pub fn append_child(&self, parent_id: usize, child: Node) -> Result<(), String> {
        let parent_opt = self.get_node(parent_id);
        if parent_opt.is_none() {
            return Err(format!("Parent node {} not found", parent_id));
        }

        let parent = parent_opt.unwrap();
        let child_id = child.id;
        self.register_node(child.clone());

        {
            let mut parent_node = parent.write().unwrap();
            parent_node.add_child(child);
        }

        // Notify mutation observers
        self.notify_mutation(MutationRecord {
            mutation_type: MutationType::ChildList,
            target: parent_id,
            added_nodes: vec![child_id],
            removed_nodes: vec![],
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value: None,
        });

        Ok(())
    }

    /// Remove child from a node with mutation tracking
    pub fn remove_child(&self, parent_id: usize, child_id: usize) -> Result<Node, String> {
        let parent_opt = self.get_node(parent_id);
        if parent_opt.is_none() {
            return Err(format!("Parent node {} not found", parent_id));
        }

        let parent = parent_opt.unwrap();
        let removed_child = {
            let mut parent_node = parent.write().unwrap();
            parent_node.remove_child(child_id)
        };

        if let Some(child) = removed_child {
            self.unregister_node(child_id);

            // Notify mutation observers
            self.notify_mutation(MutationRecord {
                mutation_type: MutationType::ChildList,
                target: parent_id,
                added_nodes: vec![],
                removed_nodes: vec![child_id],
                previous_sibling: None,
                next_sibling: None,
                attribute_name: None,
                attribute_namespace: None,
                old_value: None,
            });

            Ok(child)
        } else {
            Err(format!("Child node {} not found in parent {}", child_id, parent_id))
        }
    }

    /// Set attribute with mutation tracking
    pub fn set_attribute(&self, node_id: usize, name: String, value: String) -> Result<(), String> {
        let node_opt = self.get_node(node_id);
        if node_opt.is_none() {
            return Err(format!("Node {} not found", node_id));
        }

        let node = node_opt.unwrap();
        let old_value = {
            let mut node_ref = node.write().unwrap();
            let old = if let NodeType::Element(ref elem) = node_ref.node_type {
                elem.attributes.get(&name).cloned()
            } else {
                None
            };
            node_ref.set_attribute(name.clone(), value)?;
            old
        };

        // Notify mutation observers
        self.notify_mutation(MutationRecord {
            mutation_type: MutationType::Attributes,
            target: node_id,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            next_sibling: None,
            attribute_name: Some(name),
            attribute_namespace: None,
            old_value,
        });

        Ok(())
    }

    /// Remove attribute with mutation tracking
    pub fn remove_attribute(&self, node_id: usize, name: String) -> Result<(), String> {
        let node_opt = self.get_node(node_id);
        if node_opt.is_none() {
            return Err(format!("Node {} not found", node_id));
        }

        let node = node_opt.unwrap();
        {
            let mut node_ref = node.write().unwrap();
            node_ref.remove_attribute(&name)?;
        }

        // Notify mutation observers
        self.notify_mutation(MutationRecord {
            mutation_type: MutationType::Attributes,
            target: node_id,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            next_sibling: None,
            attribute_name: Some(name),
            attribute_namespace: None,
            old_value: None,
        });

        Ok(())
    }

    /// Set text content with mutation tracking
    pub fn set_text_content(&self, node_id: usize, text: String) -> Result<(), String> {
        let node_opt = self.get_node(node_id);
        if node_opt.is_none() {
            return Err(format!("Node {} not found", node_id));
        }

        let node = node_opt.unwrap();
        let old_value = {
            let mut node_ref = node.write().unwrap();
            let old = match &node_ref.node_type {
                NodeType::Text(ref t) => Some(t.clone()),
                NodeType::Comment(ref c) => Some(c.clone()),
                _ => None,
            };

            // Clear children for element nodes
            if matches!(node_ref.node_type, NodeType::Element(_)) {
                node_ref.children.clear();
                let text_node = Node::text(text);
                node_ref.add_child(text_node);
            } else if let NodeType::Text(ref mut t) = node_ref.node_type {
                *t = text.clone();
            }

            old
        };

        // Notify mutation observers
        self.notify_mutation(MutationRecord {
            mutation_type: MutationType::CharacterData,
            target: node_id,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value,
        });

        Ok(())
    }

    /// Execute JavaScript in the attached engine with DOM context
    pub fn execute_script(&self, script: &str) -> Result<JSValue, String> {
        if let Some(ref engine) = self.js_engine {
            let mut js = engine.write().unwrap();
            
            // Inject DOM API bindings
            self.inject_dom_api(&mut js)?;
            
            // Execute the script
            js.execute(script)
        } else {
            Err("No JavaScript engine attached".to_string())
        }
    }

    /// Inject DOM APIs into the JavaScript runtime
    fn inject_dom_api(&self, js_engine: &mut JSEngine) -> Result<(), String> {
        // This would inject native functions that map to Rust DOM methods
        // For production use, we'd use a proper FFI bridge like napi-rs
        
        // Example: document object setup
        js_engine.set_global("document", JSValue::Object(0));
        
        // In a full implementation, these would be native functions
        // that call back into DOMManager methods
        Ok(())
    }

    /// Register a node in the global registry
    fn register_node(&self, node: Node) {
        let mut registry = self.node_registry.write().unwrap();
        let node_arc = Arc::new(RwLock::new(node.clone()));
        registry.insert(node.id, node_arc);
    }

    /// Unregister a node from the global registry
    fn unregister_node(&self, node_id: usize) {
        let mut registry = self.node_registry.write().unwrap();
        registry.remove(&node_id);
    }

    /// Query selector (basic CSS selector support)
    pub fn query_selector(&self, selector: &str) -> Option<usize> {
        let root = self.root.read().unwrap();
        self.query_selector_recursive(&root, selector)
    }

    fn query_selector_recursive(&self, node: &Node, selector: &str) -> Option<usize> {
        // Simple tag name selector
        if let NodeType::Element(ref elem) = node.node_type {
            if selector.starts_with('.') {
                // Class selector
                let class_name = &selector[1..];
                if let Some(classes) = elem.attributes.get("class") {
                    if classes.split_whitespace().any(|c| c == class_name) {
                        return Some(node.id);
                    }
                }
            } else if selector.starts_with('#') {
                // ID selector
                let id = &selector[1..];
                if let Some(node_id) = elem.attributes.get("id") {
                    if node_id == id {
                        return Some(node.id);
                    }
                }
            } else {
                // Tag selector
                if elem.tag_name.to_lowercase() == selector.to_lowercase() {
                    return Some(node.id);
                }
            }
        }

        // Recursively search children
        for child in &node.children {
            if let Some(found) = self.query_selector_recursive(child, selector) {
                return Some(found);
            }
        }

        None
    }

    /// Query all elements matching selector
    pub fn query_selector_all(&self, selector: &str) -> Vec<usize> {
        let root = self.root.read().unwrap();
        let mut results = Vec::new();
        self.query_selector_all_recursive(&root, selector, &mut results);
        results
    }

    fn query_selector_all_recursive(&self, node: &Node, selector: &str, results: &mut Vec<usize>) {
        // Check if current node matches
        if let NodeType::Element(ref elem) = node.node_type {
            let matches = if selector.starts_with('.') {
                // Class selector
                let class_name = &selector[1..];
                elem.attributes.get("class")
                    .map(|classes| classes.split_whitespace().any(|c| c == class_name))
                    .unwrap_or(false)
            } else if selector.starts_with('#') {
                // ID selector
                let id = &selector[1..];
                elem.attributes.get("id")
                    .map(|node_id| node_id == id)
                    .unwrap_or(false)
            } else {
                // Tag selector
                elem.tag_name.to_lowercase() == selector.to_lowercase()
            };

            if matches {
                results.push(node.id);
            }
        }

        // Recursively search children
        for child in &node.children {
            self.query_selector_all_recursive(child, selector, results);
        }
    }

    /// Get the root node
    pub fn get_root(&self) -> Arc<RwLock<Node>> {
        Arc::clone(&self.root)
    }

    /// Flush all mutation observers (trigger callbacks)
    pub fn flush_mutations(&self) {
        let observers = self.observers.read().unwrap();
        for observer in observers.iter() {
            observer.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_append() {
        let root = Node::element("html".to_string(), HashMap::new());
        let manager = DOMManager::new(root.clone());

        let body = manager.create_element("body".to_string());
        let body_id = body.id;
        
        assert!(manager.append_child(root.id, body).is_ok());
        assert!(manager.get_node(body_id).is_some());
    }

    #[test]
    fn test_mutation_tracking() {
        let root = Node::element("html".to_string(), HashMap::new());
        let manager = DOMManager::new(root.clone());

        let observer = MutationObserver::new();
        observer.observe(root.id, MutationObserverInit {
            child_list: true,
            attributes: false,
            character_data: false,
            subtree: false,
            attribute_old_value: false,
            character_data_old_value: false,
            attribute_filter: None,
        });

        manager.register_observer(observer.clone());

        let div = manager.create_element("div".to_string());
        manager.append_child(root.id, div).unwrap();

        let records = observer.take_records();
        assert_eq!(records.len(), 1);
        assert!(matches!(records[0].mutation_type, MutationType::ChildList));
    }

    #[test]
    fn test_query_selector() {
        let mut root = Node::element("html".to_string(), HashMap::new());
        let mut body = Node::element("body".to_string(), HashMap::new());
        
        let mut div = Node::element("div".to_string(), HashMap::new());
        div.set_attribute("class".to_string(), "test".to_string()).unwrap();
        
        body.add_child(div.clone());
        root.add_child(body);

        let manager = DOMManager::new(root);

        let found = manager.query_selector(".test");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), div.id);
    }

    #[test]
    fn test_set_attribute_mutation() {
        let root = Node::element("html".to_string(), HashMap::new());
        let manager = DOMManager::new(root.clone());

        let observer = MutationObserver::new();
        observer.observe(root.id, MutationObserverInit {
            child_list: false,
            attributes: true,
            character_data: false,
            subtree: false,
            attribute_old_value: true,
            character_data_old_value: false,
            attribute_filter: None,
        });

        manager.register_observer(observer.clone());

        manager.set_attribute(root.id, "lang".to_string(), "en".to_string()).unwrap();

        let records = observer.take_records();
        assert_eq!(records.len(), 1);
        assert!(matches!(records[0].mutation_type, MutationType::Attributes));
        assert_eq!(records[0].attribute_name, Some("lang".to_string()));
    }
}
