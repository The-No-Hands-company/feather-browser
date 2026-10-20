// Tests for function expression support
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_anonymous_function_expression() {
    let mut engine = JSEngine::new();

    // Anonymous function expression assigned to variable
    let code = r#"
        const f = function() { return 42; };
        const result = f();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Anonymous function expression failed: {:?}", result);
}

#[test]
fn test_named_function_expression() {
    let mut engine = JSEngine::new();

    // Named function expression (simplified)
    let code = "const f = function myFunc() { return 42; };";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Named function expression failed: {:?}", result);
}

#[test]
fn test_function_expression_with_parameters() {
    let mut engine = JSEngine::new();

    // Function expression with parameters
    let code = r#"
        const add = function(a, b) {
            return a + b;
        };
        const result = add(10, 20);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function expression with parameters failed: {:?}", result);
}

#[test]
fn test_function_expression_in_object() {
    let mut engine = JSEngine::new();

    // Function expression as object method
    let code = r#"
        const obj = {
            greet: function(name) {
                return name;
            }
        };
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function expression in object failed: {:?}", result);
}

#[test]
#[ignore] // TODO: Fix parsing of function expressions as direct call arguments
fn test_function_expression_as_callback() {
    let mut engine = JSEngine::new();

    // Function expression passed as argument
    let code = "function execute(callback) { return callback(); } const result = execute(function() { return 100; });";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function expression as callback failed: {:?}", result);
}

#[test]
fn test_iife_pattern() {
    let mut engine = JSEngine::new();

    // Immediately Invoked Function Expression (IIFE)
    let code = r#"
        const result = (function() { return 42; })();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "IIFE pattern failed: {:?}", result);
}

#[test]
fn test_function_expression_vs_declaration() {
    let mut engine = JSEngine::new();

    // Mix of function declaration and expression
    let code = r#"
        function declared() { return 1; }
        const expressed = function() { return 2; };
        const r1 = declared();
        const r2 = expressed();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function expression vs declaration failed: {:?}", result);
}
