// PronBrow Engine - A modern web browser engine built from scratch
// Core library exports

pub mod dom;
pub mod html;
pub mod css;
pub mod js;
pub mod layout;
pub mod render;
pub mod network;
pub mod text;
pub mod image;
pub mod svg;
pub mod canvas;
pub mod storage;
pub mod events;
pub mod accessibility;
pub mod forms;
pub mod security;
pub mod compositor;
pub mod web_api;
pub mod media;
pub mod webgl;
pub mod webrtc;
pub mod indexeddb;
pub mod serviceworker;

#[cfg(feature = "napi")]
pub mod bindings;

use std::rc::Rc;
use std::cell::RefCell;

pub use dom::Dom;
pub use html::HtmlParser;
pub use css::CssParser;
pub use layout::LayoutEngine;
pub use render::RenderEngine;
pub use network::NetworkEngine;
pub use text::TextRenderer;
pub use image::ImageLoader;

#[derive(Debug, Clone)]
pub struct BrowserEngine {
    pub html_parser: HtmlParser,
    pub css_parser: CssParser,
    pub layout_engine: LayoutEngine,
    pub render_engine: RenderEngine,
    pub network_engine: NetworkEngine,
    pub js_engine: js::JSEngine,
    pub dom_bridge: Rc<RefCell<js::DOMBridge>>,
}

impl BrowserEngine {
    pub fn new() -> Self {
        Self {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            layout_engine: LayoutEngine::new(),
            render_engine: RenderEngine::new(),
            network_engine: NetworkEngine::new(),
            js_engine: js::JSEngine::new(),
            dom_bridge: Rc::new(RefCell::new(js::DOMBridge::new())),
        }
    }
    
    pub fn execute_script(&mut self, code: &str) -> Result<js::JSValue, String> {
        self.js_engine.execute(code)
    }
    
    pub fn load_and_execute_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse HTML to DOM
        let dom = self.html_parser.parse(html)?;
        
        // Register DOM with bridge
        let root_rc = Rc::new(RefCell::new(dom.root));
        self.dom_bridge.borrow_mut().setup_document(&mut self.js_engine, root_rc.clone());
        
        // Find and execute <script> tags
        let root = root_rc.borrow();
        self.dom_bridge.borrow_mut().execute_scripts(&mut self.js_engine, &root)?;
        
        Ok(())
    }

    /// Loads a URL and renders the complete page through the full rendering pipeline
    pub async fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Step 1: Fetch HTML content
        let html = self.network_engine.fetch(url).await?;
        
        // Step 2: Parse HTML to DOM
        let dom = self.html_parser.parse(&html)?;
        let root_rc = Rc::new(RefCell::new(dom.root));
        
        // Step 3: Extract and fetch all CSS resources
        let mut combined_css = String::new();
        {
            let root_node = root_rc.borrow();
            self.extract_and_fetch_css(&root_node, url, &mut combined_css).await?;
        }
        
        // Step 4: Parse CSS and build stylesheet
        let stylesheet = self.css_parser.parse(&combined_css)?;
        
        // Step 5: Apply styles to DOM (style resolution)
        let styled_node = css::style_tree(&root_rc.borrow(), &stylesheet);
        
        // Step 6: Run layout engine to compute dimensions and positions
        let viewport = layout::Dimensions {
            content: layout::Rect {
                x: 0.0,
                y: 0.0,
                width: 1920.0,  // Default viewport
                height: 1080.0,
            },
            padding: layout::EdgeSizes::default(),
            border: layout::EdgeSizes::default(),
            margin: layout::EdgeSizes::default(),
            position_offset: layout::EdgeSizes::default(),
            z_index: 0,
        };
        let layout_tree = self.layout_engine.layout_tree(&styled_node, viewport);
        
        // Step 7: Paint to canvas and render
        let bounds = layout::Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let _canvas = self.render_engine.paint(&layout_tree, bounds);
        
        // Step 8: Setup DOM bridge for JavaScript execution
        self.dom_bridge.borrow_mut().setup_document(&mut self.js_engine, root_rc.clone());
        
        // Step 9: Execute inline and external scripts
        self.dom_bridge.borrow_mut().execute_scripts(&mut self.js_engine, &root_rc.borrow())?;
        
        Ok(())
    }
    
    /// Extracts CSS links from DOM and fetches all stylesheets
    async fn extract_and_fetch_css(
        &mut self,
        node: &crate::dom::Node,
        base_url: &str,
        combined_css: &mut String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check for <link rel="stylesheet"> elements
        if node.get_tag_name() == Some("link") {
            if let (Some(rel), Some(href)) = (node.get_attribute("rel"), node.get_attribute("href")) {
                if rel == "stylesheet" {
                    let css_url = self.resolve_url(base_url, href)?;
                    let css_content = self.network_engine.fetch(&css_url).await?;
                    combined_css.push_str(&css_content);
                    combined_css.push('\n');
                }
            }
        }
        
        // Check for <style> tags
        if node.get_tag_name() == Some("style") {
            for child in &node.children {
                if let crate::dom::NodeType::Text(text) = &child.node_type {
                    combined_css.push_str(text);
                    combined_css.push('\n');
                }
            }
        }
        
        // Recursively process children
        for child in &node.children {
            Box::pin(self.extract_and_fetch_css(child, base_url, combined_css)).await?;
        }
        
        Ok(())
    }
    
    /// Resolves relative URLs to absolute URLs
    fn resolve_url(&self, base: &str, relative: &str) -> Result<String, Box<dyn std::error::Error>> {
        if relative.starts_with("http://") || relative.starts_with("https://") {
            Ok(relative.to_string())
        } else if relative.starts_with("//") {
            // Protocol-relative URL
            let protocol = if base.starts_with("https") { "https:" } else { "http:" };
            Ok(format!("{}{}", protocol, relative))
        } else if relative.starts_with('/') {
            // Absolute path
            let base_parts: Vec<&str> = base.splitn(4, '/').collect();
            if base_parts.len() >= 3 {
                Ok(format!("{}//{}{}", base_parts[0], base_parts[2], relative))
            } else {
                Err("Invalid base URL".into())
            }
        } else {
            // Relative path
            let base_parts: Vec<&str> = base.rsplitn(2, '/').collect();
            if base_parts.len() == 2 {
                Ok(format!("{}/{}", base_parts[1], relative))
            } else {
                Ok(format!("{}/{}", base, relative))
            }
        }
    }
}

impl Default for BrowserEngine {
    fn default() -> Self {
        Self::new()
    }
}
