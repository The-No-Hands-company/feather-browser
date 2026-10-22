// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PageRenderer;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{LoadEvent, PolicyDecisionType, WebView};

struct Inner {
    webview: WebView,
    suspended_url: Option<String>,
}

/// WebKit-backed PageRenderer. Wraps webkit6::WebView (GTK4-native).
#[derive(Clone)]
pub struct WebKitRenderer {
    inner: Rc<RefCell<Inner>>,
}

impl WebKitRenderer {
    pub fn new(adblock_enabled: Rc<Cell<bool>>) -> Self {
        let webview = WebView::new();
        if let Some(settings) = WebViewExt::settings(&webview) {
            settings.set_enable_javascript(true);
            settings.set_enable_webgl(true);
            settings.set_allow_modal_dialogs(false);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = format!("{}/.config/feather", home);
        crate::features::ad_blocker::AdBlocker::bundle_default_blocklist(&config_dir);
        let ad_blocker = Rc::new(crate::features::ad_blocker::AdBlocker::load(&format!(
            "{}/blocklist.txt",
            config_dir
        )));
        let blocker_ref = Rc::clone(&ad_blocker);
        webview.connect_decide_policy(move |_view, decision, decision_type| {
            if !adblock_enabled.get() {
                return false;
            }
            if decision_type == PolicyDecisionType::NavigationAction {
                if let Ok(nav) = decision
                    .clone()
                    .downcast::<webkit6::NavigationPolicyDecision>()
                {
                    if let Some(req) = nav.navigation_action().and_then(|a| a.request()) {
                        if let Some(uri) = req.uri() {
                            if blocker_ref.is_blocked(uri.as_str()) {
                                PolicyDecisionExt::ignore(&nav);
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
        Self::new(Rc::new(Cell::new(true)))
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
        self.inner.borrow().webview.clone().upcast()
    }

    fn set_zoom(&self, factor: f64) {
        self.inner.borrow().webview.set_zoom_level(factor);
    }

    fn suspend(&self) {
        let mut inner = self.inner.borrow_mut();
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
        self.inner.borrow().webview.evaluate_javascript(
            js,
            None,
            None,
            webkit6::gio::Cancellable::NONE,
            |_result| {},
        );
    }

    fn connect_url_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        self.inner.borrow().webview.connect_uri_notify(move |view| {
            if let Some(uri) = view.uri() {
                f(uri.to_string());
            }
        });
    }

    fn connect_title_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        self.inner
            .borrow()
            .webview
            .connect_title_notify(move |view| {
                if let Some(title) = view.title() {
                    f(title.to_string());
                }
            });
    }

    fn connect_load_finished(&self, f: Box<dyn Fn() + 'static>) {
        self.inner
            .borrow()
            .webview
            .connect_load_changed(move |_view, event| {
                if event == LoadEvent::Finished {
                    f();
                }
            });
    }
}
