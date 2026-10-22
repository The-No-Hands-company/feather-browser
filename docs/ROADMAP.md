# Feather Browser — Roadmap

**Current version:** 0.2.4  
**Goal:** A complete, privacy-first, zero-telemetry browser built on a native Rust engine.

---

## Versioning philosophy

| Pattern | Meaning |
|---------|---------|
| `0.x.0` | **Subsystem milestone** — a major capability lands (networking, images, JS, etc.) |
| `0.x.y` (y > 0) | **Patches within a milestone** — bugs found after `.0`, incremental improvements, edge-case fixes. These are discovered organically; they are not pre-planned. |
| `1.0.0` | **Complete browser** — renders any real website on the open web. |

> **Note on patches:** Every minor version will produce `.1`, `.2`, `.3` etc. patches — not just 0.2.x. When `0.3.0` ships and you load real pages, you'll find redirect bugs, HTML edge cases, IFC quirks. Those become `0.3.1`, `0.3.2`, and so on. This is expected and healthy. The roadmap only pre-plans the `.0` milestones and the next block of engine primitive patches — everything else is discovered by doing.

---

## 0.2.x — Engine primitives (pre-planned through 0.2.9)

These are small, focused patches that each add one engine primitive. Each has its own design spec + implementation plan in `docs/superpowers/`.

| Version | Goal |
|---------|------|
| ✅ **0.2.1** | fontdue glyph cache, CPU rasterizer renders real glyphs |
| ✅ **0.2.2** | CSS property inheritance (`color`, `font-size`), `em` unit resolution |
| ✅ **0.2.3** | `text-align: center/right`, `CssColor` constants, `pen_x` f32 accumulation |
| ✅ **0.2.4** | `text-align` inheritance, `rem`/`%` font-size → px, `<img>` placeholder, glyph cache flush-on-overflow cap |
| 🔜 **0.2.5** | **Box model** — padding, margin, border applied from CSS cascade into layout dimensions |
| 🔜 **0.2.6** | **CSS selectors** — class (`.foo`), id (`#bar`), descendant (`div p`), child (`div > p`) |
| 🔜 **0.2.7** | **Background color** — `background-color` on block elements; CSS named color keywords |
| 🔜 **0.2.8** | **CSS shorthands** — `margin`, `padding`, `border` parsed from single-value and multi-value shorthand notation |
| 🔜 **0.2.9** | **Viewport units + constraints** — `vw`, `vh`, `min-width`, `max-width`, `overflow: hidden` |

After 0.2.9 the engine has enough CSS coverage to lay out simple styled pages correctly. That unlocks 0.3.0.

---

## 0.3.0 — First browseable build

**Milestone goal:** Type a URL, load a real page, scroll through it.

- HTTP and HTTPS network fetch (non-blocking, background thread)
- Full word-wrap `InlineFormattingContext` (multi-line text, correct line-breaking)
- Scrolling (pixel-scroll via GTK4, keyboard and mouse wheel)
- FeatherRenderer wired end-to-end: fetch → parse → style → layout → paint → GTK4 DrawingArea

Pages will look rough (no images, limited CSS coverage) but they will load and be readable. This is the threshold where the engine becomes something you can actually use.

**0.3.x patches** — discovered after 0.3.0 ships.  
Expected areas: redirect chains, malformed HTML edge cases, IFC overflow, scroll clamping, HTTPS certificate errors.

---

## 0.4.0 — Real images

**Milestone goal:** Images on pages decode and paint correctly.

- PNG, JPEG, WebP image decoding (via `image` crate or similar, no new GPU dep)
- Images sized and positioned by the layout engine (replaces the grey placeholder)
- `<picture>` and `srcset` basic support
- Correct `alt` text fallback when images fail to load

**0.4.x patches** — animated GIFs, broken URL handling, CORS, large-image memory limits, `object-fit`.

---

## 0.5.0 — Full CSS coverage

**Milestone goal:** Pages that rely on modern CSS lay out correctly.

- Flexbox layout (`display: flex`, `flex-direction`, `justify-content`, `align-items`)
- CSS Grid (at minimum: `display: grid`, `grid-template-columns/rows`, `gap`)
- Pseudo-classes: `:hover`, `:focus`, `:active`, `:visited`, `:first-child`, `:last-child`, `:nth-child`
- CSS custom properties (`--var-name`, `var()`)
- Cascade specificity and `!important` fully correct
- `z-index` and stacking contexts

**0.5.x patches** — grid edge cases, flex-wrap, specificity corner cases, pseudo-element support (`::before`, `::after`).

---

## 0.6.0 — JavaScript

**Milestone goal:** Pages that require JS to render do so.

- Embedded JS engine (QuickJS — zero-telemetry, no V8, fits the zero-footprint goal)
- DOM API: `document.getElementById`, `querySelector`, `createElement`, `appendChild`, `innerHTML`
- Basic event listeners: `addEventListener('click', ...)`, `addEventListener('keydown', ...)`
- `window.location`, `window.history` (basic navigation)
- `setTimeout` / `setInterval`

**0.6.x patches** — JS engine edge cases, missing DOM methods, event propagation bugs, `console.log` dev output.

---

## 0.7.0 — Security and privacy

**Milestone goal:** Feather is safe to use on the open web.

- HTTPS certificate validation (reject invalid/expired certs by default)
- Content Security Policy (CSP) header parsing and enforcement
- Cookie management: store, send, expire, same-site attribute
- First-party isolation (no cross-site cookie sharing)
- HTTPS-only mode (opt-in)
- Ad blocker integration with the native engine (subresource blocking, not just navigation-level)

**0.7.x patches** — CSP edge cases, cookie spec compliance, mixed-content handling, HSTS.

---

## 0.8.0 — Web APIs

**Milestone goal:** Pages that use standard web APIs work end-to-end.

- DOM Events: `click`, `input`, `submit`, `keydown`, `keyup`, `scroll`, `resize`, `DOMContentLoaded`
- `fetch()` API in JS (wired to the Rust HTTP layer)
- HTML forms: `<input>`, `<textarea>`, `<select>`, `<button>`, form submission
- `localStorage` and `sessionStorage`
- History API: `pushState`, `replaceState`, `popstate` event
- `MutationObserver` (basic)

**0.8.x patches** — form validation, CORS preflight, file inputs, `XMLHttpRequest` compatibility.

---

## 0.9.0 — Polish and completeness

**Milestone goal:** The browser is solid enough for daily use.

- Accessibility tree (AT-SPI on Linux) — screen-reader support
- Print to PDF
- Extensions API (TNHC-specific, privacy-preserving, no telemetry)
- Optional GPU compositing path (Vulkan/wgpu) for low-power devices, behind a feature flag
- Performance audit: CPU and memory profiles on the top 100 pages, fix any regressions against the near-zero footprint goal

**0.9.x patches** — a11y edge cases, PDF layout, extension API hardening.

---

## 1.0.0 — Complete browser

**Definition:** Feather renders any real website on the open web.

Checklist (all must be true):
- [ ] HTML5 parser handles real-world malformed HTML without crashing
- [ ] CSS3+ coverage passes a basic web compatibility suite
- [ ] JavaScript executes on pages that require it for rendering
- [ ] HTTP/HTTPS with correct certificate validation
- [ ] Images, video (basic), and audio (basic) load
- [ ] Forms submit and responses render
- [ ] DOM events work end-to-end
- [ ] Security: CSP, cookies, first-party isolation, HTTPS-only mode
- [ ] Accessibility tree exposed to the OS
- [ ] Memory and CPU footprint remain near-zero at idle (0.2.x baseline)
- [ ] Zero telemetry, zero analytics, zero tracking — by construction

1.0 is a real release for real users, not a milestone for developers.

---

## Beyond 1.0

Each major version adds one significant feature on top of the complete browser:

- **2.0** — Built-in privacy network (onion routing or similar, no third-party VPN)
- **3.0** — Sync (bookmarks, history, open tabs) — end-to-end encrypted, self-hostable
- **4.0** — Extension system (TNHC-native, open spec, no proprietary extension store)
