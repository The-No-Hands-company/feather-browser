# Feather Browser

> Ultra-lightweight, privacy-first browser built with GTK4 and Rust.

[![CI](https://github.com/The-No-Hands-company/feather-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/The-No-Hands-company/feather-browser/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

Feather Browser is developed by [The No Hands Company](https://tnhc.dev) under the principle of **Zero Human Intervention** — software that respects your sovereignty, protects your privacy, and never works against you.

---

## ⚠️ Current State — Honest Assessment

**Feather is not a usable browser yet.** The GTK4 shell exists and compiles, and the FeatherRenderer engine can parse HTML/CSS and render text with a CPU rasterizer — but it cannot load real web pages, has no scrolling, no JavaScript, and very limited CSS coverage.

The project is in the **0.0.x engine primitive tier**: building the foundational renderer piece by piece. A browseable build is a `0.x.0` milestone goal; a full browser is `1.0.0`.

---

## Philosophy

- **Privacy is an engineering obligation**, not a checkbox.
- **No telemetry.** No analytics. No phoning home.
- **Free forever.** AGPL-3.0 — if you ship it, you open it.
- **No enshittification** — features serve the user, not advertisers.

---

## Versioning

| Tier | Meaning |
|------|---------|
| `0.0.x` | Engine primitives — renderer, layout, CSS properties, font cache, text rendering. **Current tier.** |
| `0.x.0` | Subsystem milestones — each minor bump completes a major subsystem (networking, JS engine, full CSS, images, security sandbox, web APIs). |
| `1.0.0` | Complete browser — renders any real website. HTML5, CSS3+, JavaScript, HTTP/HTTPS, security, DOM events. |
| `2.0.0+` | First new major feature added to the complete browser (e.g. privacy network, sync, extensions). |

---

## Milestone Status

| Milestone | Status |
|-----------|--------|
| **0.1.0** — GTK4 shell, tab model, WebKit rendering | ✅ Complete |
| **0.2.0** — Ad blocker, privacy URL stripping, tab hibernation | ✅ Complete |
| **0.0.6** — fontdue glyph cache, CPU rasterizer renders real text | ✅ Complete |
| **0.0.7** — CSS property inheritance, `em` unit resolution in style tree | ✅ Complete |
| **0.0.8** — `text-align: center/right`, `CssColor` constants, `pen_x` f32 accumulation | ✅ Complete |
| **0.0.9** — next engine primitive (planned: `rem`/`%` font-size, `<img>` placeholder, glyph cache LRU) | 🔜 Planned |
| **0.3.0** — First browseable pages via FeatherRenderer (networking, scrolling, full inline layout) | 🔜 Planned |

---

## Features

- **Tab management** — open, close, switch tabs with full state tracking
- **Address bar** — smart URL/search input (bare domains get `https://`, plain text goes to DuckDuckGo)
- **Ad blocker** — hosts-format blocklist, subdomain-aware, auto-seeded on first run
- **Privacy URL stripping** — removes `utm_*`, `fbclid`, `gclid`, and 15+ other tracking parameters before every navigation
- **Tab hibernation** — idle tabs are suspended after a configurable delay, freeing memory
- **WebKit rendering** — pages rendered by WebKitGTK (feature flag: `webkit-renderer`)
- **FeatherRenderer** — native HTML/CSS/layout/rasterizer pipeline, no WebKit dependency (feature flag: `feather-renderer`)
  - HTML5 parser with error recovery and void-element handling
  - CSS parser with specificity cascade, property inheritance, and `em` unit resolution
  - Block layout engine with `text-align: left/center/right`
  - Inline formatting context with glyph-accurate text placement (fontdue)
  - CPU rasterizer — software pixel buffer rendered via Cairo
  - Color constants: `CssColor::DARK`, `BLACK`, `WHITE`, `TRANSPARENT`
  - Subpixel-accurate pen advance (`f32` accumulation, not integer)
  - Referer suppressed, response capped at 10 MB, UA stylesheet hides `<head>`/`<script>` content

---

## Architecture

```
┌─────────────────────────────────────────┐
│             GTK4 Shell                  │  Window, toolbar, tab bar
├─────────────────────────────────────────┤
│          PageRenderer trait             │  Abstraction layer
├──────────────────┬──────────────────────┤
│  WebKitRenderer  │   FeatherRenderer    │  Pluggable backends
│   (0.1–0.2)      │      (0.0.6+)        │
└──────────────────┴──────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
         HTML parser     CSS parser      Style tree
              │               │          (inheritance)
              └───────────────┴───────────────┘
                              │
                       Block layout engine
                     (InlineFormattingContext)
                              │
                       CPU Rasterizer
                    (fontdue + Cairo surface)
```

The `PageRenderer` trait decouples the shell from the rendering engine entirely. Switching from WebKit to the custom FeatherRenderer is a compile-time feature flag.

**Module layout:**

```
src/
  bridge/        # PageRenderer trait + WebKitRenderer + FeatherRenderer
  engine/        # HTML parser, CSS parser, layout, rasterizer, network fetch
    dom.rs       #   DOM node tree
    html.rs      #   HTML5 parser
    css.rs       #   CSS parser + cascade + CssColor constants
    style.rs     #   Style tree builder with inheritance + em resolution
    layout.rs    #   Block layout + InlineFormattingContext + text-align
    font.rs      #   Three-level glyph cache (fontdue)
    render.rs    #   CPU rasterizer (Canvas → Cairo BGRA buffer)
    network.rs   #   HTTP/HTTPS fetch (reqwest, 10 MB cap, no Referer)
  features/      # AdBlocker, Privacy, HibernationManager
  settings.rs    # JSON config (~/.config/feather/config.json)
  shell/         # Window, toolbar, tab bar, tab manager, address bar
  main.rs        # GTK4 app entry point
  lib.rs         # Library root (for integration tests)
tests/
  engine_style_test.rs   # CSS inheritance + em resolution
  engine_layout_test.rs  # Block layout + text-align
  engine_inline_test.rs  # InlineFormattingContext
  engine_font_test.rs    # Glyph cache
  engine_render_test.rs  # CPU rasterizer + pen_x regression
```

---

## Requirements

- Rust 1.75+
- GTK4 (`libgtk-4-dev` on Debian/Ubuntu, `gtk4-devel` on Fedora)
- For `webkit-renderer` (default): `libwebkit2gtk-4.1-dev`

### Debian / Ubuntu

```sh
sudo apt-get install libgtk-4-dev libwebkit2gtk-4.1-dev
```

### Fedora

```sh
sudo dnf install gtk4-devel webkit2gtk4.1-devel
```

### Arch Linux

```sh
sudo pacman -S gtk4 webkit2gtk-4.1
```

---

## Building

```sh
# Native FeatherRenderer — no WebKit dependency required
cargo build --release --no-default-features --features feather-renderer

# WebKit renderer (default build — requires libwebkit2gtk-4.1)
cargo build --release

# Without any renderer (mock — for CI / headless testing)
cargo build --release --no-default-features
```

---

## Running

```sh
cargo run --release
```

Settings are read from `~/.config/feather/config.json` on first run. A default blocklist is written to `~/.config/feather/blocklist.txt` automatically.

---

## Testing

```sh
# Core engine tests — no system deps beyond GTK4 headers
cargo test --no-default-features

# FeatherRenderer pipeline — compile-check + run engine tests
cargo test --features feather-renderer

# Compile-check WebKit renderer
cargo check --features webkit-renderer
```

---

## Configuration

`~/.config/feather/config.json`:

```json
{
  "homepage": "https://tnhc.dev",
  "adblock_enabled": true,
  "auto_hibernate_delay_secs": 30,
  "zoom_default": 1.0,
  "renderer": "feather"
}
```

The ad blocker reads domains from `~/.config/feather/blocklist.txt` (hosts format — one domain per line, `0.0.0.0 domain.com` or bare `domain.com`).

---

## Known Limitations (engine primitive tier — 0.0.x)

- **No network loading** — FeatherRenderer cannot fetch and render live URLs yet; networking is a 0.3.0 milestone goal.
- **No scrolling** — pages taller than the viewport are clipped.
- **Inline layout is partial** — `InlineFormattingContext` handles `text-align` left/center/right, but floats, flexbox, and grid are not implemented.
- **`text-align` is not inherited** — must be set explicitly on each element; inheritance is deferred to 0.0.9.
- **No `rem` or `%` font-size** — `em` resolves correctly; `rem` falls back to 16 px base; `%` not yet converted.
- **Glyph cache is unbounded** — no LRU cap; long-running sessions accumulate glyphs. Planned fix in 0.0.9.
- **CSS coverage is limited** — shorthand properties (`margin`, `padding`, `border`), pseudo-classes, and most selectors beyond tag/class/id are not yet parsed.
- **Ad blocker blocks navigation-level requests only** — subresource blocking requires WebKit content filters, planned for a future release.
- **Shield and freeze toggles** in the toolbar are visible but not yet wired to runtime behavior.

---

## Roadmap

See [CHANGELOG.md](CHANGELOG.md) for what has shipped. Upcoming milestones:

- **0.0.9** — engine primitives: `text-align` inheritance, `rem`/`%` font-size, `<img>` placeholder, glyph cache LRU cap
- **0.3.0** — first browseable build: networking, scrolling, full inline layout
- **0.4.0** — JavaScript runtime integration
- **0.5.0** — WebKit content filters for full subresource ad/tracker blocking
- **1.0.0** — complete browser: renders any website, HTML5 + CSS3 + JS, HTTP/HTTPS, security sandbox

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Security

See [SECURITY.md](SECURITY.md) for the vulnerability disclosure policy.

---

## License

Feather Browser is licensed under the **GNU Affero General Public License v3.0 or later**.

If you distribute a modified version — including running it as a service — you must publish the source under the same license. See [LICENSE](LICENSE) for the full text.
