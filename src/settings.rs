// SPDX-License-Identifier: AGPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub adblock_enabled: bool,
    pub auto_hibernate_delay_secs: u32,
    pub homepage: String,
    pub zoom_default: f64,
    pub renderer: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adblock_enabled: true,
            auto_hibernate_delay_secs: 30,
            homepage: "https://tnhc.dev".to_string(),
            zoom_default: 1.0,
            renderer: "webkit".to_string(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &str) -> Self {
        let text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };
        match serde_json::from_str(&text) {
            Ok(settings) => settings,
            Err(e) => {
                log::warn!("Corrupt config at {}: {}. Using defaults.", path, e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn config_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.config/feather/config.json", home)
}
