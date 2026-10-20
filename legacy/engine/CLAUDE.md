# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**PronBrow Engine** is a modern web browser engine built from scratch in Rust, targeting production-grade feature parity with Chrome/Firefox. Currently implements 785+ features (64% of major browsers) across HTML5, CSS3, JavaScript, and Web APIs.

**Key Stats:**
- 20,190+ lines of Rust code across 95 source files
- 67 modules in 9 major subsystems
- Dual build: Rust library/binary + Node.js bindings via NAPI

## Build & Development Commands

### Rust Development
```bash
# Build release binary
cargo build --release

# Build debug version
cargo build

# Run the standalone browser
./target/release/pronbrow

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check without building
cargo check
```

### Node.js Bindings (NAPI)
```bash
# Build NAPI bindings for Node.js
npm run build

# Build debug NAPI bindings
npm run build:debug

# The feature flag 'napi' must be enabled for Node.js builds
cargo build --release --features napi
```

### Testing
- Main test suite: `tests/js_execution_tests.rs`
- Run single test: `cargo test <test_name>`
- All tests must pass before commits

## Architecture Overview

### Core Pipeline (src/lib.rs)
The `BrowserEngine` struct orchestrates the complete rendering pipeline:
1. **Network** → Fetch HTML/CSS/JS resources
2. **HTML Parser** → Parse to DOM tree
3. **CSS Parser** → Build stylesheet
4. **Style Resolution** → Apply styles to DOM nodes
5. **Layout Engine** → Compute box model (Flexbox/Grid)
6. **Render Engine** → Paint to canvas
7. **JS Engine** → Execute scripts with DOM bridge
8. **Compositor** → Final output

### Module Structure

**Core Parsing & Rendering:**
- `src/html/` - HTML5 parser (W3C compliant)
- `src/css/` - CSS3 parser + Flexbox/Grid layout engines
  - `flexbox.rs`, `grid.rs` - Layout implementations
  - `animation.rs`, `animator.rs` - CSS animations/transitions
  - `transform.rs` - CSS transforms
  - `pseudo.rs` - Pseudo-classes/elements
  - `media.rs` - Media queries
- `src/layout/` - Box model and layout tree construction
- `src/render/` - Rendering pipeline and paint operations
- `src/compositor/` - Compositing and final output

**JavaScript Subsystem:**
- `src/js/engine.rs` - Main JS engine entry point
- `src/js/parser.rs` - ES6+ parser and AST
- `src/js/bytecode.rs` - Bytecode compiler and VM
- `src/js/interpreter.rs` - Bytecode interpreter
- `src/js/jit.rs` - JIT compiler (partial)
- `src/js/gc.rs` - Garbage collector
- `src/js/dom_bridge.rs` - **Critical**: JS↔DOM integration layer
- `src/js/dom_executor.rs` - Script execution in DOM context
- `src/js/promise.rs` - Promise/async/await implementation
- `src/js/wasm_runtime.rs` - WebAssembly support

**Web APIs:**
- `src/web_api/` - 200+ Web APIs (timers, fetch, storage, etc.)
- `src/js/console_api.rs` - console.log/warn/error
- `src/js/fetch_api.rs` - Fetch API
- `src/js/url_api.rs` - URL/URLSearchParams
- `src/js/timer_api.rs` - setTimeout/setInterval/rAF
- `src/storage/` - localStorage/sessionStorage
- `src/indexeddb/` - IndexedDB implementation
- `src/serviceworker/` - Service Workers (partial)

**Graphics & Media:**
- `src/canvas/` - Canvas 2D API
- `src/svg/` - SVG rendering
- `src/image/` - Image loading (PNG/JPEG/WebP)
- `src/webgl/` - WebGL support (partial)
- `src/media/` - HTMLMediaElement (audio/video)
- `src/text/` - Text rendering and fonts

**Networking & Security:**
- `src/network/` - HTTP/2, WebSocket, fetch
- `src/security/` - CORS, CSP, sandboxing
- `src/webrtc/` - WebRTC (partial)

**Document & Events:**
- `src/dom/` - W3C DOM Level 4 tree structure
  - `event_system.rs` - DOM event propagation
  - `js_integration.rs` - DOM↔JS bindings
- `src/events/` - Event system
- `src/forms/` - Form handling
- `src/accessibility/` - ARIA support

**Node.js Integration:**
- `src/bindings/` - NAPI bindings (only with `--features napi`)

## Critical Architecture Details

### DOM-JavaScript Integration
The **DOM Bridge** (`src/js/dom_bridge.rs`) is the central integration point:
- Exposes DOM APIs (document, Element, Node) to JavaScript
- Manages Rc<RefCell<Node>> for shared DOM access
- Handles script execution via `execute_scripts()`
- Must be initialized before any JS-DOM interaction

**Pattern:**
```rust
let root_rc = Rc::new(RefCell::new(dom.root));
dom_bridge.borrow_mut().setup_document(&mut js_engine, root_rc.clone());
dom_bridge.borrow_mut().execute_scripts(&mut js_engine, &root_rc.borrow())?;
```

### URL Loading Pipeline
`BrowserEngine::load_url()` demonstrates the complete flow:
1. Fetch HTML via NetworkEngine
2. Parse to DOM
3. Extract `<link>` and `<style>` CSS
4. Fetch external stylesheets
5. Parse combined CSS
6. Build styled tree
7. Compute layout
8. Paint to canvas
9. Execute scripts

### Layout Engine
- **Flexbox**: `src/css/flexbox.rs` - Full flex container/item implementation
- **Grid**: `src/css/grid.rs` - CSS Grid layout
- Both integrate with main layout engine via `LayoutEngine::layout_tree()`

### Node ID System
DOM nodes use atomic counters for unique IDs:
```rust
static NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
```
Critical for event targeting and DOM queries.

## Development Notes

### Feature Flags
- `napi` - Enable Node.js bindings (required for npm builds)
- Default features: none (pure Rust library)

### Crate Types
The project builds as both:
- `rlib` - Rust library for cargo builds
- `cdylib` - C dynamic library for NAPI bindings

### Async Runtime
Uses Tokio for async operations (network, file I/O). All async code must use `tokio::runtime` or be called from async context.

### Testing Strategy
- Integration tests in `tests/` directory
- Focus on end-to-end JS execution tests
- Test DOM-JS interaction thoroughly

### CSS Architecture
The CSS subsystem is modular:
- Parse CSS → `css::CssParser`
- Match selectors → `css::matching`
- Compute styles → `css::style_tree()`
- Layout (flexbox/grid) → `layout::LayoutEngine`
- Animations handled by `css::animator`

### Current Limitations
- **JavaScript**: Partial ES6+ support, JIT incomplete
- **WebAssembly**: Runtime exists but not fully integrated
- **Media**: HTMLMediaElement stub, no actual playback
- **WebGL/WebGPU**: Stubs only
- **Service Workers**: Partial implementation
- **Multi-process**: Single-process only (no site isolation)

## Roadmap Context

The project is in **v0.4-0.5** range, targeting:
- **v0.5.0**: Complete JavaScript execution
- **v0.6.0**: Full DOM integration
- **v0.7.0**: Media support (video/audio)
- **v0.8.0**: GPU acceleration
- **v1.0.0**: Production-ready (6-12 months)

Target: **9,350+ additional features** for Chrome/Firefox parity.

## Important Patterns

### Resource Resolution
Use `BrowserEngine::resolve_url()` for relative→absolute URL conversion (handles protocol-relative, absolute paths, etc.)

### Script Execution
Always call `DOMBridge::setup_document()` before executing scripts. Scripts expect `document`, `window`, and Element APIs to be available.

### Node Ownership
DOM nodes are wrapped in `Rc<RefCell<Node>>` for shared mutable access between rendering and JS engines. Be careful with borrow conflicts.

### Error Handling
Most functions return `Result<T, Box<dyn std::error::Error>>`. The engine prioritizes graceful degradation over crashes.
