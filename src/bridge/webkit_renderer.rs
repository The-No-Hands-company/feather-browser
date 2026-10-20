// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PageRenderer;
use std::cell::RefCell;
use std::rc::Rc;
// webkit2gtk 2.0 does not expose a `prelude` module; import traits directly.
use webkit2gtk::{LoadEvent, PolicyDecisionType, SettingsExt, WebView, WebViewExt};

struct Inner {
    webview: WebView,
    suspended_url: Option<String>,
}

/// WebKit-backed PageRenderer. Wraps webkit2gtk::WebView with interior
/// mutability so all PageRenderer methods take &self.
///
/// `webkit2gtk = "2.0"` in Cargo.toml links against the `webkit2gtk-4.1`
/// system library (GTK4-native). The Rust crate exposes `gtk::Widget` (glib 0.18)
/// at the type level, but `widget()` bridges to `gtk4::Widget` via an unsafe
/// pointer cast across the glib 0.18 / glib 0.20 crate boundary — safe because
/// both versions wrap the same underlying GTK4 `GtkWidget` C struct.
#[derive(Clone)]
pub struct WebKitRenderer {
    inner: Rc<RefCell<Inner>>,
}

impl WebKitRenderer {
    // `NavigationPolicyDecisionExt::request` is deprecated since webkit2gtk 2.6;
    // we target 2.0.x where it is the correct API.
    #[allow(deprecated)]
    pub fn new() -> Self {
        let webview = WebView::new();
        // Sensible security defaults.
        if let Some(settings) = WebViewExt::settings(&webview) {
            settings.set_enable_javascript(true);
            settings.set_enable_webgl(true);
            settings.set_enable_media(true); // requires webkit2gtk feature "v2_26" (included via "v2_40")
            settings.set_allow_modal_dialogs(false);
        }
        // Wire ad blocker into WebKit policy decisions
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = format!("{}/.config/feather", home);
        crate::features::ad_blocker::AdBlocker::bundle_default_blocklist(&config_dir);
        let ad_blocker = Rc::new(crate::features::ad_blocker::AdBlocker::load(
            &format!("{}/blocklist.txt", config_dir)
        ));
        let blocker_ref = Rc::clone(&ad_blocker);
        webview.connect_decide_policy(move |_view, decision, decision_type| {
            use webkit2gtk::glib::Cast;
            use webkit2gtk::{NavigationPolicyDecisionExt, PolicyDecisionExt, URIRequestExt};
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Ok(nav) = decision.clone().downcast::<webkit2gtk::NavigationPolicyDecision>() {
                    if let Some(req) = nav.request() {
                        if let Some(uri) = req.uri() {
                            if blocker_ref.is_blocked(uri.as_str()) {
                                nav.ignore();
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });
        Self {
            inner: Rc::new(RefCell::new(Inner {
                webview,
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
        let inner = self.inner.borrow();
        // SAFETY: webkit2gtk 2.0.x links against webkit2gtk-4.1 (GTK4 C library).
        // The underlying GObject IS a GTK4 GtkWidget. The Rust crate wraps it in
        // `gtk::Widget` (glib 0.18) but both glib 0.18 and glib 0.20 represent a
        // GObject as an opaque pointer to the same C type. We cast the raw
        // *mut WebKitWebView → *mut GtkWidget (valid via C-level GObject inheritance)
        // then call `from_glib_none` which calls `g_object_ref`, giving the caller
        // an independent owning reference to the same GTK4 widget.
        unsafe {
            use gtk4::glib::translate::FromGlibPtrNone;
            use webkit2gtk::glib::translate::ToGlibPtr;
            let stash =
                ToGlibPtr::<*mut webkit2gtk::ffi::WebKitWebView>::to_glib_none(&inner.webview);
            let raw_gtk4 =
                stash.0 as *mut std::ffi::c_void as *mut gtk4::ffi::GtkWidget;
            gtk4::Widget::from_glib_none(raw_gtk4)
        }
    }

    fn set_zoom(&self, factor: f64) {
        self.inner.borrow().webview.set_zoom_level(factor);
    }

    fn suspend(&self) {
        let mut inner = self.inner.borrow_mut();
        // Guard: only stash the URL on the first suspend call. A second call
        // would otherwise overwrite suspended_url with the about:blank URI
        // loaded by the first call, causing resume() to restore the wrong page.
        if inner.suspended_url.is_none() {
            inner.suspended_url = inner.webview.uri().map(|s| s.to_string());
            inner.webview.stop_loading();
            inner.webview.load_uri("about:blank");
        }
    }

    fn resume(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(url) = inner.suspended_url.take() {
            inner.webview.load_uri(&url);
        }
    }

    fn run_script(&self, js: &str) {
        let inner = self.inner.borrow();
        // evaluate_javascript is the preferred API since WebKit 2.40 (run_javascript is deprecated).
        inner.webview.evaluate_javascript(
            js,
            None,
            None,
            webkit2gtk::gio::Cancellable::NONE,
            |_result| {},
        );
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
