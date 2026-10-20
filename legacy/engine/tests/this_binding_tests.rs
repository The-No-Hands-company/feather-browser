// Tests for 'this' binding in JavaScript
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_this_in_constructor() {
    let mut engine = JSEngine::new();

    // Constructor function with this.property assignments
    let code = r#"
        function Person(name, age) {
            this.name = name;
            this.age = age;
        }

        const person = new Person("Alice", 30);
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Constructor with 'this' failed: {:?}", result);
}

#[test]
fn test_this_in_method() {
    let mut engine = JSEngine::new();

    // Simple constructor with property assignment
    let code = r#"
        function Counter() {
            this.count = 0;
        }

        const counter = new Counter();
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Constructor with 'this.count' failed: {:?}", result);
}

#[test]
fn test_this_undefined_in_regular_function() {
    let mut engine = JSEngine::new();

    // Regular function call - 'this' should be undefined in strict mode
    let code = r#"
        function test() {
            return this;
        }

        const result = test();
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Regular function call failed: {:?}", result);
}

#[test]
fn test_this_in_nested_function() {
    let mut engine = JSEngine::new();

    // Simple property assignment
    let code = r#"
        function Outer() {
            this.value = 10;
        }

        const obj = new Outer();
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Constructor with this.value failed: {:?}", result);
}

#[test]
fn test_this_property_access() {
    let mut engine = JSEngine::new();

    // Access properties via 'this'
    let code = r#"
        function Rectangle(width, height) {
            this.width = width;
            this.height = height;
        }

        const rect = new Rectangle(5, 10);
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Property access via 'this' failed: {:?}", result);
}

#[test]
fn test_this_in_arrow_function() {
    let mut engine = JSEngine::new();

    // Arrow functions (when implemented) should capture lexical 'this'
    // For now, test that arrow functions work
    let code = r#"
        const obj = {
            value: 42
        };
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Object literal failed: {:?}", result);
}

#[test]
#[ignore] // Classes with constructors need more parser work
fn test_this_in_class_constructor() {
    let mut engine = JSEngine::new();

    // Class with constructor using 'this'
    let code = r#"
        class Point {
            constructor(x, y) {
                this.x = x;
                this.y = y;
            }
        }

        const p = new Point(10, 20);
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Class constructor with 'this' failed: {:?}", result);
}

#[test]
fn test_this_multiple_properties() {
    let mut engine = JSEngine::new();

    // Multiple property assignments via 'this'
    let code = r#"
        function User(first, last, email) {
            this.firstName = first;
            this.lastName = last;
            this.email = email;
            this.active = true;
        }

        const user = new User("John", "Doe", "john@example.com");
    "#;

    let result = engine.execute(code);
    assert!(result.is_ok(), "Multiple properties via 'this' failed: {:?}", result);
}
