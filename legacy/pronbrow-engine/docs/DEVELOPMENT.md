# Development Guide - PronBrow Engine

## Getting Started

### Prerequisites

1. **Rust Toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **System Dependencies** (Linux)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libssl-dev
   ```

### Clone & Build

```bash
git clone <repository-url>
cd pronbrow-engine
cargo build
```

### Run Tests

```bash
cargo test
cargo run
```

## Project Structure

```
pronbrow-engine/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Binary entry point (demos)
│   ├── dom/
│   │   └── mod.rs          # DOM implementation
│   ├── html/
│   │   └── mod.rs          # HTML parser
│   ├── css/
│   │   └── mod.rs          # CSS parser
│   ├── layout/
│   │   └── mod.rs          # Layout engine
│   ├── render/
│   │   └── mod.rs          # Render engine
│   ├── network/
│   │   └── mod.rs          # Network stack
│   └── js/
│       └── mod.rs          # JavaScript engine (WIP)
├── Cargo.toml              # Project manifest
├── README.md               # User documentation
├── ARCHITECTURE.md         # Architecture details
└── DEVELOPMENT.md          # This file
```

## Usage Examples

### Example 1: Parse HTML

```rust
use pronbrow_engine::HtmlParser;

fn main() {
    let html = r#"
        <html>
            <body>
                <h1>Hello World</h1>
                <p>This is a paragraph.</p>
            </body>
        </html>
    "#;

    let mut parser = HtmlParser::new();
    let dom = parser.parse(html).unwrap();
    
    // Print the DOM tree
    dom.pretty_print();
}
```

### Example 2: Parse CSS

```rust
use pronbrow_engine::CssParser;

fn main() {
    let css = r#"
        body {
            background-color: #ffffff;
            margin: 0px;
        }
        
        h1 {
            color: #333333;
            font-size: 24px;
        }
    "#;

    let mut parser = CssParser::new();
    let stylesheet = parser.parse(css).unwrap();
    
    println!("Parsed {} rules", stylesheet.rules.len());
}
```

### Example 3: Full Rendering Pipeline

```rust
use pronbrow_engine::*;
use std::collections::HashMap;

fn main() {
    // 1. Parse HTML
    let html = "<html><body><div>Hello</div></body></html>";
    let mut html_parser = HtmlParser::new();
    let dom = html_parser.parse(html).unwrap();

    // 2. Parse CSS
    let css = "div { background-color: #4285f4; width: 200px; }";
    let mut css_parser = CssParser::new();
    let stylesheet = css_parser.parse(css).unwrap();

    // 3. Create styled tree (simplified)
    let mut properties = HashMap::new();
    properties.insert("display".to_string(), 
        css::Value::Keyword("block".to_string()));
    properties.insert("background-color".to_string(),
        css::Value::Color(css::Color { r: 66, g: 133, b: 244, a: 255 }));
    
    let styled_node = layout::StyledNode::new(dom.root, properties);

    // 4. Layout
    let layout_engine = LayoutEngine::new();
    let mut viewport = layout::Dimensions::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;
    
    let layout_root = layout_engine.layout_tree(&styled_node, viewport);

    // 5. Render
    let render_engine = RenderEngine::new();
    let canvas = render_engine.paint(&layout_root, layout::Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    });

    // 6. Save output
    canvas.save_to_file("output.png").unwrap();
    println!("Rendered to output.png");
}
```

### Example 4: Fetch and Render URL

```rust
use pronbrow_engine::*;

#[tokio::main]
async fn main() {
    let url = "https://example.com";
    
    // Fetch HTML
    let network = NetworkEngine::new();
    let html = network.fetch(url).await.unwrap();
    
    // Parse and render
    let mut parser = HtmlParser::new();
    let dom = parser.parse(&html).unwrap();
    
    println!("Loaded {} from {}", dom.root.get_tag_name().unwrap(), url);
}
```

## Adding New Features

### Adding a CSS Property

1. **Add to Value enum** (`src/css/mod.rs`):
```rust
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(Color),
    // Add new type
    FontWeight(u16),  // Example
}
```

2. **Update parser** (`src/css/mod.rs`):
```rust
fn parse_value(&mut self) -> Value {
    // Add parsing logic
    if self.starts_with("bold") {
        Value::FontWeight(700)
    }
}
```

3. **Use in layout** (`src/layout/mod.rs`):
```rust
fn apply_font_weight(&self, styled_node: &StyledNode) {
    if let Some(Value::FontWeight(weight)) = styled_node.value("font-weight") {
        // Apply weight
    }
}
```

### Adding a Layout Mode

1. **Add to Display enum** (`src/layout/mod.rs`):
```rust
pub enum Display {
    Block,
    Inline,
    Flex,  // New
    None,
}
```

2. **Implement layout algorithm**:
```rust
fn layout_flex(&mut self, containing_block: &mut Dimensions) {
    // Flexbox layout logic
}
```

3. **Update layout dispatcher**:
```rust
pub fn layout(&mut self, containing_block: &mut Dimensions) {
    match self.box_type {
        BoxType::BlockNode(_) => self.layout_block(containing_block),
        BoxType::FlexNode(_) => self.layout_flex(containing_block),
        // ...
    }
}
```

## Testing

### Unit Tests

Add tests in the module file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        let mut parser = CssParser::new();
        parser.input = "#ff0000".to_string();
        parser.pos = 0;
        
        let color = parser.parse_color();
        assert_eq!(color, Value::Color(Color { 
            r: 255, g: 0, b: 0, a: 255 
        }));
    }
}
```

### Integration Tests

Create `tests/integration.rs`:

```rust
use pronbrow_engine::*;

#[test]
fn test_full_pipeline() {
    let html = "<html><body><div></div></body></html>";
    let mut parser = HtmlParser::new();
    let dom = parser.parse(html).unwrap();
    
    assert_eq!(dom.root.get_tag_name(), Some("html"));
}
```

### Run Tests

```bash
# All tests
cargo test

# Specific module
cargo test html::tests

# With output
cargo test -- --nocapture

# Single test
cargo test test_parse_color
```

## Debugging

### Enable Logging

```rust
env_logger::init();
log::info!("Parsing HTML");
log::debug!("Current position: {}", pos);
```

Run with:
```bash
RUST_LOG=debug cargo run
```

### Print DOM

```rust
dom.pretty_print();
```

### Debug Layout

```rust
println!("Dimensions: {:?}", layout_box.dimensions);
```

## Performance Profiling

### Flamegraph

```bash
cargo install flamegraph
sudo cargo flamegraph
```

### Benchmarks

Create `benches/parser.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pronbrow_engine::HtmlParser;

fn parse_benchmark(c: &mut Criterion) {
    let html = "<html><body><div>Test</div></body></html>";
    
    c.bench_function("parse html", |b| {
        b.iter(|| {
            let mut parser = HtmlParser::new();
            parser.parse(black_box(html))
        })
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "parser"
harness = false
```

Run:
```bash
cargo bench
```

## Common Tasks

### Add Dependency

```bash
cargo add <crate-name>
# Example: cargo add serde --features derive
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy
```

### Check Without Building

```bash
cargo check
```

### Build Release

```bash
cargo build --release
./target/release/pronbrow
```

## Contributing Guidelines

1. **Code Style**:
   - Run `cargo fmt` before committing
   - Follow Rust naming conventions
   - Add documentation comments

2. **Testing**:
   - Add tests for new features
   - Ensure all tests pass
   - Update integration tests

3. **Documentation**:
   - Update README.md for user-facing changes
   - Update ARCHITECTURE.md for design changes
   - Add inline documentation

4. **Commits**:
   - Clear commit messages
   - One feature per commit
   - Reference issues when applicable

## Roadmap Tasks

### High Priority
- [ ] Text rendering (use `rusttype` or `fontdue`)
- [ ] Image loading (PNG, JPEG)
- [ ] CSS selector matching
- [ ] Basic JavaScript interpreter

### Medium Priority
- [ ] CSS Flexbox
- [ ] CSS Grid
- [ ] Event handling
- [ ] Form elements

### Low Priority
- [ ] Animations
- [ ] WebGL
- [ ] Service Workers
- [ ] WebAssembly

## Resources

### Learning Resources
- [Let's Build a Browser Engine](https://limpet.net/mbrubeck/2014/08/08/toy-layout-engine-1.html)
- [HTML Spec](https://html.spec.whatwg.org/)
- [CSS Spec](https://www.w3.org/Style/CSS/)
- [Servo Documentation](https://doc.servo.org/)

### Tools
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)

### Similar Projects
- Servo (Mozilla's Rust browser)
- Robinson (Toy browser by Matt Brubeck)
- WeasyPrint (Python PDF renderer)

## FAQ

**Q: Why build from scratch instead of using Chromium?**  
A: Educational purposes and full control over architecture.

**Q: Will this be production-ready?**  
A: Eventually, but it's primarily an educational project.

**Q: What about JavaScript?**  
A: Implementing a custom interpreter, starting with ES5 basics.

**Q: Can I use this in my project?**  
A: Yes, it's MIT licensed, but it's not production-ready yet.

## Support

- Open an issue for bugs
- Discussions for questions
- PRs welcome!

---

Happy hacking! 🚀
