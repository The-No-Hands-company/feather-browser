// SPDX-License-Identifier: AGPL-3.0-or-later
use feather::features::privacy::Privacy;

#[test]
fn test_strips_utm_params() {
    let dirty = "https://example.com/page?utm_source=newsletter&utm_medium=email&id=123";
    let clean = Privacy::strip_tracking_params(dirty);
    assert!(!clean.contains("utm_source"));
    assert!(!clean.contains("utm_medium"));
    assert!(clean.contains("id=123"));
}

#[test]
fn test_strips_fbclid() {
    let dirty = "https://example.com/?fbclid=AbCdEfGh123";
    let clean = Privacy::strip_tracking_params(dirty);
    assert!(!clean.contains("fbclid"));
}

#[test]
fn test_strips_gclid() {
    let dirty = "https://example.com/article?gclid=xyz&page=1";
    let clean = Privacy::strip_tracking_params(dirty);
    assert!(!clean.contains("gclid"));
    assert!(clean.contains("page=1"));
}

#[test]
fn test_clean_url_unchanged() {
    let url = "https://tnhc.dev/projects?category=rust";
    assert_eq!(Privacy::strip_tracking_params(url), url);
}

#[test]
fn test_fragment_preserved() {
    let url = "https://example.com/page?utm_source=x#section-2";
    let clean = Privacy::strip_tracking_params(url);
    assert!(clean.contains("#section-2"));
    assert!(!clean.contains("utm_source"));
}

#[test]
fn test_all_params_tracking_removes_query_string() {
    let dirty = "https://example.com/?utm_source=x&utm_medium=y";
    let clean = Privacy::strip_tracking_params(dirty);
    assert_eq!(clean, "https://example.com/");
}

#[test]
fn test_malformed_url_returned_unchanged() {
    let bad = "not-a-url";
    assert_eq!(Privacy::strip_tracking_params(bad), bad);
}

#[test]
fn test_unknown_utm_param_stripped_by_prefix() {
    let dirty = "https://example.com/?utm_brand=x&id=42";
    let clean = Privacy::strip_tracking_params(dirty);
    assert!(!clean.contains("utm_brand"));
    assert!(clean.contains("id=42"));
}

#[test]
fn test_case_insensitive_tracking_param_stripped() {
    let dirty = "https://example.com/?FBCLID=abc&page=1";
    let clean = Privacy::strip_tracking_params(dirty);
    assert!(!clean.to_lowercase().contains("fbclid"));
    assert!(clean.contains("page=1"));
}

#[test]
fn test_ref_param_not_stripped() {
    // `ref` is a legitimate parameter used by GitHub/GitLab for branch references.
    let url = "https://github.com/org/repo/blob/main/file.txt?ref=main";
    let clean = Privacy::strip_tracking_params(url);
    assert_eq!(clean, url, "?ref=main must not be stripped");
}
