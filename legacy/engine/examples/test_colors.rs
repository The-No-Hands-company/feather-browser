// Test RGB/RGBA color parsing

use pronbrow_engine::*;

fn main() {
    println!("=== Testing RGB/RGBA Color Parsing ===\n");
    
    let mut parser = CssParser::new();
    
    // Test cases
    let test_css = vec![
        ("p { color: rgb(255, 100, 50); }", "RGB basic"),
        ("p { color: rgba(0, 128, 255, 0.5); }", "RGBA with alpha"),
        ("p { color: #f00; }", "Short hex"),
        ("p { color: #ff0000; }", "Full hex"),
        ("p { word-spacing: 10px; }", "Word spacing"),
        ("p { font-variant: small-caps; }", "Font variant"),
        ("p { white-space: pre; }", "White-space"),
    ];
    
    for (css, description) in test_css {
        print!("Testing {}: ", description);
        match parser.parse(css) {
            Ok(stylesheet) => {
                println!("✓ Parsed");
                if !stylesheet.rules.is_empty() {
                    let rule = &stylesheet.rules[0];
                    for decl in &rule.declarations {
                        println!("  - {}: {:?}", decl.name, decl.value);
                    }
                }
            }
            Err(e) => println!("✗ Failed: {}", e),
        }
        println!();
    }
    
    println!("=== All tests complete ===");
}
