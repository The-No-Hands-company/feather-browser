// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::settings::Settings;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GBox, Orientation, Stack};

pub struct FeatherWindow {
    window: ApplicationWindow,
    content_stack: Stack,
}

impl FeatherWindow {
    pub fn new(app: &Application, settings: &Settings) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Feather")
            .default_width(1280)
            .default_height(800)
            .build();

        let vbox = GBox::new(Orientation::Vertical, 0);
        let content_stack = Stack::new();
        content_stack.set_vexpand(true);
        content_stack.set_hexpand(true);

        // Placeholder label until toolbar/tab_bar added in Task 9
        let placeholder = gtk4::Label::new(Some("Feather Browser — v0.1"));
        placeholder.set_vexpand(true);
        content_stack.add_named(&placeholder, Some("placeholder"));

        vbox.append(&content_stack);
        window.set_child(Some(&vbox));

        // Open homepage tab
        let _ = settings; // used in Task 9 for homepage URL

        Self { window, content_stack }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
