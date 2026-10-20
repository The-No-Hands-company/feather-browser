// PronBrow Engine - A modern web browser engine built from scratch
// Core library exports

// Core modules (always available)
pub mod dom;
pub mod html;
pub mod network;

// CLI module (for terminal-based browsing)
#[cfg(any(feature = "cli-minimal", feature = "cli-standard", feature = "cli-full"))]
pub mod cli;

// GUI modules (feature-gated)
#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub mod css;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub mod layout;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub mod render;

// JavaScript engine
#[cfg(feature = "js-core")]
pub mod js;

// Optional modules
#[cfg(feature = "css-full")]
pub mod text;

#[cfg(any(feature = "image-png", feature = "image-jpeg", feature = "image-webp", feature = "image-svg"))]
pub mod image;

#[cfg(feature = "image-svg")]
pub mod svg;

#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(any(feature = "gui-standard", feature = "gui-full"))]
pub mod storage;

#[cfg(any(feature = "gui-standard", feature = "gui-full"))]
pub mod events;

#[cfg(any(feature = "gui-standard", feature = "gui-full"))]
pub mod accessibility;

#[cfg(feature = "forms")]
pub mod forms;

#[cfg(any(feature = "gui-standard", feature = "gui-full"))]
pub mod security;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub mod compositor;

#[cfg(any(feature = "gui-standard", feature = "gui-full"))]
pub mod web_api;

#[cfg(feature = "media")]
pub mod media;

#[cfg(feature = "webgl")]
pub mod webgl;

#[cfg(feature = "webrtc")]
pub mod webrtc;

#[cfg(feature = "indexeddb")]
pub mod indexeddb;

#[cfg(feature = "service-workers")]
pub mod serviceworker;

#[cfg(feature = "napi")]
pub mod bindings;

use std::rc::Rc;
use std::cell::RefCell;

pub use dom::Dom;
pub use html::HtmlParser;
pub use network::NetworkEngine;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub use css::CssParser;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub use layout::LayoutEngine;

#[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
pub use render::RenderEngine;

#[cfg(feature = "css-full")]
pub use text::TextRenderer;

#[cfg(any(feature = "image-png", feature = "image-jpeg", feature = "image-webp", feature = "image-svg"))]
pub use image::ImageLoader;

#[cfg(any(feature = "cli-minimal", feature = "cli-standard", feature = "cli-full"))]
pub use cli::{CLIBrowser, CLIMode};

// ============================================================================
// Optimized Browser Engine - Lazy-loading architecture
// ============================================================================

#[derive(Debug, Clone)]
pub struct BrowserEngine {
    // Core (always loaded)
    pub html_parser: HtmlParser,
    pub network_engine: NetworkEngine,
    
    // Optional subsystems (lazy-loaded)
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub css_parser: Option<CssParser>,
    
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub layout_engine: Option<LayoutEngine>,
    
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub render_engine: Option<RenderEngine>,
    
    #[cfg(feature = "js-core")]
    pub js_engine: Option<js::JSEngine>,
    
    #[cfg(feature = "js-core")]
    pub dom_bridge: Option<Rc<RefCell<js::DOMBridge>>>,
}

impl BrowserEngine {
    pub fn new() -> Self {
        Self {
            html_parser: HtmlParser::new(),
            network_engine: NetworkEngine::new(),
            
            #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
            css_parser: None,
            
            #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
            layout_engine: None,
            
            #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
            render_engine: None,
            
            #[cfg(feature = "js-core")]
            js_engine: None,
            
            #[cfg(feature = "js-core")]
            dom_bridge: None,
        }
    }
    
    /// Initialize CSS engine (lazy-loaded)
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub fn ensure_css(&mut self) {
        if self.css_parser.is_none() {
            self.css_parser = Some(CssParser::new());
        }
    }
    
    /// Initialize layout engine (lazy-loaded)
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub fn ensure_layout(&mut self) {
        if self.layout_engine.is_none() {
            self.layout_engine = Some(LayoutEngine::new());
        }
    }
    
    /// Initialize render engine (lazy-loaded)
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub fn ensure_render(&mut self) {
        if self.render_engine.is_none() {
            self.render_engine = Some(RenderEngine::new());
        }
    }
    
    /// Initialize JavaScript engine (lazy-loaded)
    #[cfg(feature = "js-core")]
    pub fn ensure_js(&mut self) {
        if self.js_engine.is_none() {
            self.js_engine = Some(js::JSEngine::new());
            self.dom_bridge = Some(Rc::new(RefCell::new(js::DOMBridge::new())));
        }
    }
    
    /// Execute JavaScript code (loads JS engine if needed)
    #[cfg(feature = "js-core")]
    pub fn execute_script(&mut self, code: &str) -> Result<js::JSValue, String> {
        self.ensure_js();
        self.js_engine.as_mut().unwrap().execute(code)
    }
    
    #[cfg(not(feature = "js-core"))]
    pub fn execute_script(&mut self, _code: &str) -> Result<String, String> {
        Err("JavaScript support not compiled in. Enable 'js-core' feature.".to_string())
    }
    
    /// Load and execute HTML with scripts
    #[cfg(feature = "js-core")]
    pub fn load_and_execute_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        let dom = self.html_parser.parse(html)?;
        
        // Load JS engine only if needed
        self.ensure_js();
        
        let root_rc = Rc::new(RefCell::new(dom.root));
        let dom_bridge = self.dom_bridge.as_ref().unwrap();
        let js_engine = self.js_engine.as_mut().unwrap();
        
        dom_bridge.borrow_mut().setup_document(js_engine, root_rc.clone());
        
        let root = root_rc.borrow();
        dom_bridge.borrow_mut().execute_scripts(js_engine, &root)?;
        
        Ok(())
    }
    
    #[cfg(not(feature = "js-core"))]
    pub fn load_and_execute_html(&mut self, html: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Just parse HTML, no script execution
        let _dom = self.html_parser.parse(html)?;
        Ok(())
    }

    /// Load URL with progressive feature loading
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    pub async fn load_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Step 1: Fetch HTML content
        let html = self.network_engine.fetch(url).await?;
        
        // Step 2: Parse HTML to DOM
        let dom = self.html_parser.parse(&html)?;
        let root_rc = Rc::new(RefCell::new(dom.root));
        
        // Step 3: Detect required features
        let needs_css = html.contains("<style") || html.contains("class=") || html.contains("id=");
        let needs_js = html.contains("<script");
        
        // Step 4: Load only needed subsystems
        if needs_css {
            self.ensure_css();
            self.ensure_layout();
            self.ensure_render();
            
            // Extract and fetch CSS
            let mut combined_css = String::new();
            {
                let root_node = root_rc.borrow();
                self.extract_and_fetch_css(&root_node, url, &mut combined_css).await?;
            }
            
            // Parse CSS
            let css_parser = self.css_parser.as_mut().unwrap();
            let stylesheet = css_parser.parse(&combined_css)?;
            
            // Apply styles and render
            let layout_engine = self.layout_engine.as_ref().unwrap();
            let render_engine = self.render_engine.as_mut().unwrap();
            
            // ... continue with rendering pipeline
        }
        
        #[cfg(feature = "js-core")]
        if needs_js {
            self.ensure_js();
            
            let dom_bridge = self.dom_bridge.as_ref().unwrap();
            let js_engine = self.js_engine.as_mut().unwrap();
            
            dom_bridge.borrow_mut().setup_document(js_engine, root_rc.clone());
            
            let root = root_rc.borrow();
            dom_bridge.borrow_mut().execute_scripts(js_engine, &root)?;
        }
        
        Ok(())
    }
    
    #[cfg(any(feature = "gui-minimal", feature = "gui-standard", feature = "gui-full"))]
    async fn extract_and_fetch_css(
        &self,
        _node: &dom::Node,
        _base_url: &str,
        _combined_css: &mut String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified version - full implementation would traverse DOM
        Ok(())
    }
}

impl Default for BrowserEngine {
    fn default() -> Self {
        Self::new()
    }
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
