// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::settings::Settings;
use gtk4::prelude::*;
use gtk4::{Box as GBox, Button, Entry, Orientation, ToggleButton};

pub struct Toolbar {
    widget: GBox,
    url_entry: Entry,
    back_btn: Button,
    fwd_btn: Button,
    reload_btn: Button,
    adblock_toggle: ToggleButton,
    freeze_toggle: ToggleButton,
}

impl Toolbar {
    pub fn new(settings: &Settings) -> Self {
        let hbox = GBox::new(Orientation::Horizontal, 4);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);

        // Nav buttons
        let back_btn = Button::from_icon_name("go-previous-symbolic");
        let fwd_btn = Button::from_icon_name("go-next-symbolic");
        let reload_btn = Button::from_icon_name("view-refresh-symbolic");

        // URL entry
        let url_entry = Entry::new();
        url_entry.set_hexpand(true);
        url_entry.set_placeholder_text(Some("Search or enter address…"));

        // Feature toggles
        let adblock_toggle = ToggleButton::with_label("Shield");
        adblock_toggle.set_active(settings.adblock_enabled);
        let freeze_toggle = ToggleButton::with_label("❄");
        freeze_toggle.set_active(settings.auto_hibernate_delay_secs > 0);

        hbox.append(&back_btn);
        hbox.append(&fwd_btn);
        hbox.append(&reload_btn);
        hbox.append(&url_entry);
        hbox.append(&adblock_toggle);
        hbox.append(&freeze_toggle);

        Self {
            widget: hbox,
            url_entry,
            back_btn,
            fwd_btn,
            reload_btn,
            adblock_toggle,
            freeze_toggle,
        }
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.widget.clone().upcast()
    }

    pub fn set_url(&self, url: &str) {
        self.url_entry.set_text(url);
    }

    pub fn connect_navigate<F: Fn(String) + 'static>(&self, f: F) {
        self.url_entry.connect_activate(move |entry| {
            let raw = entry.text().to_string();
            let cleaned = crate::shell::address_bar::AddressBarLogic::clean_url(&raw);
            let private = crate::features::privacy::Privacy::strip_tracking_params(&cleaned);
            f(private);
        });
    }

    pub fn connect_back<F: Fn() + 'static>(&self, f: F) {
        self.back_btn.connect_clicked(move |_| f());
    }

    pub fn connect_forward<F: Fn() + 'static>(&self, f: F) {
        self.fwd_btn.connect_clicked(move |_| f());
    }

    pub fn connect_reload<F: Fn() + 'static>(&self, f: F) {
        self.reload_btn.connect_clicked(move |_| f());
    }

    pub fn connect_adblock_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.adblock_toggle
            .connect_toggled(move |btn| f(btn.is_active()));
    }

    pub fn connect_freeze_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.freeze_toggle
            .connect_toggled(move |btn| f(btn.is_active()));
    }

    pub fn adblock_enabled(&self) -> bool {
        self.adblock_toggle.is_active()
    }
    pub fn freeze_enabled(&self) -> bool {
        self.freeze_toggle.is_active()
    }
}
