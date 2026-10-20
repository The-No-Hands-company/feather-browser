# PronBrow Resource Optimization Strategy

## Executive Summary

**Goal:** Create a bloat-free browser with dual modes:
- **GUI Mode:** Fast, low-memory desktop browser experience
- **CLI Mode:** Terminal browser with near-zero footprint

**Current Status:** 33,000+ LOC, full feature set but needs optimization
**Target:** <50MB RAM (CLI), <200MB RAM (GUI), <10MB binary

---

## 🎯 Core Optimization Principles

### 1. **Lazy Loading Everything**
- Load modules only when needed
- JIT compilation for JS instead of AOT
- On-demand CSS parsing (parse only visible styles)
- Deferred image decoding

### 2. **Memory Pool Architecture**
- Pre-allocated memory pools for DOM nodes
- Object pooling for frequently created/destroyed objects
- Arena allocators for short-lived allocations
- Reference counting with immediate cleanup

### 3. **Feature Flags & Conditional Compilation**
```toml
[features]
# Minimal CLI mode (text-only browsing)
cli-minimal = []

# Full CLI mode (text + basic graphics via ASCII/ANSI)
cli-full = ["cli-minimal", "image-ascii"]

# GUI mode levels
gui-basic = ["rendering", "layout-basic"]
gui-standard = ["gui-basic", "css-animations", "canvas"]
gui-full = ["gui-standard", "webgl", "webrtc", "media"]

# Fine-grained feature control
webgl = []
webrtc = []
media = ["video", "audio"]
service-workers = []
indexeddb = []
```

### 4. **Zero-Copy Architecture**
- Memory-mapped files for large resources
- String interning for repeated text
- Shared buffers between components
- COW (Copy-on-Write) for DOM operations

---

## 🖥️ CLI Mode Architecture

### CLI Mode Levels

#### **Level 1: CLI-Minimal** (~5MB RAM, ~2MB binary)
```rust
Features:
- HTML parser (text extraction only)
- Basic DOM (no styling)
- Text-only JavaScript (no graphics APIs)
- Fetch API for network requests
- localStorage (file-backed)
- No rendering engine
- No image loading
- Output: Plain text to stdout

Use Case: Web scraping, API testing, headless automation
```

#### **Level 2: CLI-Standard** (~15MB RAM, ~5MB binary)
```rust
Features:
+ ANSI color support
+ Box-drawing characters for borders
+ Sixel/iTerm2 inline images (optional)
+ ASCII art rendering for simple graphics
+ Table rendering
+ Basic CSS (colors, text styles)

Use Case: Daily browsing in terminal, email, docs
```

#### **Level 3: CLI-Full** (~30MB RAM, ~8MB binary)
```rust
Features:
+ Terminal graphics protocol (Kitty, WezTerm)
+ Canvas rendering to terminal
+ Full CSS support with ASCII approximation
+ Interactive forms
+ JavaScript with limited graphics

Use Case: Rich terminal experience for power users
```

### CLI-Specific Optimizations

```rust
// src/cli/mod.rs - New CLI subsystem
pub struct CLIBrowser {
    mode: CLIMode,
    viewport: TextViewport,  // Terminal dimensions
    dom_cache: DOMCache,     // Lightweight DOM representation
    text_renderer: ASCIIRenderer,
}

enum CLIMode {
    Minimal,    // Text only
    Standard,   // ANSI colors + basic formatting
    Full,       // Terminal graphics protocol
}

// Minimal DOM for CLI
struct LightweightDOM {
    text_nodes: Vec<String>,      // Just text content
    links: Vec<(String, String)>,  // Link text + URL
    headings: Vec<(u8, String)>,   // Level + text
    // No style information
    // No layout tree
    // No render tree
}
```

---

## 🎨 GUI Mode Optimization

### Progressive Loading Strategy

```rust
pub struct OptimizedGUIBrowser {
    // Core (always loaded)
    html_parser: LazyParser,
    minimal_dom: CompactDOM,
    
    // On-demand subsystems
    css_engine: Option<Box<CssEngine>>,      // Load when <style> encountered
    js_engine: Option<Box<JSEngine>>,        // Load when <script> encountered
    layout_engine: Option<Box<LayoutEngine>>, // Load for rendering
    canvas: Option<Box<Canvas2D>>,            // Load for <canvas>
    webgl: Option<Box<WebGLContext>>,         // Load for WebGL
    media: Option<Box<MediaEngine>>,          // Load for <video>/<audio>
}

impl OptimizedGUIBrowser {
    pub fn load_page(&mut self, url: &str) -> Result<()> {
        // 1. Fetch HTML (always needed)
        let html = self.fetch(url)?;
        
        // 2. Parse HTML (minimal pass - just structure)
        let dom = self.html_parser.parse_minimal(&html)?;
        
        // 3. Scan for features needed
        let features = self.detect_features(&dom);
        
        // 4. Load ONLY required subsystems
        if features.has_css {
            self.css_engine = Some(Box::new(CssEngine::new()));
        }
        if features.has_js {
            self.js_engine = Some(Box::new(JSEngine::new_minimal()));
        }
        
        // 5. Progressive enhancement
        self.render_progressive(dom, features)
    }
}
```

### Memory-Efficient DOM

```rust
// Replace Rc<RefCell<Node>> with compact representation
pub struct CompactDOM {
    nodes: Vec<DOMNode>,           // Flat array, index-based
    attributes: StringPool,         // Deduplicated strings
    text_content: StringPool,
    parent_map: HashMap<u32, u32>,  // Child -> Parent index
    children_map: HashMap<u32, Vec<u32>>, // Parent -> Children indices
}

// 32-byte node (vs 200+ bytes with Rc<RefCell<>>)
#[repr(C)]
pub struct DOMNode {
    tag_id: u16,           // Interned string ID
    flags: u16,            // Node type, dirty flags, etc.
    first_attr: u32,       // Index into attribute array
    text_offset: u32,      // Index into text pool
    parent_idx: u32,       // Parent node index
    first_child_idx: u32,  // First child index
    next_sibling_idx: u32, // Next sibling index
    style_idx: u32,        // Index into style array
}
```

### Smart Caching

```rust
pub struct ResourceCache {
    // LRU cache with memory limits
    images: LRUCache<Url, CompressedImage>,  // Max 50MB
    fonts: LRUCache<FontKey, Font>,          // Max 20MB
    scripts: LRUCache<Url, CompiledScript>,  // Max 30MB
    
    // Automatically compress unused resources
    compression_threshold: Duration,  // Compress after 30s unused
}

impl ResourceCache {
    fn get_or_load<T>(&mut self, key: &str) -> Result<&T> {
        if let Some(cached) = self.get(key) {
            return Ok(cached);
        }
        
        // Check memory pressure
        if self.memory_usage() > self.max_memory {
            self.evict_lru();
        }
        
        // Load resource
        let resource = self.load(key)?;
        self.insert(key, resource)
    }
}
```

---

## 📦 Modular Compilation Strategy

### Cargo.toml Feature Matrix

```toml
[features]
default = ["cli-standard"]

# CLI Modes
cli-minimal = ["html-text-only", "dom-minimal", "js-core"]
cli-standard = ["cli-minimal", "ansi-colors", "css-text", "ascii-render"]
cli-full = ["cli-standard", "terminal-graphics", "canvas-ascii"]

# GUI Modes
gui-minimal = ["rendering-basic", "layout-basic", "css-core"]
gui-standard = ["gui-minimal", "css-full", "canvas", "forms"]
gui-full = ["gui-standard", "webgl", "webrtc", "media", "service-workers"]

# Core subsystems (always available)
html-text-only = []
dom-minimal = []
js-core = []

# Optional subsystems
css-full = ["css-animations", "css-transforms", "css-grid"]
rendering-gpu = ["wgpu"]
webgl = ["rendering-gpu"]
webrtc = ["networking-p2p"]
media = ["video-decode", "audio-decode"]
service-workers = ["js-workers", "cache-api"]

# Network options
networking-minimal = []  # HTTP only
networking-full = ["websockets", "webrtc", "http2"]

# Image formats (opt-in)
image-png = ["png"]
image-jpeg = ["jpeg-decoder"]
image-webp = ["webp"]
image-svg = ["svg-render"]
image-all = ["image-png", "image-jpeg", "image-webp", "image-svg"]
```

### Binary Size Targets

| Build Configuration | Binary Size | RAM Usage | Features |
|---------------------|-------------|-----------|----------|
| `cli-minimal` | 2-3 MB | 5-10 MB | Text browsing only |
| `cli-standard` | 4-6 MB | 15-25 MB | ANSI colors, tables |
| `cli-full` | 8-12 MB | 30-50 MB | Terminal graphics |
| `gui-minimal` | 10-15 MB | 100-150 MB | Basic rendering |
| `gui-standard` | 20-30 MB | 150-250 MB | Full CSS, Canvas |
| `gui-full` | 40-60 MB | 300-500 MB | WebGL, Media, WebRTC |

---

## 🚀 Implementation Roadmap

### Phase 1: Core Refactoring (Week 1-2)
- [ ] Implement feature flags in Cargo.toml
- [ ] Refactor lib.rs to support conditional compilation
- [ ] Create CLI subsystem (`src/cli/`)
- [ ] Extract minimal DOM implementation

### Phase 2: Memory Optimization (Week 3-4)
- [ ] Replace Rc<RefCell<>> with indexed DOM
- [ ] Implement string interning
- [ ] Add memory pools for common allocations
- [ ] Create resource cache with LRU eviction

### Phase 3: CLI Browser (Week 5-6)
- [ ] CLI-minimal mode implementation
- [ ] ANSI/ASCII rendering engine
- [ ] Terminal graphics protocol support
- [ ] Interactive terminal UI

### Phase 4: GUI Optimization (Week 7-8)
- [ ] Lazy loading for subsystems
- [ ] On-demand CSS parsing
- [ ] Progressive rendering
- [ ] GPU acceleration (optional)

### Phase 5: Testing & Benchmarking (Week 9-10)
- [ ] Memory profiling across all modes
- [ ] Performance benchmarks
- [ ] Real-world website testing
- [ ] Documentation

---

## 📊 Benchmarking Targets

### Memory Usage (Target vs Current)

| Scenario | Current | Target (CLI) | Target (GUI) |
|----------|---------|--------------|--------------|
| Empty page | ~50MB | <5MB | <30MB |
| Wikipedia article | ~150MB | <15MB | <80MB |
| GitHub.com | ~300MB | <25MB | <150MB |
| YouTube | ~500MB | N/A* | <200MB |
| Gmail | ~400MB | <20MB | <180MB |

*CLI mode won't support video streaming

### Startup Time

| Mode | Target |
|------|--------|
| CLI-minimal | <50ms |
| CLI-standard | <100ms |
| GUI-minimal | <200ms |
| GUI-full | <500ms |

---

## 🔧 Technical Implementation Details

### 1. String Interning

```rust
pub struct StringPool {
    strings: Vec<String>,
    lookup: HashMap<String, u32>,
}

impl StringPool {
    pub fn intern(&mut self, s: &str) -> StringId {
        if let Some(&id) = self.lookup.get(s) {
            return StringId(id);
        }
        let id = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.lookup.insert(s.to_string(), id);
        StringId(id)
    }
    
    pub fn get(&self, id: StringId) -> &str {
        &self.strings[id.0 as usize]
    }
}
```

### 2. Memory Pool Allocator

```rust
pub struct DOMNodePool {
    nodes: Vec<DOMNode>,
    free_list: Vec<u32>,
}

impl DOMNodePool {
    pub fn allocate(&mut self) -> NodeHandle {
        if let Some(idx) = self.free_list.pop() {
            NodeHandle(idx)
        } else {
            let idx = self.nodes.len() as u32;
            self.nodes.push(DOMNode::default());
            NodeHandle(idx)
        }
    }
    
    pub fn free(&mut self, handle: NodeHandle) {
        self.free_list.push(handle.0);
    }
}
```

### 3. Lazy CSS Parser

```rust
pub struct LazyStyleEngine {
    raw_css: String,
    parsed_rules: HashMap<Selector, Vec<Declaration>>,
    parse_cache: HashSet<String>,
}

impl LazyStyleEngine {
    pub fn get_styles(&mut self, selector: &Selector) -> &[Declaration] {
        if !self.parse_cache.contains(selector.as_str()) {
            // Parse only this selector's rules on-demand
            self.parse_selector_rules(selector);
            self.parse_cache.insert(selector.to_string());
        }
        &self.parsed_rules[selector]
    }
}
```

### 4. CLI Text Renderer

```rust
pub struct ASCIIRenderer {
    viewport: (u16, u16),  // Terminal size
    buffer: Vec<Vec<Cell>>,
}

pub struct Cell {
    ch: char,
    fg: ANSIColor,
    bg: ANSIColor,
    style: TextStyle,  // Bold, italic, underline
}

impl ASCIIRenderer {
    pub fn render_dom(&mut self, dom: &LightweightDOM) -> String {
        let mut output = String::new();
        
        for node in &dom.nodes {
            match node.kind {
                NodeKind::Heading(level) => {
                    output.push_str(&self.render_heading(node, level));
                }
                NodeKind::Paragraph => {
                    output.push_str(&self.wrap_text(&node.text, self.viewport.0));
                }
                NodeKind::Link => {
                    output.push_str(&self.render_link(node));
                }
                _ => {}
            }
        }
        
        output
    }
    
    fn render_heading(&self, node: &Node, level: u8) -> String {
        let marker = "=".repeat((7 - level) as usize);
        format!("\n{} {} {}\n", marker, node.text, marker)
    }
}
```

---

## 🎯 Key Optimization Wins

### 1. **Conditional Compilation** → 50-80% binary size reduction
- CLI-minimal: Only text parsing, no rendering
- Removes unused image codecs, media decoders, GPU code

### 2. **Indexed DOM** → 70% memory reduction
- 32 bytes per node vs 200+ bytes with Rc<RefCell<>>
- Better cache locality, faster traversal

### 3. **Lazy Loading** → 60% faster startup
- Load JS engine only when <script> encountered
- Parse CSS only for visible elements
- Decode images on-demand

### 4. **String Interning** → 40% memory reduction
- Tag names, class names, IDs are heavily repeated
- One copy in memory, referenced by index

### 5. **Resource Caching** → 50% network reduction
- Smart LRU cache with compression
- Automatically free unused resources

---

## 📝 Example CLI Usage

```bash
# Minimal text-only browsing
pronbrow --mode cli-minimal https://news.ycombinator.com

# Standard CLI with colors
pronbrow --mode cli https://github.com

# Full CLI with terminal graphics
pronbrow --mode cli-full --graphics kitty https://example.com

# GUI mode with specific features
pronbrow --features gui-standard,webgl https://threejs.org

# Headless mode for automation
pronbrow --headless --execute "document.title" https://example.com
```

---

## 🔮 Future Optimizations

1. **WASM Compilation** - Compile hot paths to WASM for speed
2. **Multi-Process Architecture** - Separate process per tab (Chromium-style)
3. **GPU Acceleration** - Use wgpu for rendering (optional)
4. **Incremental Parsing** - Stream parse HTML as it downloads
5. **Content Blocking** - Built-in ad/tracker blocking saves bandwidth
6. **Compressed DOM** - Use rkyv for zero-copy deserialization

---

## 📈 Success Metrics

- [ ] CLI-minimal uses <10MB RAM on typical webpage
- [ ] GUI-standard uses <200MB RAM on typical webpage
- [ ] Binary size <10MB for CLI, <30MB for GUI
- [ ] Startup time <100ms for CLI, <500ms for GUI
- [ ] Page load 2x faster than current implementation
- [ ] Memory usage 3x lower than current implementation

---

**Next Steps:**
1. Implement feature flags
2. Create CLI subsystem skeleton
3. Benchmark current implementation
4. Begin Phase 1 refactoring
