// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::bridge::PageRenderer;
use crate::settings::Settings;
use crate::shell::tab_bar::TabBar;
use crate::shell::tab_manager::{TabId, TabManager};
use crate::shell::toolbar::Toolbar;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GBox, Orientation, Stack};
use std::cell::RefCell;
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
            let homepage = settings.homepage.clone();

            *redraw_fn.borrow_mut() = Box::new(move || {
                let mgr_ref = mgr.borrow();
                let active_id = mgr_ref.active_tab().map(|t| t.id);

                if let Some(tab) = mgr_ref.active_tab() {
                    tb.set_url(&tab.url);
                    stack.set_visible_child_name(&format!("tab-{}", tab.id));
                }

                // on_click: switch active tab, then redraw
                let mgr_c1 = Rc::clone(&mgr);
                let rd_c1 = Rc::clone(&rd);
                let on_click = move |id: TabId| {
                    { mgr_c1.borrow_mut().switch_to(id); }
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
                let on_new = move || {
                    #[cfg(feature = "webkit-renderer")]
                    let renderer: Box<dyn PageRenderer> =
                        Box::new(WebKitRenderer::new());
                    #[cfg(not(feature = "webkit-renderer"))]
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
            let renderer: Box<dyn PageRenderer> = Box::new(WebKitRenderer::new());
            #[cfg(not(feature = "webkit-renderer"))]
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

        // Initial render of the tab bar
        (redraw_fn.borrow())();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
