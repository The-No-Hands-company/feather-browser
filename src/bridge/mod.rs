// SPDX-License-Identifier: AGPL-3.0-or-later

/// The central rendering abstraction. Both WebKitRenderer and FeatherRenderer
/// implement this. The shell holds Box<dyn PageRenderer> per tab and never
/// imports webkit2gtk directly.
pub trait PageRenderer {
    // Navigation
    fn load_url(&self, url: &str);
    fn go_back(&self);
    fn go_forward(&self);
    fn reload(&self);
    fn stop(&self);

    // State queries
    fn current_url(&self) -> String;
    fn title(&self) -> String;
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;

    // GTK widget — embed into the tab content area
    fn widget(&self) -> gtk4::Widget;

    fn set_zoom(&self, factor: f64);

    // Hibernation lifecycle — called by HibernationManager
    fn suspend(&self);   // drops DOM/JS state, stashes URL
    fn resume(&self);    // reloads from stashed URL

    // Nexus integration (v1: Nexus-origin pages only)
    fn run_script(&self, js: &str);

    // Signal connectors — call once after construction
    fn connect_url_changed(&self, f: Box<dyn Fn(String) + 'static>);
    fn connect_title_changed(&self, f: Box<dyn Fn(String) + 'static>);
    fn connect_load_finished(&self, f: Box<dyn Fn() + 'static>);
}

// Re-export concrete types so callers only import bridge::*
#[cfg(feature = "webkit-renderer")]
pub use webkit_renderer::WebKitRenderer;

#[cfg(feature = "feather-renderer")]
pub use feather_renderer::FeatherRenderer;

pub use mock_renderer::MockRenderer;

mod mock_renderer;

#[cfg(feature = "webkit-renderer")]
pub mod webkit_renderer;

#[cfg(feature = "feather-renderer")]
pub mod feather_renderer;
