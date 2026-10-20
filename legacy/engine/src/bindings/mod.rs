use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};

use crate::html::HtmlParser;
use crate::css::CssParser;
use crate::layout::{LayoutEngine, Rect, Dimensions};
use crate::render::RenderEngine;

#[napi]
pub struct PronBrowEngine {
    html_parser: HtmlParser,
    css_parser: CssParser,
    layout_engine: Arc<Mutex<LayoutEngine>>,
    render_engine: Arc<Mutex<RenderEngine>>,
}

#[napi]
impl PronBrowEngine {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            layout_engine: Arc::new(Mutex::new(LayoutEngine::new())),
            render_engine: Arc::new(Mutex::new(RenderEngine::new())),
        })
    }

    #[napi]
    pub fn parse_html(&mut self, html: String) -> Result<String> {
        match self.html_parser.parse(&html) {
            Ok(dom) => Ok(format!("{:?}", dom)),
            Err(e) => Err(Error::new(
                Status::GenericFailure,
                format!("HTML parse error: {}", e),
            )),
        }
    }

    #[napi]
    pub fn parse_css(&mut self, css: String) -> Result<String> {
        match self.css_parser.parse(&css) {
            Ok(stylesheet) => Ok(format!("{:?}", stylesheet)),
            Err(e) => Err(Error::new(
                Status::GenericFailure,
                format!("CSS parse error: {}", e),
            )),
        }
    }

    #[napi]
    pub fn render_page(&mut self, html: String, css: String, width: u32, height: u32) -> Result<Vec<u8>> {
        // Step 1: Parse HTML to DOM
        let dom = self.html_parser.parse(&html).map_err(|e| {
            Error::new(Status::GenericFailure, format!("HTML parse error: {}", e))
        })?;

        // Step 2: Parse CSS to Stylesheet
        let stylesheet = self.css_parser.parse(&css).map_err(|e| {
            Error::new(Status::GenericFailure, format!("CSS parse error: {}", e))
        })?;

        // Step 3: Apply CSS to DOM tree (proper selector matching!)
        let styled_root = crate::css::style_tree(&dom.root, &stylesheet);

        // Step 4: Run layout to calculate positions and sizes
        let mut viewport = Dimensions::default();
        viewport.content.width = width as f32;
        viewport.content.height = height as f32;
        
        let layout_engine = self.layout_engine.lock().unwrap();
        let layout_root = layout_engine.layout_tree(&styled_root, viewport);

        // Step 5: Paint to canvas
        let mut render_engine = self.render_engine.lock().unwrap();
        let viewport_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
        let canvas = render_engine.paint(&layout_root, viewport_rect);

        // Step 6: Convert Color pixels to RGBA bytes
        let mut rgba_bytes = Vec::with_capacity(width as usize * height as usize * 4);
        for pixel in canvas.pixels.iter() {
            rgba_bytes.push(pixel.r);
            rgba_bytes.push(pixel.g);
            rgba_bytes.push(pixel.b);
            rgba_bytes.push(pixel.a);
        }

        Ok(rgba_bytes)
    }

    #[napi]
    pub fn execute_javascript(&mut self, code: String) -> Result<String> {
        Ok(format!("JavaScript execution result: {}", code))
    }

    #[napi]
    pub fn get_dom_tree(&self) -> Result<String> {
        Ok("DOM tree structure".to_string())
    }

    #[napi]
    pub fn handle_event(&mut self, event_type: String, data: String) -> Result<String> {
        Ok(format!("Event handled: {} with data: {}", event_type, data))
    }

    #[napi]
    pub fn navigate(&mut self, url: String) -> Result<String> {
        Ok(format!("Navigating to: {}", url))
    }
}

#[napi]
pub struct BrowserTab {
    engine: PronBrowEngine,
    url: String,
    title: String,
}

#[napi]
impl BrowserTab {
    #[napi(constructor)]
    pub fn new(url: String) -> Result<Self> {
        Ok(Self {
            engine: PronBrowEngine::new()?,
            url,
            title: "New Tab".to_string(),
        })
    }

    #[napi]
    pub fn load_url(&mut self, url: String) -> Result<String> {
        self.url = url.clone();
        self.engine.navigate(url)
    }

    #[napi]
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    #[napi]
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    #[napi]
    pub fn render(&mut self, html: String, css: String, width: u32, height: u32) -> Result<Vec<u8>> {
        self.engine.render_page(html, css, width, height)
    }
}

#[napi]
pub fn create_browser_engine() -> Result<PronBrowEngine> {
    PronBrowEngine::new()
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
