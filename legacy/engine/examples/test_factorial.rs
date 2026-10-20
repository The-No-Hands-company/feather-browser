use pronbrow_engine::js::engine::JSEngine;

fn main() {
    let mut engine = JSEngine::new();
    
    // Test 1: Simple while loop - count to 5
    println!("=== Test 1: Simple while loop ===");
    engine.execute("
        var counter = 0;
        while (counter < 5) {
            counter = counter + 1
        }
    ").unwrap();
    
    let counter_value = engine.execute("counter").unwrap();
    println!("counter = {:?} (expected 5.0)", counter_value);
    
    // Test 2: While loop with calculation
    println!("\n=== Test 2: Factorial function ===");
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
    
    let result5 = engine.execute("factorial(5)").unwrap();
    println!("factorial(5) = {:?} (expected 120.0)", result5);
    
    let result4 = engine.execute("factorial(4)").unwrap();
    println!("factorial(4) = {:?} (expected 24.0)", result4);
}
