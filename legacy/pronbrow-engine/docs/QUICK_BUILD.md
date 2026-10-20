# PronBrow Quick Build Reference

## 🚀 Fast Build Commands

### CLI Browser (Terminal)

```bash
# Recommended: Standard CLI with ANSI colors
cargo build --release --bin pronbrow-cli --features cli-standard --no-default-features

# Minimal: Text-only, smallest footprint
cargo build --release --bin pronbrow-cli --features cli-minimal --no-default-features

# Full: Terminal graphics support
cargo build --release --bin pronbrow-cli --features cli-full --no-default-features
```

### GUI Browser (Desktop)

```bash
# Recommended: Standard GUI with all common features
cargo build --release --features gui-standard --no-default-features

# Or just use default
cargo build --release

# Minimal: Basic rendering only
cargo build --release --features gui-minimal --no-default-features

# Full: Everything including WebGL, WebRTC, Media
cargo build --release --features gui-full --no-default-features
```

## 📊 Resource Usage Quick Reference

| Mode | Binary | RAM | Speed | Best For |
|------|--------|-----|-------|----------|
| `cli-minimal` | ~3 MB | ~8 MB | ⚡⚡⚡ | SSH, servers, scraping |
| `cli-standard` | ~5 MB | ~18 MB | ⚡⚡ | Daily terminal browsing |
| `cli-full` | ~10 MB | ~35 MB | ⚡ | Rich terminal experience |
| `gui-minimal` | ~12 MB | ~80 MB | ⚡⚡ | Simple HTML pages |
| `gui-standard` | ~25 MB | ~150 MB | ⚡ | Modern web (recommended) |
| `gui-full` | ~50 MB | ~300 MB | ⚪ | WebGL, media, WebRTC |

## 🎯 Usage Examples

```bash
# CLI: Browse example.com with minimal mode
./target/release/pronbrow-cli --mode minimal https://example.com

# CLI: Browse with colors (standard mode)
./target/release/pronbrow-cli https://news.ycombinator.com

# CLI: Full graphics mode
./target/release/pronbrow-cli --mode full https://github.com

# GUI: Standard browser
./target/release/pronbrow

# Help
./target/release/pronbrow-cli --help
```

## 🔧 Custom Builds

```bash
# JavaScript only (headless automation)
cargo build --release --features "js-full,networking-minimal" --no-default-features

# Web scraper (no GUI, just HTML parsing)
cargo build --release --features "networking-full" --no-default-features

# Image viewer (PNG + JPEG only)
cargo build --release --features "gui-minimal,image-png,image-jpeg" --no-default-features
```

## 📦 Feature Flags Cheat Sheet

### Common Combinations

```toml
# Minimal browser
--features cli-minimal --no-default-features

# Standard browser with JavaScript
--features "gui-standard,js-full"

# Browser without WebRTC/WebGL (lighter)
--features "gui-standard" --no-default-features

# Everything
--features gui-full
```

### Individual Features

**Core:**
- `html-text-only` - Text extraction
- `html-full` - Full HTML5 parser
- `dom-minimal` - Lightweight DOM
- `js-core` - Basic JavaScript
- `js-full` - Full ES2025 JavaScript

**CSS:**
- `css-text` - Colors only
- `css-core` - Box model
- `css-full` - All CSS3
- `css-flexbox`, `css-grid`, `css-animations`

**Web:**
- `canvas` - Canvas 2D API
- `webgl` - WebGL
- `webrtc` - WebRTC
- `media` - Audio/Video
- `indexeddb` - IndexedDB
- `service-workers` - Service Workers

**Images:**
- `image-png`, `image-jpeg`, `image-webp`, `image-svg`

## 🐛 Troubleshooting

### Build Issues

```bash
# Clear cache and rebuild
cargo clean
cargo build --release --features gui-standard --no-default-features

# Check dependencies
cargo tree --features gui-standard

# Verbose build
cargo build --release --verbose --features cli-standard --no-default-features
```

### Size Optimization

```bash
# Strip symbols for smaller binary
strip target/release/pronbrow-cli

# Enable LTO (Link-Time Optimization) - add to Cargo.toml:
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"  # Optimize for size
```

### Memory Profiling

```bash
# Linux: valgrind
valgrind --tool=massif ./target/release/pronbrow-cli https://example.com

# Linux: heaptrack  
heaptrack ./target/release/pronbrow-cli https://example.com

# Check binary size
ls -lh target/release/pronbrow*
```

## 📚 Documentation

- **[RESOURCE_OPTIMIZATION_STRATEGY.md](RESOURCE_OPTIMIZATION_STRATEGY.md)** - Full strategy & architecture
- **[OPTIMIZATION_GUIDE.md](OPTIMIZATION_GUIDE.md)** - Complete build & usage guide
- **[OPTIMIZATION_IMPLEMENTATION.md](OPTIMIZATION_IMPLEMENTATION.md)** - What we built & why
- **[README.md](README.md)** - General PronBrow documentation

## ⚡ Performance Tips

1. **Use CLI mode** for simple pages (10x lighter than GUI)
2. **Build with minimal features** you need (smaller binary, less RAM)
3. **Enable LTO** in release builds (smaller, faster)
4. **Strip binaries** after building (20-30% size reduction)
5. **Use `--no-default-features`** to avoid bloat

## 🎓 Learning Path

**Beginner:** 
```bash
cargo build --release --bin pronbrow-cli --features cli-standard --no-default-features
```

**Intermediate:**
```bash
cargo build --release --features gui-standard --no-default-features
```

**Advanced:**
```bash
# Custom feature combination for your needs
cargo build --release --features "js-full,css-full,canvas,image-png" --no-default-features
```

## 🔮 Coming Soon

- Indexed DOM (70% memory reduction)
- String interning (40% memory reduction)  
- Memory pools (50% faster allocation)
- Resource caching (60% network reduction)
- GPU acceleration (10x rendering speed)

---

**Quick Start:** Just want to try it?

```bash
# Build CLI browser
cargo build --release --bin pronbrow-cli --features cli-standard --no-default-features

# Use it
./target/release/pronbrow-cli https://example.com
```

That's it! 🎉
