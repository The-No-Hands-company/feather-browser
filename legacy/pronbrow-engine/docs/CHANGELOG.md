# Changelog

All notable changes to the PronBrow Engine will be documented in this file.

## [v0.1.9] - 2024-11-12

### ✨ Added
- **RGB/RGBA Color Support** - Full color function parsing
  - `rgb(r, g, b)` format support
  - `rgba(r, g, b, a)` format with alpha channel
  - Short hex colors: `#RGB` expands to `#RRGGBB`
  - Alpha blending for RGBA colors
  
- **Word Spacing** - Control spacing between words
  - CSS `word-spacing` property
  - Proper word boundary detection
  - Additive spacing on top of normal gaps
  
- **Font Variant (Small-Caps)** - Typography enhancement
  - CSS `font-variant: small-caps` support
  - Lowercase letters rendered as scaled uppercase
  - Maintains readability while adding style
  
- **White-Space Handling** - Complete white-space mode support
  - `white-space: normal` - Collapse sequences, wrap text
  - `white-space: nowrap` - Collapse sequences, no wrapping
  - `white-space: pre` - Preserve all whitespace, no wrapping
  - `white-space: pre-wrap` - Preserve whitespace, allow wrapping
  - `white-space: pre-line` - Collapse spaces, preserve newlines
  - Text preprocessing based on mode
  - Proper wrapping behavior per mode

### 🔧 Improved
- **CSS Parser** - Enhanced color and value parsing
  - Function-based color parsing (rgb/rgba)
  - Improved value type detection
  - Better error handling for color values
  
- **Text Rendering** - Enhanced layout algorithm
  - Word spacing calculation
  - White-space preprocessing
  - Small-caps character transformation
  - Improved line breaking logic

### 🏗️ Structure
- Updated: `src/css/mod.rs` - RGB/RGBA parsing, short hex support
- Updated: `src/text/mod.rs` - Word spacing, font-variant, white-space
- Updated: `src/render/mod.rs` - New CSS property handling
- Added: `test_new_features.html` - Test file for new features

### 📊 Statistics
- Lines of code: **+380** (1,720 → 2,100)
- New features: **8**
- Total features: **19 / ~700**
- Progress: **2.7%**

### 🎯 Demo Updates
- RGB/RGBA color demonstration
- Word spacing examples
- Small-caps text rendering
- White-space mode comparisons
- Comprehensive test file

---

## [v0.1.2] - 2024-11-12

### ✨ Added
- **Text Alignment** - Complete text alignment support
  - Left alignment (default)
  - Center alignment
  - Right alignment
  - Justify alignment (basic)
  - Per-line alignment calculation
  
- **Font Weight Support** - Bold text rendering
  - Bold font (weight >= 700)
  - Normal font (weight < 700)
  - CSS font-weight property parsing
  - Numeric font-weight values (400, 700, etc.)
  - Keyword values (bold, normal, bolder, lighter)
  
- **Font Style Support** - Italic text rendering
  - Italic font
  - Bold-italic font combination
  - CSS font-style property parsing
  - Italic and oblique keywords

### 📦 Assets
- Added `Roboto-Bold.ttf` - Bold font variant
- Added `Roboto-Italic.ttf` - Italic font variant
- Added `Roboto-BoldItalic.ttf` - Bold-italic combination

### 🏗️ Structure
- Updated: `src/text/mod.rs` - Added alignment and font styling
- Updated: `src/render/mod.rs` - Parse CSS properties for text
- Updated: `src/main.rs` - Demo showcasing new features

### 📊 Statistics
- Lines of code: **+120** (1,600 → 1,720)
- New features: **3**
- Total features: **11 / ~700**
- Progress: **1.6%**

### 🎯 Demo Updates
- Showcase text alignment (left, center, right)
- Demonstrate bold text
- Demonstrate italic text
- Combined styling examples

---

## [v0.1.1] - 2024-11-12

### ✨ Added
- **Text Rendering System** - Complete font loading and text rendering
  - Font loading support (TTF, OTF)
  - Glyph rasterization using fontdue
  - Text layout with line breaking
  - Word wrapping
  - Alpha-blended text rendering
  - Default font (Roboto) embedded
  - Text color from CSS
  - Font size from CSS
  - Letter spacing support (basic)
  - Line height support (basic)

### 📦 Dependencies
- Added `fontdue` v0.9.3 - Font loading and rasterization
- Added `ab_glyph` v0.2.32 - Additional glyph support  
- Added `image` v0.25.8 - Image processing (preparation)

### 🏗️ Structure
- New module: `src/text/mod.rs` - Text rendering implementation
- New asset: `src/assets/Roboto-Regular.ttf` - Default font
- Updated: `src/render/mod.rs` - Integrated text rendering
- Updated: `src/lib.rs` - Exported TextRenderer

### 📊 Statistics
- Lines of code: **+277** (1,323 → 1,600)
- New features: **5**
- Total features: **8 / ~700**

### 🎯 Demo Updates
- Updated demo to showcase text rendering
- New HTML content with headings and paragraphs
- CSS styling for text colors and sizes
- Generated output.png now shows actual rendered text

---

## [v0.1.0] - 2024-11-12

### 🎉 Initial Release - Foundation Complete

### ✨ Added
- **HTML Parser** - Custom recursive descent parser
  - Parses nested elements
  - Attribute support
  - Text nodes
  - Comment handling
  
- **CSS Parser** - Stylesheet parsing
  - Tag selectors
  - Class selectors
  - ID selectors
  - Color values (hex)
  - Length values (px)
  - Keyword values
  - Specificity calculation

- **DOM** - Document Object Model
  - Tree structure
  - Element nodes
  - Text nodes
  - Comment nodes
  - Pretty printing

- **Layout Engine** - Box model implementation
  - Block layout
  - Inline layout
  - Width/height calculation
  - Margin, padding, border boxes
  - Content box
  - Auto sizing

- **Render Engine** - Canvas rendering
  - Background colors
  - Box rendering
  - PNG export
  - Display command system

- **Network Engine** - HTTP client
  - HTTP/HTTPS requests
  - URL parsing
  - Async support
  - Blocking API

### 📦 Dependencies
- `reqwest` v0.11 - HTTP client
- `png` v0.17 - PNG encoding
- `tokio` v1 - Async runtime
- `serde` / `serde_json` - Serialization
- `url` v2.5 - URL parsing
- `log` / `env_logger` - Logging

### 📚 Documentation
- Complete README.md
- ARCHITECTURE.md - Technical deep dive
- DEVELOPMENT.md - Developer guide
- FEATURE_ROADMAP.md - 700+ features planned
- ELECTRON_BROWSER_ROADMAP.md - 460+ browser features
- MASTER_PROJECT_PLAN.md - Complete project strategy
- PROJECT_SUMMARY.md - Executive summary
- QUICK_START.md - Quick start guide
- GETTING_STARTED.md - Next steps

### 📊 Statistics
- Lines of code: **1,323**
- Modules: **7**
- Documentation: **~4,654 lines**
- Test coverage: Basic integration tests

### 🎯 Demo
- Working HTML → CSS → Layout → Render pipeline
- Generates 800x600 PNG output
- Shows box model rendering

---

## [Unreleased]

### 🔮 Planned for v0.2.0
- Text alignment (left, center, right, justify)
- Font weight support (bold)
- Font style support (italic)
- Text decorations (underline, strike-through)
- PNG image loading
- JPEG image loading
- Image rendering in layout
- Better CSS selector matching
- Improved error handling

### 🔮 Planned for v0.3.0
- Basic JavaScript interpreter
- DOM manipulation API
- Event system foundation
- Form elements
- Input handling

### 🔮 Planned for v0.5.0
- Flexbox layout
- CSS Grid layout
- Advanced CSS selectors
- Media queries
- CSS animations (basic)

---

## Version History

- **v0.1.1** (2024-11-12) - Text rendering
- **v0.1.0** (2024-11-12) - Initial foundation

---

## Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for guidelines on contributing to PronBrow Engine.

## License

MIT License - See [LICENSE](LICENSE) for details.
