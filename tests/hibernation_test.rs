// SPDX-License-Identifier: AGPL-3.0-or-later
use feather::bridge::MockRenderer;
use feather::features::tab_hibernation::HibernationManager;
use feather::shell::tab_manager::{Tab, TabState};
use std::time::Duration;

fn make_tab(id: u64, url: &str) -> Tab {
    let renderer = Box::new(MockRenderer::new(url));
    Tab::new(id, url, renderer)
}

#[test]
fn test_hibernation_does_not_fire_before_delay() {
    let mut mgr = HibernationManager::new(30);
    let mut tabs = vec![make_tab(1, "https://a.com"), make_tab(2, "https://b.com")];
    mgr.on_tab_switched(Some(1), 2, &mut tabs);
    // Immediately check — tab 1 should still be Idle, not Hibernated
    mgr.check_and_hibernate(&mut tabs);
    assert_eq!(tabs[0].state, TabState::Idle);
}

#[test]
fn test_hibernation_fires_after_simulated_delay() {
    let mut mgr = HibernationManager::new_with_elapsed(Duration::from_secs(0)); // instant delay
    let mut tabs = vec![make_tab(1, "https://a.com"), make_tab(2, "https://b.com")];
    mgr.on_tab_switched(Some(1), 2, &mut tabs);
    mgr.check_and_hibernate(&mut tabs);
    assert_eq!(tabs[0].state, TabState::Hibernated);
}

#[test]
fn test_active_tab_never_hibernated() {
    let mut mgr = HibernationManager::new_with_elapsed(Duration::from_secs(0));
    let mut tabs = vec![make_tab(1, "https://a.com")];
    // Tab 1 is active — even with zero delay it must not hibernate
    mgr.on_tab_switched(None, 1, &mut tabs);
    mgr.check_and_hibernate(&mut tabs);
    assert_eq!(tabs[0].state, TabState::Active);
}

#[test]
fn test_resume_on_switch_back() {
    let mut mgr = HibernationManager::new_with_elapsed(Duration::from_secs(0));
    let mut tabs = vec![make_tab(1, "https://a.com"), make_tab(2, "https://b.com")];
    // Switch away from 1, hibernate it
    mgr.on_tab_switched(Some(1), 2, &mut tabs);
    mgr.check_and_hibernate(&mut tabs);
    assert_eq!(tabs[0].state, TabState::Hibernated);
    // Switch back to 1 — should resume
    mgr.on_tab_switched(Some(2), 1, &mut tabs);
    assert_eq!(tabs[0].state, TabState::Active);
}
