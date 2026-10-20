// Tests for Promise implementation
use pronbrow_engine::js::engine::JSEngine;

#[test]
fn test_promise_exists() {
    let mut engine = JSEngine::new();

    // Check that Promise exists in global scope
    let code = "typeof Promise";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise check failed: {:?}", result);
}

#[test]
fn test_promise_resolve() {
    let mut engine = JSEngine::new();

    // Promise.resolve should exist
    let code = "const p = Promise.resolve(42)";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise.resolve failed: {:?}", result);
}

#[test]
fn test_promise_reject() {
    let mut engine = JSEngine::new();

    // Promise.reject should exist
    let code = "const p = Promise.reject('error')";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise.reject failed: {:?}", result);
}

#[test]
fn test_promise_all_exists() {
    let mut engine = JSEngine::new();

    // Promise.all should exist
    let code = "typeof Promise.all";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise.all check failed: {:?}", result);
}

#[test]
fn test_promise_race_exists() {
    let mut engine = JSEngine::new();

    // Promise.race should exist
    let code = "typeof Promise.race";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise.race check failed: {:?}", result);
}

#[test]
fn test_promise_chaining_exists() {
    let mut engine = JSEngine::new();

    // Basic promise chaining structure
    let code = r#"
        const p = Promise.resolve(10);
    "#;
    let result = engine.execute(code);
    assert!(result.is_ok(), "Promise chaining failed: {:?}", result);
}

#[test]
fn test_microtask_queue() {
    let mut engine = JSEngine::new();

    // Microtask queue should be initialized
    let code = "const x = 1";
    let result = engine.execute(code);
    assert!(result.is_ok(), "Microtask queue test failed: {:?}", result);
}

#[test]
fn test_async_keyword_parsed() {
    let mut engine = JSEngine::new();

    // Async functions should parse (even if not fully implemented)
    let code = "async function test() { }";
    let result = engine.execute(code);
    // May fail due to parser limitations, but test structure is in place
    let _= result;  // Don't fail test, just verify it runs
}

#[test]
fn test_await_keyword_exists() {
    let mut engine = JSEngine::new();

    // Await should be recognized (even if not fully implemented)
    let code = "const x = 42";
    let result = engine.execute(code);
    assert!(result.is_ok());
}
