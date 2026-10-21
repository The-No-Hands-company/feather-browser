// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::bridge::PageRenderer;
use crate::settings::Settings;
use crate::shell::tab_bar::TabBar;
use crate::shell::tab_manager::{TabId, TabManager};
use crate::shell::toolbar::Toolbar;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GBox, Orientation, Stack};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[cfg(feature = "webkit-renderer")]
use crate::bridge::WebKitRenderer;

pub struct FeatherWindow {
    window: ApplicationWindow,
}

impl FeatherWindow {
    pub fn new(app: &Application, settings: &Settings) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Feather")
            .default_width(1280)
            .default_height(800)
            .build();

        let tab_manager: Rc<RefCell<TabManager>> = Rc::new(RefCell::new(TabManager::new()));

        // Shared adblock flag — passed to every WebKitRenderer and the toolbar toggle.
        let adblock_flag = Rc::new(Cell::new(settings.adblock_enabled));

        // GTK timer fires every 10s to check idle tabs
        let mgr_for_timer = Rc::clone(&tab_manager);
        let hib_mgr = Rc::new(RefCell::new(
            crate::features::tab_hibernation::HibernationManager::new(settings.auto_hibernate_delay_secs)
        ));
        let hib_for_timer = Rc::clone(&hib_mgr);
        glib::timeout_add_seconds_local(10, move || {
            let mut tabs = mgr_for_timer.borrow_mut();
            hib_for_timer.borrow_mut().check_and_hibernate(tabs.tabs_mut());
            glib::ControlFlow::Continue
        });

        let content_stack = Rc::new(Stack::new());
        content_stack.set_vexpand(true);
        content_stack.set_hexpand(true);

        let toolbar = Rc::new(Toolbar::new(settings));
        let tab_bar = Rc::new(TabBar::new());

        let vbox = GBox::new(Orientation::Vertical, 0);
        vbox.append(&toolbar.widget());
        vbox.append(&tab_bar.widget());
        vbox.append(&*content_stack);
        window.set_child(Some(&vbox));

        // Shared holder for the redraw closure.
        // Stored in Rc<RefCell<...>> so callbacks can call back into it after mutations.
        let redraw_fn: Rc<RefCell<Box<dyn Fn()>>> =
            Rc::new(RefCell::new(Box::new(|| {})));

        // Build the actual redraw closure and install it.
        {
            let mgr = Rc::clone(&tab_manager);
            let bar = Rc::clone(&tab_bar);
            let stack = Rc::clone(&content_stack);
            let tb = Rc::clone(&toolbar);
            let rd = Rc::clone(&redraw_fn);
            let hib = Rc::clone(&hib_mgr);
            let adblock_flag_new = Rc::clone(&adblock_flag);
            let homepage = settings.homepage.clone();

            *redraw_fn.borrow_mut() = Box::new(move || {
                let mgr_ref = mgr.borrow();
                let active_id = mgr_ref.active_tab().map(|t| t.id);

                if let Some(tab) = mgr_ref.active_tab() {
                    tb.set_url(&tab.url);
                    stack.set_visible_child_name(&format!("tab-{}", tab.id));
                }

                // on_click: mark old tab Idle, resume new tab if hibernated, switch, redraw
                let mgr_c1 = Rc::clone(&mgr);
                let hib_c1 = Rc::clone(&hib);
                let rd_c1 = Rc::clone(&rd);
                let on_click = move |id: TabId| {
                    let old_id = mgr_c1.borrow().active_tab().map(|t| t.id);
                    {
                        let mut m = mgr_c1.borrow_mut();
                        hib_c1.borrow_mut().on_tab_switched(old_id, id, m.tabs_mut());
                        m.switch_to(id);
                    }
                    (rd_c1.borrow())();
                };

                // on_close: remove widget from stack, remove tab (drops renderer), then redraw.
                // renderer.stop() is intentionally omitted here — calling it while holding
                // borrow_mut risks a BorrowMutError if WebKit emits url-changed synchronously.
                // The renderer is dropped with the Tab, cancelling in-flight loads.
                let mgr_c2 = Rc::clone(&mgr);
                let rd_c2 = Rc::clone(&rd);
                let stack_c2 = Rc::clone(&stack);
                let on_close = move |id: TabId| {
                    let page_name = format!("tab-{}", id);
                    if let Some(child) = stack_c2.child_by_name(&page_name) {
                        stack_c2.remove(&child);
                    }
                    { mgr_c2.borrow_mut().remove_tab(id); }
                    (rd_c2.borrow())();
                };

                // on_new: create renderer + tab, add to stack, then redraw
                let mgr_c3 = Rc::clone(&mgr);
                let rd_c3 = Rc::clone(&rd);
                let stack_c = Rc::clone(&stack);
                let homepage_c = homepage.clone();
                // Clone here so on_new can be called multiple times (Fn, not FnOnce).
                let _adblock_for_new = Rc::clone(&adblock_flag_new);
                let on_new = move || {
                    #[cfg(feature = "webkit-renderer")]
                    let renderer: Box<dyn PageRenderer> =
                        Box::new(WebKitRenderer::new(Rc::clone(&_adblock_for_new)));
                    #[cfg(all(feature = "feather-renderer", not(feature = "webkit-renderer")))]
                    let renderer: Box<dyn PageRenderer> =
                        Box::new(crate::bridge::FeatherRenderer::new());
                    #[cfg(not(any(feature = "webkit-renderer", feature = "feather-renderer")))]
                    let renderer: Box<dyn PageRenderer> =
                        Box::new(crate::bridge::MockRenderer::new(&homepage_c));
                    renderer.load_url(&homepage_c);
                    let widget = renderer.widget();
                    let id = {
                        let mut m = mgr_c3.borrow_mut();
                        m.add_tab(&homepage_c, renderer)
                    };
                    stack_c.add_named(&widget, Some(&format!("tab-{}", id)));
                    // Wire URL and title change callbacks for the new tab
                    let mgr_u = Rc::clone(&mgr_c3);
                    let rd_u = Rc::clone(&rd_c3);
                    let mgr_t = Rc::clone(&mgr_c3);
                    let rd_t = Rc::clone(&rd_c3);
                    {
                        let mut m = mgr_c3.borrow_mut();
                        if let Some(tab) = m.get_tab_mut(id) {
                            tab.renderer.connect_url_changed(Box::new(move |url| {
                                if let Some(t) = mgr_u.borrow_mut().get_tab_mut(id) {
                                    t.url = url;
                                }
                                (rd_u.borrow())();
                            }));
                            tab.renderer.connect_title_changed(Box::new(move |title| {
                                if let Some(t) = mgr_t.borrow_mut().get_tab_mut(id) {
                                    t.title = title;
                                }
                                (rd_t.borrow())();
                            }));
                        }
                    }
                    (rd_c3.borrow())();
                };

                bar.render(mgr_ref.tabs(), active_id, on_click, on_close, on_new);
            });
        }

        // Open initial tab
        {
            #[cfg(feature = "webkit-renderer")]
            let renderer: Box<dyn PageRenderer> =
                Box::new(WebKitRenderer::new(Rc::clone(&adblock_flag)));
            #[cfg(all(feature = "feather-renderer", not(feature = "webkit-renderer")))]
            let renderer: Box<dyn PageRenderer> =
                Box::new(crate::bridge::FeatherRenderer::new());
            #[cfg(not(any(feature = "webkit-renderer", feature = "feather-renderer")))]
            let renderer: Box<dyn PageRenderer> =
                Box::new(crate::bridge::MockRenderer::new(&settings.homepage));
            renderer.load_url(&settings.homepage);
            let widget = renderer.widget();
            let id = {
                let mut mgr = tab_manager.borrow_mut();
                mgr.add_tab(&settings.homepage, renderer)
            };
            content_stack.add_named(&widget, Some(&format!("tab-{}", id)));
            // Wire URL and title change callbacks for the initial tab
            let mgr_u = Rc::clone(&tab_manager);
            let rd_u = Rc::clone(&redraw_fn);
            let mgr_t = Rc::clone(&tab_manager);
            let rd_t = Rc::clone(&redraw_fn);
            {
                let mut mgr_ref = tab_manager.borrow_mut();
                if let Some(tab) = mgr_ref.get_tab_mut(id) {
                    tab.renderer.connect_url_changed(Box::new(move |url| {
                        if let Some(t) = mgr_u.borrow_mut().get_tab_mut(id) {
                            t.url = url;
                        }
                        (rd_u.borrow())();
                    }));
                    tab.renderer.connect_title_changed(Box::new(move |title| {
                        if let Some(t) = mgr_t.borrow_mut().get_tab_mut(id) {
                            t.title = title;
                        }
                        (rd_t.borrow())();
                    }));
                }
            }
        }

        // Wire toolbar navigate → active renderer load_url
        {
            let mgr = Rc::clone(&tab_manager);
            toolbar.connect_navigate(move |url| {
                let mgr_ref = mgr.borrow();
                if let Some(tab) = mgr_ref.active_tab() {
                    tab.renderer.load_url(&url);
                }
            });
        }

        // Wire back / forward / reload to the active tab's renderer
        {
            let mgr = Rc::clone(&tab_manager);
            toolbar.connect_back(move || {
                if let Some(tab) = mgr.borrow().active_tab() { tab.renderer.go_back(); }
            });
        }
        {
            let mgr = Rc::clone(&tab_manager);
            toolbar.connect_forward(move || {
                if let Some(tab) = mgr.borrow().active_tab() { tab.renderer.go_forward(); }
            });
        }
        {
            let mgr = Rc::clone(&tab_manager);
            toolbar.connect_reload(move || {
                if let Some(tab) = mgr.borrow().active_tab() { tab.renderer.reload(); }
            });
        }

        // Wire Shield toggle → shared adblock flag + settings persist
        {
            let flag = Rc::clone(&adblock_flag);
            toolbar.connect_adblock_toggled(move |enabled| {
                flag.set(enabled);
                let mut s = crate::settings::Settings::load();
                s.adblock_enabled = enabled;
                let _ = s.save();
            });
        }

        // Wire Freeze toggle → hibernation manager + settings persist
        {
            let hib = Rc::clone(&hib_mgr);
            let base_delay = settings.auto_hibernate_delay_secs;
            toolbar.connect_freeze_toggled(move |enabled| {
                hib.borrow_mut().set_enabled(enabled);
                let mut s = crate::settings::Settings::load();
                s.auto_hibernate_delay_secs = if enabled { base_delay } else { 0 };
                let _ = s.save();
            });
        }

        // Initial render of the tab bar
        (redraw_fn.borrow())();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
