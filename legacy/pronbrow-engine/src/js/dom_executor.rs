// DOM Executor - Connects JavaScript execution to actual DOM manipulation
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::dom::{Node, NodeType, ElementData};
use super::interpreter::Interpreter;
use super::runtime::JSValue;

pub struct DOMExecutor {
    /// The actual DOM tree
    dom: Rc<RefCell<Node>>,
    /// JavaScript interpreter
    interpreter: Interpreter,
    /// Event listeners
    event_listeners: HashMap<String, Vec<EventListener>>,
    /// Mutation observers
    mutation_observers: Vec<MutationObserver>,
    /// Request animation frame callbacks
    raf_callbacks: Vec<Box<dyn Fn(f64)>>,
}

struct EventListener {
    event_type: String,
    element_id: String,
    callback: Rc<dyn Fn(&Event)>,
    options: EventListenerOptions,
}

#[derive(Clone, Debug)]
pub struct EventListenerOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

struct MutationObserver {
    callback: Rc<dyn Fn(&[MutationRecord])>,
    target: String,
    options: MutationObserverInit,
}

struct MutationObserverInit {
    attributes: bool,
    character_data: bool,
    child_list: bool,
    subtree: bool,
}

pub struct MutationRecord {
    pub mutation_type: String,
    pub target: String,
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub attribute_name: Option<String>,
    pub old_value: Option<String>,
}

pub struct Event {
    pub event_type: String,
    pub target: Option<String>,
    pub current_target: Option<String>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub timestamp: f64,
}

impl DOMExecutor {
    pub fn new(dom: Rc<RefCell<Node>>) -> Self {
        let globals = Rc::new(RefCell::new(HashMap::new()));
        let mut executor = Self {
            dom: dom.clone(),
            interpreter: Interpreter::new(globals.clone()),
            event_listeners: HashMap::new(),
            mutation_observers: Vec::new(),
            raf_callbacks: Vec::new(),
        };
        
        executor.register_dom_apis(globals);
        executor
    }

    fn register_dom_apis(&mut self, globals: Rc<RefCell<HashMap<String, JSValue>>>) {
        let dom = self.dom.clone();
        
        // document.getElementById
        let dom_clone = dom.clone();
        let get_element_by_id = move |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(id)) = args.get(0) {
                // Search DOM for element with this ID
                if let Some(element) = find_element_by_id(&dom_clone.borrow(), id) {
                    Ok(JSValue::dom_element(Rc::new(RefCell::new(element))))
                } else {
                    Ok(JSValue::Null)
                }
            } else {
                Ok(JSValue::Null)
            }
        };
        
        // document.createElement
        let create_element = move |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(tag_name)) = args.get(0) {
                let element = ElementData {
                    tag_name: tag_name.to_string(),
                    attributes: HashMap::new(),
                };
                Ok(JSValue::dom_element(Rc::new(RefCell::new(
                    Node::element(tag_name.to_string(), HashMap::new())
                ))))
            } else {
                Err("createElement requires a tag name".to_string())
            }
        };
        
        // document.querySelector
        let dom_clone = dom.clone();
        let query_selector = move |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(selector)) = args.get(0) {
                // Parse CSS selector and find first matching element
                if let Some(element) = query_selector_impl(&dom_clone.borrow(), selector) {
                    Ok(JSValue::dom_element(Rc::new(RefCell::new(element))))
                } else {
                    Ok(JSValue::Null)
                }
            } else {
                Ok(JSValue::Null)
            }
        };
        
        // document.querySelectorAll
        let dom_clone = dom.clone();
        let query_selector_all = move |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(selector)) = args.get(0) {
                let elements = query_selector_all_impl(&dom_clone.borrow(), selector);
                let mut obj = super::runtime::JSObject::new();
                for (i, elem) in elements.into_iter().enumerate() {
                    obj.set(&i.to_string(), JSValue::dom_element(Rc::new(RefCell::new(elem))));
                }
                Ok(JSValue::Object(Rc::new(RefCell::new(obj))))
            } else {
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        };
        
        // console.log
        let console_log = |args: &[JSValue]| -> Result<JSValue, String> {
            let messages: Vec<String> = args.iter()
                .map(|arg| format_js_value(arg))
                .collect();
            println!("[Console] {}", messages.join(" "));
            Ok(JSValue::Undefined)
        };
        
        // console.error
        let console_error = |args: &[JSValue]| -> Result<JSValue, String> {
            let messages: Vec<String> = args.iter()
                .map(|arg| format_js_value(arg))
                .collect();
            eprintln!("[Console Error] {}", messages.join(" "));
            Ok(JSValue::Undefined)
        };
        
        // setTimeout
        let set_timeout = |args: &[JSValue]| -> Result<JSValue, String> {
            // In a real implementation, this would schedule the callback
            log::debug!("setTimeout called with {} args", args.len());
            // Return a timer ID
            Ok(JSValue::Number(1.0))
        };
        
        // setInterval
        let set_interval = |args: &[JSValue]| -> Result<JSValue, String> {
            log::debug!("setInterval called with {} args", args.len());
            Ok(JSValue::Number(1.0))
        };
        
        // requestAnimationFrame
        let request_animation_frame = |args: &[JSValue]| -> Result<JSValue, String> {
            if args.is_empty() {
                return Err("requestAnimationFrame requires a callback".to_string());
            }
            log::debug!("requestAnimationFrame called");
            Ok(JSValue::Number(1.0))
        };
        
        // fetch API
        let fetch = |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(_url)) = args.get(0) {
                // TODO: Implement actual network request using url
                // Return a Promise that will resolve with Response
                Ok(JSValue::promise(Rc::new(RefCell::new(PromiseState::Pending))))
            } else {
                Err("fetch requires a URL".to_string())
            }
        };
        
        // localStorage.getItem
        let local_storage_get = |args: &[JSValue]| -> Result<JSValue, String> {
            if let Some(JSValue::String(_key)) = args.get(0) {
                // TODO: Access browser's localStorage with key
                Ok(JSValue::Null) // Simplified
            } else {
                Ok(JSValue::Null)
            }
        };
        
        // localStorage.setItem
        let local_storage_set = |args: &[JSValue]| -> Result<JSValue, String> {
            if args.len() >= 2 {
                log::debug!("localStorage.setItem called with key and value");
                // TODO: Store in actual localStorage
            }
            Ok(JSValue::Undefined)
        };
        
        // Register all APIs to global scope
        let mut globals_map = globals.borrow_mut();
        
        // Document object
        let mut document = super::runtime::JSObject::new();
        document.set("getElementById", JSValue::native_function(Rc::new(get_element_by_id)));
        document.set("createElement", JSValue::native_function(Rc::new(create_element)));
        document.set("querySelector", JSValue::native_function(Rc::new(query_selector)));
        document.set("querySelectorAll", JSValue::native_function(Rc::new(query_selector_all)));
        globals_map.insert("document".to_string(), JSValue::Object(Rc::new(RefCell::new(document))));
        
        // Console object
        let mut console = super::runtime::JSObject::new();
        console.set("log", JSValue::native_function(Rc::new(console_log)));
        console.set("error", JSValue::native_function(Rc::new(console_error)));
        globals_map.insert("console".to_string(), JSValue::Object(Rc::new(RefCell::new(console))));
        
        // Window object functions
        globals_map.insert("setTimeout".to_string(), JSValue::native_function(Rc::new(set_timeout)));
        globals_map.insert("setInterval".to_string(), JSValue::native_function(Rc::new(set_interval)));
        globals_map.insert("requestAnimationFrame".to_string(), JSValue::native_function(Rc::new(request_animation_frame)));
        globals_map.insert("fetch".to_string(), JSValue::native_function(Rc::new(fetch)));
        
        // localStorage object
        let mut local_storage = super::runtime::JSObject::new();
        local_storage.set("getItem", JSValue::native_function(Rc::new(local_storage_get)));
        local_storage.set("setItem", JSValue::native_function(Rc::new(local_storage_set)));
        globals_map.insert("localStorage".to_string(), JSValue::Object(Rc::new(RefCell::new(local_storage))));
    }

    pub fn execute_script(&mut self, script: &str) -> Result<JSValue, String> {
        // Parse the script
        let ast = crate::js::parser::JSParser::new(script).parse()?;
        
        // Execute the AST
        self.interpreter.evaluate(&ast)
    }

    pub fn execute_event_handler(&mut self, event_type: &str, element_id: &str, event: &Event) {
        // Find event listeners for this element and event type
        if let Some(listeners) = self.event_listeners.get(&format!("{}:{}", element_id, event_type)) {
            for listener in listeners {
                // Execute the event handler
                (listener.callback)(event);
                
                // Remove if 'once' option is set
                if listener.options.once {
                    // Remove this listener
                }
            }
        }
    }

    pub fn add_event_listener(&mut self, element_id: String, event_type: String, callback: Rc<dyn Fn(&Event)>, options: EventListenerOptions) {
        let key = format!("{}:{}", element_id, event_type);
        
        let listener = EventListener {
            event_type: event_type.clone(),
            element_id: element_id.clone(),
            callback,
            options,
        };
        
        self.event_listeners.entry(key)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn dispatch_event(&mut self, target_id: &str, event: Event) {
        // Get the target element
        let target_exists = {
            let dom_ref = self.dom.borrow();
            find_element_by_id(&dom_ref, target_id).is_some()
        };
        
        if target_exists {
            // Capture phase
            let ancestors = {
                let dom_ref = self.dom.borrow();
                get_ancestors(&dom_ref, target_id)
            };
            
            for ancestor_id in ancestors.iter().rev() {
                self.execute_event_handler(&event.event_type, ancestor_id, &event);
            }
            
            // Target phase
            self.execute_event_handler(&event.event_type, target_id, &event);
            
            // Bubble phase (if event bubbles)
            if event.bubbles {
                for ancestor_id in &ancestors {
                    self.execute_event_handler(&event.event_type, ancestor_id, &event);
                }
            }
        }
    }

    pub fn mutate_dom<F>(&mut self, mutator: F) -> Result<(), String>
    where
        F: FnOnce(&mut Node) -> Result<(), String>
    {
        // Apply the mutation
        let mut dom = self.dom.borrow_mut();
        mutator(&mut dom)?;
        
        // Notify mutation observers
        let mutations = vec![]; // Collect mutations
        for observer in &self.mutation_observers {
            (observer.callback)(&mutations);
        }
        
        Ok(())
    }

    pub fn request_animation_frame(&mut self, callback: Box<dyn Fn(f64)>) -> u32 {
        self.raf_callbacks.push(callback);
        self.raf_callbacks.len() as u32
    }

    pub fn run_animation_frame(&mut self, timestamp: f64) {
        for callback in &self.raf_callbacks {
            callback(timestamp);
        }
        self.raf_callbacks.clear();
    }
}

// Helper functions
fn find_element_by_id(node: &Node, id: &str) -> Option<Node> {
    if let NodeType::Element(ref element) = node.node_type {
        if element.attributes.get("id") == Some(&id.to_string()) {
            return Some(node.clone());
        }
    }
    
    for child in &node.children {
        if let Some(found) = find_element_by_id(child, id) {
            return Some(found);
        }
    }
    
    None
}

fn query_selector_impl(node: &Node, selector: &str) -> Option<Node> {
    // Simplified CSS selector matching
    // In a real implementation, this would parse and match full CSS selectors
    
    if selector.starts_with('#') {
        // ID selector
        let id = &selector[1..];
        return find_element_by_id(node, id);
    } else if selector.starts_with('.') {
        // Class selector
        let class_name = &selector[1..];
        return find_element_by_class(node, class_name);
    } else {
        // Tag selector
        return find_element_by_tag(node, selector);
    }
}

fn query_selector_all_impl(node: &Node, selector: &str) -> Vec<Node> {
    let mut results = Vec::new();
    
    if selector.starts_with('#') {
        if let Some(element) = find_element_by_id(node, &selector[1..]) {
            results.push(element);
        }
    } else if selector.starts_with('.') {
        find_all_by_class(node, &selector[1..], &mut results);
    } else {
        find_all_by_tag(node, selector, &mut results);
    }
    
    results
}

fn find_element_by_class(node: &Node, class_name: &str) -> Option<Node> {
    if let NodeType::Element(ref element) = node.node_type {
        if let Some(classes) = element.attributes.get("class") {
            if classes.split_whitespace().any(|c| c == class_name) {
                return Some(node.clone());
            }
        }
    }
    
    for child in &node.children {
        if let Some(found) = find_element_by_class(child, class_name) {
            return Some(found);
        }
    }
    
    None
}

fn find_element_by_tag(node: &Node, tag_name: &str) -> Option<Node> {
    if let NodeType::Element(ref element) = node.node_type {
        if element.tag_name.eq_ignore_ascii_case(tag_name) {
            return Some(node.clone());
        }
    }
    
    for child in &node.children {
        if let Some(found) = find_element_by_tag(child, tag_name) {
            return Some(found);
        }
    }
    
    None
}

fn find_all_by_class(node: &Node, class_name: &str, results: &mut Vec<Node>) {
    if let NodeType::Element(ref element) = node.node_type {
        if let Some(classes) = element.attributes.get("class") {
            if classes.split_whitespace().any(|c| c == class_name) {
                results.push(node.clone());
            }
        }
    }
    
    for child in &node.children {
        find_all_by_class(child, class_name, results);
    }
}

fn find_all_by_tag(node: &Node, tag_name: &str, results: &mut Vec<Node>) {
    if let NodeType::Element(ref element) = node.node_type {
        if element.tag_name.eq_ignore_ascii_case(tag_name) {
            results.push(node.clone());
        }
    }
    
    for child in &node.children {
        find_all_by_tag(child, tag_name, results);
    }
}

fn get_ancestors(_node: &Node, _target_id: &str) -> Vec<String> {
    // In a real implementation, maintain parent pointers
    Vec::new()
}

fn format_js_value(value: &JSValue) -> String {
    match value {
        JSValue::Undefined => "undefined".to_string(),
        JSValue::Null => "null".to_string(),
        JSValue::Boolean(b) => b.to_string(),
        JSValue::Number(n) => n.to_string(),
        JSValue::String(s) => s.to_string(),
        JSValue::Object(_) => "[object Object]".to_string(),
        JSValue::Function(_) => "[Function]".to_string(),
        _ => "[Object]".to_string(),
    }
}

// Extended JSValue for DOM integration
impl JSValue {
    pub fn dom_element(_node: Rc<RefCell<Node>>) -> Self {
        JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))) // Simplified
    }
    
    pub fn promise(_state: Rc<RefCell<PromiseState>>) -> Self {
        JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))) // Simplified
    }
    
    pub fn native_function(_callback: Rc<dyn Fn(&[JSValue]) -> Result<JSValue, String>>) -> Self {
        JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))) // Simplified
    }
}

pub enum PromiseState {
    Pending,
    Fulfilled(JSValue),
    Rejected(String),
}
