// SPDX-License-Identifier: AGPL-3.0-or-later
// tests/tab_manager_test.rs — bridge, tab model, and TabManager integration tests
// This forces the trait + mock to exist and compile
use feather::bridge::{MockRenderer, PageRenderer};
use feather::shell::tab_manager::{Tab, TabManager, TabState};

fn make_manager() -> TabManager {
    TabManager::new()
}

fn make_renderer(url: &str) -> Box<dyn PageRenderer> {
    Box::new(MockRenderer::new(url))
}

#[test]
fn test_mock_renderer_implements_trait() {
    let r: Box<dyn PageRenderer> = Box::new(MockRenderer::new("https://example.com"));
    assert_eq!(r.current_url(), "https://example.com");
    assert_eq!(r.title(), "Mock Page");
    assert!(!r.can_go_back());
}

#[test]
fn test_tab_initial_state_is_active() {
    let renderer = Box::new(MockRenderer::new("https://example.com"));
    let tab = Tab::new(1, "https://example.com", renderer);
    assert_eq!(tab.state, TabState::Active);
    assert_eq!(tab.url, "https://example.com");
}

#[test]
fn test_tab_state_transitions() {
    let renderer = Box::new(MockRenderer::new("https://a.com"));
    let mut tab = Tab::new(1, "https://a.com", renderer);
    tab.state = TabState::Idle;
    assert_eq!(tab.state, TabState::Idle);
    tab.state = TabState::Hibernated;
    assert_eq!(tab.state, TabState::Hibernated);
}

#[test]
fn test_add_tab_becomes_active() {
    let mut mgr = make_manager();
    let id = mgr.add_tab("https://tnhc.dev", make_renderer("https://tnhc.dev"));
    assert_eq!(mgr.active_tab().unwrap().id, id);
    assert_eq!(mgr.tabs().len(), 1);
}

#[test]
fn test_switch_tab_changes_active() {
    let mut mgr = make_manager();
    let id1 = mgr.add_tab("https://a.com", make_renderer("https://a.com"));
    let id2 = mgr.add_tab("https://b.com", make_renderer("https://b.com"));
    mgr.switch_to(id1);
    assert_eq!(mgr.active_tab().unwrap().id, id1);
    mgr.switch_to(id2);
    assert_eq!(mgr.active_tab().unwrap().id, id2);
}

#[test]
fn test_remove_active_tab_switches_to_neighbor() {
    let mut mgr = make_manager();
    let id1 = mgr.add_tab("https://a.com", make_renderer("https://a.com"));
    let id2 = mgr.add_tab("https://b.com", make_renderer("https://b.com"));
    mgr.switch_to(id2);
    mgr.remove_tab(id2);
    // After removing active tab, id1 should be active
    assert_eq!(mgr.active_tab().unwrap().id, id1);
    assert_eq!(mgr.tabs().len(), 1);
}

#[test]
fn test_cannot_remove_last_tab() {
    let mut mgr = make_manager();
    let id = mgr.add_tab("https://a.com", make_renderer("https://a.com"));
    mgr.remove_tab(id);
    // Still has one tab (protection against empty state)
    assert_eq!(mgr.tabs().len(), 1);
}

#[test]
fn test_switch_to_nonexistent_is_noop() {
    let mut mgr = make_manager();
    let id = mgr.add_tab("https://a.com", make_renderer("https://a.com"));
    mgr.switch_to(99999); // nonexistent
    assert_eq!(mgr.active_tab().unwrap().id, id);
}
