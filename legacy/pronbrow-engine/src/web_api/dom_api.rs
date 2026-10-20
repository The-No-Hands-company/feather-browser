// DOM API implementation
use super::DOM;
use std::collections::HashMap;

pub struct DOMImplementation {
    pub dom: DOM,
    event_listeners: HashMap<usize, Vec<EventListener>>,
}

#[derive(Clone)]
pub struct EventListener {
    pub event_type: String,
    pub callback: String,
    pub use_capture: bool,
}

impl DOMImplementation {
    pub fn new() -> Self {
        Self {
            dom: DOM::new(),
            event_listeners: HashMap::new(),
        }
    }
    
    pub fn add_event_listener(&mut self, node_id: usize, event_type: &str, callback: &str) {
        let listener = EventListener {
            event_type: event_type.to_string(),
            callback: callback.to_string(),
            use_capture: false,
        };
        
        self.event_listeners
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(listener);
    }
    
    pub fn remove_event_listener(&mut self, node_id: usize, event_type: &str) {
        if let Some(listeners) = self.event_listeners.get_mut(&node_id) {
            listeners.retain(|l| l.event_type != event_type);
        }
    }
    
    pub fn dispatch_event(&self, node_id: usize, event_type: &str) -> bool {
        if let Some(listeners) = self.event_listeners.get(&node_id) {
            for listener in listeners {
                if listener.event_type == event_type {
                    return true;
                }
            }
        }
        false
    }
    
    pub fn set_attribute(&mut self, node_id: usize, name: &str, value: &str) {
        if let Some(node) = self.dom.nodes.get_mut(&node_id) {
            node.attributes.insert(name.to_string(), value.to_string());
        }
    }
    
    pub fn get_attribute(&self, node_id: usize, name: &str) -> Option<String> {
        self.dom.nodes.get(&node_id)
            .and_then(|node| node.attributes.get(name).cloned())
    }
}
