// Simple test program to debug JavaScript execution
use pronbrow_engine::js::engine::JSEngine;

fn main() {
    let mut engine = JSEngine::new();
    
    println!("=== Testing Simple Expressions ===");
    match engine.execute("2 + 3") {
        Ok(val) => println!("2 + 3 = {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    
    println!("\n=== Testing String Concatenation ===");
    match engine.execute("'Hello' + ' ' + 'World'") {
        Ok(val) => println!("Result: {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    
    println!("\n=== Testing Variable Assignment ===");
    match engine.execute("var x = 5") {
        Ok(val) => println!("var x = 5 -> {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    match engine.execute("x = x + 1") {
        Ok(val) => println!("x = x + 1 -> {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    match engine.execute("x") {
        Ok(val) => println!("x = {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    
    println!("\n=== Testing If Statement ===");
    engine.execute("var result = 0").unwrap();
    println!("result initialized to 0");
    match engine.execute("if (true) { result = 1 }") {
        Ok(val) => println!("if (true) {{ result = 1 }} -> {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    match engine.execute("result") {
        Ok(val) => println!("result = {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
    
    println!("\n=== Testing Function Returning If Result ===");
    match engine.execute("
        function checkNumber(n) {
            if (n > 10) {
                return 'big'
            } else {
                return 'small'
            }
        }
    ") {
        Ok(_) => println!("Function defined"),
        Err(e) => println!("ERROR defining function: {}", e),
    }
    
    match engine.execute("checkNumber(15)") {
        Ok(val) => println!("checkNumber(15) = {:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
}
