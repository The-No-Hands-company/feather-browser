// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PageRenderer;
use gtk4::prelude::*;
use std::cell::RefCell;

/// Test double for PageRenderer. No GTK rendering, pure in-memory state.
pub struct MockRenderer {
    url: RefCell<String>,
    suspended: RefCell<bool>,
}

impl MockRenderer {
    pub fn new(initial_url: &str) -> Self {
        Self {
            url: RefCell::new(initial_url.to_string()),
            suspended: RefCell::new(false),
        }
    }

    pub fn is_suspended(&self) -> bool {
        *self.suspended.borrow()
    }
}

impl PageRenderer for MockRenderer {
    fn load_url(&self, url: &str) {
        *self.url.borrow_mut() = url.to_string();
    }
    fn go_back(&self) {}
    fn go_forward(&self) {}
    fn reload(&self) {}
    fn stop(&self) {}
    fn current_url(&self) -> String {
        self.url.borrow().clone()
    }
    fn title(&self) -> String {
        "Mock Page".to_string()
    }
    fn can_go_back(&self) -> bool {
        false
    }
    fn can_go_forward(&self) -> bool {
        false
    }
    fn widget(&self) -> gtk4::Widget {
        gtk4::Box::new(gtk4::Orientation::Horizontal, 0).upcast()
    }
    fn set_zoom(&self, _factor: f64) {}
    fn suspend(&self) {
        *self.suspended.borrow_mut() = true;
    }
    fn resume(&self) {
        *self.suspended.borrow_mut() = false;
    }
    fn run_script(&self, _js: &str) {}
    fn connect_url_changed(&self, _f: Box<dyn Fn(String) + 'static>) {}
    fn connect_title_changed(&self, _f: Box<dyn Fn(String) + 'static>) {}
    fn connect_load_finished(&self, _f: Box<dyn Fn() + 'static>) {}
}
