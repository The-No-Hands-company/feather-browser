// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::bridge::PageRenderer;

pub type TabId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabState {
    Active,
    Idle,
    Hibernated,
}

pub struct Tab {
    pub id: TabId,
    pub url: String,
    pub title: String,
    pub state: TabState,
    pub renderer: Box<dyn PageRenderer>,
}

impl Tab {
    pub fn new(id: TabId, url: &str, renderer: Box<dyn PageRenderer>) -> Self {
        Self {
            id,
            url: url.to_string(),
            title: String::new(),
            state: TabState::Active,
            renderer,
        }
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_id: Option<TabId>,
    next_id: TabId,
}

impl TabManager {
    pub fn new() -> Self {
        Self { tabs: Vec::new(), active_id: None, next_id: 1 }
    }

    /// Adds a new tab, makes it active, returns its id.
    pub fn add_tab(&mut self, url: &str, renderer: Box<dyn PageRenderer>) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.push(Tab::new(id, url, renderer));
        self.active_id = Some(id);
        id
    }

    /// Switches active tab. Noop if id not found.
    pub fn switch_to(&mut self, id: TabId) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_id = Some(id);
        }
    }

    /// Removes tab by id. Guards against removing the last tab.
    /// If active tab removed, switches to the nearest remaining tab.
    pub fn remove_tab(&mut self, id: TabId) {
        if self.tabs.len() <= 1 {
            return;
        }
        let pos = match self.tabs.iter().position(|t| t.id == id) {
            Some(p) => p,
            None => return,
        };
        self.tabs.remove(pos);
        if self.active_id == Some(id) {
            let new_pos = pos.saturating_sub(1).min(self.tabs.len() - 1);
            self.active_id = self.tabs.get(new_pos).map(|t| t.id);
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        let id = self.active_id?;
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let id = self.active_id?;
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        &mut self.tabs
    }

    pub fn get_tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }
}
