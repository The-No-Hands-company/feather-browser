// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::shell::tab_manager::{Tab, TabId, TabState};
use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, Label, Orientation};

pub struct TabBar {
    widget: GBox,
}

impl TabBar {
    pub fn new() -> Self {
        let hbox = GBox::new(Orientation::Horizontal, 2);
        hbox.set_margin_start(4);
        Self { widget: hbox }
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.widget.clone().upcast()
    }

    /// Redraws the entire tab strip from the current tabs slice.
    /// Call this after any TabManager mutation.
    pub fn render<F1, F2, F3>(
        &self,
        tabs: &[Tab],
        active_id: Option<TabId>,
        on_click: F1,
        on_close: F2,
        on_new: F3,
    ) where
        F1: Fn(TabId) + Clone + 'static,
        F2: Fn(TabId) + Clone + 'static,
        F3: Fn() + 'static,
    {
        // Clear existing children
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }

        for tab in tabs {
            let tab_id = tab.id;
            let is_active = Some(tab_id) == active_id;
            let is_hibernated = tab.state == TabState::Hibernated;

            let tab_box = GBox::new(Orientation::Horizontal, 2);
            let title = if is_hibernated {
                format!("❄ {}", &tab.title.chars().take(20).collect::<String>())
            } else {
                tab.title.chars().take(20).collect::<String>()
            };
            let label = Label::new(Some(if title.is_empty() { "New Tab" } else { &title }));
            let close_btn = Button::from_icon_name("window-close-symbolic");
            close_btn.set_margin_start(4);

            let click_cb = on_click.clone();
            let tab_btn = Button::new();
            tab_btn.set_child(Some(&label));
            if is_active {
                tab_btn.add_css_class("suggested-action");
            }
            tab_btn.connect_clicked(move |_| click_cb(tab_id));

            let close_cb = on_close.clone();
            close_btn.connect_clicked(move |_| close_cb(tab_id));

            tab_box.append(&tab_btn);
            tab_box.append(&close_btn);
            self.widget.append(&tab_box);
        }

        // New tab button
        let new_btn = Button::from_icon_name("tab-new-symbolic");
        let new_cb = on_new;
        new_btn.connect_clicked(move |_| new_cb());
        self.widget.append(&new_btn);
    }
}
