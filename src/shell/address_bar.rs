// SPDX-License-Identifier: AGPL-3.0-or-later

/// Pure URL-processing logic — no GTK dependency, fully unit-testable.
pub struct AddressBarLogic;

impl AddressBarLogic {
    /// Converts raw address bar input to a loadable URL.
    /// - Trims whitespace
    /// - Adds https:// if no scheme present and input looks like a domain
    /// - Falls back to DuckDuckGo search for everything else
    pub fn clean_url(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with("https://")
            || trimmed.starts_with("http://")
            || trimmed.starts_with("about:")
            || trimmed.starts_with("file://")
        {
            return trimmed.to_string();
        }
        // Looks like a domain if it contains a dot and no spaces
        if !trimmed.contains(' ') && trimmed.contains('.') {
            return format!("https://{}", trimmed);
        }
        // Treat as search query
        let encoded = trimmed.replace(' ', "+");
        format!("https://duckduckgo.com/?q={}", encoded)
    }
}
