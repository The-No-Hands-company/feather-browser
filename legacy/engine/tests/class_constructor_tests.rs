// Tests for class constructor support
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_basic_class_declaration() {
    let mut engine = JSEngine::new();

    // Basic class with constructor
    let code = r#"
        class Person {
            constructor(name) {
                this.name = name;
            }
        }
        const p = new Person("Alice");
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Basic class declaration failed: {:?}", result);
}

#[test]
fn test_class_without_constructor() {
    let mut engine = JSEngine::new();

    // Class without explicit constructor
    let code = r#"
        class Empty {
        }
        const e = new Empty();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class without constructor failed: {:?}", result);
}

#[test]
fn test_class_with_methods() {
    let mut engine = JSEngine::new();

    // Class with constructor and methods
    let code = r#"
        class Calculator {
            constructor(initial) {
                this.value = initial;
            }
            add(n) {
                return this.value + n;
            }
        }
        const calc = new Calculator(10);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class with methods failed: {:?}", result);
}

#[test]
fn test_class_multiple_properties() {
    let mut engine = JSEngine::new();

    // Class with multiple properties
    let code = r#"
        class Point {
            constructor(x, y) {
                this.x = x;
                this.y = y;
            }
        }
        const point = new Point(3, 4);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class with multiple properties failed: {:?}", result);
}

#[test]
fn test_class_simple_constructor() {
    let mut engine = JSEngine::new();

    // Simpler class test
    let code = "class C { constructor() { this.value = 42; } } const obj = new C();";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Simple class constructor failed: {:?}", result);
}

#[test]
fn test_multiple_class_instances() {
    let mut engine = JSEngine::new();

    // Multiple instances of same class
    let code = r#"
        class Counter {
            constructor(start) {
                this.count = start;
            }
        }
        const c1 = new Counter(0);
        const c2 = new Counter(100);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Multiple class instances failed: {:?}", result);
}

#[test]
fn test_class_with_default_constructor() {
    let mut engine = JSEngine::new();

    // Class that should get default constructor
    let code = r#"
        class Empty {}
        const obj = new Empty();
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Class with default constructor failed: {:?}", result);
}
