use pronbrow_engine::js::engine::JSEngine;

fn main() {
    let mut engine = JSEngine::new();
    
    // Test 1: Simple while loop
    engine.execute("
        var counter = 0;
        while (counter < 5) {
            counter = counter + 1
        }
    ").unwrap();
    
    println!("After Test 1:");
    println!("  counter = {:?}", engine.execute("counter").unwrap());
    println!("  result = {:?}", engine.execute("result").unwrap());
    println!("  i = {:?}", engine.execute("i").unwrap());
    
    // Now define factorial
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
    
    println!("\nAfter defining factorial:");
    println!("  counter = {:?}", engine.execute("counter").unwrap());
    println!("  result = {:?}", engine.execute("result").unwrap());
    println!("  i = {:?}", engine.execute("i").unwrap());
    
    println!("\nCalling factorial(5)...");
    let result = engine.execute("factorial(5)").unwrap();
    println!("factorial(5) = {:?}", result);
    
    println!("\nAfter calling factorial:");
    println!("  counter = {:?}", engine.execute("counter").unwrap());
    println!("  result = {:?}", engine.execute("result").unwrap());
    println!("  i = {:?}", engine.execute("i").unwrap());
}
