// Proxy and Reflect API
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(Arc<Mutex<JSObject>>),
    Proxy(Arc<Mutex<ProxyObject>>),
}

#[derive(Debug)]
pub struct JSObject {
    pub properties: HashMap<String, JSValue>,
}

impl JSObject {
    pub fn new() -> Self {
        JSObject {
            properties: HashMap::new(),
        }
    }
}

// Proxy handler interface
pub trait ProxyHandler: Send {
    fn get(&mut self, target: &JSObject, property: &str, _receiver: &JSValue) -> Result<JSValue, String> {
        // TODO: Use receiver for proper [[Get]] semantics with getters
        Ok(target.properties.get(property).cloned().unwrap_or(JSValue::Undefined))
    }
    
    fn set(&mut self, target: &mut JSObject, property: &str, value: JSValue, _receiver: &JSValue) -> Result<bool, String> {
        // TODO: Use receiver for proper [[Set]] semantics with setters
        target.properties.insert(property.to_string(), value);
        Ok(true)
    }
    
    fn has(&mut self, target: &JSObject, property: &str) -> Result<bool, String> {
        Ok(target.properties.contains_key(property))
    }
    
    fn delete_property(&mut self, target: &mut JSObject, property: &str) -> Result<bool, String> {
        Ok(target.properties.remove(property).is_some())
    }
    
    fn own_keys(&mut self, target: &JSObject) -> Result<Vec<String>, String> {
        Ok(target.properties.keys().cloned().collect())
    }
    
    fn get_own_property_descriptor(&mut self, target: &JSObject, property: &str) -> Result<Option<PropertyDescriptor>, String> {
        if target.properties.contains_key(property) {
            Ok(Some(PropertyDescriptor {
                value: target.properties.get(property).cloned(),
                writable: true,
                enumerable: true,
                configurable: true,
                get: None,
                set: None,
            }))
        } else {
            Ok(None)
        }
    }
    
    fn define_property(&mut self, target: &mut JSObject, property: &str, descriptor: PropertyDescriptor) -> Result<bool, String> {
        if let Some(value) = descriptor.value {
            target.properties.insert(property.to_string(), value);
        }
        Ok(true)
    }
    
    fn prevent_extensions(&mut self, _target: &mut JSObject) -> Result<bool, String> {
        Ok(true)
    }
    
    fn is_extensible(&mut self, _target: &JSObject) -> Result<bool, String> {
        Ok(true)
    }
    
    fn get_prototype_of(&mut self, _target: &JSObject) -> Result<Option<JSValue>, String> {
        Ok(None)
    }
    
    fn set_prototype_of(&mut self, _target: &mut JSObject, _proto: Option<JSValue>) -> Result<bool, String> {
        Ok(true)
    }
    
    fn apply(&mut self, _target: &JSObject, _this: &JSValue, _args: Vec<JSValue>) -> Result<JSValue, String> {
        Err("Target is not callable".to_string())
    }
    
    fn construct(&mut self, _target: &JSObject, _args: Vec<JSValue>, _new_target: &JSValue) -> Result<JSValue, String> {
        Err("Target is not a constructor".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub value: Option<JSValue>,
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
    pub get: Option<JSValue>,
    pub set: Option<JSValue>,
}

pub struct ProxyObject {
    target: Arc<Mutex<JSObject>>,
    handler: Box<dyn ProxyHandler>,
}

impl std::fmt::Debug for ProxyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyObject")
            .field("target", &self.target)
            .finish()
    }
}

impl ProxyObject {
    pub fn new(target: Arc<Mutex<JSObject>>, handler: Box<dyn ProxyHandler>) -> Self {
        ProxyObject { target, handler }
    }
    
    pub fn get(&mut self, property: &str, receiver: &JSValue) -> Result<JSValue, String> {
        let target = self.target.lock().unwrap();
        self.handler.get(&target, property, receiver)
    }
    
    pub fn set(&mut self, property: &str, value: JSValue, receiver: &JSValue) -> Result<bool, String> {
        let mut target = self.target.lock().unwrap();
        self.handler.set(&mut target, property, value, receiver)
    }
    
    pub fn has(&mut self, property: &str) -> Result<bool, String> {
        let target = self.target.lock().unwrap();
        self.handler.has(&target, property)
    }
    
    pub fn delete_property(&mut self, property: &str) -> Result<bool, String> {
        let mut target = self.target.lock().unwrap();
        self.handler.delete_property(&mut target, property)
    }
}

// Reflect API
pub struct Reflect;

impl Reflect {
    pub fn get(target: &JSObject, property: &str) -> JSValue {
        target.properties.get(property).cloned().unwrap_or(JSValue::Undefined)
    }
    
    pub fn set(target: &mut JSObject, property: &str, value: JSValue) -> bool {
        target.properties.insert(property.to_string(), value);
        true
    }
    
    pub fn has(target: &JSObject, property: &str) -> bool {
        target.properties.contains_key(property)
    }
    
    pub fn delete_property(target: &mut JSObject, property: &str) -> bool {
        target.properties.remove(property).is_some()
    }
    
    pub fn own_keys(target: &JSObject) -> Vec<String> {
        target.properties.keys().cloned().collect()
    }
    
    pub fn get_own_property_descriptor(target: &JSObject, property: &str) -> Option<PropertyDescriptor> {
        if target.properties.contains_key(property) {
            Some(PropertyDescriptor {
                value: target.properties.get(property).cloned(),
                writable: true,
                enumerable: true,
                configurable: true,
                get: None,
                set: None,
            })
        } else {
            None
        }
    }
    
    pub fn define_property(target: &mut JSObject, property: &str, descriptor: PropertyDescriptor) -> bool {
        if let Some(value) = descriptor.value {
            target.properties.insert(property.to_string(), value);
        }
        true
    }
    
    pub fn prevent_extensions(_target: &mut JSObject) -> bool {
        true
    }
    
    pub fn is_extensible(_target: &JSObject) -> bool {
        true
    }
}

// Example: Logging proxy handler
pub struct LoggingHandler;

impl ProxyHandler for LoggingHandler {
    fn get(&mut self, target: &JSObject, property: &str, _receiver: &JSValue) -> Result<JSValue, String> {
        println!("GET: {}", property);
        Ok(target.properties.get(property).cloned().unwrap_or(JSValue::Undefined))
    }
    
    fn set(&mut self, target: &mut JSObject, property: &str, value: JSValue, _receiver: &JSValue) -> Result<bool, String> {
        println!("SET: {} = {:?}", property, value);
        target.properties.insert(property.to_string(), value);
        Ok(true)
    }
}

// Example: Validation proxy handler
pub struct ValidationHandler {
    allowed_types: HashMap<String, String>,
}

impl ValidationHandler {
    pub fn new() -> Self {
        ValidationHandler {
            allowed_types: HashMap::new(),
        }
    }
    
    pub fn add_constraint(&mut self, property: String, type_name: String) {
        self.allowed_types.insert(property, type_name);
    }
}

impl ProxyHandler for ValidationHandler {
    fn set(&mut self, target: &mut JSObject, property: &str, value: JSValue, _receiver: &JSValue) -> Result<bool, String> {
        if let Some(expected_type) = self.allowed_types.get(property) {
            let actual_type = match &value {
                JSValue::Number(_) => "number",
                JSValue::String(_) => "string",
                JSValue::Boolean(_) => "boolean",
                _ => "other",
            };
            
            if expected_type != actual_type {
                return Err(format!(
                    "Type mismatch for property '{}': expected {}, got {}",
                    property, expected_type, actual_type
                ));
            }
        }
        
        target.properties.insert(property.to_string(), value);
        Ok(true)
    }
}

// Example: Negative array index handler
pub struct NegativeIndexHandler;

impl ProxyHandler for NegativeIndexHandler {
    fn get(&mut self, target: &JSObject, property: &str, _receiver: &JSValue) -> Result<JSValue, String> {
        if let Ok(index) = property.parse::<i32>() {
            if index < 0 {
                // Handle negative indexing
                let length = target.properties.get("length")
                    .and_then(|v| match v {
                        JSValue::Number(n) => Some(*n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);
                
                let actual_index = (length as i32 + index) as usize;
                let key = actual_index.to_string();
                return Ok(target.properties.get(&key).cloned().unwrap_or(JSValue::Undefined));
            }
        }
        
        Ok(target.properties.get(property).cloned().unwrap_or(JSValue::Undefined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_get_set() {
        let target = Arc::new(Mutex::new(JSObject::new()));
        let handler = Box::new(LoggingHandler);
        let mut proxy = ProxyObject::new(target.clone(), handler);
        
        let receiver = JSValue::Undefined;
        let _ = proxy.set("x", JSValue::Number(42.0), &receiver);
        let value = proxy.get("x", &receiver).unwrap();
        
        assert!(matches!(value, JSValue::Number(42.0)));
    }
    
    #[test]
    fn test_reflect() {
        let mut obj = JSObject::new();
        Reflect::set(&mut obj, "name", JSValue::String("test".to_string()));
        
        let value = Reflect::get(&obj, "name");
        assert!(matches!(value, JSValue::String(_)));
        
        assert!(Reflect::has(&obj, "name"));
        assert!(!Reflect::has(&obj, "age"));
    }
}
