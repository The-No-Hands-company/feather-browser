# Feather Browser — Architecture Design Spec

**Date:** 2026-10-20  
**Author:** Zajfan / The No Hands Company  
**Status:** Approved — ready for implementation planning  
**License:** AGPL-3.0  
**Repo:** `The-No-Hands-company/Feather-Browser`

---

## 1. Vision & Context

Feather Browser is an ultra-lightweight native browser built in Rust for Linux (and eventually all platforms). It is born from the frustration that Firefox and Chromium consume gigabytes of RAM for 10 tabs alongside normal development work. Feather targets a full 10-tab session under 300 MB.

Feather is part of the TNHC product family and will eventually be listed in the Nexus Systems ecosystem register. It is not a cloud application — it is a native client that connects to the open web and, optionally, to Nexus services. It embodies the same TNHC principles as all other TNHC products: privacy-first, free forever, no telemetry, no enshittification, built from first principles.

The existing `pronbrow`/`pronbrow-engine` project is the predecessor. Its Rust engine work (HTML5 parser, CSS3, DOM, layout) has value and will be salvaged, but the project starts clean under the Feather identity. All pronbrow files move to `legacy/` as a reference parts bin.

---

## 2. Core Architecture — Approach B (Hybrid Renderer)

Feather uses a three-layer architecture with a clean trait boundary between the UI shell and the rendering backend. This allows WebKitGTK to serve as the rendering engine today (full web compatibility immediately) while the custom Feather engine matures behind the same interface.

```
┌─────────────────────────────────────────────────┐
│  SHELL  (src/shell/)                            │
│  GTK4 window, tab strip, address bar,           │
│  settings panel, keyboard shortcuts             │
└────────────────────┬────────────────────────────┘
                     │ PageRenderer trait
┌────────────────────▼────────────────────────────┐
│  BRIDGE  (src/bridge/)                          │
│  WebKitRenderer  ←── active now                 │
│  FeatherRenderer ←── behind feature flag        │
└────────────┬───────────────────────┬────────────┘
             │                       │
┌────────────▼──────────┐ ┌──────────▼────────────┐
│  WebKitGTK6           │ │  Feather Engine        │
│  (system library)     │ │  (src/engine/)         │
│  Full web compat now  │ │  HTML/CSS/JS/DOM       │
│                       │ │  salvaged from pronbrow│
└───────────────────────┘ └───────────────────────┘
```

The shell holds `Box<dyn PageRenderer>` per tab. It never imports webkit2gtk directly — only the bridge does. When the custom engine can render a target page correctly, you activate it via a compile-time feature flag and compare output against WebKit.

---

## 3. Repository Structure

Follows the Zajfan Engineering Standard — uniform root anatomy, no ambient files, domain-first modules.

```
Feather-Browser/
├── docs/
│   ├── adr/                    # Architecture Decision Records
│   │   └── 0001-renderer-trait.md
│   └── architecture.md
├── src/
│   ├── shell/                  # GTK4 UI layer
│   │   ├── window.rs
│   │   ├── tab_manager.rs
│   │   ├── tab_bar.rs
│   │   ├── address_bar.rs
│   │   ├── toolbar.rs
│   │   └── settings.rs
│   ├── bridge/                 # Renderer abstraction
│   │   ├── mod.rs              # PageRenderer trait definition
│   │   ├── webkit_renderer.rs  # WebKit implementation
│   │   └── feather_renderer.rs # Custom engine implementation
│   ├── engine/                 # Custom Feather rendering engine
│   │   ├── html/               # HTML5 parser → DOM tree
│   │   ├── css/                # CSS3 parser + Flexbox/Grid layout
│   │   ├── dom/                # DOM tree + event system
│   │   ├── layout/             # Box model, layout tree
│   │   ├── render/             # Software rasterizer (CPU paint)
│   │   ├── network/            # HTTP fetch (reqwest)
│   │   └── assets/             # Bundled fonts (Roboto TTF)
│   └── features/
│       ├── tab_hibernation.rs
│       ├── ad_blocker.rs
│       └── privacy.rs
├── tests/
├── scripts/
│   ├── build.sh                # Canonical Linux entrypoint
│   └── build.cmd               # Canonical Windows entrypoint
├── legacy/                     # Pronbrow — reference only, never imported
├── Cargo.toml
├── CHANGELOG.md
├── README.md
└── LICENSE                     # AGPL-3.0
```

---

## 4. The PageRenderer Trait

The central seam of the architecture. Defined in `src/bridge/mod.rs`. Both `WebKitRenderer` and `FeatherRenderer` implement it. The shell only ever calls this interface.

```rust
pub trait PageRenderer {
    // Navigation
    fn load_url(&self, url: &str);
    fn go_back(&self);
    fn go_forward(&self);
    fn reload(&self);
    fn stop(&self);

    // State
    fn current_url(&self) -> String;
    fn title(&self) -> String;
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;

    // Display
    fn widget(&self) -> gtk4::Widget;   // embed into the tab pane
    fn set_zoom(&self, factor: f64);

    // Lifecycle (hibernation hooks)
    fn suspend(&self);    // drop DOM/JS state, keep URL
    fn resume(&self);     // reload from URL

    // Scripting (for Nexus integration)
    fn run_script(&self, js: &str);

    // Signals
    fn on_url_changed(&self, cb: Box<dyn Fn(&str)>);
    fn on_title_changed(&self, cb: Box<dyn Fn(&str)>);
    fn on_load_committed(&self, cb: Box<dyn Fn()>);
    fn on_load_finished(&self, cb: Box<dyn Fn()>);
}
```

**WebKitRenderer:** wraps `webkit2gtk::WebView`. `suspend()` calls `WebView::try_close()` and stashes the URL. `resume()` creates a new WebView and calls `load_url()`.

**FeatherRenderer:** wraps the custom engine pipeline. `suspend()` serializes the DOM snapshot to memory. `resume()` restores it. Active only with `--features feather-renderer`.

---

## 5. GTK4 Shell

Pure native GTK4 — no Electron, no web UI rendering itself, no npm.

**Window layout:**
```
┌─────────────────────────────────────────┐
│ [←][→][↺]  https://...  [Shield][❄]    │  ← toolbar.rs
├─────────────────────────────────────────┤
│ [Claude AI ×] [GitHub ❄ ×] [+ ]        │  ← tab_bar.rs
├─────────────────────────────────────────┤
│                                         │
│         PageRenderer::widget()          │  ← active tab's widget
│                                         │
└─────────────────────────────────────────┘
```

**Tab state machine** (owned by `tab_manager.rs`):
```
Active ──(switch away)──► Idle ──(30s timer)──► Hibernated
  ▲                                                  │
  └──────────────────(click tab)────────────────────-┘
```

- `Active`: WebView live, rendering, full RAM
- `Idle`: switched away, countdown timer running, WebView still alive
- `Hibernated`: WebView destroyed, URL stashed, snowflake badge on tab

**Settings** stored in `~/.config/feather/config.json` (plain JSON, human-readable):
- `adblock_enabled: bool`
- `auto_hibernate_delay_secs: u32` (default: 30)
- `homepage: String`
- `zoom_default: f64`
- `renderer: "webkit" | "feather"`

No dconf, no glib settings schema.

---

## 6. Features Layer

### Tab Hibernation (`src/features/tab_hibernation.rs`)
Owns a `HashMap<TabId, HibernationTimer>`. On tab switch: cancel old timer, start new countdown for tab that lost focus. On timer fire: call `renderer.suspend()`. On tab click: call `renderer.resume()`. Self-contained — shell does not orchestrate hibernation logic.

### Ad Blocker (`src/features/ad_blocker.rs`)
Hosts-file style blocklist loaded at startup from `~/.config/feather/blocklist.txt` (ships with a bundled default ~100K entries). Uses WebKit's `WebView::connect_decide_policy` signal — blocked domains get an immediate `cancel()` before any network activity. Domain lookup is a `HashSet<String>` — O(1) per request, no regex.

### Privacy (`src/features/privacy.rs`)
Two jobs:
1. **URL parameter stripping** — removes `utm_*`, `fbclid`, `gclid`, and other known tracking params before `load_url()` is called. Happens in the shell; the renderer never sees dirty URLs.
2. **Fingerprint script blocking** — a secondary blocklist (`HashSet`) of known fingerprinting/analytics endpoints (`*.doubleclick.net`, `analytics.google.com`, etc.) sharing the same WebKit intercept path as the ad blocker.

### Nexus Origin Bridge (v1 scope: minimal)
If the URL matches `*.tnhc.dev` or a configurable Nexus origin list, the ad blocker and privacy stripper are skipped. A `run_script()` bridge is exposed to those pages (and only those pages) for light integration tasks (open tab, get current URL). Full Nexus-Auth SSO integration is out of scope for v1.

---

## 7. Custom Engine — v1 Scope

The Feather engine (`src/engine/`) is feature-flagged off by default. Its v1 scope is limited to what's needed to render static HTML/CSS pages — enough to validate the pipeline and compare output against WebKit for the same page.

**In scope for v1 engine:**
- HTML5 parser → DOM tree
- CSS3 parser → computed styles (no animations)
- Flexbox layout
- Software rasterizer (CPU paint, no GPU)
- Basic HTTP fetch via reqwest

**Out of scope for v1 engine:**
- JavaScript execution (WebKit handles this)
- CSS animations and transitions
- Canvas 2D / SVG / WebGL
- WebRTC, Service Workers, IndexedDB

**JS engine note:** The pronbrow JS engine (`src/js/`) has real work in it (bytecode VM, GC, DOM bridge). It is preserved in `legacy/` and is the target for v2 engine work. It is not imported into Feather v1.

---

## 8. Legacy Salvage Plan

On day one: `git mv` all current project files into `legacy/`. Nothing from legacy is imported as a Rust dependency — it is read by the developer when writing the clean equivalent.

| Legacy module | Action |
|---|---|
| `engine/src/html/` | Rewrite into `src/engine/html/` |
| `engine/src/css/` | Rewrite into `src/engine/css/` |
| `engine/src/dom/` | Rewrite into `src/engine/dom/` |
| `engine/src/layout/` | Rewrite into `src/engine/layout/` |
| `engine/src/render/` | Rewrite into `src/engine/render/` |
| `engine/src/network/` | Rewrite into `src/engine/network/` |
| `engine/src/js/` | Preserve for v2 — do not touch in v1 |
| `engine/src/assets/` fonts | Copy directly to `src/engine/assets/` |
| `engine/src/bindings/` NAPI | Drop |
| `main.js`, `package.json`, Electron files | Drop |
| All `*.md` session/status files | Drop |

---

## 9. Build & Dependencies

```toml
# Cargo.toml [features]
default = ["webkit-renderer"]
webkit-renderer = ["dep:webkit2gtk"]
feather-renderer = []

# [dependencies]
gtk4 = "0.9"
webkit2gtk = { version = "2.0", optional = true }
reqwest = { version = "0.11", features = ["rustls-tls"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
log = "0.4"
env_logger = "0.11"
fontdue = "0.9"
```

```bash
# scripts/build.sh
cargo build --release                              # WebKit renderer (default)
cargo build --release --features feather-renderer  # custom engine
```

Single Rust binary. No npm, no Electron, no NAPI, no Node.js.

---

## 10. Milestones

| Version | Deliverable |
|---|---|
| **v0.1 — Shell** | GTK4 window, tabs, address bar, WebKit rendering, config file |
| **v0.2 — Features** | Tab hibernation, adblock, URL privacy stripping |
| **v0.3 — Engine** | HTML5+CSS3 parser wired to FeatherRenderer (static pages, feature-flagged) |
| **v0.4 — Polish** | Keyboard shortcuts, zoom, download manager, Nexus origin bridge |
| **v1.0 — Release** | Stable, packaged as .deb/.rpm/AppImage, listed on tnhc.dev |

---

## 11. TNHC Compliance

- **License:** AGPL-3.0
- **Commits:** Conventional Commits specification
- **Repo name:** `Feather-Browser` (PascalCase, TNHC standard)
- **ADRs:** `docs/adr/` — first ADR documents the renderer trait decision
- **No ambient files:** all scripts in `scripts/`, all research in `docs/`
- **Tier:** starts Tier 1 (MVP shell), targets Tier 2 (focused domain service)
- **Engineering standard:** Zajfan Standard v1.0.0 — domain-first naming, early guard clauses, deterministic resource lifetimes, `build.sh` as canonical entrypoint
- **Nexus listing:** once v0.1 ships, add entry to `Nexus-Systems/docs/NEXUS-ECOSYSTEM.md`
