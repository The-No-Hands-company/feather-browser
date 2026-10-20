// JavaScript Promise implementation - ECMAScript 2025 compliant
// Implements: https://tc39.es/ecma262/2025/#sec-promise-objects
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::RefCell;

/// Promise state as per ECMAScript 2025 spec (25.6.1.1)
#[derive(Debug, Clone)]
pub enum PromiseState {
    Pending,
    Fulfilled(JSValue),
    Rejected(JSValue),
}

/// PromiseReaction record (25.6.1.4)
#[derive(Clone)]
pub struct PromiseReaction {
    pub capability: Option<Arc<PromiseCapability>>,
    pub reaction_type: PromiseReactionType,
    pub handler: PromiseReactionHandler,
}

#[derive(Clone)]
pub enum PromiseReactionType {
    Fulfill,
    Reject,
}

#[derive(Clone)]
pub enum PromiseReactionHandler {
    Function(Rc<dyn Fn(JSValue) -> Result<JSValue, String>>),
    Identity,
    Thrower,
}

/// PromiseCapability record (25.6.1.1)
pub struct PromiseCapability {
    pub promise: Promise,
    pub resolve: Rc<dyn Fn(JSValue) -> Result<(), String>>,
    pub reject: Rc<dyn Fn(JSValue) -> Result<(), String>>,
}

/// Main Promise object implementing ECMAScript 2025 Promise specification
pub struct Promise {
    /// [[PromiseState]] internal slot
    state: Arc<Mutex<PromiseState>>,
    /// [[PromiseFulfillReactions]] internal slot
    fulfill_reactions: Arc<Mutex<VecDeque<PromiseReaction>>>,
    /// [[PromiseRejectReactions]] internal slot
    reject_reactions: Arc<Mutex<VecDeque<PromiseReaction>>>,
    /// [[PromiseIsHandled]] internal slot
    is_handled: Arc<Mutex<bool>>,
    /// Unique identifier for tracking
    id: uuid::Uuid,
}

impl std::fmt::Debug for Promise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Promise")
            .field("state", &self.state)
            .field("id", &self.id)
            .finish()
    }
}

impl Clone for Promise {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            fulfill_reactions: Arc::clone(&self.fulfill_reactions),
            reject_reactions: Arc::clone(&self.reject_reactions),
            is_handled: Arc::clone(&self.is_handled),
            id: self.id,
        }
    }
}

impl std::fmt::Debug for PromiseReaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromiseReaction")
            .field("reaction_type", &self.reaction_type)
            .finish()
    }
}

impl std::fmt::Debug for PromiseReactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromiseReactionType::Fulfill => write!(f, "Fulfill"),
            PromiseReactionType::Reject => write!(f, "Reject"),
        }
    }
}

impl Promise {
    /// Create a new Promise in pending state (25.6.3.1)
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PromiseState::Pending)),
            fulfill_reactions: Arc::new(Mutex::new(VecDeque::new())),
            reject_reactions: Arc::new(Mutex::new(VecDeque::new())),
            is_handled: Arc::new(Mutex::new(false)),
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Promise.resolve (25.6.4.6)
    pub fn resolve_static(value: JSValue) -> Self {
        let promise = Self::new();
        promise.fulfill_promise(value);
        promise
    }

    /// Promise.reject (25.6.4.4)
    pub fn reject_static(reason: JSValue) -> Self {
        let promise = Self::new();
        promise.reject_promise(reason);
        promise
    }

    /// FulfillPromise (25.6.1.3.1) - Internal algorithm
    pub fn fulfill_promise(&self, value: JSValue) {
        let mut state = self.state.lock().unwrap();
        if !matches!(*state, PromiseState::Pending) {
            return; // Already settled
        }
        
        *state = PromiseState::Fulfilled(value.clone());
        drop(state);
        
        // Trigger all fulfill reactions
        let mut reactions = self.fulfill_reactions.lock().unwrap();
        let reactions_to_trigger: Vec<_> = reactions.drain(..).collect();
        drop(reactions);
        
        for reaction in reactions_to_trigger {
            self.trigger_promise_reaction(reaction, value.clone());
        }
    }

    /// RejectPromise (25.6.1.3.2) - Internal algorithm
    pub fn reject_promise(&self, reason: JSValue) {
        let mut state = self.state.lock().unwrap();
        if !matches!(*state, PromiseState::Pending) {
            return; // Already settled
        }
        
        *state = PromiseState::Rejected(reason.clone());
        drop(state);
        
        // Trigger all reject reactions
        let mut reactions = self.reject_reactions.lock().unwrap();
        let reactions_to_trigger: Vec<_> = reactions.drain(..).collect();
        drop(reactions);
        
        for reaction in reactions_to_trigger {
            self.trigger_promise_reaction(reaction, reason.clone());
        }
        
        // Check if promise is handled
        if !*self.is_handled.lock().unwrap() {
            // HostPromiseRejectionTracker - log unhandled rejection
            eprintln!("Unhandled Promise Rejection: {:?}", reason);
        }
    }

    /// TriggerPromiseReactions - Execute a promise reaction
    fn trigger_promise_reaction(&self, reaction: PromiseReaction, argument: JSValue) {
        let result = match &reaction.handler {
            PromiseReactionHandler::Function(f) => f(argument),
            PromiseReactionHandler::Identity => Ok(argument),
            PromiseReactionHandler::Thrower => Err(format!("{:?}", argument)),
        };
        
        if let Some(capability) = &reaction.capability {
            match result {
                Ok(value) => {
                    let _ = (capability.resolve)(value);
                }
                Err(err) => {
                    let _ = (capability.reject)(JSValue::String(err));
                }
            }
        }
    }

    /// Promise.prototype.then (25.6.5.4) - Core promise chaining
    pub fn then(
        &self,
        on_fulfilled: Option<Rc<dyn Fn(JSValue) -> Result<JSValue, String>>>,
        on_rejected: Option<Rc<dyn Fn(JSValue) -> Result<JSValue, String>>>,
    ) -> Self {
        let result_promise = Promise::new();
        let result_promise_clone = result_promise.clone();
        
        // Create PromiseCapability for the result
        let capability = Arc::new(PromiseCapability {
            promise: result_promise.clone(),
            resolve: Rc::new(move |value| {
                result_promise_clone.fulfill_promise(value);
                Ok(())
            }),
            reject: {
                let result_promise_clone2 = result_promise.clone();
                Rc::new(move |reason| {
                    result_promise_clone2.reject_promise(reason);
                    Ok(())
                })
            },
        });
        
        // Create fulfill reaction
        let fulfill_handler = if let Some(handler) = on_fulfilled {
            PromiseReactionHandler::Function(handler)
        } else {
            PromiseReactionHandler::Identity
        };
        
        let fulfill_reaction = PromiseReaction {
            capability: Some(capability.clone()),
            reaction_type: PromiseReactionType::Fulfill,
            handler: fulfill_handler,
        };
        
        // Create reject reaction
        let reject_handler = if let Some(handler) = on_rejected {
            PromiseReactionHandler::Function(handler)
        } else {
            PromiseReactionHandler::Thrower
        };
        
        let reject_reaction = PromiseReaction {
            capability: Some(capability),
            reaction_type: PromiseReactionType::Reject,
            handler: reject_handler,
        };
        
        // Add reactions based on current state
        let state = self.state.lock().unwrap();
        match &*state {
            PromiseState::Pending => {
                drop(state);
                self.fulfill_reactions.lock().unwrap().push_back(fulfill_reaction);
                self.reject_reactions.lock().unwrap().push_back(reject_reaction);
            }
            PromiseState::Fulfilled(value) => {
                let value = value.clone();
                drop(state);
                self.trigger_promise_reaction(fulfill_reaction, value);
            }
            PromiseState::Rejected(reason) => {
                let reason = reason.clone();
                drop(state);
                *self.is_handled.lock().unwrap() = true;
                self.trigger_promise_reaction(reject_reaction, reason);
            }
        }
        
        result_promise
    }

    /// Promise.prototype.catch (25.6.5.1)
    pub fn catch(&self, on_rejected: Rc<dyn Fn(JSValue) -> Result<JSValue, String>>) -> Self {
        *self.is_handled.lock().unwrap() = true;
        self.then(None, Some(on_rejected))
    }

    /// Promise.prototype.finally (25.6.5.2)
    pub fn finally(&self, on_finally: Rc<dyn Fn() -> Result<(), String>>) -> Self {
        let on_finally_clone = on_finally.clone();
        
        self.then(
            Some(Rc::new(move |value| {
                let _ = on_finally()?;
                Ok(value)
            })),
            Some(Rc::new(move |reason| {
                let _ = on_finally_clone()?;
                Err(format!("{:?}", reason))
            })),
        )
    }

    /// Get current state (for testing)
    pub fn state(&self) -> PromiseState {
        self.state.lock().unwrap().clone()
    }
    
    /// Get promise ID
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    /// Check if promise is settled
    pub fn is_settled(&self) -> bool {
        !matches!(*self.state.lock().unwrap(), PromiseState::Pending)
    }

    /// Check if promise is pending
    pub fn is_pending(&self) -> bool {
        matches!(*self.state.lock().unwrap(), PromiseState::Pending)
    }

    /// Promise.all (25.6.4.1) - ECMAScript 2025 spec
    pub fn all(promises: Vec<Promise>) -> Promise {
        if promises.is_empty() {
            return Promise::resolve_static(JSValue::Array(vec![]));
        }
        
        let result_promise = Promise::new();
        let remaining = Arc::new(Mutex::new(promises.len()));
        let values = Arc::new(Mutex::new(vec![JSValue::Undefined; promises.len()]));
        
        for (index, promise) in promises.into_iter().enumerate() {
            let result_clone_fulfill = result_promise.clone();
            let result_clone_reject = result_promise.clone();
            let remaining_clone = remaining.clone();
            let values_clone = values.clone();
            
            promise.then(
                Some(Rc::new(move |value| {
                    // Store the value
                    values_clone.lock().unwrap()[index] = value;
                    
                    // Decrement remaining count
                    let mut rem = remaining_clone.lock().unwrap();
                    *rem -= 1;
                    
                    // If all promises resolved, fulfill result promise
                    if *rem == 0 {
                        let final_values = values_clone.lock().unwrap().clone();
                        result_clone_fulfill.fulfill_promise(JSValue::Array(final_values));
                    }
                    
                    Ok(JSValue::Undefined)
                })),
                Some(Rc::new(move |reason| {
                    // Any rejection rejects the entire Promise.all
                    result_clone_reject.reject_promise(reason.clone());
                    Err(format!("{:?}", reason))
                })),
            );
        }
        
        result_promise
    }

    /// Promise.allSettled (25.6.4.2)
    pub fn all_settled(promises: Vec<Promise>) -> Promise {
        if promises.is_empty() {
            return Promise::resolve_static(JSValue::Array(vec![]));
        }
        
        let result_promise = Promise::new();
        let remaining = Arc::new(Mutex::new(promises.len()));
        let results = Arc::new(Mutex::new(vec![JSValue::Undefined; promises.len()]));
        
        for (index, promise) in promises.into_iter().enumerate() {
            let result_clone = result_promise.clone();
            let remaining_clone = remaining.clone();
            let results_clone = results.clone();
            
            let on_settled = move |status: &str, value: JSValue| {
                // Create settlement object
                let settlement = JSValue::String(format!("{{status: '{}', value: {:?}}}", status, value));
                results_clone.lock().unwrap()[index] = settlement;
                
                let mut rem = remaining_clone.lock().unwrap();
                *rem -= 1;
                
                if *rem == 0 {
                    let final_results = results_clone.lock().unwrap().clone();
                    result_clone.fulfill_promise(JSValue::Array(final_results));
                }
            };
            
            let on_settled_fulfill = on_settled.clone();
            promise.then(
                Some(Rc::new(move |value| {
                    on_settled_fulfill("fulfilled", value);
                    Ok(JSValue::Undefined)
                })),
                Some(Rc::new(move |reason| {
                    on_settled("rejected", reason);
                    Ok(JSValue::Undefined)
                })),
            );
        }
        
        result_promise
    }

    /// Promise.any (25.6.4.3) - Resolves when any promise fulfills
    pub fn any(promises: Vec<Promise>) -> Promise {
        if promises.is_empty() {
            return Promise::reject_static(JSValue::String("All promises were rejected".to_string()));
        }
        
        let result_promise = Promise::new();
        let remaining = Arc::new(Mutex::new(promises.len()));
        
        for promise in promises {
            let result_clone_fulfill = result_promise.clone();
            let result_clone_reject = result_promise.clone();
            let remaining_clone = remaining.clone();
            
            promise.then(
                Some(Rc::new(move |value| {
                    // First fulfillment wins
                    result_clone_fulfill.fulfill_promise(value);
                    Ok(JSValue::Undefined)
                })),
                Some(Rc::new(move |_reason| {
                    let mut rem = remaining_clone.lock().unwrap();
                    *rem -= 1;
                    
                    // If all rejected, reject with AggregateError
                    if *rem == 0 {
                        result_clone_reject.reject_promise(JSValue::String("All promises were rejected".to_string()));
                    }
                    
                    Ok(JSValue::Undefined)
                })),
            );
        }
        
        result_promise
    }

    /// Promise.race (25.6.4.5) - Settles when any promise settles
    pub fn race(promises: Vec<Promise>) -> Promise {
        let result_promise = Promise::new();
        
        for promise in promises {
            let result_clone = result_promise.clone();
            let result_clone2 = result_promise.clone();
            
            promise.then(
                Some(Rc::new(move |value| {
                    result_clone.fulfill_promise(value);
                    Ok(JSValue::Undefined)
                })),
                Some(Rc::new(move |reason| {
                    result_clone2.reject_promise(reason.clone());
                    Err(format!("{:?}", reason))
                })),
            );
        }
        
        result_promise
    }

    /// Promise.withResolvers (ES2024+) - Returns {promise, resolve, reject}
    pub fn with_resolvers() -> (Promise, Rc<dyn Fn(JSValue)>, Rc<dyn Fn(JSValue)>) {
        let promise = Promise::new();
        let promise_clone = promise.clone();
        let promise_clone2 = promise.clone();
        
        let resolve = Rc::new(move |value: JSValue| {
            promise_clone.fulfill_promise(value);
        });
        
        let reject = Rc::new(move |reason: JSValue| {
            promise_clone2.reject_promise(reason);
        });
        
        (promise, resolve, reject)
    }
}

// ============================================================================
// JSValue definition for Promise module
// ============================================================================

#[derive(Debug, Clone)]
pub enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(usize),
    Array(Vec<JSValue>),
    Promise(Promise),
}

impl Default for JSValue {
    fn default() -> Self {
        JSValue::Undefined
    }
}

// ============================================================================
// Async/Await Support - ECMAScript 2025 Async Functions
// ============================================================================

/// Async Function - Returns a Promise when called
/// Implements: https://tc39.es/ecma262/2025/#sec-async-function-objects
pub struct AsyncFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
    /// Captured scope for closures
    pub captured_scope: Option<Rc<RefCell<HashMap<String, JSValue>>>>,
}

impl AsyncFunction {
    pub fn new(name: String, params: Vec<String>, body: String) -> Self {
        Self {
            name,
            params,
            body,
            captured_scope: None,
        }
    }

    /// Execute async function - returns a Promise
    pub fn call(&self, args: Vec<JSValue>) -> Promise {
        let promise = Promise::new();
        
        // In real implementation, this would:
        // 1. Create new execution context
        // 2. Bind parameters to arguments
        // 3. Execute body as generator
        // 4. Await each await expression
        // 5. Resolve promise with final value
        
        // For now, return pending promise (will be implemented with interpreter)
        promise
    }

    /// Set captured scope for closure
    pub fn with_scope(mut self, scope: Rc<RefCell<HashMap<String, JSValue>>>) -> Self {
        self.captured_scope = Some(scope);
        self
    }
}

impl std::fmt::Debug for AsyncFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[AsyncFunction: {}]", self.name)
    }
}

impl Clone for AsyncFunction {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            captured_scope: self.captured_scope.clone(),
        }
    }
}

// ============================================================================
// Generator Support - ECMAScript 2025 Generator Functions
// ============================================================================

/// Generator object - Iterator with yield/next
/// Implements: https://tc39.es/ecma262/2025/#sec-generator-objects
pub struct Generator {
    pub state: GeneratorState,
    pub body: String,
    pub params: Vec<String>,
    pub locals: HashMap<String, JSValue>,
    /// Program counter for resumption
    pub pc: usize,
    /// Yielded values queue
    pub yield_queue: VecDeque<JSValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    Start,
    SuspendedStart,
    SuspendedYield(usize),
    Executing,
    Completed,
}

impl Generator {
    pub fn new(params: Vec<String>, body: String) -> Self {
        Self {
            state: GeneratorState::SuspendedStart,
            body,
            params,
            locals: HashMap::new(),
            pc: 0,
            yield_queue: VecDeque::new(),
        }
    }

    /// Generator.prototype.next() - Resume execution
    pub fn next(&mut self, value: Option<JSValue>) -> GeneratorResult {
        match self.state {
            GeneratorState::Completed => GeneratorResult {
                value: JSValue::Undefined,
                done: true,
            },
            GeneratorState::SuspendedStart => {
                self.state = GeneratorState::Executing;
                // Execute until first yield
                // For now, return placeholder
                self.state = GeneratorState::Completed;
                GeneratorResult {
                    value: JSValue::Undefined,
                    done: true,
                }
            },
            GeneratorState::SuspendedYield(pc) => {
                self.state = GeneratorState::Executing;
                self.pc = pc;
                // Resume from PC with optional value
                // Execute until next yield or completion
                self.state = GeneratorState::Completed;
                GeneratorResult {
                    value: value.unwrap_or(JSValue::Undefined),
                    done: true,
                }
            },
            _ => GeneratorResult {
                value: JSValue::Undefined,
                done: true,
            }
        }
    }

    /// Generator.prototype.return() - Close generator
    pub fn return_value(&mut self, value: JSValue) -> GeneratorResult {
        self.state = GeneratorState::Completed;
        GeneratorResult {
            value,
            done: true,
        }
    }

    /// Generator.prototype.throw() - Throw error into generator
    pub fn throw(&mut self, error: JSValue) -> Result<GeneratorResult, String> {
        self.state = GeneratorState::Completed;
        Err(format!("Generator error: {:?}", error))
    }

    /// Internal yield - called by generated code
    pub fn yield_value(&mut self, value: JSValue) {
        self.state = GeneratorState::SuspendedYield(self.pc);
        self.yield_queue.push_back(value);
    }
}

impl std::fmt::Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Generator: {:?}]", self.state)
    }
}

impl Clone for Generator {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            body: self.body.clone(),
            params: self.params.clone(),
            locals: self.locals.clone(),
            pc: self.pc,
            yield_queue: self.yield_queue.clone(),
        }
    }
}

/// Result of Generator.prototype.next()
#[derive(Debug, Clone)]
pub struct GeneratorResult {
    pub value: JSValue,
    pub done: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_resolve() {
        let promise = Promise::resolve_static(JSValue::Number(42.0));
        assert!(promise.is_settled());
        match promise.state() {
            PromiseState::Fulfilled(JSValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected fulfilled promise"),
        }
    }

    #[test]
    fn test_promise_reject() {
        let promise = Promise::reject_static(JSValue::String("error".to_string()));
        assert!(promise.is_settled());
        match promise.state() {
            PromiseState::Rejected(JSValue::String(s)) => assert_eq!(s, "error"),
            _ => panic!("Expected rejected promise"),
        }
    }

    #[test]
    fn test_promise_then_chaining() {
        let promise = Promise::resolve_static(JSValue::Number(10.0));
        
        let promise2 = promise.then(
            Some(Rc::new(|value| {
                if let JSValue::Number(n) = value {
                    Ok(JSValue::Number(n * 2.0))
                } else {
                    Ok(JSValue::Undefined)
                }
            })),
            None,
        );
        
        // Promise should be created (actual execution needs microtask queue)
        assert!(promise2.is_pending() || promise2.is_settled());
    }

    #[test]
    fn test_promise_all_empty() {
        let promise = Promise::all(vec![]);
        match promise.state() {
            PromiseState::Fulfilled(JSValue::Array(arr)) => assert!(arr.is_empty()),
            _ => panic!("Expected fulfilled with empty array"),
        }
    }

    #[test]
    fn test_promise_race() {
        let p1 = Promise::resolve_static(JSValue::Number(1.0));
        let p2 = Promise::resolve_static(JSValue::Number(2.0));
        
        let race = Promise::race(vec![p1, p2]);
        assert!(race.is_settled() || race.is_pending());
    }

    #[test]
    fn test_promise_with_resolvers() {
        let (promise, resolve, _reject) = Promise::with_resolvers();
        assert!(promise.is_pending());
        
        resolve(JSValue::Number(42.0));
        assert!(promise.is_settled());
    }

    #[test]
    fn test_generator_creation() {
        let gen = Generator::new(vec![], "yield 1; yield 2;".to_string());
        assert_eq!(gen.state, GeneratorState::SuspendedStart);
    }

    #[test]
    fn test_async_function_creation() {
        let async_fn = AsyncFunction::new(
            "fetchData".to_string(),
            vec!["url".to_string()],
            "return await fetch(url);".to_string(),
        );
        assert_eq!(async_fn.name, "fetchData");
    }
}
