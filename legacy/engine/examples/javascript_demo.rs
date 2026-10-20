// JavaScript Execution Demo
use pronbrow_engine::BrowserEngine;

fn main() {
    println!("=== PronBrow JavaScript Execution Demo ===\n");
    
    let mut engine = BrowserEngine::new();
    
    // Test 1: Basic console.log
    println!("--- Test 1: Console.log ---");
    let _ = engine.execute_script("console.log(\"Hello from JavaScript!\")");
    println!();
    
    // Test 2: Variable declarations
    println!("--- Test 2: Variables ---");
    let _ = engine.execute_script("var x = 42");
    let _ = engine.execute_script("let y = \"world\"");
    let _ = engine.execute_script("const z = true");
    let _ = engine.execute_script("console.log(x)");
    let _ = engine.execute_script("console.log(y)");
    let _ = engine.execute_script("console.log(z)");
    println!();
    
    // Test 3: Arithmetic
    println!("--- Test 3: Arithmetic ---");
    let result = engine.execute_script("2 + 2");
    println!("2 + 2 = {:?}", result);
    println!();
    
    // Test 4: Boolean operations
    println!("--- Test 4: Booleans ---");
    let _ = engine.execute_script("console.log(true)");
    let _ = engine.execute_script("console.log(false)");
    println!();
    
    // Test 5: HTML with inline JavaScript
    println!("--- Test 5: HTML with Scripts ---");
    let html = r#"
        <html>
        <head>
            <title>JavaScript Test</title>
        </head>
        <body>
            <h1>JavaScript Execution</h1>
            <script>
                console.log("Script from HTML!");
                var message = "PronBrow Engine rocks!";
                console.log(message);
            </script>
            <p>Content after script</p>
            <script>
                console.log("Another script tag");
                let num = 123;
                console.log(num);
            </script>
        </body>
        </html>
    "#;
    
    match engine.load_and_execute_html(html) {
        Ok(_) => println!("✓ HTML with scripts executed successfully"),
        Err(e) => println!("✗ Error: {}", e),
    }
    println!();
    
    // Test 6: Global objects
    println!("--- Test 6: Global Objects ---");
    println!("Checking window object...");
    let window = engine.js_engine.get_global("window");
    println!("window: {:?}", window);
    
    println!("Checking document object...");
    let document = engine.js_engine.get_global("document");
    println!("document: {:?}", document);
    println!();
    
    // Test 7: Console logs
    println!("--- Test 7: All Console Logs ---");
    for (i, log) in engine.js_engine.console.logs.iter().enumerate() {
        println!("{}: {}", i + 1, log);
    }
    println!();
    
    println!("=== Demo Complete ===");
}
