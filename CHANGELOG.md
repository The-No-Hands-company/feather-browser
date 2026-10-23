# Changelog

All notable changes to Feather Browser are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versioning: [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

## [0.5.4] — 2026-10-23

### Added
- `:nth-child(An+B)` pseudo-class — full CSS2/CSS3 An+B syntax for `:nth-child`
  - Named keywords: `odd` (positions 1, 3, 5 …), `even` (positions 2, 4, 6 …)
  - Plain integer: `:nth-child(3)` matches only position 3
  - Step form: `:nth-child(3n)` matches every third element (3, 6, 9 …)
  - Full An+B: `:nth-child(2n+1)` equals `odd`; `:nth-child(-n+3)` matches first three
  - Functional pseudo-class argument stored as `"nth-child(arg)"` in `Selector.pseudo`
- CSS parser updated: `parse_simple_selector` consumes balanced `(…)` after a pseudo identifier so functional pseudo-classes survive the tokenizer unchanged

### Tests
- `engine_css_test.rs`: 5 new parsing tests in `pseudo_class_tests`:
  - `test_nth_child_functional_pseudo_parses`
  - `test_nth_child_odd_parses`
  - `test_nth_child_even_parses`
  - `test_nth_child_plain_number_parses`
  - `test_nth_child_specificity_counts_as_class`
- `engine_style_test.rs`: `nth_child_tests` module — 5 new style-matching tests:
  - `test_nth_child_odd_matches_positions_1_3_5`
  - `test_nth_child_even_matches_positions_2_4`
  - `test_nth_child_plain_number_matches_only_that_position`
  - `test_nth_child_2n_plus_1_matches_odd`
  - `test_nth_child_3n_matches_multiples_of_3`

## [0.5.3] — 2026-10-23

### Added
- `:first-child` pseudo-class — matches an element that is the first element child of its parent (text/comment nodes excluded from sibling count)
- `:last-child` pseudo-class — matches an element that is the last element child of its parent
- `:only-child` pseudo-class — matches when the element is the sole element child of its parent (equivalent to `:first-child:last-child`)
- `SiblingPos` struct threaded through the full style tree — both `simple_matches` (for the subject selector) and ancestor matching in compound selectors (e.g., `li:first-child > a`) receive accurate first/last flags
- Structural pseudo-classes count as one class-level unit in specificity (already handled by existing `pseudo` specificity rule)

### Tests
- `engine_style_test.rs`: `structural_pseudo_class_tests` module — 5 new tests:
  - `test_first_child_matches_first_element`
  - `test_last_child_matches_last_element`
  - `test_only_child_matches_sole_element`
  - `test_only_child_does_not_match_with_siblings`
  - `test_first_child_specificity_beats_plain_tag`

## [0.5.2] — 2026-10-23

### Added
- `display: inline-block` — new `BoxType::InlineBlock(StyledNode)` variant in the layout tree
- `calculate_inline_block_width`: respects explicit `width` (px, %, vw, vh) and `box-sizing: border-box`; falls back to shrink-to-fit (available container width) when width is `auto`
- `layout_inline_block`: block-level internal layout (IFC for inline children, block flow for block children) with full height constraints (`height`, `min-height`, `max-height`)
- `layout_inline_block_row`: horizontal row flow for a parent whose children are all inline / inline-block; line-wraps when a child would overflow the container width; stacks rows vertically; sets the parent's content height
- `build_layout_tree_depth`: maps `display: inline-block` → `BoxType::InlineBlock`
- `all_inline_children`: inline-block children now count as inline, triggering the row-flow path in parent blocks
- Render helpers (`get_box_opacity`, `get_border_radius`, `is_border_none`, `is_img_element`, `render_img_placeholder`, `has_overflow_hidden`, `is_visibility_hidden`, `get_color`) all handle the new `InlineBlock` variant

### Tests
- `engine_layout_test.rs`: `inline_block_tests` module — 4 new tests:
  - `test_inline_block_explicit_width_and_height`
  - `test_two_inline_blocks_side_by_side`
  - `test_inline_block_wraps_to_next_row`
  - `test_inline_block_parent_height_equals_row`

## [0.5.1] — 2026-10-23

### Added
- CSS custom properties: `--name: value` declarations stored as `Value::Keyword(raw)` in the cascade; `var(--name)` parsed as new `Value::Var(String)` variant
- Custom properties (`--*`) are inherited from parent to child automatically
- `var()` references resolved after the full cascade: each `Value::Var(name)` property is replaced with the looked-up custom property value from the element's computed map
- `Value::Var(String)` variant added to the `Value` enum in `css.rs`

### Tests
- `engine_style_test.rs`: `test_css_custom_property_resolves_color`, `test_css_custom_property_inherited_by_children`

## [0.5.0] — 2026-10-23

### Milestone: CSS Box Model & Sizing

### Added
- `box-sizing: border-box` — when set, the specified `width` is the border-box width; content width = `width - border_left - border_right - padding_left - padding_right`; `content-box` remains the default
- `min-height: Npx` — clamps block height up after `height` / IFC-derived height is computed
- `max-height: Npx` — clamps block height down; applied after `min-height`
- Both `min-height` and `max-height` also accept `%` (resolved against viewport height)

### Tests
- `engine_layout_test.rs` `box_sizing_tests`: `test_box_sizing_border_box_content_width`, `test_box_sizing_content_box_default`, `test_min_height_clamps_short_block`, `test_max_height_clamps_tall_block`

## [0.4.9] — 2026-10-23

### Added
- `:link` and `:visited` pseudo-class selector support in the CSS parser
- `pseudo: Option<String>` field added to `Selector`; stored during `parse_simple_selector` when `:` is encountered
- `specificity()` counts a pseudo-class as one class-level item `(0, 1, 0)` per pseudo
- `simple_matches()` in `style.rs` checks `:link` (matches `<a href="...">`) and `:visited` (never matches — no history in this version); unknown pseudo-classes also return false
- UA stylesheet gains `a:link{color:#0000EE;text-decoration:underline}` and `a:visited{color:#551A8B;text-decoration:underline}`

### Tests
- `engine_css_test.rs` `pseudo_class_tests`: `test_link_pseudo_parses_tag_and_pseudo`, `test_visited_pseudo_parses_correctly`, `test_pseudo_class_counts_in_specificity`, `test_no_pseudo_field_when_no_colon`
- `engine_style_test.rs` `style_tests`: `test_link_pseudo_applies_to_anchor_with_href`, `test_link_pseudo_does_not_apply_without_href`, `test_visited_pseudo_never_matches`, `test_link_pseudo_specificity_beats_plain_tag`

## [0.4.8] — 2026-10-23

### Added
- `border-radius: Npx` support: backgrounds and borders are clipped to rounded corners
- `inside_rounded_rect` geometry helper in `render.rs` — per-pixel corner test using circle distance
- `Canvas::fill_rounded_rect(rect, radius, color)` — fills within rounded shape; falls back to `fill_rect` when `radius <= 0`
- `Canvas::stroke_rounded_rect(outer, outer_r, inner, inner_r, color)` — paints only the ring between two rounded rects; used for rounded borders
- `get_border_radius(lb) -> f32` helper reads `border-radius` property (handles `Length` and bare `Number`)
- All four render paths updated: `render_background`, `render_borders`, `render_background_scrolled`, `render_borders_scrolled`

### Tests
- `engine_render_test.rs` `border_radius_tests`: `test_border_radius_clips_background_corners`, `test_border_radius_zero_fills_full_rect`, `test_border_radius_border_clips_corners`

## [0.4.7] — 2026-10-23

### Added
- `border-style: none` and `border-style: solid` support: borders are suppressed when `border-style: none` regardless of width/color; `solid` renders normally
- `border` shorthand now emits `border-style` alongside width and color (e.g. `border: 2px solid blue` → `border-style: solid`, `border: none` → `border-style: none`)
- UA stylesheet: `hr` rule updated to include `border-style: solid` so `<hr>` borders continue rendering correctly
- `is_border_none()` helper in `render.rs` gates both `render_borders` and `render_borders_scrolled`

### Tests
- `engine_css_test.rs` `border_shorthand_tests`: `test_border_shorthand_emits_style_solid`, `test_border_shorthand_emits_style_none`, `test_border_style_standalone_none`, `test_border_style_standalone_solid`
- `engine_render_test.rs` `border_style_tests`: `test_border_style_none_suppresses_border`, `test_border_style_solid_renders_border`, `test_border_none_shorthand_suppresses_border`

## [0.4.6] — 2026-10-23

### Added
- `white-space: pre` support: text with `\n` characters is split into separate line boxes; no automatic wrapping occurs (combines the newline-preservation of `pre` with nowrap semantics)
- `pre: bool` field added to `InlineFormattingContext`; when true, the layout loop splits each `InlineText` by `\n` instead of `split_whitespace`, flushes a line after each newline, and never wraps
- IFC construction in `layout_block` now decodes `"pre"` alongside `"nowrap"` from `white-space`

### Tests
- `engine_layout_test.rs` `white_space_pre_tests`: `test_pre_ifc_newline_produces_two_lines`, `test_pre_ifc_no_wrap_on_narrow_width`, `test_pre_ifc_multiple_newlines_produce_correct_line_count`, `test_white_space_pre_via_css_pipeline`

## [0.4.5] — 2026-10-23

### Added
- `white-space: nowrap` support: text inside a `nowrap` element never wraps to a new line; all words stay on a single line box regardless of container width
- `nowrap: bool` field added to `InlineFormattingContext`; the word-wrap flush in `IFC::layout` is gated on `!self.nowrap`
- `"white-space"` added to the inherited property list — descendants inherit `nowrap` unless they override

### Tests
- `engine_layout_test.rs` `white_space_tests`: `test_nowrap_ifc_produces_single_line`, `test_normal_wrap_ifc_produces_multiple_lines`, `test_white_space_nowrap_via_css_pipeline`

## [0.4.4] — 2026-10-23

### Added
- `visibility: hidden` support: elements with `visibility: hidden` are skipped during painting (background, borders, text all suppressed) but still occupy space in layout — matching CSS spec semantics
- `visibility` added to the inherited properties list; descendants inherit `hidden` unless they override with `visibility: visible`
- A `visibility: visible` child of a `visibility: hidden` parent correctly renders (the guard skips painting but still recurses into children)

### Tests
- `engine_render_test.rs` `visibility_tests`: `test_visibility_hidden_produces_no_pixels`, `test_visibility_hidden_still_occupies_space`, `test_visibility_visible_child_overrides_hidden_parent`

## [0.4.3] — 2026-10-23

### Added
- `opacity: 0.0–1.0` support: the `opacity` CSS property is now applied during rendering; background colors, border colors, and inline text colors all multiply by the element's effective opacity (parent × own), clamped to `[0.0, 1.0]`
- `with_opacity(color, opacity)` helper in `render.rs` scales a `CssColor`'s alpha channel
- Opacity cascades through the render tree: each box's effective opacity is its own `opacity` value multiplied by the parent's, matching browser compositing semantics

### Tests
- `engine_render_test.rs` `opacity_tests`: `test_opacity_zero_makes_background_invisible`, `test_opacity_half_reduces_background_alpha`

## [0.4.2] — 2026-10-23

### Added
- `<br>` forced line break: a `<br>` element in an inline context flushes the current line and starts a new one; `br{display:inline}` added to UA stylesheet
- `<hr>` horizontal rule: UA stylesheet now sets `hr{display:block;border-top-width:2px;border-color:#888888;margin-top:8px;margin-bottom:8px}` — the existing border renderer draws the visible rule
- `force_break` field on `InlineText` (internal): drives the `<br>` mechanism in `InlineFormattingContext::layout`

### Tests
- `engine_layout_test.rs`: `test_br_forces_second_line_box`, `test_br_only_produces_empty_line`, `test_hr_ua_rules_apply`

## [0.4.1] — 2026-10-23

### Added
- `<ul>` and `<ol>` list marker rendering: `ul` elements with `list-style-type:disc` inject a `•` bullet before each `li`'s inline text; `ol` elements with `list-style-type:decimal` inject `N.` numbered markers
- UA stylesheet now includes `ul{display:block;padding-left:40px;list-style-type:disc}`, `ol{display:block;padding-left:40px;list-style-type:decimal}`, `li{display:block}`
- `list-style-type` added to CSS inheritance list
- New `list_item_index` field on `LayoutBox` tracks 1-based `<li>` position within `<ol>`

### Tests
- `engine_layout_test.rs`: `test_ul_li_disc_marker_in_line_box`, `test_ol_li_decimal_marker_in_line_box`, `test_ul_li_no_marker_without_list_style`

## [0.4.0] — 2026-10-23

### Added
- UA stylesheet now includes h1–h6 block defaults: `font-weight:bold`, per-heading `font-size` (32px/24px/18px/16px/13px/11px), and top/bottom margins matching browser UA conventions
- UA stylesheet now includes `p{display:block;margin-top:16px;margin-bottom:16px}` and `body{display:block;margin:8px}` baseline block layout

### Tests
- `engine_style_test.rs`: `test_h1_ua_font_size`, `test_h2_ua_font_size`, `test_h3_ua_font_size`, `test_h1_margin_top_ua`, `test_p_ua_display_block`, `test_h1_author_font_size_overrides_ua`

## [0.3.9] — 2026-10-23

### Added
- UA stylesheet in `feather_renderer.rs` now includes `a{color:#0000EE;text-decoration:underline}` — links render blue and underlined by default, overridable by author CSS

### Tests
- `engine_style_test.rs`: `test_anchor_ua_color_applied`, `test_anchor_ua_text_decoration_applied`, `test_anchor_color_overridable_by_author`

## [0.3.8] — 2026-10-23

### Added
- `parse_rgb_function(s)` free function in `css.rs` — parses `rgb(r, g, b)` and `rgba(r, g, b, a)` into `CssColor`
- `CssParser::parse_value` now recognises `rgb()`/`rgba()` before falling back to keywords
- Alpha channel clamped to [0.0, 1.0] and converted to `u8`; integer channels clamped to [0, 255]

### Tests
- `engine_css_test.rs`: `test_rgb_color_parsed`, `test_rgba_color_parsed`, `test_rgba_fully_transparent`, `test_rgb_inline_style`

## [0.3.7] — 2026-10-23

### Added
- `apply_relative_offset()` in `layout.rs` — reads `position: relative` from styled node and shifts `dimensions.content.x/y` by `left`/`top` (or `-right`/`-bottom` when set instead)
- `position: relative` applied after `calculate_block_position` in `layout_block`; normal-flow space reserved at original coordinates

### Tests
- `engine_css_test.rs`: `test_position_relative_parsed`, `test_position_offset_top_left_parsed`
- `engine_layout_test.rs` (`position_relative_tests`): top+left offsets applied, static box unaffected, negative top shifts up

## [0.3.6] — 2026-10-22

### Added
- `TextDecoration` enum (`None`, `Underline`, `LineThrough`) in `layout.rs`
- `PlacedInlineItem::text_decoration` and `InlineText::text_decoration` fields — decoration propagates from CSS cascade through IFC layout to placed items
- `collect_inline_text` reads `text-decoration` from inline styled nodes; `layout_block` reads it from block cascade
- `render.rs`: underline drawn 1px below descent, line-through drawn at ~30% ascent height; both use item's color

### Tests
- `engine_css_test.rs`: `test_text_decoration_underline_parsed`, `test_text_decoration_line_through_parsed`, `test_text_decoration_none_parsed`
- `engine_layout_test.rs`: `test_text_decoration_underline_in_layout`, `test_text_decoration_line_through_in_layout`

## [0.3.5] — 2026-10-22

### Added
- `line-height` added to `INHERITABLE` in `style.rs` — value cascades from parent to child
- `InlineFormattingContext::line_height_override: Option<f32>` — CSS `line-height` resolved to px
  before the IFC runs; the field is `None` by default (font metrics drive line height)
- `layout_block` reads `line-height` from the block's CSS cascade:
  - `line-height: 24px` → 24 px override
  - `line-height: 1.5` (unitless) → `1.5 × font_size` px
- IFC line-height logic: `effective_lh = max(line_height_override, item_font.line_height)`
  — the override is a minimum; font metrics still win when they are taller

### Tests added
- `test_line_height_px_expands_line_box` — `line-height: 40px` produces line height ≥ 39
- `test_line_height_unitless_multiplier` — `line-height: 2` at 16 px gives line height ≥ 31

## [0.3.4] — 2026-10-22

### Added
- `NotoSans-Bold.ttf` and `NotoSans-Italic.ttf` bundled via `include_bytes!` alongside
  the existing `NotoSans-Regular.ttf`
- `FontMetrics::load_variant(size, bold, italic)` and `FontMetrics::load_cached_variant(size, bold, italic)`
  — select the bold or italic font face; `load(size)` / `load_cached(size)` remain as regular-font wrappers
- `FontMetrics::is_bold` and `FontMetrics::is_italic` fields; glyph cache key encodes font variant
  in bits 33-32 so bold and regular glyphs never collide
- `font-weight` and `font-style` added to `INHERITABLE` in `style.rs` — both properties
  now cascade from parent to child in the style tree
- `font_bold: bool` and `font_italic: bool` on `InlineText` and `PlacedInlineItem`; the
  layout pass reads `font-weight: bold` and `font-style: italic/oblique` from each styled
  node and threads the flags into the IFC and line boxes
- render loop uses `load_cached_variant` per `PlacedInlineItem` — pages using bold or italic
  CSS now paint with the correct font face

### Tests added
- `test_bold_font_loads_and_differs_from_regular` — bold 'W' advance ≥ regular 'W'
- `test_italic_font_loads_and_has_valid_metrics` — italic font loads, ascent/glyph sane
- `test_variant_cache_key_is_distinct` — regular and bold slots are independent in cache
- `test_font_weight_bold_keyword`, `test_font_weight_normal_keyword` — CSS parser emits correct Value::Keyword
- `test_font_style_italic_keyword` — CSS parser emits `italic` keyword
- `test_font_weight_bold_inherited` — `font-weight: bold` on parent div inherits to child span

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
