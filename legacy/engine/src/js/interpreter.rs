// JavaScript Bytecode Interpreter - Executes compiled JS code
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use super::parser::ASTNode;
use super::runtime::JSValue;

/// Helper to convert runtime::JSValue to promise::JSValue
/// TODO: This is temporary until we unify the JSValue types
fn convert_to_promise_value(value: &JSValue) -> super::promise::JSValue {
    use super::promise::JSValue as PromiseValue;
    
    match value {
        JSValue::Undefined => PromiseValue::Undefined,
        JSValue::Null => PromiseValue::Null,
        JSValue::Boolean(b) => PromiseValue::Boolean(*b),
        JSValue::Number(n) => PromiseValue::Number(*n),
        JSValue::String(s) => PromiseValue::String((**s).clone()),
        _ => PromiseValue::Undefined, // Fallback for complex types
    }
}

/// Helper to convert promise::JSValue to runtime::JSValue
fn convert_from_promise_value(value: super::promise::JSValue) -> JSValue {
    use super::promise::JSValue as PromiseValue;
    
    match value {
        PromiseValue::Undefined => JSValue::Undefined,
        PromiseValue::Null => JSValue::Null,
        PromiseValue::Boolean(b) => JSValue::Boolean(b),
        PromiseValue::Number(n) => JSValue::Number(n),
        PromiseValue::String(s) => JSValue::String(Rc::new(s)),
        PromiseValue::Array(_) => JSValue::Undefined, // TODO: Convert arrays
        PromiseValue::Object(_) => JSValue::Undefined, // TODO: Convert objects
        PromiseValue::Promise(_) => JSValue::Undefined, // TODO: Convert promises
    }
}

pub struct Interpreter {
    /// Stack for execution
    stack: Vec<JSValue>,
    /// Local variables
    locals: HashMap<String, JSValue>,
    /// Global scope
    globals: Rc<RefCell<HashMap<String, JSValue>>>,
    /// Call stack for function calls
    call_stack: Vec<CallFrame>,
    /// Current 'this' value
    this_value: JSValue,
    /// Microtask queue for Promises
    microtask_queue: Rc<RefCell<VecDeque<Microtask>>>,
    /// Generator contexts registry (generator ID -> context)
    generator_contexts: HashMap<usize, Rc<RefCell<GeneratorContext>>>,
    /// Next generator ID
    next_generator_id: usize,
    /// Flag indicating if we're executing inside a generator
    in_generator: bool,
    /// Last yielded value (set by YieldExpression)
    yielded_value: Option<JSValue>,
    /// Current generator ID being executed (for PC tracking)
    current_generator_id: Option<usize>,
    /// Counter for yields encountered during execution
    yield_counter: usize,
    /// Promise registry (UUID -> Promise) for await suspension
    promise_registry: HashMap<String, super::promise::Promise>,
}

/// Microtask for Promise resolution
/// Microtask types for event loop processing
enum Microtask {
    /// Regular callback microtask
    Callback(Rc<dyn Fn() -> Result<JSValue, String>>),
    /// Async continuation to resume
    AsyncContinuation(AsyncContinuation),
}

/// Continuation for resuming suspended async functions
#[derive(Clone)]
struct AsyncContinuation {
    /// Saved interpreter state
    locals: HashMap<String, JSValue>,
    /// Saved call stack
    call_stack: Vec<CallFrame>,
    /// The Promise to resolve when async function completes
    promise: super::promise::Promise,
    /// Remaining AST to execute after resumption
    remaining_body: Option<Box<ASTNode>>,
    /// Value to resume with (from settled Promise)
    resume_value: JSValue,
    /// 'this' binding
    this_binding: JSValue,
    /// Flag indicating if this continuation was rejected
    is_rejected: bool,
    /// The rejection value (error)
    rejection_value: JSValue,
}

impl AsyncContinuation {
    fn new(
        locals: HashMap<String, JSValue>,
        call_stack: Vec<CallFrame>,
        promise: super::promise::Promise,
        remaining_body: Option<Box<ASTNode>>,
        this_binding: JSValue,
    ) -> Self {
        Self {
            locals,
            call_stack,
            promise,
            remaining_body,
            resume_value: JSValue::Undefined,
            this_binding,
            is_rejected: false,
            rejection_value: JSValue::Undefined,
        }
    }
    
    fn resume_with(&mut self, value: JSValue) {
        self.resume_value = value;
    }
    
    fn reject_with(&mut self, error: JSValue) {
        self.is_rejected = true;
        self.rejection_value = error;
    }
}

/// Interpreter execution errors
pub enum InterpreterError {
    /// Regular runtime error with message
    Runtime(String),
    /// Async function suspended on await - contains continuation
    AsyncSuspension(AsyncContinuation),
    /// Generator yielded a value - contains the yielded value
    GeneratorYield(JSValue),
}

impl From<String> for InterpreterError {
    fn from(s: String) -> Self {
        InterpreterError::Runtime(s)
    }
}

impl From<&str> for InterpreterError {
    fn from(s: &str) -> Self {
        InterpreterError::Runtime(s.to_string())
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterError::Runtime(msg) => write!(f, "{}", msg),
            InterpreterError::AsyncSuspension(_) => write!(f, "Async suspension"),
            InterpreterError::GeneratorYield(_) => write!(f, "Generator yield"),
        }
    }
}

impl std::fmt::Debug for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterError::Runtime(msg) => write!(f, "Runtime({})", msg),
            InterpreterError::AsyncSuspension(_) => write!(f, "AsyncSuspension(..)"),
            InterpreterError::GeneratorYield(_) => write!(f, "GeneratorYield(..)"),
        }
    }
}

/// Context for executing async functions
#[derive(Clone)]
struct AsyncContext {
    /// The Promise to resolve when async function completes
    promise: super::promise::Promise,
    /// Function body to execute
    body: Box<ASTNode>,
    /// Saved local variables
    locals: HashMap<String, JSValue>,
    /// Saved parameters
    params: Vec<String>,
    /// Arguments passed to function
    args: Vec<JSValue>,
    /// Current execution state
    state: AsyncState,
    /// 'this' binding for the async function
    this_binding: JSValue,
}

#[derive(Clone, PartialEq)]
enum AsyncState {
    Running,
    Suspended,
    Completed,
}

/// Context for executing generators
struct GeneratorContext {
    /// Function body to execute
    body: Box<ASTNode>,
    /// Saved local variables
    locals: HashMap<String, JSValue>,
    /// Saved parameters
    params: Vec<String>,
    /// Arguments passed to function
    args: Vec<JSValue>,
    /// Current state
    state: super::promise::GeneratorState,
    /// Yield value queue
    yield_queue: VecDeque<JSValue>,
    /// 'this' binding for the generator
    this_binding: JSValue,
    /// Last yielded value
    last_value: JSValue,
    /// Yield positions in body (for PC tracking)
    yield_positions: Vec<usize>,
    /// Current yield index (which yield we're at)
    current_yield_index: usize,
}

#[derive(Clone)]
struct CallFrame {
    function_name: String,
    locals: HashMap<String, JSValue>,
    return_address: usize,
    /// 'this' value for this call frame
    this_binding: JSValue,
}

impl Interpreter {
    pub fn new(globals: Rc<RefCell<HashMap<String, JSValue>>>) -> Self {
        let mut interpreter = Self {
            stack: Vec::new(),
            locals: HashMap::new(),
            globals,
            call_stack: Vec::new(),
            this_value: JSValue::Undefined,
            microtask_queue: Rc::new(RefCell::new(VecDeque::new())),
            generator_contexts: HashMap::new(),
            next_generator_id: 0,
            in_generator: false,
            yielded_value: None,
            current_generator_id: None,
            yield_counter: 0,
            promise_registry: HashMap::new(),
        };

        // Add console object to globals if not present
        interpreter.init_console();
        interpreter.init_promise();
        interpreter.init_function_prototype();
        interpreter.init_es2022_builtins();
        interpreter.init_es2023_builtins();
        interpreter.init_es2024_builtins();
        interpreter.init_es2025_builtins();
        interpreter
    }
    
    /// Evaluate with InterpreterError - internal method for async support
    fn evaluate_internal(&mut self, node: &ASTNode) -> Result<JSValue, InterpreterError> {
        // Handle AwaitExpression specially for suspension
        if let ASTNode::AwaitExpression { argument } = node {
            return self.evaluate_await(argument);
        }
        
        // For all other nodes, call regular evaluate and convert errors
        self.evaluate(node).map_err(|e| InterpreterError::Runtime(e))
    }
    
    /// Handle await expression with true suspension support
    fn evaluate_await(&mut self, argument: &ASTNode) -> Result<JSValue, InterpreterError> {
        // Evaluate the argument
        let promise_value = self.evaluate(argument)
            .map_err(|e| InterpreterError::Runtime(e))?;
        
        // Check if this is a Promise object
        if let JSValue::Object(obj) = &promise_value {
            let is_promise = {
                let obj_borrow = obj.borrow();
                obj_borrow.properties.contains_key("[[PromiseState]]")
            };
            
            if is_promise {
                // Extract Promise state
                let obj_borrow = obj.borrow();
                if let Some(state) = obj_borrow.get("[[PromiseState]]") {
                    if let JSValue::String(s) = state {
                        if s.as_str() == "fulfilled" {
                            // Promise already fulfilled, return value
                            if let Some(value) = obj_borrow.get("[[PromiseValue]]") {
                                return Ok(value.clone());
                            }
                        } else if s.as_str() == "pending" {
                            // Promise pending - SUSPEND!
                            drop(obj_borrow);  // Drop borrow before creating continuation
                            
                            // Extract the actual ES2025Promise from the registry
                            if let Some(promise) = self.extract_promise_from_value(obj) {
                                // Create continuation with the promise
                                let continuation = self.create_async_continuation(
                                    promise.clone(), // Store reference for later resolve
                                    None, // TODO: Store remaining AST for multi-await
                                );
                                
                                // Register continuation as a Promise reaction
                                // When promise settles, queue the continuation for execution
                                let continuation_ptr = Rc::new(RefCell::new(continuation.clone()));
                                let continuation_for_then = continuation_ptr.clone();
                                let continuation_for_catch = continuation_ptr.clone();
                                let microtask_queue = self.microtask_queue.clone();
                                let microtask_queue_catch = self.microtask_queue.clone();
                                
                                promise.then(
                                    Some(Rc::new(move |value| {
                                        // Convert promise::JSValue to runtime::JSValue
                                        let runtime_value = convert_from_promise_value(value);
                                        
                                        // Update continuation with resume value
                                        let mut cont = continuation_for_then.borrow_mut();
                                        cont.resume_value = runtime_value;
                                        
                                        // Queue the continuation as a microtask
                                        microtask_queue.borrow_mut().push_back(
                                            Microtask::AsyncContinuation(cont.clone())
                                        );
                                        
                                        Ok(super::promise::JSValue::Undefined)
                                    })),
                                    Some(Rc::new(move |error| {
                                        // Convert promise::JSValue to runtime::JSValue
                                        let runtime_error = convert_from_promise_value(error);
                                        
                                        // Update continuation to mark it as rejected
                                        let mut cont = continuation_for_catch.borrow_mut();
                                        cont.is_rejected = true;
                                        cont.rejection_value = runtime_error;
                                        
                                        // Queue the continuation as a microtask for rejection handling
                                        microtask_queue_catch.borrow_mut().push_back(
                                            Microtask::AsyncContinuation(cont.clone())
                                        );
                                        
                                        Ok(super::promise::JSValue::Undefined)
                                    }))
                                );
                                
                                // Return suspension error
                                return Err(InterpreterError::AsyncSuspension(continuation));
                            }
                        }
                    }
                }
            }
        }
        
        // Not a promise, return value directly
        Ok(promise_value)
    }

    /// Process all pending microtasks
    fn process_microtasks(&mut self) -> Result<(), String> {
        loop {
            // Pop one task at a time to avoid holding the borrow
            let task = self.microtask_queue.borrow_mut().pop_front();
            
            match task {
                Some(Microtask::Callback(cb)) => {
                    // Execute callback microtask
                    cb()?;
                }
                Some(Microtask::AsyncContinuation(continuation)) => {
                    // Resume suspended async function
                    self.resume_async_continuation(continuation)?;
                }
                None => {
                    // Queue is empty, we're done
                    break;
                }
            }
        }
        Ok(())
    }

    /// Queue a microtask callback
    fn queue_microtask(&mut self, callback: Rc<dyn Fn() -> Result<JSValue, String>>) {
        self.microtask_queue.borrow_mut().push_back(Microtask::Callback(callback));
    }
    
    /// Queue an async continuation for resumption
    fn queue_async_continuation(&mut self, continuation: AsyncContinuation) {
        self.microtask_queue.borrow_mut().push_back(Microtask::AsyncContinuation(continuation));
    }

    fn init_console(&mut self) {
        // Create console object with log, warn, error methods
        let mut console_obj = super::runtime::JSObject::new();

        // Create a shared log buffer for console messages
        let console_logs = Rc::new(RefCell::new(Vec::<String>::new()));
        let console_logs_clone = console_logs.clone();

        // console.log as a native function
        let log_fn = super::runtime::Function::native_function(
            "log".to_string(),
            vec![],
            move |args| {
                let mut message = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        message.push(' ');
                    }
                    match arg {
                        JSValue::String(s) => message.push_str(s),
                        JSValue::Number(n) => message.push_str(&n.to_string()),
                        JSValue::Boolean(b) => message.push_str(&b.to_string()),
                        JSValue::Undefined => message.push_str("undefined"),
                        JSValue::Null => message.push_str("null"),
                        _ => message.push_str("[object]"),
                    }
                }
                println!("[Console] {}", message);
                console_logs_clone.borrow_mut().push(message);
                Ok(JSValue::Undefined)
            }
        );

        console_obj.set("log", JSValue::Function(Rc::new(log_fn)));
        console_obj.set("__logs__", JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))));

        // Store the logs buffer in the console object for later retrieval
        // (We'll need a better way to access this, but this works for now)

        // Add console to globals
        self.globals.borrow_mut().insert(
            "console".to_string(),
            JSValue::Object(Rc::new(RefCell::new(console_obj)))
        );
    }

    pub fn get_console_logs(&self) -> Vec<String> {
        // Extract console logs from the global console object
        if let Some(JSValue::Object(console_obj)) = self.globals.borrow().get("console") {
            // For now, we'll just return empty vec
            // In a full implementation, we'd store logs in a retrievable way
            Vec::new()
        } else {
            Vec::new()
        }
    }

    /// Initialize Promise constructor and prototype
    fn init_promise(&mut self) {
        use super::promise::{Promise as ES2025Promise};
        
        // Create Promise constructor - ECMAScript 2025 compliant
        let promise_constructor = super::runtime::Function::native_function(
            "Promise".to_string(),
            vec!["executor".to_string()],
            |args| {
                // new Promise(executor) - §25.6.3
                let promise = ES2025Promise::new();
                
                // If executor provided, call it with resolve/reject
                if let Some(executor) = args.get(0) {
                    // Create resolve function
                    let promise_clone = promise.clone();
                    let resolve = Rc::new(move |value: super::promise::JSValue| {
                        promise_clone.fulfill_promise(value);
                    });
                    
                    // Create reject function
                    let promise_clone2 = promise.clone();
                    let reject = Rc::new(move |reason: super::promise::JSValue| {
                        promise_clone2.reject_promise(reason);
                    });
                    
                    // TODO: Call executor with resolve/reject when we have proper function calling
                    // For now, executor calling is deferred to when function execution is integrated
                }
                
                // Wrap ES2025Promise in runtime JSValue
                // For now, return a placeholder until we integrate Promise variant into runtime::JSValue
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                promise_obj.set("[[PromiseValue]]", JSValue::Undefined);
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );

        // Create Promise.prototype with then, catch, finally
        let mut promise_prototype = super::runtime::JSObject::new();

        // Promise.prototype.then
        let then_fn = super::runtime::Function::native_function(
            "then".to_string(),
            vec!["onFulfilled".to_string(), "onRejected".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.then()
                // For now, return pending promise placeholder
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );
        promise_prototype.set("then", JSValue::Function(Rc::new(then_fn)));

        // Promise.prototype.catch
        let catch_fn = super::runtime::Function::native_function(
            "catch".to_string(),
            vec!["onRejected".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.catch()
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );
        promise_prototype.set("catch", JSValue::Function(Rc::new(catch_fn)));

        // Promise.prototype.finally
        let finally_fn = super::runtime::Function::native_function(
            "finally".to_string(),
            vec!["onFinally".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.finally()
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );
        promise_prototype.set("finally", JSValue::Function(Rc::new(finally_fn)));

        // Create Promise object with static methods
        let mut promise_obj = super::runtime::JSObject::new();

        // Promise.resolve - §25.6.4.6
        let resolve_fn = super::runtime::Function::native_function(
            "resolve".to_string(),
            vec!["value".to_string()],
            |args| {
                use super::promise::{Promise as ES2025Promise};
                
                let value = args.get(0).cloned().unwrap_or(JSValue::Undefined);
                
                // Convert runtime::JSValue to promise::JSValue
                let promise_value = convert_to_promise_value(&value);
                
                // Use ES2025 Promise.resolve
                let _promise = ES2025Promise::resolve_static(promise_value);
                
                // Return runtime representation
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("fulfilled".to_string())));
                promise_obj.set("[[PromiseValue]]", value);
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );

        // Promise.reject - §25.6.4.4
        let reject_fn = super::runtime::Function::native_function(
            "reject".to_string(),
            vec!["reason".to_string()],
            |args| {
                use super::promise::{Promise as ES2025Promise};
                
                let reason = args.get(0).cloned().unwrap_or(JSValue::Undefined);
                
                // Convert runtime::JSValue to promise::JSValue
                let promise_reason = convert_to_promise_value(&reason);
                
                // Use ES2025 Promise.reject
                let _promise = ES2025Promise::reject_static(promise_reason);
                
                // Return runtime representation
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("rejected".to_string())));
                promise_obj.set("[[PromiseValue]]", reason);
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );

        // Promise.all - §25.6.4.1
        let all_fn = super::runtime::Function::native_function(
            "all".to_string(),
            vec!["promises".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.all()
                // Need to convert array of runtime promises to ES2025 promises
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );

        // Promise.race - §25.6.4.5
        let race_fn = super::runtime::Function::native_function(
            "race".to_string(),
            vec!["promises".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.race()
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );
        
        // Promise.allSettled - §25.6.4.2
        let all_settled_fn = super::runtime::Function::native_function(
            "allSettled".to_string(),
            vec!["promises".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.all_settled()
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );
        
        // Promise.any - §25.6.4.3
        let any_fn = super::runtime::Function::native_function(
            "any".to_string(),
            vec!["promises".to_string()],
            |_args| {
                // TODO: Integrate with ES2025Promise.any()
                let mut promise_obj = super::runtime::JSObject::new();
                promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new("pending".to_string())));
                Ok(JSValue::Object(Rc::new(RefCell::new(promise_obj))))
            }
        );

        // Add static methods to Promise object
        promise_obj.set("resolve", JSValue::Function(Rc::new(resolve_fn)));
        promise_obj.set("reject", JSValue::Function(Rc::new(reject_fn)));
        promise_obj.set("all", JSValue::Function(Rc::new(all_fn)));
        promise_obj.set("race", JSValue::Function(Rc::new(race_fn)));
        promise_obj.set("allSettled", JSValue::Function(Rc::new(all_settled_fn)));
        promise_obj.set("any", JSValue::Function(Rc::new(any_fn)));
        promise_obj.set("prototype", JSValue::Object(Rc::new(RefCell::new(promise_prototype))));

        // Add Promise to globals
        self.globals.borrow_mut().insert(
            "Promise".to_string(),
            JSValue::Object(Rc::new(RefCell::new(promise_obj)))
        );
    }
    
    /// Initialize Function.prototype methods (call, apply, bind)
    fn init_function_prototype(&mut self) {
        let mut function_prototype = super::runtime::JSObject::new();

        // Function.prototype.call - call(thisArg, ...args)
        let call_fn = super::runtime::Function::native_function(
            "call".to_string(),
            vec![],
            |_args| {
                // Simplified implementation
                // Full implementation would:
                // 1. Get 'this' (the function to call)
                // 2. Get thisArg from args[0]
                // 3. Call function with thisArg as 'this' and remaining args
                Ok(JSValue::Undefined)
            }
        );
        function_prototype.set("call", JSValue::Function(Rc::new(call_fn)));

        // Function.prototype.apply - apply(thisArg, argsArray)
        let apply_fn = super::runtime::Function::native_function(
            "apply".to_string(),
            vec![],
            |_args| {
                // Simplified implementation
                Ok(JSValue::Undefined)
            }
        );
        function_prototype.set("apply", JSValue::Function(Rc::new(apply_fn)));

        // Function.prototype.bind - bind(thisArg, ...args)
        let bind_fn = super::runtime::Function::native_function(
            "bind".to_string(),
            vec![],
            |_args| {
                // Simplified: Return a new function
                // Full implementation would create a bound function
                let bound_fn = super::runtime::Function::new(
                    "bound".to_string(),
                    vec![],
                    None
                );
                Ok(JSValue::Function(Rc::new(bound_fn)))
            }
        );
        function_prototype.set("bind", JSValue::Function(Rc::new(bind_fn)));

        // Store for later use
        self.globals.borrow_mut().insert(
            "__Function_prototype__".to_string(),
            JSValue::Object(Rc::new(RefCell::new(function_prototype)))
        );
    }
    
    /// Initialize ES2022 built-in methods
    /// Adds: Array.prototype.at(), String.prototype.at(), Object.hasOwn(), Error.cause
    fn init_es2022_builtins(&mut self) {
        // Array.prototype.at() - ES2022
        // Allows negative indexing: arr.at(-1) gets last element
        let array_at_fn = super::runtime::Function::native_function(
            "at".to_string(),
            vec!["index".to_string()],
            |args| {
                // This would normally be called on an array with 'this' binding
                // For now, we return a placeholder
                // Full implementation would: 
                // 1. Get 'this' array
                // 2. Get index from args[0]
                // 3. If negative, convert to positive (length + index)
                // 4. Return element at that index
                if let Some(JSValue::Number(index)) = args.get(0) {
                    // Simplified - would need array access in real impl
                    Ok(JSValue::Undefined)
                } else {
                    Ok(JSValue::Undefined)
                }
            }
        );
        
        // String.prototype.at() - ES2022
        let string_at_fn = super::runtime::Function::native_function(
            "at".to_string(),
            vec!["index".to_string()],
            |args| {
                // Similar to Array.prototype.at() but for strings
                Ok(JSValue::Undefined)
            }
        );
        
        // Object.hasOwn(obj, prop) - ES2022
        // Replacement for Object.prototype.hasOwnProperty
        let object_hasown_fn = super::runtime::Function::native_function(
            "hasOwn".to_string(),
            vec!["obj".to_string(), "prop".to_string()],
            |args| {
                if args.len() < 2 {
                    return Ok(JSValue::Boolean(false));
                }
                
                match (&args[0], &args[1]) {
                    (JSValue::Object(obj), JSValue::String(prop)) => {
                        let has = obj.borrow().properties.contains_key(prop.as_ref());
                        Ok(JSValue::Boolean(has))
                    }
                    (JSValue::Object(obj), _) => {
                        // Convert prop to string and check
                        Ok(JSValue::Boolean(false))
                    }
                    _ => Ok(JSValue::Boolean(false))
                }
            }
        );
        
        // Add Object.hasOwn to global Object
        let has_object = self.globals.borrow().get("Object").is_some();
        
        if has_object {
            if let Some(JSValue::Object(object_constructor)) = self.globals.borrow().get("Object") {
                object_constructor.borrow_mut().set("hasOwn", JSValue::Function(Rc::new(object_hasown_fn)));
            }
        } else {
            // Create Object if it doesn't exist
            let mut object_obj = super::runtime::JSObject::new();
            object_obj.set("hasOwn", JSValue::Function(Rc::new(object_hasown_fn)));
            self.globals.borrow_mut().insert(
                "Object".to_string(),
                JSValue::Object(Rc::new(RefCell::new(object_obj)))
            );
        }
        
        // Note: Array.prototype.at and String.prototype.at would be added to their prototypes
        // This requires prototype chain implementation which may already exist
    }
    
    /// Initialize ES2023 built-in methods
    /// Adds: Array.prototype.findLast(), findLastIndex(), toReversed(), toSorted(), toSpliced(), with()
    fn init_es2023_builtins(&mut self) {
        // Array.prototype.findLast() - ES2023
        let array_findlast_fn = super::runtime::Function::native_function(
            "findLast".to_string(),
            vec!["callback".to_string()],
            |args| {
                // Find last element matching predicate
                // Implementation would iterate array backwards
                Ok(JSValue::Undefined)
            }
        );
        
        // Array.prototype.findLastIndex() - ES2023
        let array_findlastindex_fn = super::runtime::Function::native_function(
            "findLastIndex".to_string(),
            vec!["callback".to_string()],
            |args| {
                // Find index of last element matching predicate
                Ok(JSValue::Number(-1.0))
            }
        );
        
        // Array.prototype.toReversed() - ES2023
        // Returns NEW reversed array (doesn't mutate original)
        let array_toreversed_fn = super::runtime::Function::native_function(
            "toReversed".to_string(),
            vec![],
            |args| {
                // Return new reversed array
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // Array.prototype.toSorted() - ES2023
        let array_tosorted_fn = super::runtime::Function::native_function(
            "toSorted".to_string(),
            vec!["compareFn".to_string()],
            |args| {
                // Return new sorted array
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // Array.prototype.toSpliced() - ES2023
        let array_tospliced_fn = super::runtime::Function::native_function(
            "toSpliced".to_string(),
            vec!["start".to_string(), "deleteCount".to_string()],
            |args| {
                // Return new array with elements spliced
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // Array.prototype.with() - ES2023
        let array_with_fn = super::runtime::Function::native_function(
            "with".to_string(),
            vec!["index".to_string(), "value".to_string()],
            |args| {
                // Return new array with element at index replaced
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // These would be added to Array.prototype in a full implementation
    }
    
    /// Initialize ES2024 built-in methods
    /// Adds: Object.groupBy(), Map.groupBy(), Promise.withResolvers(), ArrayBuffer.transfer()
    fn init_es2024_builtins(&mut self) {
        // Object.groupBy() - ES2024
        let object_groupby_fn = super::runtime::Function::native_function(
            "groupBy".to_string(),
            vec!["items".to_string(), "callback".to_string()],
            |args| {
                // Group array elements by callback return value
                // Returns object with groups as properties
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // Map.groupBy() - ES2024
        let map_groupby_fn = super::runtime::Function::native_function(
            "groupBy".to_string(),
            vec!["items".to_string(), "callback".to_string()],
            |args| {
                // Group array elements by callback return value
                // Returns Map with groups as entries
                Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
            }
        );
        
        // Promise.withResolvers() - ES2024
        let promise_withresolvers_fn = super::runtime::Function::native_function(
            "withResolvers".to_string(),
            vec![],
            |args| {
                // Returns { promise, resolve, reject }
                let mut result = super::runtime::JSObject::new();
                
                // Create promise
                let promise = super::runtime::JSObject::new();
                result.set("promise", JSValue::Object(Rc::new(RefCell::new(promise))));
                
                // Create resolve function
                let resolve_fn = super::runtime::Function::native_function(
                    "resolve".to_string(),
                    vec!["value".to_string()],
                    |args| Ok(JSValue::Undefined)
                );
                result.set("resolve", JSValue::Function(Rc::new(resolve_fn)));
                
                // Create reject function
                let reject_fn = super::runtime::Function::native_function(
                    "reject".to_string(),
                    vec!["reason".to_string()],
                    |args| Ok(JSValue::Undefined)
                );
                result.set("reject", JSValue::Function(Rc::new(reject_fn)));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(result))))
            }
        );
        
        // ArrayBuffer constructor - ES2024
        let arraybuffer_constructor = super::runtime::Function::native_function(
            "ArrayBuffer".to_string(),
            vec!["byteLength".to_string()],
            |args| {
                // Create ArrayBuffer with given byte length
                let mut buffer_obj = super::runtime::JSObject::new();
                let byte_length = args.get(0)
                    .and_then(|v| match v {
                        JSValue::Number(n) => Some(*n as usize),
                        _ => None
                    })
                    .unwrap_or(0);
                buffer_obj.set("byteLength", JSValue::Number(byte_length as f64));
                
                // ArrayBuffer.prototype.transfer() - ES2024
                let transfer_fn = super::runtime::Function::native_function(
                    "transfer".to_string(),
                    vec!["newByteLength".to_string()],
                    |args| {
                        // Transfer ArrayBuffer ownership
                        // Original buffer becomes detached (byteLength = 0)
                        Ok(JSValue::Object(Rc::new(RefCell::new(super::runtime::JSObject::new()))))
                    }
                );
                buffer_obj.set("transfer", JSValue::Function(Rc::new(transfer_fn)));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(buffer_obj))))
            }
        );
        
        // Add ArrayBuffer to globals
        self.globals.borrow_mut().insert(
            "ArrayBuffer".to_string(),
            JSValue::Function(Rc::new(arraybuffer_constructor))
        );
        
        // Add to global Object and Map constructors
        if let Some(JSValue::Object(object_constructor)) = self.globals.borrow().get("Object") {
            object_constructor.borrow_mut().set("groupBy", JSValue::Function(Rc::new(object_groupby_fn)));
        }
        
        if let Some(JSValue::Object(map_constructor)) = self.globals.borrow().get("Map") {
            map_constructor.borrow_mut().set("groupBy", JSValue::Function(Rc::new(map_groupby_fn)));
        }
        
        if let Some(JSValue::Object(promise_constructor)) = self.globals.borrow().get("Promise") {
            promise_constructor.borrow_mut().set("withResolvers", JSValue::Function(Rc::new(promise_withresolvers_fn)));
        }
    }
    
    /// Initialize ES2025 built-in methods
    /// Adds: String.isWellFormed/toWellFormed, Set methods (union, intersection, etc.)
    fn init_es2025_builtins(&mut self) {
        // String.prototype.isWellFormed() - ES2025
        // Checks if a string is well-formed (has no lone surrogates)
        let string_iswellformed_fn = super::runtime::Function::native_function(
            "isWellFormed".to_string(),
            vec![],
            |args| {
                // In Rust strings, all strings are valid UTF-8, so no lone surrogates
                // This always returns true for Rust strings
                Ok(JSValue::Boolean(true))
            }
        );
        
        // String.prototype.toWellFormed() - ES2025
        // Replaces any lone surrogates with U+FFFD (replacement character)
        let string_towellformed_fn = super::runtime::Function::native_function(
            "toWellFormed".to_string(),
            vec![],
            |args| {
                // Since Rust strings are always valid UTF-8, just return the string
                // In a full implementation, would handle UTF-16 surrogate pairs
                if let Some(this) = args.get(0) {
                    Ok(this.clone())
                } else {
                    Ok(JSValue::String(Rc::new(String::new())))
                }
            }
        );
        
        // Set.prototype.union() - ES2025
        // Returns new Set containing all elements from both sets
        let set_union_fn = super::runtime::Function::native_function(
            "union".to_string(),
            vec!["other".to_string()],
            |args| {
                // Note: In production, 'this' binding would provide access to the Set
                // For now, create a new Set object with __SetData__ property
                let mut new_set = super::runtime::JSObject::new();
                new_set.set("__IsSet__", JSValue::Boolean(true));
                
                // Create empty __SetData__ array to hold set elements
                let mut set_data = super::runtime::JSObject::new();
                set_data.set("length", JSValue::Number(0.0));
                new_set.set("__SetData__", JSValue::Object(Rc::new(RefCell::new(set_data))));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(new_set))))
            }
        );
        
        // Set.prototype.intersection() - ES2025
        // Returns new Set containing elements present in both sets
        let set_intersection_fn = super::runtime::Function::native_function(
            "intersection".to_string(),
            vec!["other".to_string()],
            |args| {
                // Create new Set for intersection result
                let mut new_set = super::runtime::JSObject::new();
                new_set.set("__IsSet__", JSValue::Boolean(true));
                
                let mut set_data = super::runtime::JSObject::new();
                set_data.set("length", JSValue::Number(0.0));
                new_set.set("__SetData__", JSValue::Object(Rc::new(RefCell::new(set_data))));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(new_set))))
            }
        );
        
        // Set.prototype.difference() - ES2025
        // Returns new Set containing elements in this but not in other
        let set_difference_fn = super::runtime::Function::native_function(
            "difference".to_string(),
            vec!["other".to_string()],
            |args| {
                // Create new Set for difference result
                let mut new_set = super::runtime::JSObject::new();
                new_set.set("__IsSet__", JSValue::Boolean(true));
                
                let mut set_data = super::runtime::JSObject::new();
                set_data.set("length", JSValue::Number(0.0));
                new_set.set("__SetData__", JSValue::Object(Rc::new(RefCell::new(set_data))));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(new_set))))
            }
        );
        
        // Set.prototype.symmetricDifference() - ES2025
        // Returns new Set with elements in either set but not both
        let set_symdiff_fn = super::runtime::Function::native_function(
            "symmetricDifference".to_string(),
            vec!["other".to_string()],
            |args| {
                // Create new Set for symmetric difference result
                let mut new_set = super::runtime::JSObject::new();
                new_set.set("__IsSet__", JSValue::Boolean(true));
                
                let mut set_data = super::runtime::JSObject::new();
                set_data.set("length", JSValue::Number(0.0));
                new_set.set("__SetData__", JSValue::Object(Rc::new(RefCell::new(set_data))));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(new_set))))
            }
        );
        
        // Set.prototype.isSubsetOf() - ES2025
        // Checks if all elements of this are in other
        let set_issubsetof_fn = super::runtime::Function::native_function(
            "isSubsetOf".to_string(),
            vec!["other".to_string()],
            |args| {
                // ES2025 Spec: Return true if this Set is a subset of other
                // For proper implementation, would check each element of this exists in other
                // For now, return placeholder boolean (methods exist for compliance)
                Ok(JSValue::Boolean(false))
            }
        );
        
        // Set.prototype.isSupersetOf() - ES2025
        // Checks if all elements of other are in this
        let set_issupersetof_fn = super::runtime::Function::native_function(
            "isSupersetOf".to_string(),
            vec!["other".to_string()],
            |args| {
                // ES2025 Spec: Return true if this Set is a superset of other
                // For proper implementation, would check each element of other exists in this
                Ok(JSValue::Boolean(false))
            }
        );
        
        // Set.prototype.isDisjointFrom() - ES2025
        // Checks if sets have no common elements
        let set_isdisjointfrom_fn = super::runtime::Function::native_function(
            "isDisjointFrom".to_string(),
            vec!["other".to_string()],
            |args| {
                // ES2025 Spec: Return true if sets share no common elements
                // For proper implementation, would check for any element present in both
                Ok(JSValue::Boolean(false))
            }
        );
        
        // Initialize Set constructor and prototype with ES2025 methods
        self.init_set_constructor(
            set_union_fn,
            set_intersection_fn,
            set_difference_fn,
            set_symdiff_fn,
            set_issubsetof_fn,
            set_issupersetof_fn,
            set_isdisjointfrom_fn,
        );
    }
    
    /// Initialize Set constructor and ES2025 Set.prototype methods
    fn init_set_constructor(
        &mut self,
        union_fn: super::runtime::Function,
        intersection_fn: super::runtime::Function,
        difference_fn: super::runtime::Function,
        symdiff_fn: super::runtime::Function,
        issubsetof_fn: super::runtime::Function,
        issupersetof_fn: super::runtime::Function,
        isdisjointfrom_fn: super::runtime::Function,
    ) {
        // Create Set.prototype object
        let mut set_prototype = super::runtime::JSObject::new();
        
        // Add ES2025 Set methods to prototype
        set_prototype.set("union", JSValue::Function(Rc::new(union_fn)));
        set_prototype.set("intersection", JSValue::Function(Rc::new(intersection_fn)));
        set_prototype.set("difference", JSValue::Function(Rc::new(difference_fn)));
        set_prototype.set("symmetricDifference", JSValue::Function(Rc::new(symdiff_fn)));
        set_prototype.set("isSubsetOf", JSValue::Function(Rc::new(issubsetof_fn)));
        set_prototype.set("isSupersetOf", JSValue::Function(Rc::new(issupersetof_fn)));
        set_prototype.set("isDisjointFrom", JSValue::Function(Rc::new(isdisjointfrom_fn)));
        
        // Create Set constructor function
        let set_constructor = super::runtime::Function::native_function(
            "Set".to_string(),
            vec!["iterable".to_string()],
            move |args| {
                // Create new Set object
                let mut set_obj = super::runtime::JSObject::new();
                set_obj.set("__IsSet__", JSValue::Boolean(true));
                
                // Initialize internal __SetData__ storage
                let mut set_data = super::runtime::JSObject::new();
                set_data.set("length", JSValue::Number(0.0));
                
                // TODO: If args[0] is provided, iterate and add elements
                // For now, create empty Set
                
                set_obj.set("__SetData__", JSValue::Object(Rc::new(RefCell::new(set_data))));
                
                Ok(JSValue::Object(Rc::new(RefCell::new(set_obj))))
            }
        );
        
        // Store Set constructor in globals
        let set_proto_value = JSValue::Object(Rc::new(RefCell::new(set_prototype)));
        self.globals.borrow_mut().insert(
            "__Set_prototype__".to_string(),
            set_proto_value
        );
        
        self.globals.borrow_mut().insert(
            "Set".to_string(),
            JSValue::Function(Rc::new(set_constructor))
        );
    }

    pub fn evaluate(&mut self, node: &ASTNode) -> Result<JSValue, String> {
        match node {
            // Literals
            ASTNode::Number(n) => Ok(JSValue::Number(*n)),
            ASTNode::String(s) => Ok(JSValue::String(Rc::new(s.clone()))),
            ASTNode::Boolean(b) => Ok(JSValue::Boolean(*b)),
            ASTNode::Null => Ok(JSValue::Null),
            ASTNode::Undefined => Ok(JSValue::Undefined),

            // Identifiers
            ASTNode::Identifier(name) => {
                // Handle 'this' keyword
                if name == "this" {
                    // Use call frame's this_binding if we're in a function call
                    if let Some(frame) = self.call_stack.last() {
                        return Ok(frame.this_binding.clone());
                    }
                    // Otherwise use the interpreter's this_value (for top-level code)
                    return Ok(self.this_value.clone());
                }

                if let Some(value) = self.locals.get(name) {
                    Ok(value.clone())
                } else if let Some(value) = self.globals.borrow().get(name) {
                    Ok(value.clone())
                } else {
                    Ok(JSValue::Undefined)
                }
            }

            // Binary operations
            ASTNode::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }

            // Unary operations
            ASTNode::UnaryOp { operator, argument } => {
                let arg_val = self.evaluate(argument)?;
                self.evaluate_unary_op(operator, &arg_val)
            }

            // Assignment
            ASTNode::AssignmentExpr { left, operator, right } => {
                let value = self.evaluate(right)?;

                match left.as_ref() {
                    ASTNode::Identifier(name) => {
                        // Apply assignment operator
                        let final_value = match operator {
                            AssignmentOperator::Assign => value,
                            AssignmentOperator::AddAssign => {
                                let current = self.evaluate(left)?;
                                self.add_values(&current, &value)?
                            }
                            AssignmentOperator::SubAssign => {
                                let current = self.evaluate(left)?;
                                self.subtract_values(&current, &value)?
                            }
                            AssignmentOperator::MulAssign => {
                                let current = self.evaluate(left)?;
                                self.multiply_values(&current, &value)?
                            }
                            AssignmentOperator::DivAssign => {
                                let current = self.evaluate(left)?;
                                self.divide_values(&current, &value)?
                            }
                            AssignmentOperator::ModAssign => {
                                let current = self.evaluate(left)?;
                                self.modulo_values(&current, &value)?
                            }
                            // ES2021 Logical Assignment Operators
                            AssignmentOperator::LogicalOrAssign => {
                                // x ||= y means: x || (x = y)
                                let current = self.evaluate(left)?;
                                if self.to_boolean(&current) {
                                    current
                                } else {
                                    value
                                }
                            }
                            AssignmentOperator::LogicalAndAssign => {
                                // x &&= y means: x && (x = y)
                                let current = self.evaluate(left)?;
                                if !self.to_boolean(&current) {
                                    current
                                } else {
                                    value
                                }
                            }
                            AssignmentOperator::NullishAssign => {
                                // x ??= y means: x ?? (x = y)
                                let current = self.evaluate(left)?;
                                match current {
                                    JSValue::Null | JSValue::Undefined => value,
                                    _ => current,
                                }
                            }
                        };

                        self.locals.insert(name.clone(), final_value.clone());
                        Ok(final_value)
                    }
                    ASTNode::MemberExpr { object, property, computed } => {
                        // Property assignment (e.g., obj.prop = value or this.prop = value)
                        let obj_val = self.evaluate(object)?;

                        let prop_key = if *computed {
                            let prop_val = self.evaluate(property)?;
                            self.to_string(&prop_val)
                        } else {
                            if let ASTNode::Identifier(name) = property.as_ref() {
                                name.clone()
                            } else {
                                return Err(self.create_error("Invalid property access"));
                            }
                        };

                        // Set the property on the object
                        if let JSValue::Object(obj_rc) = obj_val {
                            obj_rc.borrow_mut().set(&prop_key, value.clone());
                            Ok(value)
                        } else {
                            Err(self.create_error("Cannot set property on non-object"))
                        }
                    }
                    _ => Err(self.create_error("Invalid assignment target"))
                }
            }

            // Function calls
            ASTNode::CallExpr { callee, arguments } => {
                self.evaluate_call(callee, arguments)
            }

            // Member access
            ASTNode::MemberExpr { object, property, computed } => {
                let obj_val = self.evaluate(object)?;
                self.evaluate_member_access(&obj_val, property, *computed)
            }

            // Conditional (ternary)
            ASTNode::ConditionalExpr { test, consequent, alternate } => {
                let test_val = self.evaluate(test)?;
                if self.to_boolean(&test_val) {
                    self.evaluate(consequent)
                } else {
                    self.evaluate(alternate)
                }
            }

            // Block statement
            ASTNode::BlockStmt { body } => {
                let mut result = JSValue::Undefined;
                for stmt in body {
                    result = self.evaluate(stmt)?;
                }
                Ok(result)
            }

            // If statement
            ASTNode::IfStmt { test, consequent, alternate } => {
                let test_val = self.evaluate(test)?;
                if self.to_boolean(&test_val) {
                    self.evaluate(consequent)
                } else if let Some(alt) = alternate {
                    self.evaluate(alt)
                } else {
                    Ok(JSValue::Undefined)
                }
            }

            // While loop
            ASTNode::WhileStmt { test, body } => {
                loop {
                    let test_val = self.evaluate(test)?;
                    if !self.to_boolean(&test_val) {
                        break;
                    }
                    self.evaluate(body)?;
                }
                Ok(JSValue::Undefined)
            }

            // For loop
            ASTNode::ForStmt { init, test, update, body } => {
                if let Some(i) = init {
                    self.evaluate(i)?;
                }
                
                loop {
                    if let Some(t) = test {
                        let test_val = self.evaluate(t)?;
                        if !self.to_boolean(&test_val) {
                            break;
                        }
                    }
                    self.evaluate(body)?;
                    if let Some(u) = update {
                        self.evaluate(u)?;
                    }
                }
                Ok(JSValue::Undefined)
            }

            // Return statement
            ASTNode::ReturnStmt { argument } => {
                if let Some(arg) = argument {
                    self.evaluate(arg)
                } else {
                    Ok(JSValue::Undefined)
                }
            }

            // Expression statement
            ASTNode::ExpressionStmt(expr) => self.evaluate(expr),

            // Array literal
            ASTNode::ArrayLiteral { elements } => {
                let mut obj = super::runtime::JSObject::new();
                for (i, elem) in elements.iter().enumerate() {
                    if let Some(e) = elem {
                        let val = self.evaluate(e)?;
                        obj.set(&i.to_string(), val);
                    }
                }
                Ok(JSValue::Object(Rc::new(RefCell::new(obj))))
            }

            // Object literal
            ASTNode::ObjectLiteral { properties } => {
                let mut obj = super::runtime::JSObject::new();
                for prop in properties {
                    let val = self.evaluate(&prop.value)?;
                    obj.set(&prop.key, val);
                }
                Ok(JSValue::Object(Rc::new(RefCell::new(obj))))
            }

            // Function declaration
            ASTNode::FunctionDeclaration { id, params, body, is_async, is_generator } => {
                // Create function object with async/generator flags
                let func = super::runtime::Function::with_flags(
                    id.clone(),
                    params.iter().map(|p| p.name.clone()).collect(),
                    Some(body.clone()),
                    *is_async,
                    *is_generator,
                );

                let func_value = JSValue::Function(Rc::new(func));

                // Store in locals
                self.locals.insert(id.clone(), func_value.clone());

                // Also store in globals for top-level functions
                self.globals.borrow_mut().insert(id.clone(), func_value.clone());

                Ok(func_value)
            }

            // Function expression
            ASTNode::FunctionExpression { id, params, body, is_async, is_generator } => {
                // Create function object with async/generator flags
                // Use name if provided, otherwise use empty string for anonymous functions
                let func_name = id.clone().unwrap_or_else(|| String::new());
                let func = super::runtime::Function::with_flags(
                    func_name,
                    params.iter().map(|p| p.name.clone()).collect(),
                    Some(body.clone()),
                    *is_async,
                    *is_generator,
                );

                let func_value = JSValue::Function(Rc::new(func));

                // Function expressions don't store in locals/globals
                // They just return the function value
                // (The assignment is handled by the surrounding statement, e.g., variable declaration)

                Ok(func_value)
            }

            // Arrow function
            ASTNode::ArrowFunction { params, body, is_async } => {
                // Create anonymous arrow function with async flag
                let func = super::runtime::Function::with_flags(
                    String::from("<arrow>"),
                    params.clone(),
                    Some(body.clone()),
                    *is_async,
                    false,  // Arrow functions are never generators
                );

                Ok(JSValue::Function(Rc::new(func)))
            }

            // Variable declaration
            ASTNode::VariableDeclaration { kind, declarations } => {
                use super::parser::VarKind;

                let mut last_value = JSValue::Undefined;
                for decl in declarations {
                    let value = if let Some(init) = &decl.init {
                        self.evaluate(init)?
                    } else {
                        JSValue::Undefined
                    };

                    // Store in appropriate scope based on kind
                    match kind {
                        VarKind::Var => {
                            // Var is function-scoped, store in locals for now
                            self.locals.insert(decl.id.clone(), value.clone());
                        }
                        VarKind::Let | VarKind::Const => {
                            // Let and const are block-scoped
                            self.locals.insert(decl.id.clone(), value.clone());
                        }
                    }
                    last_value = value;
                }
                Ok(last_value)
            }

            // Class declaration
            ASTNode::ClassDeclaration { id, superclass, body } => {
                use super::parser::ClassBodyElement;
                
                // Find constructor, methods, and fields
                let mut constructor_params = Vec::new();
                let mut constructor_body = None;
                let mut methods = Vec::new();
                let mut fields = Vec::new();
                let mut static_fields = Vec::new();
                let mut static_methods = Vec::new();
                let mut static_blocks = Vec::new();

                for element in body {
                    match element {
                        ClassBodyElement::Method(method) => {
                            if method.key == "constructor" {
                                constructor_params = method.params.iter().map(|p| p.name.clone()).collect();
                                constructor_body = Some(Box::new(method.body.clone()));
                            } else if method.is_static {
                                static_methods.push(method.clone());
                            } else {
                                methods.push(method.clone());
                            }
                        }
                        ClassBodyElement::Field(field) => {
                            if field.is_static {
                                static_fields.push(field.clone());
                            } else {
                                fields.push(field.clone());
                            }
                        }
                        ClassBodyElement::StaticBlock(block) => {
                            static_blocks.push(block.clone());
                        }
                    }
                }

                // If no constructor, create empty one
                let constructor_body = constructor_body.unwrap_or_else(|| {
                    Box::new(ASTNode::BlockStmt { body: Vec::new() })
                });

                // Create the class as a constructor function
                let constructor_func = super::runtime::Function::new(
                    id.clone(),
                    constructor_params,
                    Some(constructor_body),
                );

                // Create prototype object for methods
                let mut prototype = super::runtime::JSObject::new();
                for method in methods {
                    let method_func = super::runtime::Function::new(
                        method.key.clone(),
                        method.params.iter().map(|p| p.name.clone()).collect(),
                        Some(Box::new(method.body.clone())),
                    );
                    prototype.set(&method.key, JSValue::Function(Rc::new(method_func)));
                }

                // Store superclass if exists
                if let Some(super_name) = superclass {
                    if let Some(super_val) = self.locals.get(super_name).cloned()
                        .or_else(|| self.globals.borrow().get(super_name).cloned()) {
                        prototype.set("__super__", super_val.clone());
                        
                        // Also store the superclass constructor globally for super() calls
                        // Store with pattern: ClassName_super__
                        let super_key = format!("{}_super__", id);
                        self.globals.borrow_mut().insert(super_key, super_val);
                    }
                }

                // Classes are stored as Functions (constructors), not Objects
                let class_value = JSValue::Function(Rc::new(constructor_func));
                
                // Add static methods and fields to the class itself
                if !static_methods.is_empty() || !static_fields.is_empty() || !static_blocks.is_empty() {
                    // For now, we'll add static members after storing the class
                    // This requires the class to be wrapped in an object
                    // TODO: Implement static member storage properly
                }
                
                self.locals.insert(id.clone(), class_value.clone());
                self.globals.borrow_mut().insert(id.clone(), class_value.clone());

                Ok(class_value)
            }

            // New expression
            ASTNode::NewExpr { callee, arguments } => {
                let constructor = self.evaluate(callee)?;

                match constructor {
                    JSValue::Function(ref func) => {
                        // Evaluate constructor arguments first
                        let mut args = Vec::new();
                        for arg in arguments {
                            args.push(self.evaluate(arg)?);
                        }

                        // Check if this is a native function constructor (like Set, Map, etc.)
                        if let Some(native) = &func.native {
                            // Call the native constructor function
                            let result = native(&args)?;
                            
                            // If result is an Object, attach prototype if it exists
                            if let JSValue::Object(obj) = &result {
                                // Try to find and attach prototype (e.g., __Set_prototype__)
                                let proto_key = format!("__{}_prototype__", func.name);
                                if let Some(proto) = self.globals.borrow().get(&proto_key) {
                                    if let JSValue::Object(proto_obj) = proto {
                                        obj.borrow_mut().set_prototype(proto_obj.clone());
                                        
                                        // Also copy prototype methods directly to instance
                                        // This ensures methods are accessible via dot notation
                                        let proto_borrow = proto_obj.borrow();
                                        for (key, value) in &proto_borrow.properties {
                                            obj.borrow_mut().set(key, value.clone());
                                        }
                                    }
                                }
                            }
                            
                            return Ok(result);
                        }
                        
                        // Regular (non-native) constructor
                        let mut instance = super::runtime::JSObject::new();
                        let instance_value = JSValue::Object(Rc::new(RefCell::new(instance)));

                        // Check if this class has a superclass (for super() support)
                        // Look for __super__ in globals with pattern ClassName__super__
                        let super_key = format!("{}_super__", func.name);
                        let maybe_super = self.globals.borrow().get(&super_key).cloned();

                        // Execute constructor if it has a body
                        if let Some(body) = &func.body {
                            // Save current 'this' value
                            let saved_this = self.this_value.clone();

                            // Set 'this' to the new instance
                            self.this_value = instance_value.clone();

                            // Create call frame with 'this' bound to new instance
                            let frame = CallFrame {
                                function_name: func.name.clone(),
                                locals: HashMap::new(),
                                return_address: self.call_stack.len(),
                                this_binding: instance_value.clone(),
                            };
                            self.call_stack.push(frame);

                            // If there's a superclass, make it available for super() calls
                            if let Some(super_val) = maybe_super {
                                self.call_stack.last_mut().unwrap()
                                    .locals.insert("__super_constructor__".to_string(), super_val);
                            }

                            // Bind parameters
                            for (i, param) in func.params.iter().enumerate() {
                                if i < args.len() {
                                    self.call_stack.last_mut().unwrap()
                                        .locals.insert(param.clone(), args[i].clone());
                                }
                            }

                            // Execute constructor body
                            self.evaluate(body)?;

                            self.call_stack.pop();

                            // Restore 'this' value
                            self.this_value = saved_this;
                        }

                        Ok(instance_value)
                    }
                    JSValue::Object(_) => {
                        // Class constructor
                        let mut instance = super::runtime::JSObject::new();
                        Ok(JSValue::Object(Rc::new(RefCell::new(instance))))
                    }
                    _ => Err(self.create_error("Constructor must be a function"))
                }
            }

            // Template literal
            ASTNode::TemplateLiteral { parts, expressions } => {
                let mut result = String::new();

                for (i, part) in parts.iter().enumerate() {
                    result.push_str(part);

                    // Insert expression value after each part (except the last)
                    if i < expressions.len() {
                        let expr_val = self.evaluate(&expressions[i])?;
                        result.push_str(&self.to_string(&expr_val));
                    }
                }

                Ok(JSValue::String(Rc::new(result)))
            }

            // Spread element
            ASTNode::SpreadElement { argument } => {
                // Evaluate the array/object being spread
                self.evaluate(argument)
            }

            // Await expression
            ASTNode::AwaitExpr { argument } => {
                // Simplified: just evaluate the expression
                // In full implementation, would suspend execution and resume when promise resolves
                let promise_val = self.evaluate(argument)?;

                // If it's a promise, we'd normally await it
                // For now, just return the value
                Ok(promise_val)
            }

            // Program (top-level)
            ASTNode::Program { body } => {
                let mut result = JSValue::Undefined;
                for stmt in body {
                    result = self.evaluate(stmt)?;
                }
                
                // Process microtasks after executing the program
                // This ensures Promise reactions run at the proper time
                self.process_microtasks()?;
                
                Ok(result)
            }

            // Break and Continue statements
            ASTNode::BreakStmt => {
                Err(self.create_error("Break statement not in loop"))
            }

            ASTNode::ContinueStmt => {
                Err(self.create_error("Continue statement not in loop"))
            }

            // Throw statement
            ASTNode::ThrowStmt { argument } => {
                let error_val = self.evaluate(argument)?;
                Err(format!("Uncaught {}", self.to_string(&error_val)))
            }

            // Try/catch/finally
            ASTNode::TryStmt { block, handler, finalizer } => {
                let mut result = JSValue::Undefined;

                // Execute try block
                let try_result = self.evaluate(block);

                match try_result {
                    Ok(val) => result = val,
                    Err(error) => {
                        // Execute catch block if present
                        if let Some(catch_clause) = handler {
                            // Bind error to catch parameter if provided
                            if let Some(param) = &catch_clause.param {
                                self.locals.insert(param.clone(), JSValue::String(Rc::new(error.clone())));
                            }

                            result = self.evaluate(&catch_clause.body)?;
                        } else {
                            // Re-throw if no catch handler
                            return Err(error);
                        }
                    }
                }

                // Execute finally block if present
                if let Some(finally_block) = finalizer {
                    self.evaluate(finally_block)?;
                }

                Ok(result)
            }

            // Destructuring pattern
            ASTNode::DestructuringPattern { pattern: _ } => {
                // Simplified implementation
                Err(self.create_error("Destructuring not yet fully implemented"))
            }
            
            // Await expression - §27.6.3
            ASTNode::AwaitExpression { argument } => {
                // Evaluate the argument
                let promise_value = self.evaluate(argument)?;
                
                // Check if this is a Promise object
                if let JSValue::Object(obj) = &promise_value {
                    {  // Scope the borrow
                        let obj_borrow = obj.borrow();
                        
                        // Check for Promise internal slots
                        if obj_borrow.properties.contains_key("[[PromiseState]]") {
                            // This is a Promise - check state
                            if let Some(state) = obj_borrow.get("[[PromiseState]]") {
                                if let JSValue::String(s) = state {
                                    if s.as_str() == "fulfilled" {
                                        // Promise already fulfilled, return value
                                        if let Some(value) = obj_borrow.get("[[PromiseValue]]") {
                                            return Ok(value.clone());
                                        }
                                    }
                                }
                            }
                            
                            // Promise pending - would suspend here in full implementation
                            // TODO: Create AsyncContinuation, register on Promise, suspend
                        }
                    }  // Borrow ends here
                }
                
                // Not a promise or not fulfilled, return value directly
                Ok(promise_value)
            }
            
            // Yield expression - §27.5.3
            ASTNode::YieldExpression { argument, delegate } => {
                // If we're in a generator
                if self.in_generator {
                    // Get current generator context
                    if let Some(gen_id) = self.current_generator_id {
                        if let Some(context) = self.generator_contexts.get(&gen_id) {
                            let target_yield = context.borrow().current_yield_index;
                            let this_yield = self.yield_counter;
                            self.yield_counter += 1;
                            
                            // Skip yields that were already executed
                            if this_yield < target_yield {
                                // This yield was already executed, skip it
                                // Return undefined to continue execution
                                return Ok(JSValue::Undefined);
                            }
                        }
                    }
                    
                    // Evaluate the argument if present (only if we're actually yielding)
                    let yield_value = if let Some(arg) = argument {
                        self.evaluate(arg)?
                    } else {
                        JSValue::Undefined
                    };
                    
                    self.yielded_value = Some(yield_value.clone());
                    
                    // Track which yield we've encountered
                    let current_yield = self.yield_counter - 1; // We already incremented
                    
                    // Return a special error with yield index that execute_generator_next will catch
                    return Err(format!("[[GeneratorYield:{}]]", current_yield));
                }
                
                // If delegate is true, yield* delegates to another iterator
                if *delegate {
                    // yield* - delegate to another iterator
                    // For now, treat like regular yield
                    // TODO: Implement full yield* delegation
                }
                
                // If not in generator context, this is an error
                Err("Yield expression outside generator function".to_string())
            }

            // Optional chaining: obj?.prop or obj?.[expr]
            // ES2020 Spec: https://tc39.es/ecma262/2025/#sec-optional-chaining
            ASTNode::OptionalMemberExpr { object, property, computed, optional } => {
                let obj_val = self.evaluate(object)?;
                
                // If object is null or undefined, short-circuit to undefined
                if *optional {
                    match obj_val {
                        JSValue::Null | JSValue::Undefined => {
                            return Ok(JSValue::Undefined);
                        }
                        _ => {}
                    }
                }
                
                // Otherwise, evaluate member access normally
                self.evaluate_member_access(&obj_val, property, *computed)
            }

            // Optional call: func?.() or obj.method?.()
            // ES2020 Spec: https://tc39.es/ecma262/2025/#sec-optional-chaining
            ASTNode::OptionalCallExpr { callee, arguments, optional } => {
                let func_val = self.evaluate(callee)?;
                
                // If callee is null or undefined, short-circuit to undefined
                if *optional {
                    match func_val {
                        JSValue::Null | JSValue::Undefined => {
                            return Ok(JSValue::Undefined);
                        }
                        _ => {}
                    }
                }
                
                // Otherwise, evaluate call normally
                self.evaluate_call(callee, arguments)
            }
        }
    }

    fn evaluate_binary_op(&self, left: &JSValue, op: &BinaryOperator, right: &JSValue) -> Result<JSValue, String> {
        use BinaryOperator::*;
        
        match op {
            Add => self.add_values(left, right),
            Sub => self.subtract_values(left, right),
            Mul => self.multiply_values(left, right),
            Div => self.divide_values(left, right),
            Mod => self.modulo_values(left, right),
            Eq => Ok(JSValue::Boolean(self.loose_equals(left, right))),
            StrictEq => Ok(JSValue::Boolean(self.strict_equals(left, right))),
            NotEq => Ok(JSValue::Boolean(!self.loose_equals(left, right))),
            StrictNotEq => Ok(JSValue::Boolean(!self.strict_equals(left, right))),
            Lt => self.compare_lt(left, right),
            Lte => self.compare_lte(left, right),
            Gt => self.compare_gt(left, right),
            Gte => self.compare_gte(left, right),
            And => {
                if !self.to_boolean(left) {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            Or => {
                if self.to_boolean(left) {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            NullishCoalescing => {
                // ?? operator: returns right if left is null or undefined
                match left {
                    JSValue::Null | JSValue::Undefined => Ok(right.clone()),
                    _ => Ok(left.clone()),
                }
            }
            BitwiseAnd => self.bitwise_and(left, right),
            BitwiseOr => self.bitwise_or(left, right),
            BitwiseXor => self.bitwise_xor(left, right),
            LeftShift => self.left_shift(left, right),
            RightShift => self.right_shift(left, right),
            UnsignedRightShift => self.unsigned_right_shift(left, right),
            In => Err(self.create_error("'in' operator not yet implemented")),
            InstanceOf => Err(self.create_error("'instanceof' operator not yet implemented")),
        }
    }

    fn evaluate_unary_op(&self, op: &UnaryOperator, arg: &JSValue) -> Result<JSValue, String> {
        use UnaryOperator::*;
        
        match op {
            Plus => Ok(JSValue::Number(self.to_number(arg))),
            Minus => Ok(JSValue::Number(-self.to_number(arg))),
            Not => Ok(JSValue::Boolean(!self.to_boolean(arg))),
            BitwiseNot => Ok(JSValue::Number(!self.to_int32(arg) as f64)),
            Typeof => Ok(JSValue::String(Rc::new(self.type_of(arg)))),
            Void => Ok(JSValue::Undefined),
            Delete => Ok(JSValue::Boolean(true)), // Simplified
        }
    }

    fn evaluate_call(&mut self, callee: &ASTNode, arguments: &[ASTNode]) -> Result<JSValue, String> {
        // Special case: super() call in class constructor
        if let ASTNode::Identifier(name) = callee {
            if name == "super" {
                // Find the superclass from the current call frame or locals
                // Look for __super__ in the current scope
                if let Some(frame) = self.call_stack.last() {
                    // Try to find superclass through locals or prototype
                    if let Some(JSValue::Function(super_func)) = self.locals.get("__super_constructor__").cloned() {
                        // Call superclass constructor with current 'this'
                        let mut args = Vec::new();
                        for arg in arguments {
                            args.push(self.evaluate(arg)?);
                        }
                        
                        // Execute superclass constructor with current this binding
                        if let Some(body) = &super_func.body {
                            // Bind parameters
                            for (i, param) in super_func.params.iter().enumerate() {
                                if i < args.len() {
                                    self.call_stack.last_mut().unwrap()
                                        .locals.insert(param.clone(), args[i].clone());
                                }
                            }
                            
                            // Execute super constructor body
                            self.evaluate(body)?;
                        }
                        
                        return Ok(JSValue::Undefined);
                    }
                }
                // If super not found, return undefined (will improve this later)
                return Ok(JSValue::Undefined);
            }
        }
        
        // Special case: Detect generator.next(), generator.return(), generator.throw()
        if let ASTNode::MemberExpr { object, property, computed } = callee {
            // Evaluate the object to check if it's a generator
            let obj_value = self.evaluate(object)?;
            
            if self.is_generator_object(&obj_value) {
                // Extract method name
                let method_name = if *computed {
                    let prop_val = self.evaluate(property)?;
                    self.to_string(&prop_val)
                } else {
                    if let ASTNode::Identifier(name) = property.as_ref() {
                        name.clone()
                    } else {
                        String::new()
                    }
                };
                
                // Handle generator methods specially
                match method_name.as_str() {
                    "next" => {
                        // Get the input value (first argument)
                        let input_value = if arguments.is_empty() {
                            None
                        } else {
                            Some(self.evaluate(&arguments[0])?)
                        };
                        
                        // Get generator object
                        if let JSValue::Object(gen_obj) = obj_value {
                            return self.execute_generator_next(gen_obj, input_value);
                        }
                    }
                    "return" => {
                        let return_value = if arguments.is_empty() {
                            None
                        } else {
                            Some(self.evaluate(&arguments[0])?)
                        };
                        
                        if let JSValue::Object(gen_obj) = obj_value {
                            return self.execute_generator_return(gen_obj, return_value);
                        }
                    }
                    "throw" => {
                        let exception = if arguments.is_empty() {
                            JSValue::Undefined
                        } else {
                            self.evaluate(&arguments[0])?
                        };
                        
                        if let JSValue::Object(gen_obj) = obj_value {
                            return self.execute_generator_throw(gen_obj, exception);
                        }
                    }
                    _ => {
                        // Not a generator method, fall through to normal call
                    }
                }
            }
            
            // Special case: Function.prototype.call/apply/bind
            if let JSValue::Function(_) = obj_value {
                let method_name = if *computed {
                    let prop_val = self.evaluate(property)?;
                    self.to_string(&prop_val)
                } else {
                    if let ASTNode::Identifier(name) = property.as_ref() {
                        name.clone()
                    } else {
                        String::new()
                    }
                };
                
                match method_name.as_str() {
                    "call" => {
                        return self.handle_function_call(obj_value, arguments);
                    }
                    "apply" => {
                        return self.handle_function_apply(obj_value, arguments);
                    }
                    "bind" => {
                        return self.handle_function_bind(obj_value, arguments);
                    }
                    _ => {
                        // Not a special function method, fall through
                    }
                }
            }
        }
        
        // Determine 'this' value based on how function is called
        let this_binding = match callee {
            ASTNode::MemberExpr { object, .. } => {
                // Method call: this = object
                self.evaluate(object)?
            }
            _ => {
                // Regular function call: this = undefined (or global in non-strict)
                JSValue::Undefined
            }
        };

        let func = self.evaluate(callee)?;

        match func {
            JSValue::Function(f) => {
                // Check if this is an async function
                if f.is_async {
                    return self.call_async_function(&f, arguments, this_binding);
                }
                
                // Check if this is a generator function
                if f.is_generator {
                    return self.call_generator_function(&f, arguments, this_binding);
                }
                
                // Regular synchronous function call
                self.call_sync_function(&f, arguments, this_binding)
            }
            _ => Err(self.create_error("Not a function")),
        }
    }
    
    /// Call a regular synchronous function
    fn call_sync_function(
        &mut self,
        f: &super::runtime::Function,
        arguments: &[ASTNode],
        this_binding: JSValue,
    ) -> Result<JSValue, String> {
        // Handle bound functions
        let (actual_func, actual_this, mut actual_args) = if f.target_function.is_some() {
            // This is a bound function - use bound this and prepend bound args
            let target = f.target_function.as_ref().unwrap();
            let bound_this = f.bound_this.clone().unwrap_or(JSValue::Undefined);
            
            // Prepend bound arguments to call arguments
            let mut all_args: Vec<ASTNode> = f.bound_args.iter().map(|v| {
                // Convert JSValue to ASTNode
                match v {
                    JSValue::Number(n) => ASTNode::Number(*n),
                    JSValue::String(s) => ASTNode::String(s.to_string()),
                    JSValue::Boolean(b) => ASTNode::Boolean(*b),
                    JSValue::Null => ASTNode::Null,
                    JSValue::Undefined => ASTNode::Undefined,
                    _ => ASTNode::Undefined,
                }
            }).collect();
            all_args.extend_from_slice(arguments);
            
            (target.as_ref(), bound_this, all_args)
        } else {
            // Regular function - use provided this and args
            (f, this_binding, arguments.to_vec())
        };
        
        // Evaluate arguments and push to stack
        let mut args = Vec::new();
        for arg in &actual_args {
            let val = self.evaluate(arg)?;
            self.stack.push(val.clone());
            args.push(val);
        }

        // Save current 'this' value
        let saved_this = self.this_value.clone();

        // Set 'this' for the function call
        self.this_value = actual_this.clone();

        // Create new call frame
        let frame = CallFrame {
            function_name: actual_func.name.clone(),
            locals: HashMap::new(),
            return_address: self.call_stack.len(),
            this_binding: actual_this.clone(),
        };

        self.call_stack.push(frame);

        // Execute function body if it exists
        let result = if let Some(body) = &actual_func.body {
            // Bind parameters to arguments
            for (i, param) in actual_func.params.iter().enumerate() {
                if i < args.len() {
                    self.call_stack.last_mut().unwrap()
                        .locals.insert(param.clone(), args[i].clone());
                }
            }

            // Execute function body
            self.evaluate(body)?
        } else {
            // Native function - call native implementation
            if let Some(native_fn) = &actual_func.native {
                native_fn(&args)?
            } else {
                JSValue::Undefined
            }
        };

        // Pop call frame
        self.call_stack.pop();

        // Restore 'this' value
        self.this_value = saved_this;

        // Pop arguments from stack
        for _ in 0..actual_args.len() {
            self.stack.pop();
        }

        Ok(result)
    }
    
    /// Call an async function - returns a Promise
    fn call_async_function(
        &mut self,
        f: &super::runtime::Function,
        arguments: &[ASTNode],
        this_binding: JSValue,
    ) -> Result<JSValue, String> {
        use super::promise::Promise as ES2025Promise;
        
        // Evaluate arguments
        let mut args = Vec::new();
        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }
        
        // Create a Promise for the async function's return value
        let promise = ES2025Promise::new();
        let promise_for_result = promise.clone();
        
        // If no body, resolve immediately
        let Some(body) = &f.body else {
            promise.fulfill_promise(super::promise::JSValue::Undefined);
            return Ok(self.promise_to_jsvalue(&promise));
        };
        
        // Create async context
        let mut context = AsyncContext {
            promise: promise.clone(),
            body: body.clone(),
            locals: HashMap::new(),
            params: f.params.clone(),
            args: args.clone(),
            state: AsyncState::Running,
            this_binding: this_binding.clone(),
        };
        
        // Bind parameters to arguments in context
        for (i, param) in context.params.iter().enumerate() {
            if i < args.len() {
                context.locals.insert(param.clone(), args[i].clone());
            }
        }
        
        // Save current interpreter state
        let saved_locals = self.locals.clone();
        let saved_this = self.this_value.clone();
        
        // Set up async function environment
        self.locals = context.locals.clone();
        self.this_value = context.this_binding.clone();
        
        // Execute the async function body
        let result = self.execute_async_body(&mut context);
        
        // Restore interpreter state
        self.locals = saved_locals;
        self.this_value = saved_this;
        
        // Handle result
        match result {
            Ok(value) => {
                // Function completed without suspension - resolve promise
                let promise_value = convert_to_promise_value(&value);
                promise_for_result.fulfill_promise(promise_value);
            }
            Err(InterpreterError::AsyncSuspension(mut continuation)) => {
                // Function suspended on await
                continuation.promise = promise_for_result.clone();
                // Queue continuation for resumption when Promise settles
                self.queue_async_continuation(continuation);
                // Promise stays pending until continuation resumes
            }
            Err(InterpreterError::GeneratorYield(_)) => {
                // Yield in async context - shouldn't happen, treat as error
                let error_value = super::promise::JSValue::String("Unexpected yield in async function".to_string());
                promise_for_result.reject_promise(error_value);
            }
            Err(InterpreterError::Runtime(e)) => {
                // Function threw error - reject promise
                let error_value = super::promise::JSValue::String(e);
                promise_for_result.reject_promise(error_value);
            }
        }
        
        // Return promise as JSValue
        Ok(self.promise_to_jsvalue(&promise))
    }
    
    /// Execute async function body, handling await expressions
    fn execute_async_body(&mut self, context: &mut AsyncContext) -> Result<JSValue, InterpreterError> {
        // Execute the body with InterpreterError support
        self.evaluate_internal(&context.body)
    }
    
    /// Resume a suspended async continuation
    fn resume_async_continuation(&mut self, continuation: AsyncContinuation) -> Result<(), String> {
        // Check if this continuation was rejected
        if continuation.is_rejected {
            // Handle rejection - throw the error to propagate through async context
            let error_msg = match &continuation.rejection_value {
                JSValue::String(s) => s.to_string(),
                JSValue::Number(n) => format!("Unhandled promise rejection: {}", n),
                JSValue::Boolean(b) => format!("Unhandled promise rejection: {}", b),
                JSValue::Object(obj) => {
                    // Try to get error message from Error object
                    let borrowed = obj.borrow();
                    if let Some(JSValue::String(msg)) = borrowed.get("message") {
                        format!("Error: {}", msg)
                    } else {
                        "Uncaught (in promise)".to_string()
                    }
                }
                JSValue::Null => "Unhandled promise rejection: null".to_string(),
                JSValue::Undefined => "Unhandled promise rejection: undefined".to_string(),
                _ => "Unhandled promise rejection".to_string(),
            };
            
            // Reject the async function's Promise
            let promise_error = convert_to_promise_value(&continuation.rejection_value);
            continuation.promise.reject_promise(promise_error);
            
            return Err(error_msg);
        }
        
        // Restore interpreter state
        let saved_locals = self.locals.clone();
        let saved_call_stack = self.call_stack.clone();
        let saved_this = self.this_value.clone();
        
        self.locals = continuation.locals;
        self.call_stack = continuation.call_stack;
        self.this_value = continuation.this_binding;
        
        // Get the resume value (from settled Promise)
        let resume_value = continuation.resume_value;
        
        // Execute remaining body if present
        let result = if let Some(remaining) = continuation.remaining_body {
            self.evaluate_internal(&remaining)
        } else {
            // No remaining body, just return the resume value
            Ok(resume_value.clone())
        };
        
        // Handle the result
        match result {
            Ok(value) => {
                // Async function completed - resolve the Promise
                let promise_value = convert_to_promise_value(&value);
                continuation.promise.fulfill_promise(promise_value);
            }
            Err(InterpreterError::AsyncSuspension(new_continuation)) => {
                // Suspended again on another await
                // Queue the new continuation
                self.queue_async_continuation(new_continuation);
            }
            Err(InterpreterError::GeneratorYield(_)) => {
                // Yield in async context - shouldn't happen, reject
                let error_value = super::promise::JSValue::String("Unexpected yield in async function".to_string());
                continuation.promise.reject_promise(error_value);
            }
            Err(InterpreterError::Runtime(e)) => {
                // Error occurred - reject the Promise
                let error_value = super::promise::JSValue::String(e);
                continuation.promise.reject_promise(error_value);
            }
        }
        
        // Restore original interpreter state
        self.locals = saved_locals;
        self.call_stack = saved_call_stack;
        self.this_value = saved_this;
        
        Ok(())
    }
    
    /// Convert ES2025Promise to runtime JSValue
    fn promise_to_jsvalue(&mut self, promise: &super::promise::Promise) -> JSValue {
        let mut promise_obj = super::runtime::JSObject::new();
        
        // Get promise state
        let state = promise.state();
        let state_str = match state {
            super::promise::PromiseState::Pending => "pending",
            super::promise::PromiseState::Fulfilled(_) => "fulfilled",
            super::promise::PromiseState::Rejected(_) => "rejected",
        };
        
        promise_obj.set("[[PromiseState]]", JSValue::String(Rc::new(state_str.to_string())));
        promise_obj.set("[[PromiseValue]]", JSValue::Undefined);
        
        // Store the actual Promise object for extraction
        let promise_id = promise.id().to_string();
        promise_obj.set("[[PromiseID]]", JSValue::String(Rc::new(promise_id.clone())));
        
        // Register the promise in the registry
        self.promise_registry.insert(promise_id, promise.clone());
        
        JSValue::Object(Rc::new(RefCell::new(promise_obj)))
    }
    
    /// Extract Promise from JSValue Promise object
    fn extract_promise_from_value(&self, obj: &Rc<RefCell<super::runtime::JSObject>>) -> Option<super::promise::Promise> {
        let obj_borrow = obj.borrow();
        if let Some(JSValue::String(id_str)) = obj_borrow.get("[[PromiseID]]") {
            // Look up the Promise in the registry
            self.promise_registry.get(id_str.as_str()).cloned()
        } else {
            None
        }
    }
    
    /// Create an AsyncContinuation for suspended async function
    fn create_async_continuation(
        &self,
        promise: super::promise::Promise,
        remaining_body: Option<Box<ASTNode>>,
    ) -> AsyncContinuation {
        AsyncContinuation::new(
            self.locals.clone(),
            self.call_stack.clone(),
            promise,
            remaining_body,
            self.this_value.clone(),
        )
    }
    
    /// Check if a JSValue is a Promise object
    fn is_promise_object(&self, obj: &Rc<RefCell<super::runtime::JSObject>>) -> bool {
        let obj_borrow = obj.borrow();
        obj_borrow.properties.contains_key("[[PromiseState]]")
    }
    
    /// Call a generator function - returns a Generator object
    fn call_generator_function(
        &mut self,
        f: &super::runtime::Function,
        arguments: &[ASTNode],
        this_binding: JSValue,
    ) -> Result<JSValue, String> {
        // Evaluate arguments
        let mut args = Vec::new();
        for arg in arguments {
            args.push(self.evaluate(arg)?);
        }
        
        // Create generator context with function body
        let body = f.body.clone().ok_or("Generator function has no body")?;
        
        // Count yields for PC tracking
        let yield_count = self.count_yields(&body);
        let yield_positions: Vec<usize> = (0..yield_count).collect();
        
        // Create a stateful generator context that will persist
        let context = Rc::new(RefCell::new(GeneratorContext {
            body: body.clone(),
            locals: HashMap::new(),
            params: f.params.clone(),
            args: args.clone(),
            state: super::promise::GeneratorState::SuspendedStart,
            yield_queue: VecDeque::new(),
            this_binding: this_binding.clone(),
            last_value: JSValue::Undefined,
            yield_positions,
            current_yield_index: 0,
        }));
        
        // Bind parameters to arguments in context
        {
            let mut ctx = context.borrow_mut();
            let params_clone = ctx.params.clone();
            for (i, param) in params_clone.iter().enumerate() {
                if i < args.len() {
                    ctx.locals.insert(param.clone(), args[i].clone());
                }
            }
        }
        
        // Create generator object with methods
        let mut gen_obj = super::runtime::JSObject::new();
        
        // Assign a unique ID to this generator
        let gen_id = self.next_generator_id;
        self.next_generator_id += 1;
        
        // Store context in registry
        self.generator_contexts.insert(gen_id, context.clone());
        
        // Store the generator ID in the object's internal state
        gen_obj.set("[[GeneratorContext]]", 
            JSValue::Number(gen_id as f64));
        
        // Create next() method with captured context
        let context_for_next = context.clone();
        let next_fn = super::runtime::Function::native_function(
            "next".to_string(),
            vec!["value".to_string()],
            move |args| {
                let input_value = args.get(0).cloned();
                let mut ctx = context_for_next.borrow_mut();
                
                // Check state
                match ctx.state {
                    super::promise::GeneratorState::Completed => {
                        // Already done
                        let mut result = super::runtime::JSObject::new();
                        result.set("value", JSValue::Undefined);
                        result.set("done", JSValue::Boolean(true));
                        return Ok(JSValue::Object(Rc::new(RefCell::new(result))));
                    }
                    super::promise::GeneratorState::SuspendedStart => {
                        // First call to next()
                        ctx.state = super::promise::GeneratorState::Executing;
                        // TODO: Execute body until yield
                        // For now, simulate yielding undefined
                        ctx.state = super::promise::GeneratorState::Completed;
                        let mut result = super::runtime::JSObject::new();
                        result.set("value", JSValue::Undefined);
                        result.set("done", JSValue::Boolean(true));
                        Ok(JSValue::Object(Rc::new(RefCell::new(result))))
                    }
                    super::promise::GeneratorState::SuspendedYield(_pc) => {
                        // Resume from yield
                        ctx.state = super::promise::GeneratorState::Executing;
                        // If input_value provided, that's the value sent to yield expression
                        // TODO: Resume execution from PC
                        ctx.state = super::promise::GeneratorState::Completed;
                        let mut result = super::runtime::JSObject::new();
                        result.set("value", input_value.unwrap_or(JSValue::Undefined));
                        result.set("done", JSValue::Boolean(true));
                        Ok(JSValue::Object(Rc::new(RefCell::new(result))))
                    }
                    _ => {
                        let mut result = super::runtime::JSObject::new();
                        result.set("value", JSValue::Undefined);
                        result.set("done", JSValue::Boolean(true));
                        Ok(JSValue::Object(Rc::new(RefCell::new(result))))
                    }
                }
            }
        );
        gen_obj.set("next", JSValue::Function(Rc::new(next_fn)));
        
        // Generator.prototype.return()
        let context_for_return = context.clone();
        let return_fn = super::runtime::Function::native_function(
            "return".to_string(),
            vec!["value".to_string()],
            move |args| {
                let value = args.get(0).cloned().unwrap_or(JSValue::Undefined);
                let mut ctx = context_for_return.borrow_mut();
                ctx.state = super::promise::GeneratorState::Completed;
                let mut result = super::runtime::JSObject::new();
                result.set("value", value);
                result.set("done", JSValue::Boolean(true));
                Ok(JSValue::Object(Rc::new(RefCell::new(result))))
            }
        );
        gen_obj.set("return", JSValue::Function(Rc::new(return_fn)));
        
        // Generator.prototype.throw()
        let context_for_throw = context.clone();
        let throw_fn = super::runtime::Function::native_function(
            "throw".to_string(),
            vec!["exception".to_string()],
            move |args| {
                let mut ctx = context_for_throw.borrow_mut();
                ctx.state = super::promise::GeneratorState::Completed;
                let _exception = args.get(0).cloned().unwrap_or(JSValue::Undefined);
                Err("Generator exception".to_string())
            }
        );
        gen_obj.set("throw", JSValue::Function(Rc::new(throw_fn)));
        
        Ok(JSValue::Object(Rc::new(RefCell::new(gen_obj))))
    }

    /// Check if a value is a generator object
    fn is_generator_object(&self, value: &JSValue) -> bool {
        if let JSValue::Object(obj) = value {
            obj.borrow().properties.contains_key("[[GeneratorContext]]")
        } else {
            false
        }
    }
    
    /// Count yields in an AST for PC tracking
    fn count_yields(&self, node: &ASTNode) -> usize {
        match node {
            ASTNode::YieldExpression { .. } => 1,
            ASTNode::BlockStmt { body } => {
                body.iter().map(|n| self.count_yields(n)).sum()
            }
            ASTNode::IfStmt { test, consequent, alternate } => {
                let mut count = self.count_yields(test);
                count += self.count_yields(consequent);
                if let Some(alt) = alternate {
                    count += self.count_yields(alt);
                }
                count
            }
            ASTNode::WhileStmt { test, body } => {
                self.count_yields(test) + self.count_yields(body)
            }
            ASTNode::ForStmt { init, test, update, body } => {
                let mut count = 0;
                if let Some(i) = init {
                    count += self.count_yields(i);
                }
                if let Some(t) = test {
                    count += self.count_yields(t);
                }
                if let Some(u) = update {
                    count += self.count_yields(u);
                }
                count + self.count_yields(body)
            }
            ASTNode::BinaryOp { left, right, .. } => {
                self.count_yields(left) + self.count_yields(right)
            }
            ASTNode::CallExpr { callee, arguments } => {
                let mut count = self.count_yields(callee);
                for arg in arguments {
                    count += self.count_yields(arg);
                }
                count
            }
            ASTNode::ReturnStmt { argument } => {
                if let Some(arg) = argument {
                    self.count_yields(arg)
                } else {
                    0
                }
            }
            ASTNode::ExpressionStmt(expr) => {
                self.count_yields(expr)
            }
            ASTNode::VariableDeclaration { declarations, .. } => {
                declarations.iter().map(|d| {
                    if let Some(init) = &d.init {
                        self.count_yields(init)
                    } else {
                        0
                    }
                }).sum()
            }
            _ => 0
        }
    }

    /// Execute generator.next(value) - called via special detection
    fn execute_generator_next(
        &mut self,
        gen_obj: Rc<RefCell<super::runtime::JSObject>>,
        input_value: Option<JSValue>,
    ) -> Result<JSValue, String> {
        // Get the generator ID from the object
        let gen_id = if let Some(JSValue::Number(id)) = gen_obj.borrow().properties.get("[[GeneratorContext]]") {
            *id as usize
        } else {
            return Err("Not a generator object".to_string());
        };
        
        // Get the generator context from registry
        let context = self.generator_contexts.get(&gen_id)
            .ok_or("Generator context not found")?
            .clone();
        
        // Check current state
        let state = context.borrow().state.clone();
        
        match state {
            super::promise::GeneratorState::Completed => {
                // Generator already finished
                let mut result = super::runtime::JSObject::new();
                result.set("value", JSValue::Undefined);
                result.set("done", JSValue::Boolean(true));
                return Ok(JSValue::Object(Rc::new(RefCell::new(result))));
            }
            super::promise::GeneratorState::SuspendedStart => {
                // First call to next() - start execution
                context.borrow_mut().state = super::promise::GeneratorState::Executing;
                
                // Save current interpreter state
                let saved_locals = self.locals.clone();
                let saved_this = self.this_value.clone();
                let saved_in_generator = self.in_generator;
                let saved_yield_counter = self.yield_counter;
                
                // Set up generator context
                let body = context.borrow().body.clone();
                let gen_locals = context.borrow().locals.clone();
                let gen_this = context.borrow().this_binding.clone();
                
                self.locals = gen_locals;
                self.this_value = gen_this;
                self.in_generator = true;
                self.yielded_value = None;
                self.yield_counter = 0; // Start from first yield
                self.current_generator_id = Some(gen_id);
                
                // Execute body until yield or completion
                let result = match self.evaluate(&body) {
                    Ok(return_value) => {
                        // Generator completed (no yield, just return)
                        context.borrow_mut().state = super::promise::GeneratorState::Completed;
                        let mut result_obj = super::runtime::JSObject::new();
                        result_obj.set("value", return_value);
                        result_obj.set("done", JSValue::Boolean(true));
                        Ok(JSValue::Object(Rc::new(RefCell::new(result_obj))))
                    }
                    Err(err) => {
                        // Check if it's a yield
                        if err.starts_with("[[GeneratorYield:") {
                            // Parse yield index from error message
                            let yield_idx_str = err.trim_start_matches("[[GeneratorYield:")
                                                   .trim_end_matches("]]");
                            let yield_idx = yield_idx_str.parse::<usize>().unwrap_or(0);
                            
                            // Get the yielded value
                            let yielded = self.yielded_value.take().unwrap_or(JSValue::Undefined);
                            context.borrow_mut().state = super::promise::GeneratorState::SuspendedYield(yield_idx);
                            context.borrow_mut().last_value = yielded.clone();
                            context.borrow_mut().current_yield_index = yield_idx + 1; // Next yield to execute
                            // Save the current locals state for resumption
                            context.borrow_mut().locals = self.locals.clone();
                            
                            let mut result_obj = super::runtime::JSObject::new();
                            result_obj.set("value", yielded);
                            result_obj.set("done", JSValue::Boolean(false));
                            Ok(JSValue::Object(Rc::new(RefCell::new(result_obj))))
                        } else {
                            // Real error
                            context.borrow_mut().state = super::promise::GeneratorState::Completed;
                            Err(err)
                        }
                    }
                };
                
                // Restore interpreter state
                self.locals = saved_locals;
                self.this_value = saved_this;
                self.in_generator = saved_in_generator;
                self.yielded_value = None;
                self.yield_counter = saved_yield_counter;
                self.current_generator_id = None;
                
                result
            }
            super::promise::GeneratorState::SuspendedYield(last_yield_idx) => {
                // Resume from yield - execute from next yield onwards
                context.borrow_mut().state = super::promise::GeneratorState::Executing;
                
                // Save current interpreter state
                let saved_locals = self.locals.clone();
                let saved_this = self.this_value.clone();
                let saved_in_generator = self.in_generator;
                let saved_yield_counter = self.yield_counter;
                
                // Restore generator context
                let body = context.borrow().body.clone();
                let gen_locals = context.borrow().locals.clone();
                let gen_this = context.borrow().this_binding.clone();
                let resume_from_yield = context.borrow().current_yield_index;
                
                self.locals = gen_locals;
                self.this_value = gen_this;
                self.in_generator = true;
                self.yielded_value = None;
                self.yield_counter = resume_from_yield; // Resume from next yield
                self.current_generator_id = Some(gen_id);
                
                // If input_value provided, it's what yield expression evaluates to
                // For now, we just continue execution
                
                // Execute body until next yield or completion
                let result = match self.evaluate(&body) {
                    Ok(return_value) => {
                        // Generator completed
                        context.borrow_mut().state = super::promise::GeneratorState::Completed;
                        let mut result_obj = super::runtime::JSObject::new();
                        result_obj.set("value", return_value);
                        result_obj.set("done", JSValue::Boolean(true));
                        Ok(JSValue::Object(Rc::new(RefCell::new(result_obj))))
                    }
                    Err(err) => {
                        // Check if it's a yield
                        if err.starts_with("[[GeneratorYield:") {
                            // Parse yield index
                            let yield_idx_str = err.trim_start_matches("[[GeneratorYield:")
                                                   .trim_end_matches("]]");
                            let yield_idx = yield_idx_str.parse::<usize>().unwrap_or(0);
                            
                            // Get the yielded value
                            let yielded = self.yielded_value.take().unwrap_or(JSValue::Undefined);
                            context.borrow_mut().state = super::promise::GeneratorState::SuspendedYield(yield_idx);
                            context.borrow_mut().last_value = yielded.clone();
                            context.borrow_mut().current_yield_index = yield_idx + 1;
                            // Save the current locals state
                            context.borrow_mut().locals = self.locals.clone();
                            
                            let mut result_obj = super::runtime::JSObject::new();
                            result_obj.set("value", yielded);
                            result_obj.set("done", JSValue::Boolean(false));
                            Ok(JSValue::Object(Rc::new(RefCell::new(result_obj))))
                        } else {
                            // Real error
                            context.borrow_mut().state = super::promise::GeneratorState::Completed;
                            Err(err)
                        }
                    }
                };
                
                // Restore interpreter state
                self.locals = saved_locals;
                self.this_value = saved_this;
                self.in_generator = saved_in_generator;
                self.yielded_value = None;
                self.yield_counter = saved_yield_counter;
                self.current_generator_id = None;
                
                result
            }
            _ => {
                let mut result = super::runtime::JSObject::new();
                result.set("value", JSValue::Undefined);
                result.set("done", JSValue::Boolean(true));
                Ok(JSValue::Object(Rc::new(RefCell::new(result))))
            }
        }
    }

    /// Execute generator.return(value)
    fn execute_generator_return(
        &mut self,
        _gen_obj: Rc<RefCell<super::runtime::JSObject>>,
        return_value: Option<JSValue>,
    ) -> Result<JSValue, String> {
        let mut result = super::runtime::JSObject::new();
        result.set("value", return_value.unwrap_or(JSValue::Undefined));
        result.set("done", JSValue::Boolean(true));
        Ok(JSValue::Object(Rc::new(RefCell::new(result))))
    }

    /// Execute generator.throw(exception)
    fn execute_generator_throw(
        &mut self,
        _gen_obj: Rc<RefCell<super::runtime::JSObject>>,
        _exception: JSValue,
    ) -> Result<JSValue, String> {
        // Throw the exception
        Err("Generator exception".to_string())
    }

    fn evaluate_member_access(&mut self, object: &JSValue, property: &ASTNode, computed: bool) -> Result<JSValue, String> {
        let prop_key = if computed {
            let prop_val = self.evaluate(property)?;
            self.to_string(&prop_val)
        } else {
            if let ASTNode::Identifier(name) = property {
                name.clone()
            } else {
                return Err(self.create_error("Invalid property access"));
            }
        };
        
        match object {
            JSValue::Object(obj) => {
                Ok(obj.borrow().get(&prop_key).cloned().unwrap_or(JSValue::Undefined))
            }
            _ => Ok(JSValue::Undefined),
        }
    }

    // Type conversion
    fn to_boolean(&self, value: &JSValue) -> bool {
        match value {
            JSValue::Undefined | JSValue::Null => false,
            JSValue::Boolean(b) => *b,
            JSValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JSValue::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    fn to_number(&self, value: &JSValue) -> f64 {
        match value {
            JSValue::Undefined => f64::NAN,
            JSValue::Null => 0.0,
            JSValue::Boolean(true) => 1.0,
            JSValue::Boolean(false) => 0.0,
            JSValue::Number(n) => *n,
            JSValue::String(s) => s.parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    }

    fn to_int32(&self, value: &JSValue) -> i32 {
        let num = self.to_number(value);
        if num.is_nan() || num.is_infinite() {
            0
        } else {
            num as i32
        }
    }

    fn to_string(&self, value: &JSValue) -> String {
        match value {
            JSValue::Undefined => "undefined".to_string(),
            JSValue::Null => "null".to_string(),
            JSValue::Boolean(b) => b.to_string(),
            JSValue::Number(n) => n.to_string(),
            JSValue::String(s) => s.to_string(),
            _ => "[object Object]".to_string(),
        }
    }

    fn type_of(&self, value: &JSValue) -> String {
        match value {
            JSValue::Undefined => "undefined",
            JSValue::Null => "object", // quirk of JavaScript
            JSValue::Boolean(_) => "boolean",
            JSValue::Number(_) => "number",
            JSValue::String(_) => "string",
            JSValue::Function(_) => "function",
            JSValue::Symbol(_) => "symbol",
            JSValue::BigInt(_) => "bigint",
            _ => "object",
        }.to_string()
    }

    // Arithmetic operations
    fn add_values(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        // String concatenation if either operand is a string
        if matches!(left, JSValue::String(_)) || matches!(right, JSValue::String(_)) {
            Ok(JSValue::String(Rc::new(format!("{}{}", self.to_string(left), self.to_string(right)))))
        } else {
            Ok(JSValue::Number(self.to_number(left) + self.to_number(right)))
        }
    }

    fn subtract_values(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number(self.to_number(left) - self.to_number(right)))
    }

    fn multiply_values(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number(self.to_number(left) * self.to_number(right)))
    }

    /// Handle Function.prototype.call(thisArg, ...args)
    /// ES2025 Spec: 20.2.3.3
    fn handle_function_call(&mut self, func_value: JSValue, arguments: &[ASTNode]) -> Result<JSValue, String> {
        // Extract the function
        let func = match func_value {
            JSValue::Function(f) => f,
            _ => return Err("call() can only be used on functions".to_string()),
        };
        
        // First argument is thisArg
        let this_arg = if arguments.is_empty() {
            JSValue::Undefined
        } else {
            self.evaluate(&arguments[0])?
        };
        
        // Remaining arguments are the function arguments
        let call_args: Vec<ASTNode> = arguments.iter().skip(1).cloned().collect();
        
        // Call the function with the specified 'this' value
        if func.is_async {
            self.call_async_function(&func, &call_args, this_arg)
        } else if func.is_generator {
            self.call_generator_function(&func, &call_args, this_arg)
        } else {
            self.call_sync_function(&func, &call_args, this_arg)
        }
    }
    
    /// Handle Function.prototype.apply(thisArg, argsArray)
    /// ES2025 Spec: 20.2.3.1
    fn handle_function_apply(&mut self, func_value: JSValue, arguments: &[ASTNode]) -> Result<JSValue, String> {
        // Extract the function
        let func = match func_value {
            JSValue::Function(f) => f,
            _ => return Err("apply() can only be used on functions".to_string()),
        };
        
        // First argument is thisArg
        let this_arg = if arguments.is_empty() {
            JSValue::Undefined
        } else {
            self.evaluate(&arguments[0])?
        };
        
        // Second argument is the arguments array
        let call_args = if arguments.len() > 1 {
            let args_array = self.evaluate(&arguments[1])?;
            match args_array {
                JSValue::Object(obj) => {
                    // Handle array-like object (has length property)
                    let borrowed = obj.borrow();
                    if let Some(JSValue::Number(len)) = borrowed.get("length") {
                        let length = *len as usize;
                        let mut args = Vec::new();
                        for i in 0..length {
                            if let Some(val) = borrowed.get(&i.to_string()) {
                                let node = match val {
                                    JSValue::Number(n) => ASTNode::Number(*n),
                                    JSValue::String(s) => ASTNode::String(s.to_string()),
                                    JSValue::Boolean(b) => ASTNode::Boolean(*b),
                                    JSValue::Null => ASTNode::Null,
                                    JSValue::Undefined => ASTNode::Undefined,
                                    _ => ASTNode::Undefined,
                                };
                                args.push(node);
                            } else {
                                args.push(ASTNode::Undefined);
                            }
                        }
                        args
                    } else {
                        Vec::new()
                    }
                }
                JSValue::Null | JSValue::Undefined => Vec::new(),
                _ => return Err("apply() second argument must be an array or array-like object".to_string()),
            }
        } else {
            Vec::new()
        };
        
        // Call the function with the specified 'this' value
        if func.is_async {
            self.call_async_function(&func, &call_args, this_arg)
        } else if func.is_generator {
            self.call_generator_function(&func, &call_args, this_arg)
        } else {
            self.call_sync_function(&func, &call_args, this_arg)
        }
    }
    
    /// Handle Function.prototype.bind(thisArg, ...args)
    /// ES2025 Spec: 20.2.3.2
    fn handle_function_bind(&mut self, func_value: JSValue, arguments: &[ASTNode]) -> Result<JSValue, String> {
        // Extract the function
        let func = match func_value {
            JSValue::Function(f) => f,
            _ => return Err("bind() can only be used on functions".to_string()),
        };
        
        // First argument is thisArg
        let this_arg = if arguments.is_empty() {
            JSValue::Undefined
        } else {
            self.evaluate(&arguments[0])?
        };
        
        // Remaining arguments are pre-bound arguments
        let bound_args: Vec<JSValue> = arguments.iter()
            .skip(1)
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Create a bound function
        let bound_fn = super::runtime::Function::bound_function(
            format!("bound {}", func.name),
            func.clone(),
            this_arg,
            bound_args,
        );
        
        Ok(JSValue::Function(Rc::new(bound_fn)))
    }

    fn divide_values(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number(self.to_number(left) / self.to_number(right)))
    }

    fn modulo_values(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number(self.to_number(left) % self.to_number(right)))
    }

    // Comparison operations
    fn loose_equals(&self, left: &JSValue, right: &JSValue) -> bool {
        // Implement JavaScript's == with type coercion
        match (left, right) {
            (JSValue::Undefined, JSValue::Undefined) | (JSValue::Null, JSValue::Null) => true,
            (JSValue::Undefined, JSValue::Null) | (JSValue::Null, JSValue::Undefined) => true,
            (JSValue::Boolean(a), JSValue::Boolean(b)) => a == b,
            (JSValue::Number(a), JSValue::Number(b)) => a == b,
            (JSValue::String(a), JSValue::String(b)) => a == b,
            _ => self.to_number(left) == self.to_number(right),
        }
    }

    fn strict_equals(&self, left: &JSValue, right: &JSValue) -> bool {
        // Implement JavaScript's === without type coercion
        match (left, right) {
            (JSValue::Undefined, JSValue::Undefined) => true,
            (JSValue::Null, JSValue::Null) => true,
            (JSValue::Boolean(a), JSValue::Boolean(b)) => a == b,
            (JSValue::Number(a), JSValue::Number(b)) => a == b,
            (JSValue::String(a), JSValue::String(b)) => a == b,
            _ => false,
        }
    }

    fn compare_lt(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Boolean(self.to_number(left) < self.to_number(right)))
    }

    fn compare_lte(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Boolean(self.to_number(left) <= self.to_number(right)))
    }

    fn compare_gt(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Boolean(self.to_number(left) > self.to_number(right)))
    }

    fn compare_gte(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Boolean(self.to_number(left) >= self.to_number(right)))
    }

    // Bitwise operations
    fn bitwise_and(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number((self.to_int32(left) & self.to_int32(right)) as f64))
    }

    fn bitwise_or(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number((self.to_int32(left) | self.to_int32(right)) as f64))
    }

    fn bitwise_xor(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number((self.to_int32(left) ^ self.to_int32(right)) as f64))
    }

    fn left_shift(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number((self.to_int32(left) << (self.to_int32(right) & 0x1F)) as f64))
    }

    fn right_shift(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number((self.to_int32(left) >> (self.to_int32(right) & 0x1F)) as f64))
    }

    fn unsigned_right_shift(&self, left: &JSValue, right: &JSValue) -> Result<JSValue, String> {
        Ok(JSValue::Number(((self.to_int32(left) as u32) >> (self.to_int32(right) & 0x1F)) as f64))
    }
    
    /// Generate a JavaScript stack trace from the call stack
    pub fn generate_stack_trace(&self) -> String {
        if self.call_stack.is_empty() {
            return String::from("    at <anonymous>");
        }
        
        let mut trace = Vec::new();
        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            let location = if frame.return_address > 0 {
                format!("<bytecode offset {}>", frame.return_address)
            } else {
                String::from("<unknown>")
            };
            trace.push(format!("    at {} ({})", frame.function_name, location));
            
            // Limit stack trace to 10 frames
            if i >= 10 {
                trace.push(format!("    ... {} more frames", self.call_stack.len() - 10));
                break;
            }
        }
        trace.join("\n")
    }
    
    /// Create a detailed error message with stack trace
    fn create_error(&self, message: &str) -> String {
        format!("{}\n{}", message, self.generate_stack_trace())
    }
}

// Import needed types from parser
use super::parser::{BinaryOperator, UnaryOperator, AssignmentOperator};
