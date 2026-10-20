# Changelog

All notable changes to Feather Browser are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versioning: [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

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
