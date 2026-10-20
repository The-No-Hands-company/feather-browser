// ES6+ Features: Generators, Iterators, Symbols
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Symbol implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    id: u64,
    description: Option<String>,
}

static mut SYMBOL_COUNTER: u64 = 0;

impl Symbol {
    pub fn new(description: Option<String>) -> Self {
        unsafe {
            SYMBOL_COUNTER += 1;
            Symbol {
                id: SYMBOL_COUNTER,
                description,
            }
        }
    }
    
    pub fn for_key(key: &str) -> Self {
        // Global symbol registry (simplified)
        Symbol::new(Some(key.to_string()))
    }
    
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    pub fn to_string(&self) -> String {
        if let Some(desc) = &self.description {
            format!("Symbol({})", desc)
        } else {
            "Symbol()".to_string()
        }
    }
}

// Well-known symbols
pub struct WellKnownSymbols {
    pub iterator: Symbol,
    pub async_iterator: Symbol,
    pub has_instance: Symbol,
    pub is_concat_spreadable: Symbol,
    pub to_primitive: Symbol,
    pub to_string_tag: Symbol,
}

impl WellKnownSymbols {
    pub fn new() -> Self {
        WellKnownSymbols {
            iterator: Symbol::for_key("Symbol.iterator"),
            async_iterator: Symbol::for_key("Symbol.asyncIterator"),
            has_instance: Symbol::for_key("Symbol.hasInstance"),
            is_concat_spreadable: Symbol::for_key("Symbol.isConcatSpreadable"),
            to_primitive: Symbol::for_key("Symbol.toPrimitive"),
            to_string_tag: Symbol::for_key("Symbol.toStringTag"),
        }
    }
}

// Iterator protocol
pub trait Iterator {
    fn next(&mut self) -> IteratorResult;
}

#[derive(Debug, Clone)]
pub struct IteratorResult {
    pub value: JSValue,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Symbol(Symbol),
    Object(Arc<Mutex<JSObject>>),
    Array(Vec<JSValue>),
}

#[derive(Debug)]
pub struct JSObject {
    properties: HashMap<String, JSValue>,
    symbols: HashMap<Symbol, JSValue>,
}

impl JSObject {
    pub fn new() -> Self {
        JSObject {
            properties: HashMap::new(),
            symbols: HashMap::new(),
        }
    }
    
    pub fn set_property(&mut self, key: String, value: JSValue) {
        self.properties.insert(key, value);
    }
    
    pub fn get_property(&self, key: &str) -> Option<JSValue> {
        self.properties.get(key).cloned()
    }
    
    pub fn set_symbol(&mut self, symbol: Symbol, value: JSValue) {
        self.symbols.insert(symbol, value);
    }
    
    pub fn get_symbol(&self, symbol: &Symbol) -> Option<JSValue> {
        self.symbols.get(symbol).cloned()
    }
}

// Array Iterator
pub struct ArrayIterator {
    array: Vec<JSValue>,
    index: usize,
}

impl ArrayIterator {
    pub fn new(array: Vec<JSValue>) -> Self {
        ArrayIterator { array, index: 0 }
    }
}

impl Iterator for ArrayIterator {
    fn next(&mut self) -> IteratorResult {
        if self.index < self.array.len() {
            let value = self.array[self.index].clone();
            self.index += 1;
            IteratorResult { value, done: false }
        } else {
            IteratorResult {
                value: JSValue::Undefined,
                done: true,
            }
        }
    }
}

// Generator implementation
pub struct Generator {
    state: GeneratorState,
    yielded_values: Vec<JSValue>,
    current_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    Suspended,
    Executing,
    Completed,
}

impl Generator {
    pub fn new() -> Self {
        Generator {
            state: GeneratorState::Suspended,
            yielded_values: Vec::new(),
            current_index: 0,
        }
    }
    
    pub fn yield_value(&mut self, value: JSValue) {
        self.yielded_values.push(value);
    }
    
    pub fn next(&mut self) -> IteratorResult {
        if self.current_index < self.yielded_values.len() {
            let value = self.yielded_values[self.current_index].clone();
            self.current_index += 1;
            IteratorResult { value, done: false }
        } else {
            self.state = GeneratorState::Completed;
            IteratorResult {
                value: JSValue::Undefined,
                done: true,
            }
        }
    }
    
    pub fn return_value(&mut self, value: JSValue) -> IteratorResult {
        self.state = GeneratorState::Completed;
        IteratorResult { value, done: true }
    }
    
    pub fn throw_error(&mut self, error: JSValue) -> Result<IteratorResult, JSValue> {
        self.state = GeneratorState::Completed;
        Err(error)
    }
}

// Async Iterator
pub trait AsyncIterator {
    fn next(&mut self) -> Box<dyn std::future::Future<Output = IteratorResult> + Unpin>;
}

// for...of loop support
pub struct ForOfLoop;

impl ForOfLoop {
    pub fn iterate<I: Iterator>(mut iterator: I, mut callback: impl FnMut(JSValue)) {
        loop {
            let result = iterator.next();
            if result.done {
                break;
            }
            callback(result.value);
        }
    }
}

// Spread operator implementation
pub struct SpreadOperator;

impl SpreadOperator {
    pub fn spread_array(arrays: Vec<Vec<JSValue>>) -> Vec<JSValue> {
        let mut result = Vec::new();
        for array in arrays {
            result.extend(array);
        }
        result
    }
    
    pub fn spread_object(objects: Vec<Arc<Mutex<JSObject>>>) -> JSObject {
        let mut result = JSObject::new();
        
        for obj in objects {
            let obj = obj.lock().unwrap();
            for (key, value) in &obj.properties {
                result.set_property(key.clone(), value.clone());
            }
        }
        
        result
    }
}

// Rest parameters
pub struct RestParameters;

impl RestParameters {
    pub fn collect(args: Vec<JSValue>, param_count: usize) -> (Vec<JSValue>, Vec<JSValue>) {
        let params: Vec<JSValue> = args.iter().take(param_count).cloned().collect();
        let rest: Vec<JSValue> = args.iter().skip(param_count).cloned().collect();
        (params, rest)
    }
}

// Destructuring assignment
pub struct Destructuring;

impl Destructuring {
    pub fn destructure_array(pattern: Vec<String>, value: Vec<JSValue>) -> HashMap<String, JSValue> {
        let mut bindings = HashMap::new();
        
        for (i, name) in pattern.iter().enumerate() {
            let val = value.get(i).cloned().unwrap_or(JSValue::Undefined);
            bindings.insert(name.clone(), val);
        }
        
        bindings
    }
    
    pub fn destructure_object(pattern: Vec<String>, object: &JSObject) -> HashMap<String, JSValue> {
        let mut bindings = HashMap::new();
        
        for name in pattern {
            let value = object.get_property(&name).unwrap_or(JSValue::Undefined);
            bindings.insert(name, value);
        }
        
        bindings
    }
    
    pub fn destructure_with_rest(
        pattern: Vec<String>,
        rest_name: Option<String>,
        value: Vec<JSValue>,
    ) -> HashMap<String, JSValue> {
        let mut bindings = HashMap::new();
        
        for (i, name) in pattern.iter().enumerate() {
            let val = value.get(i).cloned().unwrap_or(JSValue::Undefined);
            bindings.insert(name.clone(), val);
        }
        
        if let Some(rest) = rest_name {
            let rest_values: Vec<JSValue> = value.iter().skip(pattern.len()).cloned().collect();
            bindings.insert(rest, JSValue::Array(rest_values));
        }
        
        bindings
    }
}

// Template literals
pub struct TemplateLiteral;

impl TemplateLiteral {
    pub fn process(parts: Vec<String>, expressions: Vec<JSValue>) -> String {
        let mut result = String::new();
        
        for (i, part) in parts.iter().enumerate() {
            result.push_str(part);
            if i < expressions.len() {
                result.push_str(&Self::to_string(&expressions[i]));
            }
        }
        
        result
    }
    
    fn to_string(value: &JSValue) -> String {
        match value {
            JSValue::Undefined => "undefined".to_string(),
            JSValue::Null => "null".to_string(),
            JSValue::Boolean(b) => b.to_string(),
            JSValue::Number(n) => n.to_string(),
            JSValue::String(s) => s.clone(),
            JSValue::Symbol(s) => s.to_string(),
            JSValue::Object(_) => "[object Object]".to_string(),
            JSValue::Array(arr) => {
                let strs: Vec<String> = arr.iter().map(Self::to_string).collect();
                strs.join(",")
            }
        }
    }
    
    pub fn tagged_template<F>(tag: F, parts: Vec<String>, expressions: Vec<JSValue>) -> JSValue
    where
        F: Fn(Vec<String>, Vec<JSValue>) -> JSValue,
    {
        tag(parts, expressions)
    }
}

// Default parameters
pub struct DefaultParameters;

impl DefaultParameters {
    pub fn apply_defaults(
        params: Vec<(String, Option<JSValue>)>,
        args: Vec<JSValue>,
    ) -> HashMap<String, JSValue> {
        let mut bindings = HashMap::new();
        
        for (i, (name, default)) in params.iter().enumerate() {
            let value = args.get(i).cloned().or_else(|| default.clone()).unwrap_or(JSValue::Undefined);
            bindings.insert(name.clone(), value);
        }
        
        bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_creation() {
        let sym1 = Symbol::new(Some("test".to_string()));
        let sym2 = Symbol::new(Some("test".to_string()));
        assert_ne!(sym1, sym2); // Symbols are unique
    }
    
    #[test]
    fn test_array_iterator() {
        let arr = vec![
            JSValue::Number(1.0),
            JSValue::Number(2.0),
            JSValue::Number(3.0),
        ];
        let mut iter = ArrayIterator::new(arr);
        
        let result1 = iter.next();
        assert!(!result1.done);
        assert!(matches!(result1.value, JSValue::Number(1.0)));
        
        let result2 = iter.next();
        assert!(!result2.done);
        
        let result3 = iter.next();
        assert!(!result3.done);
        
        let result4 = iter.next();
        assert!(result4.done);
    }
    
    #[test]
    fn test_spread_array() {
        let arr1 = vec![JSValue::Number(1.0), JSValue::Number(2.0)];
        let arr2 = vec![JSValue::Number(3.0), JSValue::Number(4.0)];
        
        let result = SpreadOperator::spread_array(vec![arr1, arr2]);
        assert_eq!(result.len(), 4);
    }
    
    #[test]
    fn test_template_literal() {
        let parts = vec!["Hello ".to_string(), "!".to_string()];
        let expressions = vec![JSValue::String("World".to_string())];
        
        let result = TemplateLiteral::process(parts, expressions);
        assert_eq!(result, "Hello World!");
    }
}
