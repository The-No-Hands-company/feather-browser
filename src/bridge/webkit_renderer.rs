// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PageRenderer;
use std::cell::RefCell;
use std::rc::Rc;
// webkit2gtk 2.0 does not expose a `prelude` module; import traits directly.
use webkit2gtk::{LoadEvent, SettingsExt, WebView, WebViewExt};

struct Inner {
    webview: WebView,
    /// GTK4 placeholder widget.
    ///
    /// webkit2gtk 2.0 targets GTK3 internally, so its `WebView` cannot be
    /// directly embedded in a GTK4 widget hierarchy. This placeholder is
    /// returned by `widget()` until the project migrates to webkit2gtk-6.0
    /// (the GTK4-native binding) or a GTK3/GTK4 bridge is in place.
    // TODO: replace with actual WebKit GTK4 widget when webkit2gtk-6.0 is adopted.
    placeholder: gtk4::Box,
    suspended_url: Option<String>,
}

/// WebKit-backed PageRenderer. Wraps webkit2gtk::WebView with interior
/// mutability so all PageRenderer methods take &self.
///
/// **Compatibility note**: webkit2gtk 2.0 is GTK3-based; the `widget()` method
/// returns a GTK4 placeholder box. All navigation and scripting calls are
/// forwarded to the underlying WebView. Upgrade to webkit2gtk-6.0 for full
/// GTK4 widget integration.
#[derive(Clone)]
pub struct WebKitRenderer {
    inner: Rc<RefCell<Inner>>,
}

impl WebKitRenderer {
    pub fn new() -> Self {
        let webview = WebView::new();
        // Sensible security defaults.
        if let Some(settings) = WebViewExt::settings(&webview) {
            settings.set_enable_javascript(true);
            settings.set_enable_webgl(true);
            // TODO: set_enable_media requires webkit2gtk feature "v2_26"; add it to
            // Cargo.toml's webkit2gtk features = ["v2_26"] to enable this call.
            settings.set_allow_modal_dialogs(false);
        }
        // TODO: replace with a real GTK4-native WebKit widget once webkit2gtk-6.0 is used.
        let placeholder = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        Self {
            inner: Rc::new(RefCell::new(Inner {
                webview,
                placeholder,
                suspended_url: None,
            })),
        }
    }
}

impl Default for WebKitRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl PageRenderer for WebKitRenderer {
    fn load_url(&self, url: &str) {
        self.inner.borrow().webview.load_uri(url);
    }

    fn go_back(&self) {
        self.inner.borrow().webview.go_back();
    }

    fn go_forward(&self) {
        self.inner.borrow().webview.go_forward();
    }

    fn reload(&self) {
        self.inner.borrow().webview.reload();
    }

    fn stop(&self) {
        self.inner.borrow().webview.stop_loading();
    }

    fn current_url(&self) -> String {
        self.inner
            .borrow()
            .webview
            .uri()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    fn title(&self) -> String {
        self.inner
            .borrow()
            .webview
            .title()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    fn can_go_back(&self) -> bool {
        self.inner.borrow().webview.can_go_back()
    }

    fn can_go_forward(&self) -> bool {
        self.inner.borrow().webview.can_go_forward()
    }

    fn widget(&self) -> gtk4::Widget {
        // TODO: webkit2gtk 2.0 targets GTK3; cannot directly embed its WebView in GTK4.
        // Returns a GTK4 Box placeholder. Replace when webkit2gtk-6.0 is adopted.
        use gtk4::prelude::Cast;
        self.inner.borrow().placeholder.clone().upcast()
    }

    fn set_zoom(&self, factor: f64) {
        self.inner.borrow().webview.set_zoom_level(factor);
    }

    fn suspend(&self) {
        let mut inner = self.inner.borrow_mut();
        let url = inner.webview.uri().map(|s| s.to_string());
        inner.suspended_url = url;
        inner.webview.stop_loading();
        inner.webview.load_uri("about:blank");
    }

    fn resume(&self) {
        let inner = self.inner.borrow();
        if let Some(ref url) = inner.suspended_url {
            inner.webview.load_uri(url);
        }
    }

    fn run_script(&self, js: &str) {
        let inner = self.inner.borrow();
        // webkit2gtk 2.0 uses `run_javascript`; `evaluate_javascript` requires v2_40.
        inner
            .webview
            .run_javascript(js, webkit2gtk::gio::Cancellable::NONE, |_result| {});
    }

    fn connect_url_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        let inner = self.inner.borrow();
        inner.webview.connect_uri_notify(move |view| {
            if let Some(uri) = view.uri() {
                f(uri.to_string());
            }
        });
    }

    fn connect_title_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        let inner = self.inner.borrow();
        inner.webview.connect_title_notify(move |view| {
            if let Some(title) = view.title() {
                f(title.to_string());
            }
        });
    }

    fn connect_load_finished(&self, f: Box<dyn Fn() + 'static>) {
        let inner = self.inner.borrow();
        inner.webview.connect_load_changed(move |_view, event| {
            if event == LoadEvent::Finished {
                f();
            }
        });
    }
}
