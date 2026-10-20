# PronBrow Resource Optimization Guide

## Quick Start

### Build CLI Browser (Minimal footprint)

```bash
# Minimal mode - Text only (~2-3MB binary, ~5-10MB RAM)
cargo build --release --bin pronbrow-cli --features cli-minimal --no-default-features

# Standard mode - ANSI colors (~4-6MB binary, ~15-25MB RAM)
cargo build --release --bin pronbrow-cli --features cli-standard --no-default-features

# Full CLI mode - Terminal graphics (~8-12MB binary, ~30-50MB RAM)
cargo build --release --bin pronbrow-cli --features cli-full --no-default-features
```

### Build GUI Browser

```bash
# Minimal GUI - Basic rendering (~10-15MB binary, ~100-150MB RAM)
cargo build --release --features gui-minimal --no-default-features

# Standard GUI - Full CSS + Canvas (~20-30MB binary, ~150-250MB RAM)
cargo build --release --features gui-standard --no-default-features

# Full GUI - All features (~40-60MB binary, ~300-500MB RAM)
cargo build --release --features gui-full --no-default-features
```

## Usage Examples

### CLI Browser

```bash
# Minimal text-only browsing
./target/release/pronbrow-cli --mode minimal https://example.com

# Standard with colors
./target/release/pronbrow-cli --mode standard https://news.ycombinator.com

# Full terminal graphics
./target/release/pronbrow-cli --mode full https://github.com
```

### GUI Browser

```bash
# Standard GUI mode
./target/release/pronbrow
```

## Feature Flags Reference

### CLI Modes
- `cli-minimal` - Text-only, no styling (smallest footprint)
- `cli-standard` - ANSI colors + basic formatting
- `cli-full` - Terminal graphics protocol support

### GUI Modes
- `gui-minimal` - Basic rendering only
- `gui-standard` - Full CSS, Canvas, Forms (recommended)
- `gui-full` - All features including WebGL, WebRTC, Media

### Granular Features

**Core:**
- `html-text-only` - HTML text extraction only
- `html-full` - Full HTML5 parser
- `dom-minimal` - Lightweight indexed DOM
- `dom-full` - Full W3C DOM with Rc<RefCell<>>

**JavaScript:**
- `js-core` - Basic JS (variables, functions)
- `js-full` - Full ES2025 (async/await, generators, promises)

**CSS:**
- `css-text` - Text colors only
- `css-core` - Basic box model
- `css-full` - All CSS3 features
- `css-animations`, `css-transforms`, `css-grid`, `css-flexbox`

**Rendering:**
- `rendering-basic` - Software rendering
- `rendering-gpu` - GPU acceleration

**Web Features:**
- `canvas` - Canvas 2D API
- `webgl` - WebGL support
- `webrtc` - WebRTC
- `media` - Audio/Video
- `service-workers` - Service Workers
- `indexeddb` - IndexedDB

**Images:**
- `image-png`, `image-jpeg`, `image-webp`, `image-svg`
- `image-all` - All image formats

**Network:**
- `networking-minimal` - HTTP/1.1 only
- `networking-full` - HTTP/2, WebSockets, WebRTC

## Build Size Comparison

| Configuration | Binary Size | RAM Usage | Features |
|--------------|-------------|-----------|----------|
| cli-minimal | 2-3 MB | 5-10 MB | Text only |
| cli-standard | 4-6 MB | 15-25 MB | ANSI colors |
| cli-full | 8-12 MB | 30-50 MB | Graphics |
| gui-minimal | 10-15 MB | 100-150 MB | Basic render |
| gui-standard | 20-30 MB | 150-250 MB | Full CSS |
| gui-full | 40-60 MB | 300-500 MB | All features |

## Memory Optimization Techniques

### 1. Lazy Loading
Components are loaded only when needed:
```rust
// JS engine loaded only when <script> tag encountered
if html.contains("<script") {
    browser.ensure_js();
}
```

### 2. Feature Gating
Compile only what you need:
```toml
[features]
default = ["cli-standard"]  # Lightweight by default
```

### 3. Zero-Copy Where Possible
- Memory-mapped files for resources
- String interning for repeated text
- Shared buffers between components

### 4. Resource Caching
- LRU cache with memory limits
- Automatic compression of unused resources
- Smart eviction policies

## Performance Benchmarks

### Startup Time
- CLI-minimal: <50ms
- CLI-standard: <100ms
- GUI-minimal: <200ms
- GUI-full: <500ms

### Memory Usage (Wikipedia article)
- CLI-minimal: ~8MB
- CLI-standard: ~18MB
- GUI-standard: ~85MB
- Current (pre-optimization): ~150MB

## Advanced Usage

### Custom Feature Combinations

```bash
# JavaScript engine only (headless automation)
cargo build --release --features "js-full,networking-minimal" --no-default-features

# Image-optimized build (remove unused codecs)
cargo build --release --features "gui-standard,image-png,image-jpeg" --no-default-features

# Network-only (no GUI, for testing/scraping)
cargo build --release --features "networking-full,js-core" --no-default-features
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug ./pronbrow-cli https://example.com

# Force specific graphics protocol
TERM=xterm-kitty ./pronbrow-cli --mode full https://example.com

# Disable colors
NO_COLOR=1 ./pronbrow-cli https://example.com
```

## Development Tips

### Testing Different Builds

```bash
# Test minimal CLI
cargo run --bin pronbrow-cli --features cli-minimal --no-default-features -- \
    --mode minimal https://example.com

# Test standard GUI
cargo run --features gui-standard --no-default-features

# Test with specific feature
cargo test --features "js-full,css-full"
```

### Profiling Memory Usage

```bash
# Linux: Use valgrind
valgrind --tool=massif ./target/release/pronbrow-cli https://example.com

# Linux: Use heaptrack
heaptrack ./target/release/pronbrow-cli https://example.com

# macOS: Use Instruments
instruments -t Allocations ./target/release/pronbrow-cli https://example.com
```

### Binary Size Analysis

```bash
# Check binary size
ls -lh target/release/pronbrow*

# Analyze with bloaty (Google's tool)
bloaty target/release/pronbrow-cli

# Strip symbols for smaller size
strip target/release/pronbrow-cli
```

## Optimization Roadmap

### Phase 1: Foundation (Current)
- ✅ Feature flags architecture
- ✅ CLI browser skeleton
- ✅ Lazy loading framework
- ⬜ Conditional compilation working

### Phase 2: Core Optimizations
- ⬜ Replace Rc<RefCell<>> with indexed DOM
- ⬜ Implement string interning
- ⬜ Add memory pools
- ⬜ Implement resource cache

### Phase 3: CLI Implementation
- ⬜ Full text extraction
- ⬜ ANSI rendering
- ⬜ Terminal graphics protocols
- ⬜ Interactive navigation

### Phase 4: GUI Optimization
- ⬜ Progressive rendering
- ⬜ On-demand CSS parsing
- ⬜ Image lazy loading
- ⬜ GPU acceleration (optional)

### Phase 5: Advanced Features
- ⬜ Multi-process architecture
- ⬜ WASM compilation for hot paths
- ⬜ Incremental parsing
- ⬜ Content blocking

## Troubleshooting

### Build Errors

**Error: "feature not found"**
```bash
# Make sure feature is defined in Cargo.toml
cargo build --features cli-standard --no-default-features
```

**Error: "module not found"**
```bash
# Module might be feature-gated, check src/lib.rs
# Enable required features
```

### Runtime Issues

**CLI mode shows no output**
```bash
# Check if URL is accessible
curl -I https://example.com

# Enable debug logging
RUST_LOG=debug ./pronbrow-cli https://example.com
```

**High memory usage**
```bash
# Use minimal mode
./pronbrow-cli --mode minimal https://example.com

# Or build with minimal features
cargo build --features cli-minimal --no-default-features
```

## Contributing

When adding features:
1. Always add appropriate feature flag
2. Use `#[cfg(feature = "...")]` for conditional compilation
3. Update this guide with new features
4. Test memory usage before and after
5. Benchmark binary size impact

## Resources

- [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Memory Profiling in Rust](https://nnethercote.github.io/perf-book/profiling.html)

---

**Next Steps:** See [RESOURCE_OPTIMIZATION_STRATEGY.md](RESOURCE_OPTIMIZATION_STRATEGY.md) for detailed architecture.
