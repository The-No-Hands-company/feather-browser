# Implementation Plan - ES2025 Compliance
## Date: November 18, 2025
## Status: Phase 2 - 82% Complete

---

## 🎯 CURRENT PRIORITIES

Based on your requirements, here are the next features to implement:

### 1. **Rejection Handling for Async/Await** (Priority: HIGH)
**Status:** 30% complete  
**Time Estimate:** 2-3 hours  
**Impact:** Completes async/await to 100%

**What's Missing:**
- `.catch()` handler in `evaluate_await`
- Error conversion from Promise rejection to JavaScript error
- Integration with try/catch blocks
- Stack trace preservation

**Implementation Steps:**
1. Add reject handler to Promise.then() in evaluate_await
2. Queue error continuations as microtasks
3. Handle rejection in resume_async_continuation
4. Test error propagation

### 2. **Function.prototype.call/apply/bind** (Priority: HIGH)
**Status:** 10% complete (stubs only)  
**Time Estimate:** 4-6 hours  
**Impact:** ES2025 core compliance

**Current State:**
- Stub implementations exist
- Need access to function context
- Requires 'this' binding mechanism

**Implementation Steps:**
1. Implement Function.prototype.call()
2. Implement Function.prototype.apply()
3. Implement Function.prototype.bind()
4. Create bound function objects
5. Test all three methods

### 3. **Generator PC-Based Resumption** (Priority: MEDIUM)
**Status:** 80% complete  
**Time Estimate:** 2-3 hours  
**Impact:** Full generator iteration support

**What's Missing:**
- Statement-level PC tracking
- Skip already-executed yields
- Proper multi-yield iteration

**Implementation Steps:**
1. Add statement PC to execution
2. Track executed statements
3. Skip to next unexecuted yield
4. Test multi-yield scenarios

### 4. **Microtask Queue Integration** (Priority: LOW)
**Status:** 100% complete ✅  
**Notes:** Already fully functional

---

## 📋 DETAILED IMPLEMENTATION

### Task 1: Async Rejection Handling

**File:** `src/js/interpreter.rs`

```rust
fn evaluate_await(&mut self, argument: &ASTNode) -> Result<JSValue, InterpreterError> {
    // ... existing code ...
    
    // Clone for reject handler
    let continuation_for_catch = Rc::new(RefCell::new(continuation.clone()));
    let microtask_queue_catch = self.microtask_queue.clone();
    
    promise.then(
        Some(Rc::new(move |value| {
            // ... existing fulfill handler ...
        })),
        Some(Rc::new(move |error| {
            // NEW: Handle rejection
            let runtime_error = convert_from_promise_value(error);
            let mut cont = continuation_for_catch.borrow_mut();
            
            // Mark continuation as rejected
            cont.is_rejected = true;
            cont.rejection_value = runtime_error;
            
            // Queue for resumption
            microtask_queue_catch.borrow_mut().push_back(
                Microtask::AsyncContinuation(cont.clone())
            );
            
            Ok(super::promise::JSValue::Undefined)
        }))
    );
    
    // ... rest of code ...
}
```

**Add to AsyncContinuation:**
```rust
struct AsyncContinuation {
    // ... existing fields ...
    is_rejected: bool,
    rejection_value: JSValue,
}
```

**Update resume_async_continuation:**
```rust
fn resume_async_continuation(&mut self, continuation: AsyncContinuation) -> Result<(), String> {
    // ... restore state ...
    
    if continuation.is_rejected {
        // Throw the rejection as an error
        return Err(format!("Unhandled promise rejection: {:?}", continuation.rejection_value));
    }
    
    // ... rest of resumption ...
}
```

---

### Task 2: Function.prototype Methods

**File:** `src/js/interpreter.rs`

#### 2.1: Function.prototype.call()

```rust
// Update init_function_prototype()
let call_fn = super::runtime::Function::native_function(
    "call".to_string(),
    vec![],
    move |args| {
        // args[0] = the function itself (bound 'this')
        // args[1] = thisArg
        // args[2..] = arguments to pass
        
        if args.is_empty() {
            return Err("Function.prototype.call requires a function".to_string());
        }
        
        // Get the function to call
        let func = match &args[0] {
            JSValue::Function(f) => f.clone(),
            _ => return Err("call() can only be used on functions".to_string()),
        };
        
        // Get thisArg (or undefined if not provided)
        let this_arg = args.get(1).cloned().unwrap_or(JSValue::Undefined);
        
        // Get remaining arguments
        let call_args: Vec<JSValue> = args.iter().skip(2).cloned().collect();
        
        // Call the function with thisArg as 'this'
        // TODO: Need access to interpreter to call function
        // For now, call without 'this' binding
        (func.body.as_ref().unwrap())(&call_args)
    }
);
```

**Problem:** Native functions can't access interpreter to properly call user functions with 'this' binding.

**Solution:** Create a method-based approach:

```rust
// Add to Interpreter impl
pub fn call_function_with_this(
    &mut self,
    func: &super::runtime::Function,
    this_binding: JSValue,
    args: Vec<JSValue>,
) -> Result<JSValue, String> {
    // Save current 'this'
    let saved_this = self.this_value.clone();
    
    // Set new 'this'
    self.this_value = this_binding;
    
    // Call function
    let result = if let Some(body) = &func.body {
        // Native function
        body(&args)
    } else {
        // User-defined function
        self.call_user_function(func, &args)
    };
    
    // Restore 'this'
    self.this_value = saved_this;
    
    result
}
```

Then expose this through a special global:

```rust
// In init_function_prototype, store interpreter reference
// This requires rethinking the architecture...
```

**Alternative Approach:** Use MemberExpression handling:

```rust
// In evaluate() for MemberExpression
ASTNode::MemberExpression { object, property, computed } => {
    // ... existing code ...
    
    // Check if property is 'call', 'apply', or 'bind'
    if let JSValue::Function(_) = object_value {
        if property_name == "call" {
            return self.handle_function_call(object_value, /* ... */);
        }
        // ... etc
    }
}
```

#### 2.2: Function.prototype.apply()

```rust
fn handle_function_apply(
    &mut self,
    func: JSValue,
    args: Vec<JSValue>,
) -> Result<JSValue, String> {
    // args[0] = thisArg
    // args[1] = array of arguments
    
    let this_arg = args.get(0).cloned().unwrap_or(JSValue::Undefined);
    
    let call_args = match args.get(1) {
        Some(JSValue::Array(arr)) => arr.clone(),
        Some(JSValue::Object(obj)) => {
            // Handle array-like object
            // Get 'length' property and iterate
            vec![] // TODO: Implement
        }
        _ => vec![],
    };
    
    self.call_function_with_this_binding(func, this_arg, call_args)
}
```

#### 2.3: Function.prototype.bind()

```rust
fn handle_function_bind(
    &mut self,
    func: JSValue,
    args: Vec<JSValue>,
) -> Result<JSValue, String> {
    // args[0] = thisArg
    // args[1..] = bound arguments
    
    let this_arg = args.get(0).cloned().unwrap_or(JSValue::Undefined);
    let bound_args: Vec<JSValue> = args.iter().skip(1).cloned().collect();
    
    // Create a bound function
    let original_func = match func {
        JSValue::Function(f) => f,
        _ => return Err("bind() can only be used on functions".to_string()),
    };
    
    let bound_fn = super::runtime::Function::bound_function(
        format!("bound {}", original_func.name),
        original_func.clone(),
        this_arg,
        bound_args,
    );
    
    Ok(JSValue::Function(Rc::new(bound_fn)))
}
```

**Add to Function struct in runtime.rs:**

```rust
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Rc<dyn Fn(&[JSValue]) -> Result<JSValue, String>>>,
    pub ast_body: Option<Box<ASTNode>>,
    pub is_async: bool,
    pub is_generator: bool,
    // NEW: Bound function support
    pub bound_this: Option<JSValue>,
    pub bound_args: Option<Vec<JSValue>>,
    pub target_function: Option<Rc<Function>>,
}
```

---

### Task 3: Generator PC Tracking

**File:** `src/js/interpreter.rs`

**Current Issue:** Generator re-executes all yields on each next() call, only returning the Nth one.

**Solution:** Track which statements have been executed:

```rust
struct GeneratorContext {
    // ... existing fields ...
    
    // NEW: Statement execution tracking
    executed_statements: HashSet<usize>,
    statement_pcs: Vec<usize>, // PC for each statement in body
}
```

**Update execute_generator_next:**

```rust
fn execute_generator_next(...) -> Result<JSValue, String> {
    // ... existing code ...
    
    // When resuming from yield:
    super::promise::GeneratorState::SuspendedYield(last_yield_idx) => {
        // Set yield_counter to skip already-executed yields
        self.yield_counter = last_yield_idx + 1;
        
        // Mark previous statements as executed
        context.borrow_mut().mark_statements_executed_until(last_yield_idx);
        
        // Execute body
        let result = self.evaluate_with_pc(&body, &context.borrow().executed_statements)?;
        
        // ... rest of code ...
    }
}
```

**Add PC-aware evaluation:**

```rust
fn evaluate_with_pc(
    &mut self,
    node: &ASTNode,
    executed: &HashSet<usize>,
) -> Result<JSValue, String> {
    match node {
        ASTNode::BlockStatement { body } => {
            for (idx, stmt) in body.iter().enumerate() {
                if executed.contains(&idx) {
                    continue; // Skip already executed
                }
                
                self.evaluate(stmt)?;
            }
            Ok(JSValue::Undefined)
        }
        // ... other nodes ...
    }
}
```

---

## 🧪 TESTING PLAN

### Test 1: Async Rejection
```javascript
async function test() {
    try {
        await Promise.reject(new Error("Test error"));
    } catch (e) {
        console.log("Caught:", e);
    }
}
test();
```

### Test 2: Function.prototype.call
```javascript
function greet(greeting) {
    console.log(greeting + ", " + this.name);
}

const person = { name: "Alice" };
greet.call(person, "Hello");
// Expected: "Hello, Alice"
```

### Test 3: Function.prototype.apply
```javascript
function sum(a, b, c) {
    return a + b + c;
}

const result = sum.apply(null, [1, 2, 3]);
// Expected: 6
```

### Test 4: Function.prototype.bind
```javascript
function multiply(a, b) {
    return a * b;
}

const double = multiply.bind(null, 2);
console.log(double(5));
// Expected: 10
```

### Test 5: Generator Multi-Yield
```javascript
function* gen() {
    yield 1;
    yield 2;
    yield 3;
}

const g = gen();
console.log(g.next().value); // 1
console.log(g.next().value); // 2
console.log(g.next().value); // 3
```

---

## 📊 PRIORITY ORDERING

1. **Async Rejection Handling** (2-3 hours) - Completes async/await
2. **Generator PC Tracking** (2-3 hours) - Fixes generator iteration
3. **Function.prototype.call** (2 hours) - Most commonly used
4. **Function.prototype.apply** (1 hour) - Similar to call
5. **Function.prototype.bind** (2 hours) - Requires bound function support

**Total Estimate:** 9-11 hours of focused work

---

## 🎯 EXPECTED OUTCOME

After completing these tasks:

- **Async/Await:** 90% → 100% ✅
- **Generators:** 80% → 95% ✅
- **Function Methods:** 10% → 100% ✅
- **Phase 2:** 82% → 95% ✅
- **ES2025 Compliance:** 70.8% → 75.5% ✅

---

**Ready to begin implementation!**
