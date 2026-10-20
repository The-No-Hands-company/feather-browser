// JavaScript Scope and Closures
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum ScopeValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Arc<Function>),
    Object(Arc<Mutex<HashMap<String, ScopeValue>>>),
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
    pub closure: Arc<Mutex<Scope>>,
}

pub struct Scope {
    variables: HashMap<String, ScopeValue>,
    parent: Option<Arc<Mutex<Scope>>>,
    kind: ScopeKind,
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("kind", &self.kind)
            .field("variables", &self.variables)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
}

impl Scope {
    pub fn new_global() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
            kind: ScopeKind::Global,
        }
    }
    
    pub fn new_function(parent: Arc<Mutex<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
            kind: ScopeKind::Function,
        }
    }
    
    pub fn new_block(parent: Arc<Mutex<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
            kind: ScopeKind::Block,
        }
    }
    
    pub fn declare(&mut self, name: String, value: ScopeValue, _kind: VarKind) {
        // TODO: Use kind for proper var/let/const semantics (hoisting, TDZ, etc.)
        self.variables.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<ScopeValue> {
        if let Some(value) = self.variables.get(name) {
            return Some(value.clone());
        }
        
        if let Some(parent) = &self.parent {
            return parent.lock().unwrap().get(name);
        }
        
        None
    }
    
    pub fn set(&mut self, name: &str, value: ScopeValue) -> Result<(), String> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return Ok(());
        }
        
        if let Some(parent) = &self.parent {
            return parent.lock().unwrap().set(name, value);
        }
        
        Err(format!("Variable '{}' is not defined", name))
    }
    
    pub fn has(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return true;
        }
        
        if let Some(parent) = &self.parent {
            return parent.lock().unwrap().has(name);
        }
        
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarKind {
    Var,   // Function-scoped
    Let,   // Block-scoped
    Const, // Block-scoped, immutable
}

pub struct ScopeManager {
    current: Arc<Mutex<Scope>>,
}

impl ScopeManager {
    pub fn new() -> Self {
        ScopeManager {
            current: Arc::new(Mutex::new(Scope::new_global())),
        }
    }
    
    pub fn enter_function_scope(&mut self) {
        let parent = self.current.clone();
        self.current = Arc::new(Mutex::new(Scope::new_function(parent)));
    }
    
    pub fn enter_block_scope(&mut self) {
        let parent = self.current.clone();
        self.current = Arc::new(Mutex::new(Scope::new_block(parent)));
    }
    
    pub fn exit_scope(&mut self) {
        let parent = self.current.lock().unwrap().parent.clone();
        if let Some(parent) = parent {
            self.current = parent;
        }
    }
    
    pub fn declare(&mut self, name: String, value: ScopeValue, kind: VarKind) {
        self.current.lock().unwrap().declare(name, value, kind);
    }
    
    pub fn get(&self, name: &str) -> Option<ScopeValue> {
        self.current.lock().unwrap().get(name)
    }
    
    pub fn set(&mut self, name: &str, value: ScopeValue) -> Result<(), String> {
        self.current.lock().unwrap().set(name, value)
    }
    
    pub fn current_scope(&self) -> Arc<Mutex<Scope>> {
        self.current.clone()
    }
}

// Closure implementation
pub struct Closure {
    function: Arc<Function>,
    captured_scope: Arc<Mutex<Scope>>,
}

impl Closure {
    pub fn new(function: Arc<Function>, scope: Arc<Mutex<Scope>>) -> Self {
        Closure {
            function,
            captured_scope: scope,
        }
    }
    
    pub fn call(&self, args: Vec<ScopeValue>) -> Result<ScopeValue, String> {
        // Create new scope with captured scope as parent
        let exec_scope = Arc::new(Mutex::new(Scope::new_function(self.captured_scope.clone())));
        
        // Bind parameters
        let mut scope = exec_scope.lock().unwrap();
        for (i, param) in self.function.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(ScopeValue::Undefined);
            scope.declare(param.clone(), value, VarKind::Let);
        }
        drop(scope);
        
        // Execute function body (simplified - would need interpreter)
        Ok(ScopeValue::Undefined)
    }
}

// this binding
pub struct ThisBinding {
    value: ScopeValue,
}

impl ThisBinding {
    pub fn new(value: ScopeValue) -> Self {
        ThisBinding { value }
    }
    
    pub fn get(&self) -> ScopeValue {
        self.value.clone()
    }
    
    pub fn set(&mut self, value: ScopeValue) {
        self.value = value;
    }
}

pub struct ExecutionContext {
    scope: Arc<Mutex<Scope>>,
    this_binding: ThisBinding,
    strict_mode: bool,
}

impl ExecutionContext {
    pub fn new(scope: Arc<Mutex<Scope>>) -> Self {
        ExecutionContext {
            scope,
            this_binding: ThisBinding::new(ScopeValue::Undefined),
            strict_mode: false,
        }
    }
    
    pub fn with_this(mut self, this: ScopeValue) -> Self {
        self.this_binding.set(this);
        self
    }
    
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
    
    pub fn get_this(&self) -> ScopeValue {
        self.this_binding.get()
    }
    
    /// Get the scope for this execution context
    pub fn get_scope(&self) -> Arc<Mutex<Scope>> {
        Arc::clone(&self.scope)
    }
    
    /// Execute in the context's scope
    pub fn execute_in_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Scope) -> R,
    {
        let mut scope = self.scope.lock().unwrap();
        f(&mut scope)
    }
    
    /// Check if executing in strict mode
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scope_variable() {
        let mut scope_mgr = ScopeManager::new();
        scope_mgr.declare("x".to_string(), ScopeValue::Number(42.0), VarKind::Let);
        
        let value = scope_mgr.get("x").unwrap();
        assert!(matches!(value, ScopeValue::Number(42.0)));
    }
    
    #[test]
    fn test_nested_scope() {
        let mut scope_mgr = ScopeManager::new();
        scope_mgr.declare("x".to_string(), ScopeValue::Number(1.0), VarKind::Let);
        
        scope_mgr.enter_block_scope();
        scope_mgr.declare("y".to_string(), ScopeValue::Number(2.0), VarKind::Let);
        
        assert!(scope_mgr.get("x").is_some());
        assert!(scope_mgr.get("y").is_some());
        
        scope_mgr.exit_scope();
        assert!(scope_mgr.get("x").is_some());
        assert!(scope_mgr.get("y").is_none());
    }
}
