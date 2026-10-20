// JavaScript Engine - Embedded V8-like execution environment
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct JSEngine {
    global_scope: HashMap<String, JSValue>,
    heap: Vec<JSObject>,
    call_stack: Vec<StackFrame>,
    pub console: Console,
    /// Use AST-based execution instead of string matching
    use_ast_mode: bool,
}

#[derive(Debug, Clone)]
pub struct Console {
    pub logs: Vec<String>,
}

impl Console {
    pub fn log(&mut self, message: &str) {
        self.logs.push(message.to_string());
        println!("[Console] {}", message);
    }

    pub fn error(&mut self, message: &str) {
        self.logs.push(format!("ERROR: {}", message));
        eprintln!("[Console Error] {}", message);
    }

    pub fn warn(&mut self, message: &str) {
        self.logs.push(format!("WARN: {}", message));
        println!("[Console Warn] {}", message);
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub locals: HashMap<String, JSValue>,
}

#[derive(Debug, Clone)]
pub enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(usize), // heap index
    Function(JSFunction),
    Array(Vec<JSValue>),
}

#[derive(Debug, Clone)]
pub struct JSFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct JSObject {
    pub properties: HashMap<String, JSValue>,
    pub prototype: Option<usize>,
}

/// Result of executing a statement - distinguishes between normal values and early returns
#[derive(Debug, Clone)]
enum StatementResult {
    /// Normal statement execution result
    Value(JSValue),
    /// Early return from function
    Return(JSValue),
}

impl JSEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            global_scope: HashMap::new(),
            heap: Vec::new(),
            call_stack: Vec::new(),
            console: Console { logs: Vec::new() },
            use_ast_mode: true, // Default to AST mode for better ES6+ support
        };

        // Initialize global objects
        engine.init_globals();
        engine
    }

    /// Create engine with legacy string-based execution
    pub fn new_legacy() -> Self {
        let mut engine = Self {
            global_scope: HashMap::new(),
            heap: Vec::new(),
            call_stack: Vec::new(),
            console: Console { logs: Vec::new() },
            use_ast_mode: false,
        };

        engine.init_globals();
        engine
    }

    /// Enable or disable AST mode
    pub fn set_ast_mode(&mut self, enabled: bool) {
        self.use_ast_mode = enabled;
    }

    fn init_globals(&mut self) {
        // Primitive values
        self.global_scope.insert("undefined".to_string(), JSValue::Undefined);
        self.global_scope.insert("null".to_string(), JSValue::Null);
        self.global_scope.insert("NaN".to_string(), JSValue::Number(f64::NAN));
        self.global_scope.insert("Infinity".to_string(), JSValue::Number(f64::INFINITY));
        
        // Create window object
        let mut window_props = HashMap::new();
        window_props.insert("innerWidth".to_string(), JSValue::Number(1920.0));
        window_props.insert("innerHeight".to_string(), JSValue::Number(1080.0));
        let window_idx = self.create_object(window_props);
        self.global_scope.insert("window".to_string(), JSValue::Object(window_idx));
        
        // Create document object
        let document_props = HashMap::new();
        let document_idx = self.create_object(document_props);
        self.global_scope.insert("document".to_string(), JSValue::Object(document_idx));
    }
    
    pub fn set_global(&mut self, name: &str, value: JSValue) {
        self.global_scope.insert(name.to_string(), value);
    }
    
    pub fn get_global(&self, name: &str) -> Option<JSValue> {
        self.global_scope.get(name).cloned()
    }

    /// Push a new stack frame for function execution
    pub fn push_frame(&mut self, function_name: String) {
        self.call_stack.push(StackFrame {
            function_name,
            locals: HashMap::new(),
        });
    }

    /// Pop the current stack frame
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.call_stack.pop()
    }

    /// Get current stack depth for debugging
    pub fn stack_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Get stack trace for error reporting
    pub fn get_stack_trace(&self) -> Vec<String> {
        self.call_stack.iter()
            .map(|frame| frame.function_name.clone())
            .collect()
    }

    pub fn execute(&mut self, code: &str) -> Result<JSValue, String> {
        // Use AST mode if enabled
        if self.use_ast_mode {
            return self.execute_with_ast(code);
        }

        // Legacy string-based execution
        let trimmed = code.trim();

        // If code contains multiple statements (detected by semicolons or newlines with content),
        // handle it as a block
        if self.contains_multiple_statements(trimmed) {
            return self.execute_block(trimmed);
        }

        self.execute_single_statement(trimmed)
    }

    /// Execute JavaScript code using AST parser and interpreter
    pub fn execute_with_ast(&mut self, code: &str) -> Result<JSValue, String> {
        use super::parser::JSParser;
        use super::interpreter::Interpreter;

        // Parse code into AST
        let mut parser = JSParser::new(code);
        let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

        // Convert our JSValue to runtime JSValue for interpreter
        let globals = Rc::new(RefCell::new(HashMap::new()));

        // Copy global scope to interpreter globals
        {
            let mut globals_ref = globals.borrow_mut();
            for (key, value) in &self.global_scope {
                let runtime_value = self.convert_to_runtime_value(value);
                globals_ref.insert(key.clone(), runtime_value);
            }
        }

        // Create interpreter and evaluate
        let mut interpreter = Interpreter::new(globals.clone());
        let result = interpreter.evaluate(&ast)?;

        // Convert result back to our JSValue
        let our_value = self.convert_from_runtime_value(&result);

        // Update global scope from interpreter
        {
            let globals_ref = globals.borrow();
            for (key, value) in globals_ref.iter() {
                self.global_scope.insert(key.clone(), self.convert_from_runtime_value(value));
            }
        }

        Ok(our_value)
    }

    /// Convert engine JSValue to runtime JSValue
    fn convert_to_runtime_value(&self, value: &JSValue) -> super::runtime::JSValue {
        use super::runtime::JSValue as RuntimeValue;

        match value {
            JSValue::Undefined => RuntimeValue::Undefined,
            JSValue::Null => RuntimeValue::Null,
            JSValue::Boolean(b) => RuntimeValue::Boolean(*b),
            JSValue::Number(n) => RuntimeValue::Number(*n),
            JSValue::String(s) => RuntimeValue::String(Rc::new(s.clone())),
            JSValue::Object(idx) => {
                if let Some(obj) = self.heap.get(*idx) {
                    let mut runtime_obj = super::runtime::JSObject::new();
                    for (key, val) in &obj.properties {
                        runtime_obj.set(key, self.convert_to_runtime_value(val));
                    }
                    RuntimeValue::Object(Rc::new(RefCell::new(runtime_obj)))
                } else {
                    RuntimeValue::Undefined
                }
            }
            JSValue::Function(func) => {
                let runtime_func = super::runtime::Function::new(
                    func.name.clone(),
                    func.params.clone(),
                    None, // Body would need to be converted to AST
                );
                RuntimeValue::Function(Rc::new(runtime_func))
            }
            JSValue::Array(arr) => {
                let mut runtime_obj = super::runtime::JSObject::new();
                for (i, val) in arr.iter().enumerate() {
                    runtime_obj.set(&i.to_string(), self.convert_to_runtime_value(val));
                }
                runtime_obj.set("length", RuntimeValue::Number(arr.len() as f64));
                RuntimeValue::Object(Rc::new(RefCell::new(runtime_obj)))
            }
        }
    }

    /// Convert runtime JSValue back to engine JSValue
    fn convert_from_runtime_value(&self, value: &super::runtime::JSValue) -> JSValue {
        use super::runtime::JSValue as RuntimeValue;

        match value {
            RuntimeValue::Undefined => JSValue::Undefined,
            RuntimeValue::Null => JSValue::Null,
            RuntimeValue::Boolean(b) => JSValue::Boolean(*b),
            RuntimeValue::Number(n) => JSValue::Number(*n),
            RuntimeValue::String(s) => JSValue::String(s.to_string()),
            RuntimeValue::Object(_) => JSValue::Undefined, // Simplified for now
            RuntimeValue::Function(_) => JSValue::Undefined, // Simplified for now
            RuntimeValue::Symbol(_) => JSValue::Undefined,
            RuntimeValue::BigInt(_) => JSValue::Undefined,
        }
    }

    fn execute_single_statement(&mut self, trimmed: &str) -> Result<JSValue, String> {
        match self.execute_single_statement_internal(trimmed)? {
            StatementResult::Value(v) => Ok(v),
            StatementResult::Return(v) => Ok(v),
        }
    }
    
    fn execute_single_statement_internal(&mut self, trimmed: &str) -> Result<StatementResult, String> {
        // Handle console.log
        if trimmed.starts_with("console.log(") && trimmed.ends_with(")") {
            let content = &trimmed[12..trimmed.len()-1];
            let value = self.evaluate_expression(content)?;
            let message = self.value_to_string(&value);
            self.console.log(&message);
            return Ok(StatementResult::Value(JSValue::Undefined));
        }

        // Handle console.error
        if trimmed.starts_with("console.error(") && trimmed.ends_with(")") {
            let content = &trimmed[14..trimmed.len()-1];
            let value = self.evaluate_expression(content)?;
            let message = self.value_to_string(&value);
            self.console.error(&message);
            return Ok(StatementResult::Value(JSValue::Undefined));
        }

        // Handle console.warn
        if trimmed.starts_with("console.warn(") && trimmed.ends_with(")") {
            let content = &trimmed[13..trimmed.len()-1];
            let value = self.evaluate_expression(content)?;
            let message = self.value_to_string(&value);
            self.console.warn(&message);
            return Ok(StatementResult::Value(JSValue::Undefined));
        }

        // Handle function declarations: function name(params) { body }
        if trimmed.starts_with("function ") {
            return Ok(StatementResult::Value(self.handle_function_declaration(trimmed)?));
        }

        // Handle if statements: if (condition) { ... } else { ... }
        if trimmed.starts_with("if ") || trimmed.starts_with("if(") {
            return self.handle_if_statement_internal(trimmed);
        }

        // Handle while loops: while (condition) { ... }
        if trimmed.starts_with("while ") || trimmed.starts_with("while(") {
            return self.handle_while_loop_internal(trimmed);
        }

        // Handle for loops: for (init; condition; update) { ... }
        if trimmed.starts_with("for ") || trimmed.starts_with("for(") {
            return self.handle_for_loop_internal(trimmed);
        }

        // Handle variable declarations
        if trimmed.starts_with("var ") || trimmed.starts_with("let ") || trimmed.starts_with("const ") {
            return Ok(StatementResult::Value(self.handle_variable_declaration(trimmed)?));
        }

        // Handle variable assignments (name = value) without var/let/const
        // Be careful not to match comparison operators (==, !=, <=, >=)
        if let Some(eq_pos) = trimmed.find('=') {
            // Check that this is actually an assignment, not a comparison
            let before_eq_char = if eq_pos > 0 { 
                trimmed.chars().nth(eq_pos - 1) 
            } else { 
                None 
            };
            let after_eq_char = trimmed.chars().nth(eq_pos + 1);
            
            // Skip if it's ==, !=, <=, >=, ===, !==
            let is_comparison = matches!(before_eq_char, Some('=' | '!' | '<' | '>')) || 
                               matches!(after_eq_char, Some('='));
            
            if !is_comparison {
                let before_eq = trimmed[..eq_pos].trim();
                // Check if left side is a simple identifier (not an expression)
                if !before_eq.is_empty() && 
                   before_eq.chars().all(|c| c.is_alphanumeric() || c == '_') &&
                   !before_eq.chars().next().unwrap().is_numeric() {
                    return Ok(StatementResult::Value(self.handle_variable_declaration(trimmed)?));
                }
            }
        }

        // Handle return statements
        if trimmed.starts_with("return ") {
            let expr = &trimmed[7..].trim_end_matches(';');
            return Ok(StatementResult::Return(self.evaluate_expression(expr)?));
        }

        // Evaluate as expression
        Ok(StatementResult::Value(self.evaluate_expression(trimmed)?))
    }

    fn handle_function_declaration(&mut self, code: &str) -> Result<JSValue, String> {
        // Parse: function name(param1, param2) { body }
        let code = code.trim_start_matches("function").trim();
        
        // Find function name
        let paren_pos = code.find('(').ok_or("Invalid function declaration")?;
        let name = code[..paren_pos].trim().to_string();
        
        // Find parameters
        let close_paren = code.find(')').ok_or("Invalid function declaration")?;
        let params_str = &code[paren_pos+1..close_paren];
        let params: Vec<String> = if params_str.trim().is_empty() {
            Vec::new()
        } else {
            params_str.split(',').map(|s| s.trim().to_string()).collect()
        };
        
        // Find function body
        let body_start = code.find('{').ok_or("Invalid function declaration")?;
        let body_end = code.rfind('}').ok_or("Invalid function declaration")?;
        let body = code[body_start+1..body_end].to_string();
        
        // Create function and store it
        let func = JSFunction {
            name: name.clone(),
            params,
            body,
        };
        
        self.global_scope.insert(name, JSValue::Function(func));
        Ok(JSValue::Undefined)
    }

    fn handle_if_statement(&mut self, code: &str) -> Result<JSValue, String> {
        match self.handle_if_statement_internal(code)? {
            StatementResult::Value(v) => Ok(v),
            StatementResult::Return(v) => Ok(v),
        }
    }
    
    fn handle_if_statement_internal(&mut self, code: &str) -> Result<StatementResult, String> {
        // Parse: if (condition) { consequent } else { alternate }
        let code = code.trim_start_matches("if").trim();
        
        // Find condition
        let cond_start = code.find('(').ok_or("Invalid if statement")?;
        let cond_end = self.find_matching_paren(code, cond_start)?;
        let condition = &code[cond_start+1..cond_end];
        
        // Evaluate condition
        let cond_value = self.evaluate_expression(condition)?;
        let cond_bool = self.to_boolean(&cond_value);
        
        // Find consequent block
        let rest = code[cond_end+1..].trim();
        let rest_trimmed = rest.trim().to_string();
        let (consequent, alternate_code) = if rest_trimmed.starts_with('{') {
            let block_end = self.find_matching_brace(&rest_trimmed, 0)?;
            let block = rest_trimmed[1..block_end].to_string();
            let after = rest_trimmed[block_end+1..].trim().to_string();
            (block, after)
        } else {
            // Single statement without braces
            let semi_pos = rest_trimmed.find(';').unwrap_or(rest_trimmed.len());
            let stmt = rest_trimmed[..semi_pos].to_string();
            let after = rest_trimmed[semi_pos+1..].trim().to_string();
            (stmt, after)
        };
        
        if cond_bool {
            return self.execute_block_internal(&consequent);
        }
        
        // Handle else clause
        if alternate_code.starts_with("else") {
            let else_code = alternate_code.trim_start_matches("else").trim().to_string();
            if else_code.starts_with('{') {
                let block_end = self.find_matching_brace(&else_code, 0)?;
                let block = else_code[1..block_end].to_string();
                return self.execute_block_internal(&block);
            } else if else_code.starts_with("if") {
                // else if
                return self.handle_if_statement_internal(&else_code);
            } else {
                // Single statement
                return self.execute_single_statement_internal(&else_code);
            }
        }
        
        Ok(StatementResult::Value(JSValue::Undefined))
    }

    fn handle_while_loop(&mut self, code: &str) -> Result<JSValue, String> {
        match self.handle_while_loop_internal(code)? {
            StatementResult::Value(v) => Ok(v),
            StatementResult::Return(v) => Ok(v),
        }
    }
    
    fn handle_while_loop_internal(&mut self, code: &str) -> Result<StatementResult, String> {
        // Parse: while (condition) { body }
        let code = code.trim_start_matches("while").trim();
        
        // Find condition
        let cond_start = code.find('(').ok_or("Invalid while loop")?;
        let cond_end = self.find_matching_paren(code, cond_start)?;
        let condition = &code[cond_start+1..cond_end];
        
        // Find body
        let rest = &code[cond_end+1..].trim();
        let body = if rest.starts_with('{') {
            let block_end = self.find_matching_brace(rest, 0)?;
            &rest[1..block_end]
        } else {
            rest
        };
        
        // Execute loop
        let mut result = JSValue::Undefined;
        loop {
            let cond_value = self.evaluate_expression(condition)?;
            if !self.to_boolean(&cond_value) {
                break;
            }
            
            // Check if body returns early
            match self.execute_block_internal(body)? {
                StatementResult::Value(v) => result = v,
                StatementResult::Return(v) => return Ok(StatementResult::Return(v)),
            }
        }
        
        Ok(StatementResult::Value(result))
    }

    fn handle_for_loop(&mut self, code: &str) -> Result<JSValue, String> {
        match self.handle_for_loop_internal(code)? {
            StatementResult::Value(v) => Ok(v),
            StatementResult::Return(v) => Ok(v),
        }
    }
    
    fn handle_for_loop_internal(&mut self, code: &str) -> Result<StatementResult, String> {
        // Parse: for (init; condition; update) { body }
        let code = code.trim_start_matches("for").trim();
        
        // Find loop header
        let header_start = code.find('(').ok_or("Invalid for loop")?;
        let header_end = self.find_matching_paren(code, header_start)?;
        let header = &code[header_start+1..header_end];
        
        // Parse init, condition, update
        let parts: Vec<&str> = header.split(';').collect();
        if parts.len() != 3 {
            return Err("Invalid for loop syntax".to_string());
        }
        
        let init = parts[0].trim();
        let condition = parts[1].trim();
        let update = parts[2].trim();
        
        // Find body
        let rest = &code[header_end+1..].trim();
        let body = if rest.starts_with('{') {
            let block_end = self.find_matching_brace(rest, 0)?;
            &rest[1..block_end]
        } else {
            rest
        };
        
        // Execute loop
        if !init.is_empty() {
            self.execute_single_statement(init)?;
        }
        
        let mut result = JSValue::Undefined;
        loop {
            if !condition.is_empty() {
                let cond_value = self.evaluate_expression(condition)?;
                if !self.to_boolean(&cond_value) {
                    break;
                }
            }
            
            // Check if body returns early
            match self.execute_block_internal(body)? {
                StatementResult::Value(v) => result = v,
                StatementResult::Return(v) => return Ok(StatementResult::Return(v)),
            }
            
            if !update.is_empty() {
                self.execute_single_statement(update)?;
            }
        }
        
        Ok(StatementResult::Value(result))
    }

    fn find_matching_paren(&self, code: &str, start: usize) -> Result<usize, String> {
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let chars: Vec<char> = code.chars().collect();
        
        if start >= chars.len() || chars[start] != '(' {
            return Err("Start position is not an opening parenthesis".to_string());
        }
        
        for i in start..chars.len() {
            let c = chars[i];
            
            // Track string state to skip parentheses in strings
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
            }
            
            if in_string {
                continue;
            }
            
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }
        
        Err("Unmatched parenthesis".to_string())
    }

    fn find_matching_brace(&self, code: &str, start: usize) -> Result<usize, String> {
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let chars: Vec<char> = code.chars().collect();
        
        if start >= chars.len() || chars[start] != '{' {
            return Err("Start position is not an opening brace".to_string());
        }
        
        for i in start..chars.len() {
            let c = chars[i];
            
            // Track string state to skip braces in strings
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
            }
            
            if in_string {
                continue;
            }
            
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }
        
        Err("Unmatched brace".to_string())
    }

    /// Find the first assignment operator (=) outside of strings and parens
    fn find_assignment_operator(&self, code: &str) -> Option<usize> {
        let mut in_string = false;
        let mut string_char = ' ';
        let mut paren_depth = 0;
        let chars: Vec<char> = code.chars().collect();
        
        for i in 0..chars.len() {
            let c = chars[i];
            
            // Track string state
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
            }
            
            if in_string {
                continue;
            }
            
            // Track parenthesis depth
            if c == '(' {
                paren_depth += 1;
            } else if c == ')' {
                paren_depth -= 1;
            }
            
            // Check for assignment operator (not ==, !=, <=, >=)
            if c == '=' && paren_depth == 0 {
                let before = if i > 0 { chars.get(i - 1) } else { None };
                let after = chars.get(i + 1);
                
                // Skip if it's part of a comparison operator
                let is_comparison = matches!(before, Some('=' | '!' | '<' | '>')) ||
                                   matches!(after, Some('='));
                
                if !is_comparison {
                    return Some(i);
                }
            }
        }
        
        None
    }

    fn handle_variable_declaration(&mut self, code: &str) -> Result<JSValue, String> {
        let trimmed = code.trim_end_matches(';');
        
        // Find the assignment operator position
        let eq_pos = self.find_assignment_operator(trimmed)
            .ok_or("No assignment operator found")?;
        
        let left = trimmed[..eq_pos].trim();
        let right = trimmed[eq_pos + 1..].trim();
        
        // Check if this is an assignment without declaration (variable update)
        if !trimmed.starts_with("var ") && !trimmed.starts_with("let ") && !trimmed.starts_with("const ") {
            // This is a variable assignment: name = value
            let name = left;
            let value = self.evaluate_expression(right)?;
            
            // Update in current frame if exists, otherwise global
            if let Some(frame) = self.call_stack.last_mut() {
                if frame.locals.contains_key(name) {
                    frame.locals.insert(name.to_string(), value.clone());
                    return Ok(value);
                }
            }
            
            self.global_scope.insert(name.to_string(), value.clone());
            return Ok(value);
        }
        
        // Handle var/let/const declarations
        let name = left.split_whitespace().last().ok_or("Invalid variable name")?;
        let value = self.evaluate_expression(right)?;
        
        // Store in current frame if in function, otherwise global
        if let Some(frame) = self.call_stack.last_mut() {
            frame.locals.insert(name.to_string(), value.clone());
        } else {
            self.global_scope.insert(name.to_string(), value.clone());
        }
        
        Ok(value)
    }

    fn evaluate_expression(&mut self, expr: &str) -> Result<JSValue, String> {
        let expr = expr.trim();
        
        // Handle quoted strings - check if ENTIRE expression is a single string literal
        if expr.len() >= 2 {
            let first_char = expr.chars().next().unwrap();
            if first_char == '"' || first_char == '\'' {
                // Check if this is a complete string literal (no operators outside quotes)
                let mut in_string = false;
                let mut string_char = ' ';
                let mut found_end = false;
                
                for (i, c) in expr.chars().enumerate() {
                    if i == 0 {
                        in_string = true;
                        string_char = c;
                        continue;
                    }
                    
                    if in_string && c == string_char {
                        in_string = false;
                        if i == expr.len() - 1 {
                            // Entire expression is a string literal
                            found_end = true;
                        }
                        break;
                    }
                }
                
                if found_end {
                    return Ok(JSValue::String(expr[1..expr.len()-1].to_string()));
                }
            }
        }
        
        // Handle parentheses for grouping - but only if they wrap the ENTIRE expression
        if expr.starts_with('(') && expr.ends_with(')') {
            // Check if these parentheses actually match (entire expression is wrapped)
            if let Ok(matching) = self.find_matching_paren(expr, 0) {
                if matching == expr.len() - 1 {
                    // The parentheses wrap the entire expression
                    return self.evaluate_expression(&expr[1..expr.len()-1]);
                }
            }
        }

        // Handle function calls: name(args)
        if let Some(paren_pos) = expr.find('(') {
            // Check if this is actually a function call by verifying:
            // 1. The name before '(' is a valid identifier
            // 2. There's a matching closing paren at the end
            let func_name = expr[..paren_pos].trim();
            
            // Verify it's a valid identifier (alphanumeric + underscore, not starting with digit)
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') &&
               !func_name.chars().next().unwrap().is_numeric() {
                
                // Find the matching closing paren
                if let Ok(close_paren) = self.find_matching_paren(expr, paren_pos) {
                    if close_paren == expr.len() - 1 {
                        // This is a function call
                        let args_str = &expr[paren_pos+1..close_paren];
                        
                        // Handle console methods specially
                        if func_name == "console.log" || func_name == "console.error" || func_name == "console.warn" {
                            // Already handled above, but just in case
                            let value = self.evaluate_expression(args_str)?;
                            let message = self.value_to_string(&value);
                            if func_name.ends_with("log") {
                                self.console.log(&message);
                            } else if func_name.ends_with("error") {
                                self.console.error(&message);
                            } else {
                                self.console.warn(&message);
                            }
                            return Ok(JSValue::Undefined);
                        }
                        
                        // Check if it's a function - clone it to avoid borrow issues
                        if let Some(JSValue::Function(func)) = self.global_scope.get(func_name).cloned() {
                            return self.execute_function(&func, args_str);
                        }
                    }
                }
            }
        }
        
        // Number
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(JSValue::Number(num));
        }

        // Boolean
        match expr {
            "true" => return Ok(JSValue::Boolean(true)),
            "false" => return Ok(JSValue::Boolean(false)),
            "undefined" => return Ok(JSValue::Undefined),
            "null" => return Ok(JSValue::Null),
            _ => {}
        }

        // Binary operations - process in precedence order
        // Logical OR (lowest precedence)
        if let Some(result) = self.try_binary_op(expr, "||")? {
            return Ok(result);
        }
        // Logical AND
        if let Some(result) = self.try_binary_op(expr, "&&")? {
            return Ok(result);
        }
        // Equality operators
        for op in &["===", "!==", "==", "!="] {
            if let Some(result) = self.try_binary_op(expr, op)? {
                return Ok(result);
            }
        }
        // Comparison operators
        for op in &["<=", ">=", "<", ">"] {
            if let Some(result) = self.try_binary_op(expr, op)? {
                return Ok(result);
            }
        }
        // Addition and subtraction
        for op in &["+", "-"] {
            if let Some(result) = self.try_binary_op(expr, op)? {
                return Ok(result);
            }
        }
        // Multiplication, division, modulo (highest precedence)
        for op in &["*", "/", "%"] {
            if let Some(result) = self.try_binary_op(expr, op)? {
                return Ok(result);
            }
        }

        // Variable lookup - check current frame first, then global
        if let Some(frame) = self.call_stack.last() {
            if let Some(value) = frame.locals.get(expr) {
                return Ok(value.clone());
            }
        }
        if let Some(value) = self.global_scope.get(expr) {
            return Ok(value.clone());
        }

        // If nothing else matches, it's undefined
        Ok(JSValue::Undefined)
    }

    fn try_binary_op(&mut self, expr: &str, op: &str) -> Result<Option<JSValue>, String> {
        // Find the rightmost operator at depth 0, but skip parentheses and strings
        // This ensures right-to-left processing for same precedence operators
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut last_op_pos: Option<usize> = None;
        
        let chars: Vec<char> = expr.chars().collect();
        let op_chars: Vec<char> = op.chars().collect();
        
        for i in 0..chars.len() {
            let c = chars[i];
            
            // Track string state
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
            }
            if in_string {
                continue;
            }
            
            // Track parenthesis depth
            if c == '(' || c == '[' {
                depth += 1;
            } else if c == ')' || c == ']' {
                depth -= 1;
            } else if depth == 0 && i + op_chars.len() <= chars.len() {
                // Check if operator matches at this position
                let matches = op_chars.iter().enumerate().all(|(j, &ch)| {
                    chars.get(i + j) == Some(&ch)
                });
                
                if matches {
                    // Make sure the character before and after are not part of another operator
                    let before_ok = i == 0 || {
                        let prev = chars[i.saturating_sub(1)];
                        // For single-char ops, check we're not part of a multi-char op
                        if op.len() == 1 {
                            !(prev == '=' || prev == '!' || prev == '<' || prev == '>' || 
                              prev == '&' || prev == '|' || prev == '+' || prev == '-' || 
                              prev == '*' || prev == '/')
                        } else {
                            true
                        }
                    };
                    
                    let after_ok = i + op.len() >= chars.len() || {
                        let next = chars[i + op.len()];
                        // For single-char ops, check we're not part of a multi-char op
                        if op.len() == 1 {
                            !(next == '=' || (op == "&" && next == '&') || (op == "|" && next == '|'))
                        } else {
                            true
                        }
                    };
                    
                    if before_ok && after_ok {
                        last_op_pos = Some(i);
                    }
                }
            }
        }
        
        if let Some(pos) = last_op_pos {
            let left = expr[..pos].trim();
            let right = expr[pos + op.len()..].trim();
            
            if !left.is_empty() && !right.is_empty() {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                return Ok(Some(match op {
                    "+" => self.add(left_val, right_val)?,
                    "-" => self.subtract(left_val, right_val)?,
                    "*" => self.multiply(left_val, right_val)?,
                    "/" => self.divide(left_val, right_val)?,
                    "%" => self.modulo(left_val, right_val)?,
                    "==" | "===" => JSValue::Boolean(self.equals(&left_val, &right_val)),
                    "!=" | "!==" => JSValue::Boolean(!self.equals(&left_val, &right_val)),
                    "<" => self.less_than(left_val, right_val)?,
                    ">" => self.greater_than(left_val, right_val)?,
                    "<=" => self.less_equal(left_val, right_val)?,
                    ">=" => self.greater_equal(left_val, right_val)?,
                    "&&" => JSValue::Boolean(self.to_boolean(&left_val) && self.to_boolean(&right_val)),
                    "||" => JSValue::Boolean(self.to_boolean(&left_val) || self.to_boolean(&right_val)),
                    _ => return Err(format!("Unknown operator: {}", op)),
                }));
            }
        }
        Ok(None)
    }

    fn execute_function(&mut self, func: &JSFunction, args_str: &str) -> Result<JSValue, String> {
        // Parse arguments
        let args = self.parse_arguments(args_str)?;
        
        // Create new stack frame
        self.push_frame(func.name.clone());
        
        // Bind parameters to arguments
        for (i, param) in func.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(JSValue::Undefined);
            if let Some(frame) = self.call_stack.last_mut() {
                frame.locals.insert(param.clone(), value);
            }
        }
        
        // Execute function body
        let result = self.execute_block(&func.body)?;
        
        // Pop stack frame
        self.pop_frame();
        
        Ok(result)
    }

    fn parse_arguments(&mut self, args_str: &str) -> Result<Vec<JSValue>, String> {
        if args_str.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        
        for c in args_str.chars() {
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char {
                    in_string = false;
                }
                current_arg.push(c);
            } else if in_string {
                current_arg.push(c);
            } else if c == '(' || c == '[' || c == '{' {
                depth += 1;
                current_arg.push(c);
            } else if c == ')' || c == ']' || c == '}' {
                depth -= 1;
                current_arg.push(c);
            } else if c == ',' && depth == 0 {
                args.push(self.evaluate_expression(&current_arg)?);
                current_arg.clear();
            } else {
                current_arg.push(c);
            }
        }
        
        if !current_arg.trim().is_empty() {
            args.push(self.evaluate_expression(&current_arg)?);
        }
        
        Ok(args)
    }

    fn contains_multiple_statements(&self, code: &str) -> bool {
        // Check if code contains multiple statements by looking for semicolons
        // or multiple lines with substantial content (not just in strings/blocks)
        let statements = self.split_statements(code);
        statements.len() > 1
    }

    fn execute_block(&mut self, code: &str) -> Result<JSValue, String> {
        match self.execute_block_internal(code)? {
            StatementResult::Value(v) => Ok(v),
            StatementResult::Return(v) => Ok(v),
        }
    }
    
    fn execute_block_internal(&mut self, code: &str) -> Result<StatementResult, String> {
        let mut result = JSValue::Undefined;
        let statements = self.split_statements(code);
        
        for stmt in statements {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Handle return statements
            if trimmed.starts_with("return ") {
                let expr = &trimmed[7..].trim_end_matches(';');
                let value = self.evaluate_expression(expr)?;
                return Ok(StatementResult::Return(value));
            }
            
            // Execute statement and check if it returned early
            match self.execute_single_statement_internal(trimmed)? {
                StatementResult::Value(v) => result = v,
                StatementResult::Return(v) => return Ok(StatementResult::Return(v)),
            }
        }
        
        Ok(StatementResult::Value(result))
    }

    fn split_statements(&self, code: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current = String::new();
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
                current.push(c);
            } else if in_string {
                current.push(c);
            } else if c == '(' {
                paren_depth += 1;
                current.push(c);
            } else if c == ')' {
                paren_depth -= 1;
                current.push(c);
            } else if c == '{' {
                brace_depth += 1;
                current.push(c);
            } else if c == '}' {
                brace_depth -= 1;
                current.push(c);
                
                // After a closing brace at depth 0, check if we should split
                if brace_depth == 0 && paren_depth == 0 {
                    let trimmed = current.trim();
                    
                    // Check if this is a loop or if statement
                    let is_loop = trimmed.starts_with("while ") || trimmed.starts_with("while(") ||
                                  trimmed.starts_with("for ") || trimmed.starts_with("for(");
                    let is_if = trimmed.starts_with("if ") || trimmed.starts_with("if(");
                    
                    if is_loop || is_if {
                        // Look ahead past whitespace
                        let mut j = i + 1;
                        while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                            j += 1;
                        }
                        
                        if j < chars.len() {
                            let rest = &chars[j..];
                            let rest_str: String = rest.iter().take(10).collect();
                            
                            // For loops: split before return/var/if/while/for
                            // For if: only split before var/let/const (not before return!)
                            let should_split = if is_loop {
                                chars[j] != '}' &&
                                chars[j] != ')' &&
                                chars[j] != ';' &&
                                (rest_str.starts_with("var ") ||
                                 rest_str.starts_with("let ") ||
                                 rest_str.starts_with("const ") ||
                                 rest_str.starts_with("return ") ||
                                 rest_str.starts_with("if ") ||
                                 rest_str.starts_with("if(") ||
                                 rest_str.starts_with("while ") ||
                                 rest_str.starts_with("while(") ||
                                 rest_str.starts_with("for ") ||
                                 rest_str.starts_with("for(") ||
                                 rest_str.starts_with("function "))
                            } else {  // is_if
                                // For if: split before var/let/const OR before an identifier (assignment)
                                // But NOT before else, }, ), ;, or return
                                !rest_str.starts_with("else") &&
                                chars[j] != '}' &&
                                chars[j] != ')' &&
                                chars[j] != ';' &&
                                !rest_str.starts_with("return ") &&
                                (rest_str.starts_with("var ") ||
                                 rest_str.starts_with("let ") ||
                                 rest_str.starts_with("const ") ||
                                 // Check if it's an identifier (starts with letter or _)
                                 chars[j].is_alphabetic() || chars[j] == '_')
                            };
                            
                            if should_split {
                                statements.push(current.trim().to_string());
                                current.clear();
                            }
                        }
                    }
                }
            } else if c == ';' && brace_depth == 0 && paren_depth == 0 {
                // Only split on semicolons outside of braces AND parentheses
                statements.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(c);
            }
            
            i += 1;
        }
        
        if !current.trim().is_empty() {
            statements.push(current.trim().to_string());
        }
        
        statements
    }
    
    fn add(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (left, right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Number(l + r)),
            (JSValue::String(l), JSValue::String(r)) => Ok(JSValue::String(format!("{}{}", l, r))),
            (l, r) => Ok(JSValue::String(format!("{}{}", self.value_to_string(&l), self.value_to_string(&r)))),
        }
    }
    
    fn to_number(&self, value: &JSValue) -> f64 {
        match value {
            JSValue::Number(n) => *n,
            JSValue::Boolean(true) => 1.0,
            JSValue::Boolean(false) => 0.0,
            JSValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            JSValue::Null => 0.0,
            JSValue::Undefined => f64::NAN,
            _ => f64::NAN,
        }
    }
    
    fn subtract(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Number(l - r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Number(l - r))
            }
        }
    }
    
    fn multiply(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Number(l * r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Number(l * r))
            }
        }
    }
    
    fn divide(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => {
                if *r == 0.0 {
                    Ok(JSValue::Number(f64::INFINITY))
                } else {
                    Ok(JSValue::Number(l / r))
                }
            },
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                if r == 0.0 {
                    Ok(JSValue::Number(f64::INFINITY))
                } else {
                    Ok(JSValue::Number(l / r))
                }
            }
        }
    }
    
    fn modulo(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Number(l % r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Number(l % r))
            }
        }
    }
    
    fn less_than(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Boolean(l < r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Boolean(l < r))
            }
        }
    }
    
    fn greater_than(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Boolean(l > r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Boolean(l > r))
            }
        }
    }
    
    fn less_equal(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Boolean(l <= r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Boolean(l <= r))
            }
        }
    }
    
    fn greater_equal(&self, left: JSValue, right: JSValue) -> Result<JSValue, String> {
        match (&left, &right) {
            (JSValue::Number(l), JSValue::Number(r)) => Ok(JSValue::Boolean(l >= r)),
            _ => {
                let l = self.to_number(&left);
                let r = self.to_number(&right);
                Ok(JSValue::Boolean(l >= r))
            }
        }
    }
    
    fn equals(&self, left: &JSValue, right: &JSValue) -> bool {
        match (left, right) {
            (JSValue::Number(l), JSValue::Number(r)) => l == r,
            (JSValue::String(l), JSValue::String(r)) => l == r,
            (JSValue::Boolean(l), JSValue::Boolean(r)) => l == r,
            (JSValue::Null, JSValue::Null) => true,
            (JSValue::Undefined, JSValue::Undefined) => true,
            _ => false,
        }
    }
    
    fn to_boolean(&self, value: &JSValue) -> bool {
        match value {
            JSValue::Boolean(b) => *b,
            JSValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JSValue::String(s) => !s.is_empty(),
            JSValue::Null | JSValue::Undefined => false,
            JSValue::Object(_) | JSValue::Function(_) | JSValue::Array(_) => true,
        }
    }

    pub fn create_object(&mut self, properties: HashMap<String, JSValue>) -> usize {
        let obj = JSObject {
            properties,
            prototype: None,
        };
        self.heap.push(obj);
        self.heap.len() - 1
    }

    pub fn get_property(&self, obj_index: usize, key: &str) -> Option<JSValue> {
        self.heap.get(obj_index)?.properties.get(key).cloned()
    }

    pub fn set_property(&mut self, obj_index: usize, key: String, value: JSValue) {
        if let Some(obj) = self.heap.get_mut(obj_index) {
            obj.properties.insert(key, value);
        }
    }
    
    fn value_to_string(&self, value: &JSValue) -> String {
        match value {
            JSValue::Undefined => "undefined".to_string(),
            JSValue::Null => "null".to_string(),
            JSValue::Boolean(b) => b.to_string(),
            JSValue::Number(n) => n.to_string(),
            JSValue::String(s) => s.clone(),
            JSValue::Object(idx) => format!("[Object {}]", idx),
            JSValue::Function(f) => format!("[Function: {}]", f.name),
            JSValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

impl Default for JSEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Event Loop for async operations
#[derive(Debug)]
pub struct EventLoop {
    tasks: Vec<Task>,
    microtasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub callback: String,
    pub delay: u64,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            microtasks: Vec::new(),
        }
    }

    pub fn queue_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn queue_microtask(&mut self, task: Task) {
        self.microtasks.push(task);
    }

    pub fn run(&mut self, engine: &mut JSEngine) {
        // Process microtasks first
        while let Some(task) = self.microtasks.pop() {
            let _ = engine.execute(&task.callback);
        }

        // Process tasks
        while let Some(task) = self.tasks.pop() {
            let _ = engine.execute(&task.callback);
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_engine() {
        let mut engine = JSEngine::new();

        // Test variable declaration
        let result = engine.execute("var x = 42");
        assert!(result.is_ok(), "Variable declaration failed: {:?}", result);

        // Test console.log (AST mode has its own console, legacy mode uses engine.console)
        let result = engine.execute("console.log(\"Hello World\")");
        assert!(result.is_ok(), "Console.log failed: {:?}", result);

        // Test in legacy mode where console logs are tracked
        // Create separate instance to avoid RefCell borrow conflicts
        {
            let mut legacy_engine = JSEngine::new_legacy();
            let result = legacy_engine.execute("console.log(\"Legacy mode\")");
            assert!(result.is_ok());
            assert_eq!(legacy_engine.console.logs.len(), 1);
        }
    }
}
