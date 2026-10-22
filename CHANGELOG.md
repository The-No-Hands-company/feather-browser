# Changelog

All notable changes to Feather Browser are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versioning: [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Engine primitives in progress (0.0.9)
- `text-align` inheritance through the style tree
- `rem`/`%` font-size → px conversion
- `<img>` placeholder rendering (grey box with alt-text label)
- LRU cap on the glyph cache (unbounded growth fix)

---

## [0.0.8] — 2026-10-22

### Added
- `CssColor::DARK` constant (`rgb(30, 30, 30)`) in `css.rs` alongside `BLACK`, `WHITE`, `TRANSPARENT`
- `TextAlign` enum (`Left` | `Center` | `Right`) in `layout.rs`
- `text-align: center` and `text-align: right` support in `InlineFormattingContext` — items shifted per line after flush
- `layout_block()` reads `text-align` from the CSS cascade and passes `TextAlign` to the IFC

### Fixed
- `pen_x` accumulation in `render_text()` converted from `i32` to `f32` — eliminates subpixel drift on long text lines
- Default text color in `render_text()` and `render_text_scrolled()` uses `CssColor::DARK` constant (was a repeated literal)

### Tests added
- `test_dark_color_constant_value` — verifies `CssColor::DARK` RGBA values
- `test_pen_advance_f32_accumulation` — regression guard for pen_x drift
- `test_text_align_left_is_default`, `test_text_align_center_offsets_items`, `test_text_align_right_offsets_items`, `test_text_align_center_via_css`

---

## [0.0.7] — 2026-10-22

### Added
- CSS property inheritance: `color`, `font-size`, `font-family`, `text-align`, `line-height`, `font-weight`, `font-style` propagate from parent to child nodes in the style tree
- `em` unit resolution: `font-size: 1.5em` resolves against the computed parent font-size; fallback to 16 px base when parent is unset
- `style_tree()` and `style_node()` in `style.rs` now accept a parent `ComputedStyle` and thread it through the tree

### Tests added
- `test_color_inheritance`, `test_font_size_inheritance`, `test_em_resolution`, `test_em_child_of_em`, `test_explicit_value_overrides_inherited`, `test_non_inherited_property_not_propagated`

---

## [0.0.6] — 2026-10-22

### Added
- `font.rs` — three-level glyph cache: per-size `fontdue::Font` instances, rasterized `GlyphBitmap` per `(char, px_size)`, `rasterize_cached()` API
- `GlyphBitmap` struct with `width`, `height`, `advance_width: f32`, `bearing_y: i32`, `bitmap: Vec<u8>`
- `blit_glyph()` in `render.rs` — alpha-composite glyph bitmap onto BGRA `Canvas`
- `render_text()` and `render_text_scrolled()` use fontdue for actual glyph rasterization (replaces grey-rectangle placeholder)
- `InlineFormattingContext` in `layout.rs` — lays out inline items, accumulates line width, breaks on available width
- Anonymous box wrapping for inline children of block boxes
- `parent_color` threaded through `BoxType::Anonymous` so inline text inherits the correct color

### Fixed
- Text was rendered as grey rectangles; now renders actual font glyphs at the correct baseline

### Tests added
- `engine_font_test.rs` — glyph cache hit/miss, advance width, multi-size independence
- `engine_render_test.rs` — pen-advance regression test (threshold 25 columns)
- `engine_inline_test.rs` — IFC item placement, line breaking, available-width clamping

---

## [0.2.0] — 2026-10-21

### Added
- **Ad blocker** — hosts-format blocklist, subdomain-aware (`stats.doubleclick.net` → blocks `doubleclick.net`), auto-seeded default list on first run
- **Privacy URL stripping** — removes `utm_*` (all variants by prefix), `fbclid`, `gclid`, `msclkid`, `dclid`, `twclid`, `mc_cid`, `mc_eid`, `_ga`, `_gl`, `igshid` before every navigation
- **Tab hibernation** — idle tabs suspended after configurable delay (`auto_hibernate_delay_secs`), renderer freed via `PageRenderer::suspend()`
- **FeatherRenderer stub** — `feather-renderer` Cargo feature compiles, ready for v0.3 engine implementation
- Shield (ad block) and freeze (hibernation) toggles in the toolbar

### Fixed
- `resume()` now clears `suspended_url` so a second suspend/resume cycle works correctly
- Closing a tab removes its widget from the content stack, stopping audio/JS/network leaks
- Tab URL and title update in real time after navigation
- `extract_domain` correctly handles URLs with userinfo (`user@host`) and queries without a path
- `utm_*` stripped by prefix, not just an explicit list — future `utm_` variants auto-covered
- `?ref=` parameter no longer stripped (was breaking GitHub/GitLab branch links)

---

## [0.1.0] — 2026-10-21

### Added
- GTK4 application shell with `ApplicationWindow`
- `PageRenderer` trait — abstraction layer between the shell and rendering backends
- `WebKitRenderer` — wraps `webkit2gtk::WebView`, gated by `webkit-renderer` Cargo feature
- `MockRenderer` — test double implementing the full `PageRenderer` trait
- Tab data model — `Tab`, `TabId`, `TabState` (Active / Idle / Hibernated)
- `TabManager` — add, remove, switch tabs; guards against closing the last tab
- Toolbar — address bar, back/forward/reload buttons, Shield and freeze toggles
- `AddressBarLogic::clean_url()` — bare domains → `https://`, plain text → DuckDuckGo
- Tab bar — renders active/idle/hibernated states, new-tab button
- Full shell wiring — toolbar navigate → renderer load, tab bar click/close/new
- Settings — `~/.config/feather/config.json`, JSON serialisation with sane defaults
- `scripts/build.sh` and `scripts/build.cmd`

---

[Unreleased]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/The-No-Hands-company/feather-browser/releases/tag/v0.1.0
