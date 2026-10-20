// SPDX-License-Identifier: AGPL-3.0-or-later
mod features;

use feather::settings::Settings;
use feather::shell::window::FeatherWindow;
use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    env_logger::init();
    let app = Application::builder()
        .application_id("dev.tnhc.feather")
        .build();
    app.connect_activate(|app| {
        let settings = Settings::load();
        let window = FeatherWindow::new(app, &settings);
        window.present();
    });
    app.run();
}
