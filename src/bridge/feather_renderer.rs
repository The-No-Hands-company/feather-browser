// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PageRenderer;
use crate::engine::{
    css::CssParser,
    dom::NodeType,
    html::HtmlParser,
    layout::{LayoutEngine, Rect},
    network,
    render::{canvas_to_bgra, Painter},
    style::style_tree,
};
use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::cell::RefCell;
use std::rc::Rc;

struct Inner {
    url: String,
    title: String,
    history: Vec<String>,
    history_pos: usize,
    pixel_buf: Option<Vec<u8>>,
    pixel_w: u32,
    pixel_h: u32,
    drawing_area: DrawingArea,
    url_cbs: Vec<Box<dyn Fn(String) + 'static>>,
    title_cbs: Vec<Box<dyn Fn(String) + 'static>>,
    load_done_cbs: Vec<Box<dyn Fn() + 'static>>,
    suspended_url: Option<String>,
}

#[derive(Clone)]
pub struct FeatherRenderer {
    inner: Rc<RefCell<Inner>>,
}

impl FeatherRenderer {
    pub fn new() -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);

        let inner = Rc::new(RefCell::new(Inner {
            url: String::new(),
            title: "Feather".into(),
            history: Vec::new(),
            history_pos: 0,
            pixel_buf: None,
            pixel_w: 0,
            pixel_h: 0,
            drawing_area: drawing_area.clone(),
            url_cbs: Vec::new(),
            title_cbs: Vec::new(),
            load_done_cbs: Vec::new(),
            suspended_url: None,
        }));

        {
            // Fix 2: use a Weak reference to avoid an Rc cycle in the draw closure.
            let inner_weak = Rc::downgrade(&inner);
            drawing_area.set_draw_func(move |_da, ctx, w, h| {
                let inner_rc = match inner_weak.upgrade() {
                    Some(rc) => rc,
                    None => return,
                };
                let state = inner_rc.borrow();
                match &state.pixel_buf {
                    None => {
                        ctx.set_source_rgb(0.15, 0.15, 0.15);
                        let _ = ctx.paint();
                    }
                    Some(buf) if buf.len() == (state.pixel_w * state.pixel_h * 4) as usize => {
                        let buf_clone: Vec<u8> = buf.clone();
                        let pw = state.pixel_w;
                        let ph = state.pixel_h;
                        drop(state);

                        match cairo::ImageSurface::create_for_data(
                            buf_clone,
                            cairo::Format::ARgb32,
                            pw as i32,
                            ph as i32,
                            (pw * 4) as i32,
                        ) {
                            Ok(surface) => {
                                let scale_x = w as f64 / pw as f64;
                                let scale_y = h as f64 / ph as f64;
                                ctx.scale(scale_x, scale_y);
                                ctx.set_source_surface(&surface, 0.0, 0.0).ok();
                                ctx.paint().ok();
                            }
                            Err(_) => {
                                ctx.set_source_rgb(0.5, 0.0, 0.0);
                                ctx.paint().ok();
                            }
                        }
                    }
                    Some(_) => {
                        ctx.set_source_rgb(0.9, 0.9, 0.9);
                        ctx.paint().ok();
                    }
                }
            });
        }

        Self { inner }
    }

    fn render_page(&self, html: &str, url: &str) {
        let (w, h) = {
            let s = self.inner.borrow();
            (
                s.drawing_area.width().max(800) as u32,
                s.drawing_area.height().max(600) as u32,
            )
        };

        let dom = HtmlParser::parse(html);
        // Fix 7: prepend UA stylesheet so head/style/script/title default to display:none.
        let ua_css = "head{display:none}style{display:none}script{display:none}title{display:none}";
        let css_text = format!("{} {}", ua_css, extract_style_tags(&dom.root, 0));
        let sheet = CssParser::parse(&css_text);
        let styled = style_tree(&dom.root, &sheet);
        let title = extract_title(&dom.root, 0).unwrap_or_else(|| url.to_string());

        let layout_root = LayoutEngine::build(&styled, w as f32);
        let viewport = Rect { x: 0.0, y: 0.0, width: w as f32, height: h as f32 };
        let canvas = Painter::paint(&layout_root, viewport);
        let bgra = canvas_to_bgra(&canvas);

        {
            let mut s = self.inner.borrow_mut();
            s.pixel_buf = Some(bgra);
            s.pixel_w = w;
            s.pixel_h = h;
            s.title = title.clone();
        }

        // Take each callback list before invoking to avoid re-entrant borrow panics.
        let url_str = url.to_string();
        let url_cbs = std::mem::take(&mut self.inner.borrow_mut().url_cbs);
        for cb in &url_cbs { cb(url_str.clone()); }
        self.inner.borrow_mut().url_cbs = url_cbs;

        let title_cbs = std::mem::take(&mut self.inner.borrow_mut().title_cbs);
        for cb in &title_cbs { cb(title.clone()); }
        self.inner.borrow_mut().title_cbs = title_cbs;

        self.inner.borrow().drawing_area.queue_draw();

        let load_done_cbs = std::mem::take(&mut self.inner.borrow_mut().load_done_cbs);
        for cb in &load_done_cbs { cb(); }
        self.inner.borrow_mut().load_done_cbs = load_done_cbs;
    }

    /// Fetch + render without touching history. Used by go_back, go_forward, reload.
    fn fetch_and_render_url(&self, url: String) {
        self.inner.borrow_mut().url = url.clone();
        let renderer = self.clone();
        let url_clone = url.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
        std::thread::spawn(move || {
            let result = network::fetch_page(&url_clone)
                .map(|r| r.html)
                .map_err(|e| e.0);
            let _ = tx.send(result);
        });
        glib::idle_add_local(move || {
            match rx.try_recv() {
                Ok(Ok(html)) => {
                    renderer.render_page(&html, &url);
                    glib::ControlFlow::Break
                }
                Ok(Err(e)) => {
                    // Fix 4: escape error message to prevent HTML injection.
                    let error_html = format!(
                        "<html><body><h2>Load failed</h2><p>{}</p></body></html>",
                        html_escape(&e)
                    );
                    renderer.render_page(&error_html, &url);
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            }
        });
    }
}

impl Default for FeatherRenderer {
    fn default() -> Self { Self::new() }
}

impl PageRenderer for FeatherRenderer {
    fn load_url(&self, url: &str) {
        let url = url.to_string();
        {
            let mut s = self.inner.borrow_mut();
            // Fix 3: compare against history[history_pos], not history.last().
            if s.history.get(s.history_pos).map(|u| u.as_str()) != Some(&url) {
                let pos = s.history_pos + 1;
                s.history.truncate(pos);
                s.history.push(url.clone());
                s.history_pos = s.history.len() - 1;
            }
            s.url = url.clone();
        }

        let renderer = self.clone();
        let url_clone = url.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
        std::thread::spawn(move || {
            let result = network::fetch_page(&url_clone)
                .map(|r| r.html)
                .map_err(|e| e.0);
            let _ = tx.send(result);
        });

        glib::idle_add_local(move || {
            match rx.try_recv() {
                Ok(Ok(html)) => {
                    renderer.render_page(&html, &url);
                    glib::ControlFlow::Break
                }
                Ok(Err(e)) => {
                    // Fix 4: escape error message to prevent HTML injection.
                    let error_html = format!(
                        "<html><body><h2>Load failed</h2><p>{}</p></body></html>",
                        html_escape(&e)
                    );
                    renderer.render_page(&error_html, &url);
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            }
        });
    }

    fn go_back(&self) {
        let url = {
            let mut s = self.inner.borrow_mut();
            if s.history_pos == 0 { return; }
            s.history_pos -= 1;
            s.history[s.history_pos].clone()
        };
        self.fetch_and_render_url(url);
    }

    fn go_forward(&self) {
        let url = {
            let mut s = self.inner.borrow_mut();
            if s.history_pos + 1 >= s.history.len() { return; }
            s.history_pos += 1;
            s.history[s.history_pos].clone()
        };
        self.fetch_and_render_url(url);
    }

    fn reload(&self) {
        let url = self.inner.borrow().url.clone();
        if !url.is_empty() { self.fetch_and_render_url(url); }
    }

    fn stop(&self) {}

    fn current_url(&self) -> String { self.inner.borrow().url.clone() }

    fn title(&self) -> String { self.inner.borrow().title.clone() }

    fn can_go_back(&self) -> bool { self.inner.borrow().history_pos > 0 }

    fn can_go_forward(&self) -> bool {
        let s = self.inner.borrow();
        s.history_pos + 1 < s.history.len()
    }

    fn widget(&self) -> gtk4::Widget {
        self.inner.borrow().drawing_area.clone().upcast()
    }

    fn set_zoom(&self, _factor: f64) {}

    fn suspend(&self) {
        let url = {
            let mut s = self.inner.borrow_mut();
            if s.suspended_url.is_some() { return; }
            let u = s.url.clone();
            s.suspended_url = Some(u.clone());
            s.pixel_buf = None;
            u
        };
        log::debug!("[FeatherRenderer] suspended: {}", url);
    }

    // Fix 3: resume() uses fetch_and_render_url to avoid corrupting history.
    fn resume(&self) {
        let url = self.inner.borrow_mut().suspended_url.take();
        if let Some(u) = url {
            if !u.is_empty() {
                self.fetch_and_render_url(u);
            }
        }
    }

    fn run_script(&self, _js: &str) {}

    fn connect_url_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        self.inner.borrow_mut().url_cbs.push(f);
    }

    fn connect_title_changed(&self, f: Box<dyn Fn(String) + 'static>) {
        self.inner.borrow_mut().title_cbs.push(f);
    }

    fn connect_load_finished(&self, f: Box<dyn Fn() + 'static>) {
        self.inner.borrow_mut().load_done_cbs.push(f);
    }
}

/// Fix 4: Escape HTML special characters to prevent injection in error pages.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

/// Fix 1: depth-capped traversal to avoid stack overflow on deep DOMs.
fn extract_style_tags(node: &crate::engine::dom::Node, depth: usize) -> String {
    if depth >= 512 { return String::new(); }
    let mut css = String::new();
    if let NodeType::Element(e) = &node.node_type {
        if e.tag_name == "style" {
            for child in &node.children {
                if let NodeType::Text(t) = &child.node_type {
                    css.push_str(t);
                }
            }
        }
    }
    for child in &node.children {
        css.push_str(&extract_style_tags(child, depth + 1));
    }
    css
}

/// Fix 1: depth-capped traversal to avoid stack overflow on deep DOMs.
fn extract_title(node: &crate::engine::dom::Node, depth: usize) -> Option<String> {
    if depth >= 512 { return None; }
    if let NodeType::Element(e) = &node.node_type {
        if e.tag_name == "title" {
            let text: String = node.children.iter().filter_map(|c| {
                if let NodeType::Text(t) = &c.node_type { Some(t.clone()) } else { None }
            }).collect();
            if !text.is_empty() { return Some(text.trim().to_string()); }
        }
    }
    for child in &node.children {
        if let Some(t) = extract_title(child, depth + 1) { return Some(t); }
    }
    None
}
