// SPDX-License-Identifier: AGPL-3.0-or-later
// FeatherRenderer: custom engine backend (stub — full implementation in v0.3 plan)
use super::PageRenderer;
use gtk4::prelude::*;
use std::cell::RefCell;

pub struct FeatherRenderer {
    url: RefCell<String>,
}

impl FeatherRenderer {
    pub fn new() -> Self {
        Self { url: RefCell::new(String::new()) }
    }
}

impl Default for FeatherRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl PageRenderer for FeatherRenderer {
    fn load_url(&self, url: &str) {
        log::info!("[FeatherRenderer] load_url: {}", url);
        *self.url.borrow_mut() = url.to_string();
    }
    fn go_back(&self) { log::info!("[FeatherRenderer] go_back (stub)"); }
    fn go_forward(&self) { log::info!("[FeatherRenderer] go_forward (stub)"); }
    fn reload(&self) { log::info!("[FeatherRenderer] reload (stub)"); }
    fn stop(&self) {}
    fn current_url(&self) -> String { self.url.borrow().clone() }
    fn title(&self) -> String { format!("FeatherRenderer — {}", self.url.borrow()) }
    fn can_go_back(&self) -> bool { false }
    fn can_go_forward(&self) -> bool { false }
    fn widget(&self) -> gtk4::Widget {
        let label = gtk4::Label::new(Some(
            "Feather Native Engine\n(v0.3 plan — not yet implemented)"
        ));
        label.set_vexpand(true);
        label.upcast()
    }
    fn set_zoom(&self, _factor: f64) {}
    fn suspend(&self) {}
    fn resume(&self) {}
    fn run_script(&self, _js: &str) {}
    fn connect_url_changed(&self, _f: Box<dyn Fn(String) + 'static>) {}
    fn connect_title_changed(&self, _f: Box<dyn Fn(String) + 'static>) {}
    fn connect_load_finished(&self, _f: Box<dyn Fn() + 'static>) {}
}
