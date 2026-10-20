// Test async/await and generator parsing

use pronbrow_engine::js::parser::JSParser;

#[test]
fn test_async_function_declaration() {
    let code = "async function fetchData() { return await fetch(); }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    // Should parse without errors
    println!("Async function parsed: {:?}", ast);
}

#[test]
fn test_async_arrow_function() {
    let code = "const getData = async () => { return await api.call(); }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    println!("Async arrow function parsed: {:?}", ast);
}

#[test]
fn test_generator_function() {
    let code = "function* counter() { yield 1; yield 2; yield 3; }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    println!("Generator function parsed: {:?}", ast);
}

#[test]
fn test_async_generator() {
    let code = "async function* asyncGen() { yield await fetch(); }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    println!("Async generator parsed: {:?}", ast);
}

#[test]
fn test_yield_delegate() {
    let code = "function* delegator() { yield* otherGenerator(); }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    println!("Yield delegate parsed: {:?}", ast);
}

#[test]
fn test_await_in_expression() {
    let code = "async function test() { const x = await Promise.resolve(42); }";
    let mut parser = JSParser::new(code);
    let ast = parser.parse().unwrap();
    
    println!("Await expression parsed: {:?}", ast);
}
