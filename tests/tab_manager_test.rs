// SPDX-License-Identifier: AGPL-3.0-or-later
// tests/tab_manager_test.rs — bridge, tab model, and TabManager integration tests
// This forces the trait + mock to exist and compile
use feather::bridge::{MockRenderer, PageRenderer};
use feather::shell::tab_manager::{Tab, TabState};

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
