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
