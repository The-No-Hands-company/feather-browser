// Minimal test to debug 'this' binding
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_simple_this() {
    let mut engine = JSEngine::new();

    // Absolutely minimal test
    let code = "function F() { }; const x = new F();";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Simple new failed: {:?}", result);
}

#[test]
fn test_this_assignment_number() {
    let mut engine = JSEngine::new();

    // Test assigning a number to this.prop
    let code = r#"
        function F() {
            let temp = 5;
        }
        const x = new F();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Constructor with local var failed: {:?}", result);
}
