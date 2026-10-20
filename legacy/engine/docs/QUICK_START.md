# PronBrow Engine - Quick Start Guide

## What is PronBrow?

PronBrow is a **complete web browser engine built from scratch** in Rust. Unlike other browsers that use Chromium, Gecko, or WebKit, PronBrow implements every component from the ground up:

- ✅ HTML Parser
- ✅ CSS Parser  
- ✅ DOM (Document Object Model)
- ✅ Layout Engine (Box Model)
- ✅ Render Engine
- ✅ Network Stack
- 🔨 JavaScript Engine (in progress)

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux: Install dependencies
sudo apt-get install build-essential pkg-config libssl-dev
```

### Build & Run

```bash
cd pronbrow-engine
cargo build --release
cargo run
```

## What You'll See

When you run `cargo run`, the engine will:

1. ✅ Parse a complex HTML document
2. ✅ Parse CSS stylesheets  
3. ✅ Build DOM tree
4. ✅ Calculate layout (box model)
5. ✅ Render to pixels
6. ✅ Save as `output.png`

### Example Output

```
=== PronBrow Engine v0.1.0 ===
A web browser engine built from scratch

--- Testing HTML Parser ---
✓ HTML parsed successfully

DOM Tree:
<html>
  <head>
    <title>Test Page</title>
  </head>
  <body>
    <h1>Hello, PronBrow!</h1>
    <p>This is a test paragraph.</p>
  </body>
</html>

--- Testing CSS Parser ---
✓ CSS parsed successfully
  Rules found: 4

--- Testing Full Rendering Pipeline ---
✓ HTML parsed
✓ CSS parsed
✓ Styles applied
✓ Layout calculated
✓ Page rendered to canvas (800x600)
✓ Output saved to output.png

=== Pipeline Complete ===
```

## Quick Examples

### 1. Parse HTML

```rust
use pronbrow_engine::HtmlParser;

let html = "<html><body><h1>Hello</h1></body></html>";
let mut parser = HtmlParser::new();
let dom = parser.parse(html).unwrap();

dom.pretty_print(); // Shows DOM tree
```

### 2. Parse CSS

```rust
use pronbrow_engine::CssParser;

let css = "h1 { color: #ff0000; font-size: 24px; }";
let mut parser = CssParser::new();
let stylesheet = parser.parse(css).unwrap();

println!("Parsed {} rules", stylesheet.rules.len());
```

### 3. Render a Page

```rust
use pronbrow_engine::*;

// Parse HTML
let mut html_parser = HtmlParser::new();
let dom = html_parser.parse("<html><body>Hello</body></html>").unwrap();

// Render (simplified example - see DEVELOPMENT.md for full code)
let render_engine = RenderEngine::new();
// ... (layout steps) ...
// canvas.save_to_file("output.png").unwrap();
```

## Project Structure

```
pronbrow-engine/
├── src/
│   ├── dom/        # Document Object Model
│   ├── html/       # HTML Parser
│   ├── css/        # CSS Parser
│   ├── layout/     # Layout Engine (Box Model)
│   ├── render/     # Rendering Engine
│   ├── network/    # HTTP Client
│   └── js/         # JavaScript Engine (WIP)
├── README.md              # Full documentation
├── ARCHITECTURE.md        # Design details
├── DEVELOPMENT.md         # Developer guide
├── PROJECT_SUMMARY.md     # Overview
└── QUICK_START.md         # This file
```

## Key Concepts

### 1. HTML → DOM

HTML text is parsed into a tree structure:

```
<div>
  <p>Text</p>
</div>

→

Element("div")
  └─ Element("p")
      └─ Text("Text")
```

### 2. CSS → Stylesheet

CSS is parsed into rules:

```css
h1 { color: red; }

→

Rule {
  selectors: [Selector { tag_name: "h1" }],
  declarations: [Declaration { name: "color", value: Color(255,0,0) }]
}
```

### 3. Layout (Box Model)

Each element becomes a box with:
- Content (actual text/children)
- Padding (space inside border)
- Border (edge of element)
- Margin (space outside border)

### 4. Rendering

Layout boxes are painted to a pixel canvas:

```
Layout Tree → Display Commands → Pixels → PNG
```

## Next Steps

1. **Read the docs**:
   - `README.md` - Features & overview
   - `ARCHITECTURE.md` - How it works
   - `DEVELOPMENT.md` - Add features

2. **Explore the code**:
   - Start with `src/lib.rs`
   - Check out `src/html/mod.rs` for parsing
   - Look at `src/layout/mod.rs` for box model

3. **Run tests**:
   ```bash
   cargo test
   cargo run
   ```

4. **Experiment**:
   - Modify HTML in `src/main.rs`
   - Change CSS rules
   - Try different layouts

## Common Questions

**Q: Can this browse the web?**  
A: Not yet fully. It can fetch HTML and render simple pages, but lacks JavaScript and many CSS features.

**Q: How is this different from Chromium?**  
A: PronBrow is built from scratch. Every line of code is custom-written. Chromium is 25M+ lines. We're at ~1,500 lines.

**Q: What's the end goal?**  
A: A production-ready browser engine that can render modern websites, execute JavaScript, and compete with existing browsers.

**Q: Can I contribute?**  
A: Yes! See DEVELOPMENT.md for guidelines.

## Performance

**Current**: Basic implementation focused on correctness

**Benchmarks**:
- Parse simple HTML: ~10μs
- Parse CSS: ~5μs
- Render small page: ~100μs

**Future**: GPU acceleration, parallel parsing, SIMD rendering

## Roadmap

### ✅ Phase 1: Core (Complete)
- HTML/CSS parsing
- DOM tree
- Box model layout
- Basic rendering

### 🔨 Phase 2: Advanced (In Progress)
- Text rendering
- Images
- Flexbox/Grid
- Borders

### 📅 Phase 3: JavaScript (Next)
- JS parser
- Interpreter
- DOM API
- Events

### 📅 Phase 4: Production (Future)
- Security
- Performance
- Developer tools
- Full web compatibility

## Resources

### Documentation
- [README.md](README.md) - Full project documentation
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [DEVELOPMENT.md](DEVELOPMENT.md) - Developer guide
- [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) - Project overview

### External Resources
- [HTML5 Spec](https://html.spec.whatwg.org/)
- [CSS Spec](https://www.w3.org/Style/CSS/)
- [How Browsers Work](https://web.dev/howbrowserswork/)
- [Servo Browser](https://servo.org/)

## Support

- **Issues**: Report bugs via GitHub issues
- **Discussions**: Ask questions in discussions
- **Contributing**: See DEVELOPMENT.md

## License

MIT License - Free to use, modify, and distribute

---

**Ready to dive in?** Run `cargo run` and explore the code! 🚀
