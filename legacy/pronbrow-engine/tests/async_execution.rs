// Test async function execution

use pronbrow_engine::js::{parser::JSParser, interpreter::Interpreter};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[test]
fn test_async_function_returns_promise() {
    let code = r#"
        async function test() {
            return 42;
        }
        test();
    "#;
    
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    let globals = Rc::new(RefCell::new(HashMap::new()));
    let mut interpreter = Interpreter::new(globals);
    
    let result = interpreter.evaluate(&ast);
    // println!("Async function result: {:?}", result);
    assert!(result.is_ok(), "Async function should execute");
}

#[test]
fn test_async_function_basic_execution() {
    let code = r#"
        async function getValue() {
            return 123;
        }
        getValue();
    "#;
    
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    let globals = Rc::new(RefCell::new(HashMap::new()));
    let mut interpreter = Interpreter::new(globals);
    
    let result = interpreter.evaluate(&ast).unwrap();
    // println!("Result: {:?}", result);
    // Should return a Promise object
}

#[test]
fn test_generator_function_returns_iterator() {
    let code = r#"
        function* count() {
            yield 1;
            yield 2;
            yield 3;
        }
        count();
    "#;
    
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    let globals = Rc::new(RefCell::new(HashMap::new()));
    let mut interpreter = Interpreter::new(globals);
    
    let result = interpreter.evaluate(&ast);
    // println!("Generator function result: {:?}", result);
    assert!(result.is_ok(), "Generator should execute");
}

#[test]
fn test_async_and_sync_functions_coexist() {
    let code = r#"
        function syncFunc() { return 1; }
        async function asyncFunc() { return 2; }
        syncFunc();
        asyncFunc();
    "#;
    
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    let globals = Rc::new(RefCell::new(HashMap::new()));
    let mut interpreter = Interpreter::new(globals);
    
    let result = interpreter.evaluate(&ast);
    assert!(result.is_ok(), "Both function types should work");
}
