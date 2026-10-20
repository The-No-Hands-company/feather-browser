# PronBrow Engine Architecture

## Overview

PronBrow is a complete web browser engine built from the ground up in Rust. This document details the internal architecture and design decisions.

## High-Level Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser Engine                            │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Network    │→ │     HTML     │→ │     DOM      │          │
│  │   Engine     │  │    Parser    │  │   Builder    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         ↓                                     ↓                   │
│  ┌──────────────┐                    ┌──────────────┐          │
│  │     CSS      │                    │    Style     │          │
│  │    Parser    │→───────────────────│  Calculator  │          │
│  └──────────────┘                    └──────────────┘          │
│                                              ↓                   │
│                                      ┌──────────────┐          │
│                                      │   Layout     │          │
│                                      │   Engine     │          │
│                                      └──────────────┘          │
│                                              ↓                   │
│                                      ┌──────────────┐          │
│                                      │   Render     │          │
│                                      │   Engine     │          │
│                                      └──────────────┘          │
│                                              ↓                   │
│                                      ┌──────────────┐          │
│                                      │   Display    │          │
│                                      └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Network Engine

**Location**: `src/network/mod.rs`

**Purpose**: Fetches resources from the web

**Key Components**:
- HTTP/HTTPS client using `reqwest`
- URL parsing and normalization
- Request/response handling
- Future: Caching, HTTP/2, HTTP/3

**API**:
```rust
pub struct NetworkEngine {
    client: reqwest::blocking::Client,
}

impl NetworkEngine {
    pub async fn fetch(&self, url: &str) -> Result<String, Box<dyn Error>>
    pub fn fetch_blocking(&self, url: &str) -> Result<String, Box<dyn Error>>
    pub fn parse_url(&self, input: &str) -> String
}
```

**Flow**:
1. User enters URL or search query
2. NetworkEngine.parse_url() normalizes input
3. fetch() retrieves the resource
4. Returns HTML/CSS/JS as string

### 2. HTML Parser

**Location**: `src/html/mod.rs`

**Purpose**: Converts HTML text into a DOM tree

**Algorithm**: Recursive descent parser

**Key Components**:
```rust
pub struct HtmlParser {
    pos: usize,      // Current position in input
    input: String,   // HTML source
}
```

**Parsing Process**:
1. Tokenize HTML (implicit)
2. Build tree structure
3. Handle nested elements
4. Parse attributes
5. Create text nodes

**Features**:
- Self-closing tags
- Attribute parsing with single/double quotes
- Whitespace handling
- Comment support

**Example**:
```html
<div class="container">
  <h1>Title</h1>
</div>
```

Becomes:
```
Element("div", {class: "container"})
  ├─ Element("h1")
  │   └─ Text("Title")
```

### 3. DOM (Document Object Model)

**Location**: `src/dom/mod.rs`

**Purpose**: Tree structure representing the document

**Core Types**:
```rust
pub enum NodeType {
    Element(ElementData),    // <div>, <p>, etc.
    Text(String),           // Text content
    Comment(String),        // <!-- comments -->
}

pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

pub struct Dom {
    pub root: Node,
}
```

**Operations**:
- Tree traversal
- Element lookup by tag name
- Attribute access
- Pretty printing for debugging

**Properties**:
- Immutable once built (for now)
- Future: Dynamic DOM manipulation via JavaScript

### 4. CSS Parser

**Location**: `src/css/mod.rs`

**Purpose**: Parses CSS and manages styles

**Core Types**:
```rust
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

pub struct Selector {
    pub tag_name: Option<String>,    // div
    pub id: Option<String>,          // #myid
    pub classes: Vec<String>,        // .class1 .class2
}

pub struct Declaration {
    pub name: String,      // "color"
    pub value: Value,      // #ff0000
}

pub enum Value {
    Keyword(String),                    // "block", "auto"
    Length(f32, Unit),                  // 10px, 2em
    Color(Color),                       // #ff0000
}
```

**Specificity Calculation**:
```rust
impl Selector {
    pub fn specificity(&self) -> (usize, usize, usize) {
        // (ID count, class count, tag count)
        // Example: #id.class1.class2 div = (1, 2, 1)
    }
}
```

**User Agent Stylesheet**:
```css
html { display: block; }
body { display: block; margin: 8px; }
h1 { display: block; font-size: 2em; font-weight: bold; }
```

### 5. Style Computation

**Integrated in**: `src/layout/mod.rs`

**Purpose**: Match selectors to DOM nodes and compute final styles

**Process**:
1. For each DOM node
2. Find matching CSS rules
3. Sort by specificity
4. Cascade and inherit values
5. Compute final property values

**Result**:
```rust
pub struct StyledNode {
    pub node: Node,
    pub properties: PropertyMap,  // HashMap<String, Value>
    pub children: Vec<StyledNode>,
}
```

### 6. Layout Engine

**Location**: `src/layout/mod.rs`

**Purpose**: Calculate positions and sizes (Box Model)

**Box Model**:
```
┌─────────────────────────────────────┐
│           Margin (transparent)       │
│  ┌───────────────────────────────┐  │
│  │      Border                    │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │     Padding             │  │  │
│  │  │  ┌───────────────────┐  │  │  │
│  │  │  │    Content        │  │  │  │
│  │  │  │   (text/children) │  │  │  │
│  │  │  └───────────────────┘  │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

**Data Structure**:
```rust
pub struct Dimensions {
    pub content: Rect,           // Content box
    pub padding: EdgeSizes,      // Padding
    pub border: EdgeSizes,       // Border
    pub margin: EdgeSizes,       // Margin
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}
```

**Layout Types**:
- **Block Layout**: Stacks vertically, takes full width
- **Inline Layout**: Flows horizontally, wraps at line breaks
- **Anonymous Blocks**: Created for inline content in block containers

**Layout Algorithm**:
1. Calculate width (top-down)
2. Calculate position (top-down)
3. Layout children (recursive)
4. Calculate height (bottom-up)

### 7. Render Engine

**Location**: `src/render/mod.rs`

**Purpose**: Paint the layout tree to pixels

**Pipeline**:
```
Layout Tree → Display List → Rasterization → Pixels
```

**Display Commands**:
```rust
pub enum DisplayCommand {
    SolidColor(Color, Rect),
    // Future: Text, Image, Gradient, etc.
}
```

**Canvas**:
```rust
pub struct Canvas {
    pub pixels: Vec<Color>,  // RGBA buffer
    pub width: usize,
    pub height: usize,
}
```

**Rendering Process**:
1. Walk layout tree
2. Generate display commands
3. Paint each command to canvas
4. Export to PNG (or display on screen)

**Future Optimizations**:
- Layer compositing
- GPU acceleration
- Dirty rectangle tracking
- SIMD pixel operations

### 8. JavaScript Engine

**Location**: `src/js/mod.rs`

**Status**: Placeholder (in development)

**Planned Features**:
- Lexer & Parser (AST generation)
- Bytecode compiler
- Virtual machine
- Garbage collector
- DOM API bindings

## Data Flow

### Complete Page Load Flow

```
1. User Input
   ↓
2. URL Normalization (Network Engine)
   ↓
3. HTTP Request (Network Engine)
   ↓
4. HTML Response
   ↓
5. HTML Parsing (HTML Parser)
   ↓
6. DOM Tree (DOM)
   ↓
7. CSS Fetching (Network Engine)
   ↓
8. CSS Parsing (CSS Parser)
   ↓
9. Style Matching & Computation
   ↓
10. Styled Tree (StyledNode)
    ↓
11. Layout Calculation (Layout Engine)
    ↓
12. Layout Tree (LayoutBox)
    ↓
13. Display List Generation (Render Engine)
    ↓
14. Rasterization (Render Engine)
    ↓
15. Canvas/Screen Output
```

## Performance Considerations

### Memory Management

**Ownership Model** (Rust):
- DOM nodes own their children
- Layout boxes reference styled nodes
- No reference counting needed
- Zero-cost abstractions

### Optimizations

**Current**:
- Single-threaded (simple, correct)
- Stack-based parsing
- Minimal allocations

**Future**:
- Parallel CSS parsing
- Parallel layout (subtrees)
- GPU rasterization
- Incremental rendering
- JIT compilation for JS

## Security Architecture

### Planned Security Features

1. **Sandboxing**:
   - Process isolation
   - Separate renderer processes
   - Restricted system calls

2. **CORS**:
   - Cross-Origin Resource Sharing
   - Same-origin policy enforcement

3. **CSP**:
   - Content Security Policy
   - Script/style whitelisting

4. **TLS/SSL**:
   - Certificate validation
   - HTTPS enforcement

## Testing Strategy

### Unit Tests

Each component has unit tests:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_html_parsing() { ... }
    
    #[test]
    fn test_css_specificity() { ... }
}
```

### Integration Tests

Full pipeline tests in `main.rs`:
- HTML → DOM
- CSS → Stylesheet
- Layout calculation
- Rendering output

### Future Tests

- Conformance tests (HTML5, CSS3)
- Performance benchmarks
- Fuzz testing
- Visual regression tests

## Extensibility

### Plugin Architecture (Future)

```rust
pub trait Extension {
    fn on_dom_ready(&mut self, dom: &Dom);
    fn on_render(&mut self, canvas: &Canvas);
}
```

### Custom Elements (Future)

```rust
engine.register_element("custom-widget", CustomWidget::new());
```

## Browser Features Comparison

| Feature | Chrome | Firefox | Safari | PronBrow |
|---------|--------|---------|--------|----------|
| HTML5 | ✅ | ✅ | ✅ | 🔨 Basic |
| CSS3 | ✅ | ✅ | ✅ | 🔨 Basic |
| JavaScript | ✅ | ✅ | ✅ | ❌ Planned |
| WebGL | ✅ | ✅ | ✅ | ❌ Future |
| WebRTC | ✅ | ✅ | ✅ | ❌ Future |
| Service Workers | ✅ | ✅ | ✅ | ❌ Future |
| GPU Acceleration | ✅ | ✅ | ✅ | ❌ Future |

Legend: ✅ Full support, 🔨 In progress, ❌ Not implemented

## Code Statistics

**Current (v0.1.0)**:
- Total Lines: ~1,500
- Modules: 7
- Dependencies: 8
- Rust Edition: 2021

## Future Roadmap

### Short Term (1-3 months)
- Text rendering with font support
- Image loading (PNG, JPEG)
- Basic JavaScript interpreter
- CSS Flexbox

### Medium Term (3-6 months)
- Full JavaScript engine
- Canvas API
- WebGL basics
- Multi-threading

### Long Term (6-12 months)
- Production-ready security
- Full Web API support
- Developer tools
- Mobile version

## Resources & References

- **HTML Spec**: https://html.spec.whatwg.org/
- **CSS Spec**: https://www.w3.org/Style/CSS/
- **DOM Spec**: https://dom.spec.whatwg.org/
- **Servo**: https://servo.org/
- **Chromium**: https://www.chromium.org/

---

Last Updated: 2024-11-12
