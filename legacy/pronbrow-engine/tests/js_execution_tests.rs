// Comprehensive JavaScript Execution Tests
// Tests for function execution, control flow, expressions, and operators

use pronbrow_engine::js::engine::{JSEngine, JSValue};

#[test]
fn test_arithmetic_expressions() {
    let mut engine = JSEngine::new();
    
    // Basic arithmetic
    assert_eq_number(&engine.execute("2 + 3").unwrap(), 5.0);
    assert_eq_number(&engine.execute("10 - 4").unwrap(), 6.0);
    assert_eq_number(&engine.execute("3 * 4").unwrap(), 12.0);
    assert_eq_number(&engine.execute("15 / 3").unwrap(), 5.0);
    assert_eq_number(&engine.execute("17 % 5").unwrap(), 2.0);
    
    // Complex expressions with precedence
    assert_eq_number(&engine.execute("2 + 3 * 4").unwrap(), 14.0);
    assert_eq_number(&engine.execute("(2 + 3) * 4").unwrap(), 20.0);
    assert_eq_number(&engine.execute("10 / 2 + 3").unwrap(), 8.0);
    assert_eq_number(&engine.execute("10 / (2 + 3)").unwrap(), 2.0);
}

#[test]
fn test_comparison_operators() {
    let mut engine = JSEngine::new();
    
    // Equality
    assert_eq_boolean(&engine.execute("5 == 5").unwrap(), true);
    assert_eq_boolean(&engine.execute("5 == 6").unwrap(), false);
    assert_eq_boolean(&engine.execute("5 != 6").unwrap(), true);
    assert_eq_boolean(&engine.execute("5 != 5").unwrap(), false);
    
    // Relational
    assert_eq_boolean(&engine.execute("5 < 10").unwrap(), true);
    assert_eq_boolean(&engine.execute("10 < 5").unwrap(), false);
    assert_eq_boolean(&engine.execute("5 > 3").unwrap(), true);
    assert_eq_boolean(&engine.execute("3 > 5").unwrap(), false);
    assert_eq_boolean(&engine.execute("5 <= 5").unwrap(), true);
    assert_eq_boolean(&engine.execute("5 >= 5").unwrap(), true);
    assert_eq_boolean(&engine.execute("5 <= 4").unwrap(), false);
}

#[test]
fn test_logical_operators() {
    let mut engine = JSEngine::new();
    
    // Logical AND
    assert_eq_boolean(&engine.execute("true && true").unwrap(), true);
    assert_eq_boolean(&engine.execute("true && false").unwrap(), false);
    assert_eq_boolean(&engine.execute("false && true").unwrap(), false);
    
    // Logical OR
    assert_eq_boolean(&engine.execute("true || false").unwrap(), true);
    assert_eq_boolean(&engine.execute("false || true").unwrap(), true);
    assert_eq_boolean(&engine.execute("false || false").unwrap(), false);
    
    // Complex logical expressions
    assert_eq_boolean(&engine.execute("(5 > 3) && (10 < 20)").unwrap(), true);
    assert_eq_boolean(&engine.execute("(5 > 10) || (3 < 8)").unwrap(), true);
}

#[test]
fn test_variable_declarations() {
    let mut engine = JSEngine::new();
    
    // var declarations
    engine.execute("var x = 10").unwrap();
    assert_eq_number(&engine.execute("x").unwrap(), 10.0);
    
    // let declarations
    engine.execute("let y = 20").unwrap();
    assert_eq_number(&engine.execute("y").unwrap(), 20.0);
    
    // const declarations
    engine.execute("const z = 30").unwrap();
    assert_eq_number(&engine.execute("z").unwrap(), 30.0);
    
    // String variables
    engine.execute("var message = 'Hello'").unwrap();
    assert_eq_string(&engine.execute("message").unwrap(), "Hello");
    
    // Boolean variables
    engine.execute("let flag = true").unwrap();
    assert_eq_boolean(&engine.execute("flag").unwrap(), true);
}

#[test]
fn test_function_declarations_and_calls() {
    let mut engine = JSEngine::new();
    
    // Simple function with no parameters
    engine.execute("function greet() { return 'Hello' }").unwrap();
    assert_eq_string(&engine.execute("greet()").unwrap(), "Hello");
    
    // Function with parameters
    engine.execute("function add(a, b) { return a + b }").unwrap();
    assert_eq_number(&engine.execute("add(5, 3)").unwrap(), 8.0);
    assert_eq_number(&engine.execute("add(10, 20)").unwrap(), 30.0);
    
    // Function with multiple operations
    engine.execute("function multiply(x, y) { return x * y }").unwrap();
    assert_eq_number(&engine.execute("multiply(4, 7)").unwrap(), 28.0);
    
    // Function returning computed value
    engine.execute("function square(n) { return n * n }").unwrap();
    assert_eq_number(&engine.execute("square(5)").unwrap(), 25.0);
    assert_eq_number(&engine.execute("square(9)").unwrap(), 81.0);
}

#[test]
fn test_if_statements() {
    let mut engine = JSEngine::new();
    
    // Simple if statement
    engine.execute("var result = 0").unwrap();
    engine.execute("if (true) { result = 1 }").unwrap();
    assert_eq_number(&engine.execute("result").unwrap(), 1.0);
    
    // if with false condition
    engine.execute("var result2 = 0").unwrap();
    engine.execute("if (false) { result2 = 1 }").unwrap();
    assert_eq_number(&engine.execute("result2").unwrap(), 0.0);
    
    // if-else statement
    engine.execute("
        function checkNumber(n) {
            if (n > 10) {
                return 'big'
            } else {
                return 'small'
            }
        }
    ").unwrap();
    assert_eq_string(&engine.execute("checkNumber(15)").unwrap(), "big");
    assert_eq_string(&engine.execute("checkNumber(5)").unwrap(), "small");
}

#[test]
fn test_while_loops() {
    let mut engine = JSEngine::new();
    
    // Simple while loop - count to 5
    engine.execute("
        var counter = 0;
        while (counter < 5) {
            counter = counter + 1
        }
    ").unwrap();
    assert_eq_number(&engine.execute("counter").unwrap(), 5.0);
    
    // While loop with calculation
    engine.execute("
        function factorial(n) {
            var result = 1;
            var i = 1;
            while (i <= n) {
                result = result * i;
                i = i + 1
            }
            return result
        }
    ").unwrap();
    assert_eq_number(&engine.execute("factorial(5)").unwrap(), 120.0);
    assert_eq_number(&engine.execute("factorial(4)").unwrap(), 24.0);
}

#[test]
fn test_for_loops() {
    let mut engine = JSEngine::new();
    
    // Basic for loop
    engine.execute("
        var sum = 0;
        for (var i = 1; i <= 5; i = i + 1) {
            sum = sum + i
        }
    ").unwrap();
    assert_eq_number(&engine.execute("sum").unwrap(), 15.0);
    
    // For loop in function
    engine.execute("
        function sumRange(n) {
            var total = 0;
            for (var i = 1; i <= n; i = i + 1) {
                total = total + i
            }
            return total
        }
    ").unwrap();
    assert_eq_number(&engine.execute("sumRange(10)").unwrap(), 55.0);
    assert_eq_number(&engine.execute("sumRange(100)").unwrap(), 5050.0);
}

#[test]
fn test_fibonacci_function() {
    let mut engine = JSEngine::new();
    
    // Fibonacci using iteration
    engine.execute("
        function fibonacci(n) {
            if (n <= 1) {
                return n
            }
            var a = 0;
            var b = 1;
            for (var i = 2; i <= n; i = i + 1) {
                var temp = a + b;
                a = b;
                b = temp
            }
            return b
        }
    ").unwrap();
    
    assert_eq_number(&engine.execute("fibonacci(0)").unwrap(), 0.0);
    assert_eq_number(&engine.execute("fibonacci(1)").unwrap(), 1.0);
    assert_eq_number(&engine.execute("fibonacci(5)").unwrap(), 5.0);
    assert_eq_number(&engine.execute("fibonacci(10)").unwrap(), 55.0);
}

#[test]
fn test_console_operations() {
    let mut engine = JSEngine::new();
    
    engine.execute("console.log('Test message')").unwrap();
    assert!(engine.console.logs.len() > 0);
    assert!(engine.console.logs.last().unwrap().contains("Test message"));
    
    engine.execute("console.error('Error message')").unwrap();
    assert!(engine.console.logs.last().unwrap().contains("ERROR"));
    
    engine.execute("console.warn('Warning message')").unwrap();
    assert!(engine.console.logs.last().unwrap().contains("WARN"));
}

#[test]
fn test_string_operations() {
    let mut engine = JSEngine::new();
    
    // String concatenation
    assert_eq_string(&engine.execute("'Hello' + ' ' + 'World'").unwrap(), "Hello World");
    
    // String with variables
    engine.execute("var name = 'Alice'").unwrap();
    assert_eq_string(&engine.execute("'Hello ' + name").unwrap(), "Hello Alice");
}

#[test]
fn test_complex_expressions_in_functions() {
    let mut engine = JSEngine::new();
    
    // Function with complex logic
    engine.execute("
        function isPrime(n) {
            if (n <= 1) {
                return false
            }
            if (n <= 3) {
                return true
            }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) {
                    return false
                }
                i = i + 1
            }
            return true
        }
    ").unwrap();
    
    assert_eq_boolean(&engine.execute("isPrime(2)").unwrap(), true);
    assert_eq_boolean(&engine.execute("isPrime(3)").unwrap(), true);
    assert_eq_boolean(&engine.execute("isPrime(4)").unwrap(), false);
    assert_eq_boolean(&engine.execute("isPrime(17)").unwrap(), true);
    assert_eq_boolean(&engine.execute("isPrime(20)").unwrap(), false);
}

// Helper functions for assertions
fn assert_eq_number(value: &JSValue, expected: f64) {
    match value {
        JSValue::Number(n) => assert_eq!(*n, expected),
        _ => panic!("Expected number, got {:?}", value),
    }
}

fn assert_eq_boolean(value: &JSValue, expected: bool) {
    match value {
        JSValue::Boolean(b) => assert_eq!(*b, expected),
        _ => panic!("Expected boolean, got {:?}", value),
    }
}

fn assert_eq_string(value: &JSValue, expected: &str) {
    match value {
        JSValue::String(s) => assert_eq!(s, expected),
        _ => panic!("Expected string, got {:?}", value),
    }
}
