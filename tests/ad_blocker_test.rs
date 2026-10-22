// SPDX-License-Identifier: AGPL-3.0-or-later
use feather::features::ad_blocker::AdBlocker;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_known_ad_domain_is_blocked() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("blocklist.txt");
    fs::write(
        &path,
        "0.0.0.0 doubleclick.net\n0.0.0.0 googleadservices.com\n",
    )
    .unwrap();
    let blocker = AdBlocker::load(path.to_str().unwrap());
    assert!(blocker.is_blocked("https://doubleclick.net/ad/123"));
    assert!(blocker.is_blocked("https://googleadservices.com/pagead/"));
}

#[test]
fn test_nexus_domain_is_not_blocked() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("blocklist.txt");
    fs::write(&path, "0.0.0.0 doubleclick.net\n").unwrap();
    let blocker = AdBlocker::load(path.to_str().unwrap());
    assert!(!blocker.is_blocked("https://tnhc.dev"));
    assert!(!blocker.is_blocked("https://nexus.tnhc.dev/api"));
}

#[test]
fn test_malformed_url_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("blocklist.txt");
    fs::write(&path, "0.0.0.0 doubleclick.net\n").unwrap();
    let blocker = AdBlocker::load(path.to_str().unwrap());
    // Should return false, not panic
    assert!(!blocker.is_blocked("not-a-url"));
    assert!(!blocker.is_blocked(""));
    assert!(!blocker.is_blocked("://missing-scheme"));
}

#[test]
fn test_missing_blocklist_file_creates_empty_blocker() {
    let blocker = AdBlocker::load("/nonexistent/path/blocklist.txt");
    assert!(!blocker.is_blocked("https://example.com"));
}

#[test]
fn test_subdomain_of_blocked_domain_is_blocked() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("blocklist.txt");
    fs::write(&path, "0.0.0.0 doubleclick.net\n").unwrap();
    let blocker = AdBlocker::load(path.to_str().unwrap());
    assert!(blocker.is_blocked("https://stats.doubleclick.net/foo"));
}
