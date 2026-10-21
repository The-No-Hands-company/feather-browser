# Feather Browser

> Ultra-lightweight, privacy-first browser built with GTK4 and Rust.

[![CI](https://github.com/The-No-Hands-company/feather-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/The-No-Hands-company/feather-browser/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

Feather Browser is developed by [The No Hands Company](https://tnhc.dev) under the principle of **Zero Human Intervention** — software that respects your sovereignty, protects your privacy, and never works against you.

---

## Philosophy

- **Privacy is an engineering obligation**, not a checkbox.
- **No telemetry.** No analytics. No phoning home.
- **Free forever.** AGPL-3.0 — if you ship it, you open it.
- **No enshittification** — features serve the user, not advertisers.

---

## Status

| Milestone | Status |
|-----------|--------|
| v0.1 — GTK4 shell, tabs, WebKit rendering | ✅ Complete |
| v0.2 — Ad blocker, privacy URL stripping, tab hibernation | ✅ Complete |
| v0.3 — FeatherRenderer (custom engine, no WebKit) | ✅ Complete |
| v0.4 — Async fetch, text rendering, scroll, full CSS | 🔜 Next |

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
  - CSS parser with specificity cascade
  - Block layout engine
  - CPU rasterizer — software pixel buffer rendered via Cairo
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
│   (v0.1–v0.2)    │      (v0.3+)         │
└──────────────────┴──────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
         HTML parser     CSS parser      Layout engine
              │               │               │
              └───────────────┴───────────────┘
                              │
                       CPU Rasterizer
                       (Cairo surface)
```

The `PageRenderer` trait decouples the shell from the rendering engine entirely. Switching from WebKit to the custom FeatherRenderer is a compile-time feature flag.

**Module layout:**

```
src/
  bridge/        # PageRenderer trait + WebKitRenderer + FeatherRenderer
  engine/        # HTML parser, CSS parser, layout, rasterizer, network fetch
    dom.rs       #   DOM node tree
    html.rs      #   HTML5 parser
    css.rs       #   CSS parser + cascade
    style.rs     #   Style tree builder
    layout.rs    #   Block layout engine
    render.rs    #   CPU rasterizer (Canvas → Cairo BGRA buffer)
    network.rs   #   HTTP/HTTPS fetch (reqwest, 10 MB cap, no Referer)
  features/      # AdBlocker, Privacy, HibernationManager
  settings.rs    # JSON config (~/.config/feather/config.json)
  shell/         # Window, toolbar, tab bar, tab manager, address bar
  main.rs        # GTK4 app entry point
  lib.rs         # Library root (for integration tests)
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
# Default build (includes WebKit renderer)
cargo build --release

# Native FeatherRenderer — no WebKit dependency required
cargo build --release --no-default-features --features feather-renderer

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
# Full test suite — 36 tests, no system deps needed beyond GTK4 headers
cargo test --no-default-features

# Compile-check the FeatherRenderer pipeline
cargo check --features feather-renderer

# Compile-check the WebKit renderer
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
  "renderer": "webkit"
}
```

The ad blocker reads domains from `~/.config/feather/blocklist.txt` (hosts format — one domain per line, `0.0.0.0 domain.com` or bare `domain.com`).

---

## Known Limitations (v0.3)

- **FeatherRenderer renders block layout only** — floats, flexbox, grid, and inline text flow are not yet implemented.
- **Text is rendered as grey rectangles** — font rasterization (fontdue integration) is deferred to v0.4.
- **No scrolling** — pages taller than the viewport are clipped.
- **CSS coverage is limited** — shorthand properties (`margin`, `padding`, `border`), pseudo-classes, and most selectors beyond tag/class/id are not yet parsed.
- **Ad blocker blocks navigation-level requests only** — subresource loads (scripts, images, XHR) require WebKit content filters, planned for a future release.
- **Tab hibernation timer fires every 10s** but `on_tab_switched` is not yet wired to tab bar clicks — hibernation is partially functional.
- **Shield and freeze toggles** in the toolbar are visible but not yet wired to runtime behavior.

---

## Roadmap

- **v0.3** — FeatherRenderer: custom layout/paint engine replacing WebKit entirely ✅
- **v0.4** — Async fetch pipeline, font rasterization, scrolling, inline layout, broader CSS support
- **v0.5** — WebKit content filters for full subresource ad/tracker blocking
- **v0.6** — Ephemeral mode, per-site permissions, first-party isolation

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
