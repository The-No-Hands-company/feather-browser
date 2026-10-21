// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::shell::tab_manager::{Tab, TabId, TabState};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct HibernationManager {
    delay: Duration,
    idle_since: HashMap<TabId, Instant>,
    simulated_elapsed: Option<Duration>, // for tests only
    enabled: bool,
}

impl HibernationManager {
    pub fn new(delay_secs: u32) -> Self {
        Self {
            delay: Duration::from_secs(delay_secs as u64),
            idle_since: HashMap::new(),
            simulated_elapsed: None,
            enabled: delay_secs > 0,
        }
    }

    /// Test-only constructor: treats every recorded idle tab as immediately overdue.
    /// Do NOT call from production code — it sets delay to zero, hibernating all idle
    /// tabs on the first check regardless of how long they have been idle.
    pub fn new_with_elapsed(_elapsed: Duration) -> Self {
        Self {
            delay: Duration::from_secs(0),
            idle_since: HashMap::new(),
            simulated_elapsed: Some(Duration::from_secs(999)),
            enabled: true,
        }
    }

    /// Enables or disables the hibernation timer at runtime (wired to the Freeze toggle).
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Call on every tab switch. Marks old tab as Idle, resumes new tab if Hibernated.
    pub fn on_tab_switched(&mut self, old_id: Option<TabId>, new_id: TabId, tabs: &mut Vec<Tab>) {
        // Mark old tab as Idle, start idle timer
        if let Some(old) = old_id {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == old) {
                if tab.state == TabState::Active {
                    tab.state = TabState::Idle;
                    self.idle_since.insert(old, Instant::now());
                }
            }
        }
        // Resume new tab if it was hibernated
        if let Some(tab) = tabs.iter_mut().find(|t| t.id == new_id) {
            if tab.state == TabState::Hibernated {
                tab.renderer.resume();
            }
            tab.state = TabState::Active;
            self.idle_since.remove(&new_id);
        }
    }

    /// Call periodically (or after a timer) to hibernate overdue idle tabs.
    pub fn check_and_hibernate(&mut self, tabs: &mut Vec<Tab>) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        let overdue: Vec<TabId> = self
            .idle_since
            .iter()
            .filter(|(_, &since)| {
                if self.simulated_elapsed.is_some() {
                    true
                } else {
                    now.duration_since(since) >= self.delay
                }
            })
            .map(|(&id, _)| id)
            .collect();

        for id in overdue {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == id && t.state == TabState::Idle) {
                tab.renderer.suspend();
                tab.state = TabState::Hibernated;
                self.idle_since.remove(&id);
            }
        }
    }
}
