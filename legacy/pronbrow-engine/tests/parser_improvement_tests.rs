// Tests for parser improvements
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_multiple_statements_with_semicolons() {
    let mut engine = JSEngine::new();

    // This used to fail with "Unexpected token: Semicolon"
    let code = "let x = 1; let y = 2; let z = 3;";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Multiple statements failed: {:?}", result);
}

#[test]
fn test_function_then_usage() {
    let mut engine = JSEngine::new();

    // Function declaration followed by usage
    let code = r#"
        function add(a, b) {
            return a + b;
        }
        let result = add(2, 3);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function declaration and usage failed: {:?}", result);
}

#[test]
fn test_empty_statements() {
    let mut engine = JSEngine::new();

    // Multiple semicolons should be handled gracefully
    let code = ";;let x = 1;;";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Empty statements failed: {:?}", result);
}

#[test]
fn test_mixed_declarations() {
    let mut engine = JSEngine::new();

    // Mix of var, let, const
    let code = "var a = 1; let b = 2; const c = 3;";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Mixed declarations failed: {:?}", result);
}

#[test]
fn test_expression_statements() {
    let mut engine = JSEngine::new();

    // Expression statements with semicolons
    let code = "1 + 1; 2 * 2; 3 - 3;";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Expression statements failed: {:?}", result);
}

#[test]
fn test_block_with_statements() {
    let mut engine = JSEngine::new();

    // Block containing multiple statements
    let code = r#"
        {
            let x = 1;
            let y = 2;
            let z = x + y;
        }
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Block with statements failed: {:?}", result);
}

#[test]
fn test_parser_error_messages() {
    let mut engine = JSEngine::new();

    // Invalid syntax should give better error
    let code = "let x =";  // Incomplete
    let result = engine.execute(code);
    assert!(result.is_err(), "Should fail on incomplete statement");

    // Error should contain position information
    if let Err(msg) = result {
        assert!(msg.contains("position") || msg.contains("Parse"),
                "Error should mention position or parsing: {}", msg);
    }
}

#[test]
fn test_statement_after_return() {
    let mut engine = JSEngine::new();

    // Return in function followed by more code
    let code = r#"
        function test() {
            return 42;
        }
        let x = test();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Statement after function failed: {:?}", result);
}
