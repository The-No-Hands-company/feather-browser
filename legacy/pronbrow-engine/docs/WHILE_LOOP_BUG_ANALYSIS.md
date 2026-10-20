# While Loop Variable Update Bug - Analysis

## Date
November 14, 2025

## Summary
A critical bug in the JavaScript engine causes incorrect return values when:
1. A while/for loop contains multiple statements in its body
2. The function returns the FIRST variable updated in each iteration
3. WITHOUT reading that variable after the loop ends

## Symptom
Functions return `n+1` instead of the correct calculated value.

Example:
```javascript
function factorial(n) {
    var result = 1;
    var i = 1;
    while (i <= n) {
        result = result * i;  // First statement
        i = i + 1              // Second statement  
    }
    return result  // Returns n+1 instead of n!
}

factorial(5)  // Returns 6 instead of 120
factorial(3)  // Returns 4 instead of 6
```

## Root Cause
When `execute_block` executes multiple statements within a loop iteration:
- The FIRST variable update appears to be lost
- The LAST statement's result becomes the loop iteration's result
- Subsequent variable reads work correctly
- But immediate returns of the first-updated variable fail

## Test Results

### ✓ WORKS
1. **Single statement in loop**
   ```javascript
   while (x < 3) { x = x + 1 }  // ✓ Returns 3
   ```

2. **Return second variable**
   ```javascript
   while (b <= 3) { a = a * b; b = b + 1 }
   return b  // ✓ Returns 4 (correct)
   ```

3. **Read first variable after loop**
   ```javascript
   while (b <= 3) { a = a * b; b = b + 1 }
   var dummy = a;  // Force evaluation
   return a  // ✓ Returns 6 (correct!)
   ```

4. **Reverse statement order**
   ```javascript
   while (b <= 3) { b = b + 1; a = a * (b - 1) }
   return a  // ✓ Returns 6 (correct!)
   ```

5. **For loops with update clause**
   ```javascript
   for (var i = 1; i <= 3; i = i + 1) { result = result * i }
   return result  // ✓ Returns 6 (correct!)
   ```

### ✗ BROKEN
1. **Multiple statements, return first variable**
   ```javascript
   while (b <= 3) { a = a * b; b = b + 1 }
   return a  // ✗ Returns 4 instead of 6
   ```

## Workarounds

### Workaround 1: Add console.log
```javascript
function factorial(n) {
    var result = 1;
    var i = 1;
    while (i <= n) {
        console.log('i=' + i);  // Forces proper evaluation
        result = result * i;
        i = i + 1
    }
    return result  // ✓ Now returns correct value
}
```

### Workaround 2: Read variable after loop
```javascript
function factorial(n) {
    var result = 1;
    var i = 1;
    while (i <= n) {
        result = result * i;
        i = i + 1
    }
    var dummy = result;  // Force evaluation
    return result  // ✓ Now returns correct value
}
```

### Workaround 3: Use for loop with update clause
```javascript
function factorial(n) {
    var result = 1;
    for (var i = 1; i <= 3; i = i + 1) {
        result = result * i
    }
    return result  // ✓ Works correctly
}
```

## Investigation Notes

1. **Not a scoping issue** - Variables are properly declared in the frame
2. **Not a state pollution issue** - Happens even in fresh engines
3. **Not specific to while loops** - Also affects for loops with multi-statement bodies
4. **Related to execute_block** - The bug appears when `execute_block` processes multiple statements within a loop

## Suspected Cause

The issue likely lies in how `handle_variable_declaration` updates variables in the call stack frame when called multiple times within `execute_block`:

```rust
fn handle_variable_declaration(&mut self, code: &str) -> Result<JSValue, String> {
    // ...
    if let Some(frame) = self.call_stack.last_mut() {
        if frame.locals.contains_key(name) {
            frame.locals.insert(name.to_string(), value.clone());
            return Ok(value);
        }
    }
    // ...
}
```

Hypothesis: When `execute_block` calls `execute_single_statement` multiple times in sequence, the first variable update may not be properly persisted in the frame's HashMap, or the frame reference is somehow invalidated between calls.

## Recommended Fix Direction

1. **Add instrumentation** to `handle_variable_declaration` to log when variables are updated in frames
2. **Verify HashMap updates** persist across multiple calls to `last_mut()`
3. **Check if** `execute_block` needs to explicitly maintain frame state between statement executions
4. **Consider** whether the issue is with lazy evaluation or caching in variable lookup

## Impact

**Current test results:** 9/12 passing (75%)

**Affected tests:**
- `test_while_loops` - factorial function
- `test_fibonacci_function` - iterative fibonacci
- `test_complex_expressions_in_functions` - isPrime function

All three tests involve while loops with multiple statements returning the first-updated variable.

## Next Steps

1. Add debug logging to understand exact execution flow
2. Test if the issue is specific to HashMap operations
3. Consider refactoring variable storage mechanism
4. Verify that mutable borrows to call_stack.last_mut() work as expected

---

**Status:** Under investigation  
**Priority:** High (blocks 3/12 tests)  
**Workarounds:** Available (see above)
