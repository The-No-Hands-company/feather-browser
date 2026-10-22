# Changelog

All notable changes to Feather Browser are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versioning: [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

## [0.3.3] — 2026-10-22

### Added
- `margin` shorthand fully covered by tests: 1-value and 2-value already existed;
  3-value (top, left/right, bottom) and 4-value (top, right, bottom, left) added
- New CSS parser tests: `test_margin_shorthand_3_values`, `test_margin_shorthand_4_values`
- New layout tests: `test_margin_4_value_shorthand_in_layout`,
  `test_margin_2_value_shorthand_in_layout` — verify shorthand values
  arrive in `LayoutBox::dimensions.margin.*` through the full style + layout pipeline

## [0.3.2] — 2026-10-22

### Added
- `padding` shorthand fully covered by tests: 1-value (all sides equal),
  2-value (top/bottom, left/right), 3-value (top, left/right, bottom),
  4-value (top, right, bottom, left — already tested in CSS layer since v0.2.5)
- New CSS parser tests: `test_padding_shorthand_1_value`,
  `test_padding_shorthand_2_values`, `test_padding_shorthand_3_values`
- New layout tests: `test_padding_4_value_shorthand_in_layout`,
  `test_padding_2_value_shorthand_in_layout` — verify shorthand values
  arrive in `LayoutBox::dimensions.padding.*` through the full style + layout pipeline

## [0.3.1] — 2026-10-22

### Added
- `link_stylesheets(root: &Node) -> Vec<String>` in `src/engine/html.rs` — collects `href` values
  from every `<link rel="stylesheet">` in the DOM; `rel` is case-insensitive, non-stylesheet links
  (`rel="icon"`, `rel="prefetch"`, etc.) are ignored; depth-capped at 512 to prevent stack overflow
- `resolve_url(href: &str, base: &str) -> Option<String>` in `src/bridge/feather_renderer.rs` —
  resolves absolute, protocol-relative (`//`), root-relative (`/path`), and relative (`path`)
  hrefs against the page's base URL
- Both fetch closures (`load_url` and `fetch_and_render_url`) now collect linked stylesheets on
  the background thread, fetch each one, and pass the concatenated CSS as `extra_css` to
  `render_page`; the `render_page` signature gains a third `extra_css: &str` parameter

## [0.3.0] — 2026-10-22

### Milestone — First Browseable Pages

The FeatherRenderer pipeline is now fully wired: a URL typed in the address bar is fetched
asynchronously, parsed, styled, laid out, and painted to the GTK4 DrawingArea without blocking
the UI thread. Pages taller than the viewport are scrollable with the mouse wheel or touchpad.

### Added
- **Async fetch pipeline** — `glib::ThreadPool::push_future` runs the blocking `network::fetch_page`
  off the GTK main thread; result is shipped back via `glib::MainContext::spawn_local`
- **Vertical scrolling** — `EventControllerScroll` tracks `scroll_offset`; re-renders on each tick
  and clamps to `(doc_height − viewport_height).max(0)` so over-scrolling is impossible
- **Word-wrap** — `InlineFormattingContext::layout` splits each text run into words and wraps at the
  available-width boundary; `text-align: left/center/right` applied per line
- **UA stylesheet** — `head`, `style`, `script`, `title` default to `display: none` so page chrome
  never leaks into the visible content area
- **Layout tree stored in `Inner`** — `layout_root: Option<LayoutBox>` retained so scroll
  re-renders do not re-parse the DOM
- **Error pages** — failed fetches render a minimal HTML error page rather than leaving the
  drawing area blank

## [0.2.9] — 2026-10-22

### Added
- `display: none` layout tests (2): hidden element produces no layout box; hidden div with children leaves the tree empty
- `overflow: hidden` render clipping: children whose entire border-box lies fully outside the parent's border-box are skipped in both `render_box` (feather-renderer) and `render_box_plain` paths
- `test_display_none_produces_no_pixels`: render test verifying zero red pixels when top-level element is `display:none`
- `test_overflow_hidden_skips_fully_outside_child`: render test verifying zero red pixels when child is fully below a zero-height `overflow:hidden` parent

### Known Limitations
- `overflow: hidden` clips only fully-outside children; partially-overlapping children are still painted (scissor/clip-path not yet implemented)

## [0.2.8] — 2026-10-22

### Added
- `vw` and `vh` viewport units: `50vw` resolves to 50% of containing width; `50vh` resolves to 50% of viewport height
- `LayoutEngine::build_with_viewport(root, containing_width, viewport_height)` — explicit viewport height for `vh` resolution; `build` defaults to 600px
- `min-width` clamps block width up; `max-width` clamps block width down (px and %)
- Explicit `height` (px/vh/vw/%) now applied for inline children too (was only block-children path)

## [0.2.7] — 2026-10-22

### Added
- 38 additional CSS named colors in `CssColor::from_keyword`: cyan/aqua, magenta/fuchsia, lime, navy, teal, maroon, olive, silver, coral, salmon, skyblue, tomato, indigo, violet, brown, tan, lightgray/grey, darkgray/grey, whitesmoke, beige, gold, crimson, hotpink, lightblue, lightgreen, lightyellow, darkblue, darkgreen, darkred, darkorange, deeppink, deepskyblue, forestgreen, limegreen, mediumblue, midnightblue, royalblue, slategray/grey, steelblue

## [0.2.6] — 2026-10-22

### Added
- CSS selector tests: class (`.foo`), id (`#bar`), descendant (`div p`), child (`div > p`) — 4 new tests in `engine_style_test.rs`

## [0.2.5] — 2026-10-22

### Added
- `border` CSS shorthand: `border: 1px solid black` now expands to `border-*-width` (×4) + `border-color`; `border-{side}` one-side shorthand also supported
- `style=""` inline attribute parsed and applied with correct specificity (overrides any stylesheet rule)
- Box model end-to-end tests: padding, margin, border, background-color verified through full layout + render pipeline

## [0.2.4] — 2026-10-22

### Added
- `text-align` inheritance through the style tree — `"text-align"` added to `INHERITABLE` in `style.rs`
- `rem` and `%` font-size units now resolve to px (`rem` uses 16 px CSS default base; `%` resolves relative to parent font-size)
- `<img>` placeholder rendering — grey fill (180,180,180) + 1px border (140,140,140) + centred 12px alt text; dimensions read from `width`/`height` HTML attributes (default 100×50)
- Glyph cache flush-on-overflow cap: `GLYPH_CACHE_CAP = 512`; cache clears on overflow (O(1) amortised, no new dependencies)

---

## [0.2.3] — 2026-10-22

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

## [0.2.2] — 2026-10-22

### Added
- CSS property inheritance: `color`, `font-size`, `font-family`, `text-align`, `line-height`, `font-weight`, `font-style` propagate from parent to child nodes in the style tree
- `em` unit resolution: `font-size: 1.5em` resolves against the computed parent font-size; fallback to 16 px base when parent is unset
- `style_tree()` and `style_node()` in `style.rs` now accept a parent `ComputedStyle` and thread it through the tree

### Tests added
- `test_color_inheritance`, `test_font_size_inheritance`, `test_em_resolution`, `test_em_child_of_em`, `test_explicit_value_overrides_inherited`, `test_non_inherited_property_not_propagated`

---

## [0.2.1] — 2026-10-22

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
- **FeatherRenderer stub** — `feather-renderer` Cargo feature compiles, ready for engine implementation
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

[Unreleased]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/The-No-Hands-company/feather-browser/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/The-No-Hands-company/feather-browser/releases/tag/v0.1.0
