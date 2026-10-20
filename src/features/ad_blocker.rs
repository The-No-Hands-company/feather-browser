// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashSet;
use std::fs;

pub struct AdBlocker {
    blocked_domains: HashSet<String>,
}

impl AdBlocker {
    /// Loads a hosts-format blocklist. Returns empty blocker if file missing.
    pub fn load(path: &str) -> Self {
        let blocked_domains = fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .filter_map(parse_hosts_line)
            .collect();
        Self { blocked_domains }
    }

    /// Returns true if the URL's domain (or any parent domain) is in the blocklist.
    pub fn is_blocked(&self, url: &str) -> bool {
        let domain = match extract_domain(url) {
            Some(d) => d,
            None => return false,
        };
        // Check domain and all parent domains (e.g. stats.doubleclick.net → doubleclick.net)
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let candidate = parts[i..].join(".");
            if self.blocked_domains.contains(&candidate) {
                return true;
            }
        }
        false
    }

    /// Writes a minimal default blocklist to config_dir/blocklist.txt if absent.
    pub fn bundle_default_blocklist(config_dir: &str) {
        let path = format!("{}/blocklist.txt", config_dir);
        if std::path::Path::new(&path).exists() { return; }
        let _ = fs::create_dir_all(config_dir);
        let default = include_str!("../../assets/default_blocklist.txt");
        let _ = fs::write(&path, default);
    }
}

fn parse_hosts_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with('#') || line.is_empty() { return None; }
    // Hosts format: "0.0.0.0 domain.com" or just "domain.com"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        Some(parts[1].to_lowercase())
    } else if parts.len() == 1 {
        Some(parts[0].to_lowercase())
    } else {
        None
    }
}

fn extract_domain(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    // Strip userinfo (user:pass@host)
    let after_userinfo = match after_scheme.split_once('@') {
        Some((_, rest)) => rest,
        None => after_scheme,
    };
    // Take up to the first of '/', '?', '#'
    let host = after_userinfo
        .split(|c| c == '/' || c == '?' || c == '#')
        .next()?;
    // Strip port
    let host = host.split(':').next()?;
    // Strip trailing dot (FQDN)
    let host = host.trim_end_matches('.');
    if host.is_empty() { return None; }
    Some(host.to_lowercase())
}
