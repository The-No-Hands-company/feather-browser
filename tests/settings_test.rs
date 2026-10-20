// SPDX-License-Identifier: AGPL-3.0-or-later
// The crate is bin-only (no lib target), so the settings module is included
// directly by path rather than imported as `feather::settings`.
#[path = "../src/settings.rs"]
#[allow(dead_code)]
mod settings;

use settings::Settings;
use tempfile::TempDir;

#[test]
fn test_load_returns_defaults_when_no_file() {
    let s = Settings::load_from("/nonexistent/path/config.json");
    assert_eq!(s.adblock_enabled, true);
    assert_eq!(s.auto_hibernate_delay_secs, 30);
    assert_eq!(s.zoom_default, 1.0);
    assert_eq!(s.renderer, "webkit");
}

#[test]
fn test_save_and_load_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    let path_str = path.to_str().unwrap();

    let mut s = Settings::load_from(path_str);
    s.adblock_enabled = false;
    s.auto_hibernate_delay_secs = 60;
    s.homepage = "https://tnhc.dev".to_string();
    s.save_to(path_str).unwrap();

    let loaded = Settings::load_from(path_str);
    assert_eq!(loaded.adblock_enabled, false);
    assert_eq!(loaded.auto_hibernate_delay_secs, 60);
    assert_eq!(loaded.homepage, "https://tnhc.dev");
}

#[test]
fn test_load_ignores_unknown_fields() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"adblock_enabled":false,"unknown_future_field":42}"#).unwrap();
    let s = Settings::load_from(path.to_str().unwrap());
    assert_eq!(s.adblock_enabled, false);
    // unknown field silently ignored, no panic
}
