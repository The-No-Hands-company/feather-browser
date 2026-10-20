// Debug test for class constructor this binding
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_function_constructor_works() {
    let mut engine = JSEngine::new();

    // Regular function constructor (should work)
    let code = r#"
        function Person(name) {
            this.name = name;
        }
        const p = new Person("Alice");
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Function constructor failed: {:?}", result);
}

#[test]
fn test_class_constructor_minimal() {
    let mut engine = JSEngine::new();

    // Minimal class test - just declaration
    let code = "class C {}";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class declaration failed: {:?}", result);
}

#[test]
fn test_class_instantiation_empty() {
    let mut engine = JSEngine::new();

    // Empty class with instantiation
    let code = "class C {} const obj = new C();";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Empty class instantiation failed: {:?}", result);
}

#[test]
fn test_class_constructor_body_no_this() {
    let mut engine = JSEngine::new();

    // Constructor with body but no 'this'
    let code = r#"
        class C {
            constructor() {
                const x = 1;
            }
        }
        const obj = new C();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class constructor without this failed: {:?}", result);
}

#[test]
fn test_class_constructor_with_this() {
    let mut engine = JSEngine::new();

    // Constructor with 'this' assignment
    let code = r#"
        class C {
            constructor() {
                this.x = 1;
            }
        }
        const obj = new C();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class constructor with this failed: {:?}", result);
}
