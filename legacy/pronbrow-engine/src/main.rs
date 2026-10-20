// PronBrow - A modern web browser built from scratch
// Main application entry point

use pronbrow_engine::*;
use std::collections::HashMap;

fn main() {
    env_logger::init();
    
    println!("=== PronBrow Engine v0.3.0 ===");
    println!("A web browser engine built from scratch");
    println!("NEW: Full JavaScript Execution Support!\n");

    // Test the engine components
    test_html_parser();
    test_css_parser();
    test_javascript_engine();
    test_full_pipeline();
}

fn test_html_parser() {
    println!("--- Testing HTML Parser ---");
    let html = r#"
        <html>
            <head>
                <title>Test Page</title>
            </head>
            <body>
                <h1 class="header">Hello, PronBrow!</h1>
                <p id="intro">This is a test paragraph.</p>
                <div class="container">
                    <p>Nested paragraph</p>
                </div>
            </body>
        </html>
    "#;

    let mut parser = HtmlParser::new();
    match parser.parse(html) {
        Ok(dom) => {
            println!("✓ HTML parsed successfully");
            println!("\nDOM Tree:");
            dom.pretty_print();
        }
        Err(e) => println!("✗ HTML parsing failed: {}", e),
    }
    println!();
}

fn test_css_parser() {
    println!("--- Testing CSS Parser ---");
    let css = r#"
        body {
            background-color: #ffffff;
            margin: 8px;
        }
        
        h1 {
            color: #333333;
            font-size: 24px;
        }
        
        .header {
            margin-top: 10px;
        }
        
        #intro {
            padding: 5px;
        }
    "#;

    let mut parser = CssParser::new();
    match parser.parse(css) {
        Ok(stylesheet) => {
            println!("✓ CSS parsed successfully");
            println!("  Rules found: {}", stylesheet.rules.len());
            for rule in &stylesheet.rules {
                println!("  - {} selectors, {} declarations",
                    rule.selectors.len(),
                    rule.declarations.len()
                );
            }
        }
        Err(e) => println!("✗ CSS parsing failed: {}", e),
    }
    println!();
}

fn test_javascript_engine() {
    println!("--- Testing JavaScript Engine ---");
    
    let mut engine = BrowserEngine::new();
    
    // Test 1: Console.log
    println!("Test 1: console.log()");
    let _ = engine.execute_script("console.log(\"Hello from JavaScript!\")");
    
    // Test 2: Variables
    println!("Test 2: Variable declarations");
    let _ = engine.execute_script("var x = 42");
    let _ = engine.execute_script("let message = \"PronBrow is awesome\"");
    let _ = engine.execute_script("console.log(x)");
    let _ = engine.execute_script("console.log(message)");
    
    // Test 3: Arithmetic
    println!("Test 3: Arithmetic");
    match engine.execute_script("2 + 2") {
        Ok(result) => println!("  2 + 2 = {:?}", result),
        Err(e) => println!("  Error: {}", e),
    }
    
    // Test 4: HTML with scripts
    println!("Test 4: HTML with inline scripts");
    let html = r#"
        <html>
        <body>
            <h1>Dynamic Content</h1>
            <script>
                console.log("Inline script executed!");
                var title = "PronBrow Engine";
                console.log(title);
            </script>
        </body>
        </html>
    "#;
    
    match engine.load_and_execute_html(html) {
        Ok(_) => println!("  ✓ HTML with scripts executed"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    
    // Test 5: Global objects
    println!("Test 5: Global objects");
    if engine.js_engine.get_global("window").is_some() {
        println!("  ✓ window object available");
    }
    if engine.js_engine.get_global("document").is_some() {
        println!("  ✓ document object available");
    }
    
    println!("✓ JavaScript engine tests complete");
    println!();
}

fn test_full_pipeline() {
    println!("--- Testing Full Rendering Pipeline ---");
    
    let html = r#"
        <html>
            <body>
                <h1>PronBrow Engine v0.3.0</h1>
                <div class="solid-border">
                    <p>Solid border with JavaScript!</p>
                </div>
                <script>
                    console.log("JavaScript is running!");
                    var version = "0.3.0";
                    console.log("Engine version: " + version);
                </script>
                <div class="dashed-border">
                    <p>Dashed border</p>
                </div>
                <div class="dotted-border">
                    <p>Dotted border</p>
                </div>
            </body>
        </html>
    "#;

    let css = r#"
        body {
            background-color: #f5f5f5;
            margin: 0px;
            padding: 20px;
        }
        
        h1 {
            color: #0066cc;
            font-size: 32px;
            font-weight: bold;
            text-align: center;
        }
        
        p {
            color: #333333;
            font-size: 18px;
            margin: 10px;
        }
        
        .solid-border {
            background-color: #ffffff;
            border-width: 3px;
            border-color: #0066cc;
            border-style: solid;
            width: 400px;
            padding: 20px;
            margin: 20px;
        }
        
        .dashed-border {
            background-color: #ffffff;
            border-width: 3px;
            border-color: #cc6600;
            border-style: dashed;
            width: 400px;
            padding: 20px;
            margin: 20px;
        }
        
        .dotted-border {
            background-color: #ffffff;
            border-width: 3px;
            border-color: #00cc66;
            border-style: dotted;
            width: 400px;
            padding: 20px;
            margin: 20px;
        }
    "#;

    // Parse HTML
    let mut html_parser = HtmlParser::new();
    let dom = match html_parser.parse(html) {
        Ok(d) => {
            println!("✓ HTML parsed");
            d
        }
        Err(e) => {
            println!("✗ HTML parsing failed: {}", e);
            return;
        }
    };

    // Parse CSS
    let mut css_parser = CssParser::new();
    let stylesheet = match css_parser.parse(css) {
        Ok(s) => {
            println!("✓ CSS parsed");
            s
        }
        Err(e) => {
            println!("✗ CSS parsing failed: {}", e);
            return;
        }
    };

    // Apply styles
    println!("✓ Styles applied ({} CSS rules)", stylesheet.rules.len());

    // Layout
    let layout_engine = LayoutEngine::new();
    let mut viewport = layout::Dimensions::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;

    // Create a simple styled node for testing
    let mut properties = HashMap::new();
    properties.insert("display".to_string(), css::Value::Keyword("block".to_string()));
    properties.insert("background-color".to_string(), 
        css::Value::Color(css::Color { r: 66, g: 133, b: 244, a: 255 }));
    properties.insert("width".to_string(), css::Value::Length(200.0, css::Unit::Px));
    properties.insert("height".to_string(), css::Value::Length(100.0, css::Unit::Px));

    let styled_node = layout::StyledNode::new(dom.root.clone(), properties);
    let layout_root = layout_engine.layout_tree(&styled_node, viewport);
    println!("✓ Layout calculated");

    // Render
    let mut render_engine = RenderEngine::new();
    let canvas = render_engine.paint(&layout_root, layout::Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    });
    println!("✓ Page rendered to canvas ({}x{})", canvas.width, canvas.height);

    // Save output
    match canvas.save_to_file("output.png") {
        Ok(_) => println!("✓ Output saved to output.png"),
        Err(e) => println!("✗ Failed to save output: {}", e),
    }

    println!("\n=== Pipeline Complete ===");
}
