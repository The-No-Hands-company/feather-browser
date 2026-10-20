# PronBrow - AI Development Instructions

## Project Mission
PronBrow is the next-generation web browser built from scratch with its own rendering engine. This is not a prototype, MVP, or proof-of-concept. Every line of code must be production-ready, industry-grade, and built to compete with Chrome, Firefox, Safari, and emerging engines like Ladybird.

## Core Principles

### 1. No Compromises
- **NO simplifications** - Implement full specifications
- **NO workarounds** - Fix root causes, not symptoms
- **NO placeholders** - Every function must be complete
- **NO stubs** - No "TODO: implement later" code
- **NO shortcuts** - Follow best practices always

### 2. Production-Ready Only
- All code must be production-grade from day one
- Implement complete error handling and edge cases
- Write comprehensive tests for every feature
- Document all public APIs and complex logic
- Follow industry best practices and standards

### 3. Standards Compliance
- **HTML5** - Full WHATWG HTML Living Standard compliance
- **CSS3** - Complete W3C CSS specifications (including Grid, Flexbox, Animations)
- **ECMAScript** - Full ES2024+ support
- **Web APIs** - Complete implementation of all standard Web APIs
- **ARIA** - Full accessibility support
- **Security** - Complete CSP, CORS, Same-Origin Policy implementation

### 4. Architecture Excellence
- **Rust Engine** - High-performance native rendering engine
- **Modular Design** - Clean separation of concerns
- **Thread Safety** - Proper concurrent programming
- **Memory Efficiency** - Zero-copy where possible, smart resource management
- **Performance** - JIT compilation, GPU acceleration, optimized algorithms

## Development Standards

### Code Quality
- Write idiomatic Rust code following official style guide
- Use type safety to prevent runtime errors
- Implement proper error handling with Result<T, E>
- Write self-documenting code with clear naming
- Add comments only for complex algorithms or non-obvious decisions

### Testing Requirements
- Unit tests for all business logic
- Integration tests for module interactions
- End-to-end tests for user-facing features
- Performance benchmarks for critical paths
- Security tests for all attack surfaces

### Performance Standards
- Render pages in <16ms (60 FPS)
- Parse HTML/CSS/JS as fast as Chrome
- Memory usage comparable to industry leaders
- Battery-efficient on mobile devices
- Smooth animations and scrolling

### Security Standards
- Implement full sandbox for renderer processes
- Complete site isolation
- Secure IPC between processes
- Memory safety guarantees (Rust's ownership model)
- Regular security audits and fuzzing

## Feature Implementation Checklist

Before implementing any feature:
1. ✅ Research complete specification (W3C, WHATWG, MDN)
2. ✅ Design architecture and data structures
3. ✅ Write comprehensive tests first (TDD when possible)
4. ✅ Implement full functionality (no partial implementations)
5. ✅ Handle all edge cases and error conditions
6. ✅ Optimize for performance
7. ✅ Document public APIs
8. ✅ Test against real websites
9. ✅ Validate standards compliance
10. ✅ Security review

## Project Structure

### Engine Components (`pronbrow-engine/`)
- **HTML Parser** - Tokenizer, tree construction, error recovery
- **CSS Parser** - Selector parsing, cascade, specificity
- **Style Engine** - Style resolution, computed values, inheritance
- **Layout Engine** - Box model, Flexbox, Grid, multi-column
- **Paint Engine** - Rasterization, GPU acceleration, compositing
- **JavaScript Engine** - Parser, bytecode compiler, VM, JIT
- **DOM Implementation** - Complete DOM API, event system
- **Network Stack** - HTTP/2, HTTP/3, WebSocket, caching
- **Security** - Sandbox, CSP, CORS, site isolation

### Browser Application (`pronbrow/`)
- **Electron Integration** - IPC, native bindings
- **Browser UI** - Tabs, navigation, bookmarks
- **DevTools** - Inspector, console, network, performance
- **Extensions** - WebExtensions API compatibility
- **Settings** - User preferences, profiles, sync

## Technology Decisions

### Why Rust for Engine?
- Memory safety without garbage collection
- Performance comparable to C++
- Fearless concurrency
- Strong type system prevents bugs
- Modern tooling (Cargo, rustfmt, clippy)

### Why Electron for Browser UI?
- Cross-platform (Linux, Windows, macOS)
- Rapid UI development with web technologies
- Easy integration with native Rust engine
- Large ecosystem of tools and libraries
- Familiar development experience

### Performance Strategy
- **Multi-process architecture** - Separate processes for UI, renderer, GPU
- **Multi-threading** - Parallel parsing, layout, paint
- **GPU acceleration** - Hardware compositing, WebGL, Canvas
- **JIT compilation** - Hot code optimization
- **Caching** - Aggressive caching at all levels

## Current Development Focus

### Phase 1: Core Engine (In Progress)
- ✅ HTML parser (tokenizer, tree builder)
- ✅ CSS parser (selectors, properties, values)
- ✅ Style resolution (cascade, inheritance, specificity)
- ✅ Layout engine (box model, positioning, float)
- ✅ Paint engine (canvas rendering, basic graphics)
- 🔄 JavaScript engine (parser, interpreter, runtime)
- 🔄 DOM APIs (manipulation, traversal, events)
- 📋 Network stack (HTTP/2, caching, cookies)
- 📋 Security (sandbox, CSP, CORS)

### Phase 2: Browser Integration
- 📋 Rust ↔ Node.js bindings (neon/napi-rs)
- 📋 Electron app architecture
- 📋 Browser UI components
- 📋 Tab management and navigation
- 📋 DevTools integration

### Phase 3: Advanced Features
- 📋 WebGL and Canvas 2D/3D
- 📋 Web Workers and Service Workers
- 📋 IndexedDB and Web Storage
- 📋 WebRTC and Media APIs
- 📋 Extension system

### Phase 4: Optimization & Polish
- 📋 Performance profiling and optimization
- 📋 Memory leak detection and fixes
- 📋 Security hardening
- 📋 Cross-browser testing
- 📋 Documentation and examples

## AI Assistant Guidelines

### When Writing Code
1. **Research First** - Understand the full specification before coding
2. **Design Before Implementation** - Plan data structures and algorithms
3. **Test-Driven** - Write tests before or alongside implementation
4. **Complete Implementation** - Never leave TODOs or placeholders
5. **Optimize** - Don't sacrifice performance for convenience
6. **Document** - Add doc comments for public APIs

### When Fixing Bugs
1. **Root Cause Analysis** - Find and fix the actual problem
2. **No Band-Aids** - Don't add workarounds for broken code
3. **Test Coverage** - Add tests to prevent regression
4. **Verify** - Test the fix thoroughly before considering it done

### When Refactoring
1. **Preserve Behavior** - Don't break existing functionality
2. **Improve Architecture** - Make code more maintainable
3. **Add Tests** - Ensure refactoring doesn't break anything
4. **Document Changes** - Update docs to reflect new structure

### Communication Style
- Be concise and technical
- Provide context for decisions
- Explain trade-offs when applicable
- Reference specifications and standards
- Show code examples for clarity

## Build & Test Commands

```bash
# Engine (Rust)
cd pronbrow-engine/
cargo build --release          # Production build
cargo test                     # Run all tests
cargo bench                    # Run benchmarks
cargo clippy                   # Linting
cargo fmt                      # Format code

# Browser (Electron)
cd pronbrow/
npm install                    # Install dependencies
npm start                      # Run development
npm run build                  # Production build
npm test                       # Run tests
npm run lint                   # Linting

# Integration
npm run build:engine           # Build Rust engine with Node bindings
npm run test:integration       # Full integration tests
```

## Resources & References

### Specifications
- [WHATWG HTML Living Standard](https://html.spec.whatwg.org/)
- [W3C CSS Specifications](https://www.w3.org/Style/CSS/specs.en.html)
- [ECMAScript Language Specification](https://tc39.es/ecma262/)
- [Web APIs - MDN](https://developer.mozilla.org/en-US/docs/Web/API)

### Browser Engineering
- [Web Browser Engineering Book](https://browser.engineering/)
- [Servo Architecture](https://github.com/servo/servo/wiki/Design)
- [Chromium Design Docs](https://www.chromium.org/developers/design-documents/)
- [WebKit Architecture](https://webkit.org/code/browse/)

### Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Success Criteria

PronBrow will be considered successful when it can:
1. ✅ Render modern websites correctly (HTML5, CSS3, ES2024+)
2. ✅ Pass W3C/WHATWG conformance tests
3. ✅ Achieve performance parity with Chrome/Firefox
4. ✅ Provide complete DevTools for debugging
5. ✅ Support WebExtensions for extensibility
6. ✅ Maintain security on par with industry leaders
7. ✅ Run on Linux, Windows, and macOS
8. ✅ Handle real-world websites and web applications

---

**Remember**: Every commit should move us closer to a browser that developers trust and users love. No compromises. No shortcuts. Only excellence.
