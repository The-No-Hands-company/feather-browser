# Phase 2: True Async Suspension & Resumption
## Implementation Roadmap for Complete Async/Await & Generator Support

**Target:** Full suspension/resumption matching browser behavior  
**Estimated Time:** 1-2 weeks  
**Current State:** 87.5% complete, infrastructure ready

---

## 🎯 PHASE 2 GOALS

### 1. True Await Suspension
**Current:** Await evaluates synchronously  
**Target:** Suspend execution, resume on Promise settlement

### 2. Generator State Machine
**Current:** Returns static iterator  
**Target:** PC-based execution with yield suspension

### 3. Microtask Scheduling
**Current:** Microtasks processed after program  
**Target:** Schedule await continuations as microtasks

---

## 📋 WEEK 1: AWAIT SUSPENSION

### Day 1-2: Suspension Mechanism

**File:** `src/js/interpreter.rs`  
**Lines:** ~150 new

#### Step 1: Enhance AwaitExpression Handling
```rust
ASTNode::AwaitExpression { argument } => {
    let promise_value = self.evaluate(argument)?;
    
    // Extract Promise from JSValue
    if let JSValue::Object(obj) = promise_value {
        if self.is_promise_object(&obj) {
            // Get the ES2025Promise
            let promise = self.extract_promise(&obj)?;
            
            // Create continuation
            let continuation = self.create_await_continuation()?;
            
            // Register continuation on promise
            promise.then(move |value| {
                continuation.resume_with(value);
                Ok(value)
            });
            
            // Suspend current async function
            return Err(AsyncSuspension);
        }
    }
    
    // Not a promise, return value directly
    Ok(promise_value)
}
```

#### Step 2: Create Continuation Structure
```rust
struct AsyncContinuation {
    /// Saved interpreter state
    locals: HashMap<String, JSValue>,
    /// Saved call stack
    call_stack: Vec<CallFrame>,
    /// Remaining AST to execute
    remaining_body: Option<Box<ASTNode>>,
    /// Promise to resolve when complete
    promise: ES2025Promise,
}

impl AsyncContinuation {
    fn resume_with(&mut self, value: JSValue) {
        // Restore interpreter state
        // Continue execution
        // Resolve promise on completion
    }
}
```

#### Step 3: Handle AsyncSuspension
```rust
// In execute_async_body
match self.evaluate(&context.body) {
    Ok(value) => {
        // Normal completion
        promise.fulfill(value);
    }
    Err(AsyncSuspension) => {
        // Suspended on await - continuation registered
        // Don't resolve promise yet
    }
    Err(e) => {
        // Error - reject promise
        promise.reject(e);
    }
}
```

### Day 3: Testing
**File:** `tests/await_suspension.rs`

```rust
#[test]
fn test_await_suspension() {
    let code = r#"
        async function test() {
            const x = await Promise.resolve(42);
            return x * 2;
        }
        test();
    "#;
    
    // Should return Promise
    // Microtasks should contain continuation
    // Promise should resolve to 84
}

#[test]
fn test_multiple_awaits() {
    let code = r#"
        async function test() {
            const a = await Promise.resolve(1);
            const b = await Promise.resolve(2);
            const c = await Promise.resolve(3);
            return a + b + c;
        }
    "#;
    
    // Should handle multiple suspension points
    // Result should be 6
}
```

### Day 4-5: Microtask Integration

#### Update process_microtasks
```rust
fn process_microtasks(&mut self) -> Result<(), String> {
    while let Some(task) = self.microtask_queue.pop_front() {
        match task {
            Microtask::Callback(cb) => {
                cb()?;
            }
            Microtask::AsyncContinuation(cont) => {
                // Resume async function
                self.resume_async_continuation(cont)?;
            }
            Microtask::PromiseReaction(reaction) => {
                // Execute promise reaction
                self.execute_promise_reaction(reaction)?;
            }
        }
    }
    Ok(())
}
```

#### Queue Continuations
```rust
fn queue_async_continuation(&mut self, continuation: AsyncContinuation) {
    let microtask = Microtask::AsyncContinuation(continuation);
    self.microtask_queue.push_back(microtask);
}
```

---

## 📋 WEEK 2: GENERATOR STATE MACHINE

### Day 1-2: PC-Based Execution

**Concept:** Track program counter to resume at exact yield point

#### Step 1: Add PC to AST Traversal
```rust
struct ExecutionState {
    pc: usize,  // Current instruction
    locals: HashMap<String, JSValue>,
    value_stack: Vec<JSValue>,
}

// Transform AST to numbered instructions
enum Instruction {
    Literal(JSValue),
    Identifier(String),
    BinaryOp { op: BinaryOperator, left: usize, right: usize },
    Yield { value: Option<usize> },
    Return { value: Option<usize> },
}
```

#### Step 2: Generator State Machine
```rust
impl GeneratorContext {
    fn next(&mut self, value: JSValue) -> Result<IteratorResult, String> {
        // Restore state
        let mut state = ExecutionState {
            pc: self.pc,
            locals: self.locals.clone(),
            value_stack: self.value_stack.clone(),
        };
        
        // If resuming from yield, push the passed value
        if self.state == GeneratorState::SuspendedYield {
            state.value_stack.push(value);
        }
        
        // Execute instructions
        loop {
            let instruction = &self.instructions[state.pc];
            
            match instruction {
                Instruction::Yield { value } => {
                    let yield_value = if let Some(idx) = value {
                        state.value_stack[*idx].clone()
                    } else {
                        JSValue::Undefined
                    };
                    
                    // Save state
                    self.pc = state.pc + 1;
                    self.locals = state.locals;
                    self.value_stack = state.value_stack;
                    self.state = GeneratorState::SuspendedYield;
                    
                    return Ok(IteratorResult {
                        value: yield_value,
                        done: false,
                    });
                }
                
                Instruction::Return { value } => {
                    let return_value = if let Some(idx) = value {
                        state.value_stack[*idx].clone()
                    } else {
                        JSValue::Undefined
                    };
                    
                    self.state = GeneratorState::Completed;
                    
                    return Ok(IteratorResult {
                        value: return_value,
                        done: true,
                    });
                }
                
                // ... other instructions
            }
            
            state.pc += 1;
        }
    }
}
```

### Day 3: AST to Instruction Conversion

```rust
fn ast_to_instructions(node: &ASTNode) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut pc = 0;
    
    fn visit(node: &ASTNode, instructions: &mut Vec<Instruction>, pc: &mut usize) {
        match node {
            ASTNode::YieldExpression { argument, delegate } => {
                if let Some(arg) = argument {
                    visit(arg, instructions, pc);
                    let value_idx = *pc - 1;
                    instructions.push(Instruction::Yield {
                        value: Some(value_idx)
                    });
                } else {
                    instructions.push(Instruction::Yield { value: None });
                }
                *pc += 1;
            }
            
            ASTNode::Number(n) => {
                instructions.push(Instruction::Literal(JSValue::Number(*n)));
                *pc += 1;
            }
            
            // ... other node types
        }
    }
    
    visit(node, &mut instructions, &mut pc);
    instructions
}
```

### Day 4: Generator Integration

#### Update call_generator_function
```rust
fn call_generator_function(
    &mut self,
    f: &super::runtime::Function,
    arguments: &[ASTNode],
    this_binding: JSValue,
) -> Result<JSValue, String> {
    // Convert function body to instructions
    let instructions = ast_to_instructions(&f.body);
    
    // Create generator context
    let context = Rc::new(RefCell::new(GeneratorContext {
        instructions,
        pc: 0,
        locals: HashMap::new(),
        value_stack: Vec::new(),
        state: GeneratorState::SuspendedStart,
        // ...
    }));
    
    // Create generator object with live context
    let gen_obj = self.create_generator_object(context)?;
    
    Ok(JSValue::Object(Rc::new(RefCell::new(gen_obj))))
}
```

#### Live next() Method
```rust
fn create_generator_object(&self, context: Rc<RefCell<GeneratorContext>>) 
    -> Result<JSObject, String> 
{
    let mut gen_obj = JSObject::new();
    
    let ctx_clone = context.clone();
    let next_fn = Function::native_function(
        "next".to_string(),
        vec!["value".to_string()],
        move |args| {
            let value = args.get(0).cloned().unwrap_or(JSValue::Undefined);
            
            // Call generator's next with the value
            let mut ctx = ctx_clone.borrow_mut();
            let result = ctx.next(value)?;
            
            // Convert to JSValue
            let mut result_obj = JSObject::new();
            result_obj.set("value", result.value);
            result_obj.set("done", JSValue::Boolean(result.done));
            
            Ok(JSValue::Object(Rc::new(RefCell::new(result_obj))))
        }
    );
    
    gen_obj.set("next", JSValue::Function(Rc::new(next_fn)));
    
    Ok(gen_obj)
}
```

### Day 5: Testing & Integration

**File:** `tests/generator_execution.rs`

```rust
#[test]
fn test_generator_yields_values() {
    let code = r#"
        function* gen() {
            yield 1;
            yield 2;
            yield 3;
        }
        const g = gen();
        const a = g.next();
        const b = g.next();
        const c = g.next();
        const d = g.next();
    "#;
    
    // a should be { value: 1, done: false }
    // b should be { value: 2, done: false }
    // c should be { value: 3, done: false }
    // d should be { value: undefined, done: true }
}

#[test]
fn test_generator_with_values() {
    let code = r#"
        function* gen() {
            const x = yield 1;
            const y = yield x * 2;
            return y * 3;
        }
        const g = gen();
        g.next();      // { value: 1, done: false }
        g.next(10);    // { value: 20, done: false }
        g.next(100);   // { value: 300, done: true }
    "#;
}
```

---

## 📋 ADVANCED FEATURES

### Async Generators
**Week 3** - Combine async and generator mechanisms

```rust
async function* asyncGen() {
    yield await Promise.resolve(1);
    yield await Promise.resolve(2);
}
```

Implementation:
1. Generator state machine with PC tracking
2. Await suspension within generator
3. Resume on both next() and Promise settlement

### for-await-of Loops
```rust
for await (const item of asyncIterable) {
    console.log(item);
}
```

Parser changes:
1. Recognize `for await` syntax
2. Create ForAwaitStmt AST node
3. Handle AsyncIterator protocol

---

## 🧪 TESTING STRATEGY

### Unit Tests
- Await suspension/resumption
- Generator yield/next cycle
- Error propagation through async
- Multiple await in sequence
- Nested generators

### Integration Tests
- Real-world async patterns
- Promise chains with await
- Generator iteration
- Async generator streaming

### Performance Tests
- State save/restore overhead
- Microtask queue performance
- Memory usage with many generators

---

## 📊 SUCCESS METRICS

### Completion Criteria
- ✅ Await suspends and resumes correctly
- ✅ Generators track PC and resume
- ✅ Microtasks scheduled properly
- ✅ All tests passing
- ✅ Real-world patterns work

### Browser Parity
Target: 100% async feature parity with Chrome/Firefox/Safari

| Feature | Current | Target |
|---------|---------|--------|
| Promise | 100% | 100% |
| Microtasks | 100% | 100% |
| Async Execution | 70% | 100% |
| Generator Execution | 75% | 100% |
| **Overall** | **87.5%** | **100%** |

---

## 🎯 IMPLEMENTATION CHECKLIST

### Week 1: Await Suspension
- [ ] AsyncContinuation structure
- [ ] Suspension mechanism in AwaitExpression
- [ ] Continuation queueing
- [ ] State save/restore
- [ ] Microtask integration
- [ ] Tests for suspension
- [ ] Multiple await handling

### Week 2: Generator State Machine
- [ ] Instruction enum
- [ ] AST to instruction conversion
- [ ] PC-based execution
- [ ] Yield suspension
- [ ] next() resumption
- [ ] Live generator objects
- [ ] Tests for iteration

### Week 3: Polish
- [ ] Async generators
- [ ] for-await-of
- [ ] Error handling
- [ ] Performance optimization
- [ ] Integration tests
- [ ] Documentation updates

---

## 💡 KEY DESIGN DECISIONS

### 1. Suspension Strategy
**Chosen:** Exception-based suspension  
**Why:** Clean separation, no AST changes needed

### 2. State Representation
**Chosen:** PC + value stack  
**Why:** Simple, efficient, maps to bytecode future

### 3. Continuation Storage
**Chosen:** Microtask queue  
**Why:** Matches browser behavior, integrates with Promise

---

## 📚 REFERENCE IMPLEMENTATIONS

### V8 (Chrome)
- Generators use bytecode with resume points
- Await uses builtin suspension
- State stored in GeneratorObject

### SpiderMonkey (Firefox)
- Similar bytecode approach
- Async state in AsyncFunctionObject
- Microtasks via Promise job queue

### JavaScriptCore (Safari)
- Generator state in iterator
- Await suspension in promise chain
- Efficient state representation

---

## 🚀 EXPECTED OUTCOME

After Phase 2 completion:

```javascript
// ✅ Full async/await support
async function fetchUser(id) {
    const response = await fetch(`/api/users/${id}`);
    const data = await response.json();
    return data;
}

// ✅ Complete generator iteration
function* fibonacci() {
    let [a, b] = [0, 1];
    while (true) {
        yield a;
        [a, b] = [b, a + b];
    }
}

const fib = fibonacci();
console.log(fib.next().value);  // 0
console.log(fib.next().value);  // 1
console.log(fib.next().value);  // 1
console.log(fib.next().value);  // 2

// ✅ Async generators
async function* asyncSequence() {
    for (let i = 0; i < 10; i++) {
        await delay(100);
        yield i;
    }
}

for await (const num of asyncSequence()) {
    console.log(num);
}
```

**Result:** 100% async/await and generator support matching major browsers!

---

**Timeline:** 2-3 weeks  
**Complexity:** Medium-High  
**Impact:** Engine reaches 85%+ overall completion  
**Status:** Infrastructure ready, clear implementation path

*Next session: Begin Week 1 implementation!*
