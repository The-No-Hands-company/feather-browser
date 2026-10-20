// CLI Browser Module - Terminal-based web browsing with minimal resource usage
//
// This module provides three modes:
// - Minimal: Text-only extraction (<10MB RAM)
// - Standard: ANSI colors + basic formatting (<25MB RAM)
// - Full: Terminal graphics protocol support (<50MB RAM)

mod interactive;

pub use interactive::{InteractiveCLI, InteractiveConfig, NavigationHistory, RenderedPage};

use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CLIMode {
    Minimal,   // Text only, no styling
    Standard,  // ANSI colors, basic formatting
    Full,      // Terminal graphics protocol
}

pub struct CLIBrowser {
    mode: CLIMode,
    viewport: TerminalViewport,
    
    #[cfg(feature = "ansi-colors")]
    color_enabled: bool,
    
    #[cfg(feature = "terminal-graphics")]
    graphics_protocol: Option<GraphicsProtocol>,
}

#[derive(Debug, Clone)]
pub struct TerminalViewport {
    pub width: u16,
    pub height: u16,
}

#[cfg(feature = "terminal-graphics")]
#[derive(Debug, Clone, Copy)]
pub enum GraphicsProtocol {
    Kitty,
    ITerm2,
    Sixel,
}

impl CLIBrowser {
    pub fn new(mode: CLIMode) -> Self {
        let viewport = Self::detect_terminal_size();
        
        Self {
            mode,
            viewport,
            
            #[cfg(feature = "ansi-colors")]
            color_enabled: Self::supports_color(),
            
            #[cfg(feature = "terminal-graphics")]
            graphics_protocol: Self::detect_graphics_protocol(),
        }
    }
    
    /// Detect terminal dimensions
    fn detect_terminal_size() -> TerminalViewport {
        // Try to get terminal size from environment or tty
        #[cfg(unix)]
        {
            if let Ok((w, h)) = term_size::dimensions() {
                return TerminalViewport {
                    width: w as u16,
                    height: h as u16,
                };
            }
        }
        
        // Default fallback
        TerminalViewport {
            width: 80,
            height: 24,
        }
    }
    
    #[cfg(feature = "ansi-colors")]
    fn supports_color() -> bool {
        // Check if terminal supports colors
        std::env::var("TERM")
            .map(|term| !term.contains("dumb") && term != "unknown")
            .unwrap_or(false)
    }
    
    #[cfg(feature = "terminal-graphics")]
    fn detect_graphics_protocol() -> Option<GraphicsProtocol> {
        // Detect which graphics protocol the terminal supports
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("kitty") {
                return Some(GraphicsProtocol::Kitty);
            }
            if term.contains("iterm") {
                return Some(GraphicsProtocol::ITerm2);
            }
        }
        
        // Check for Sixel support via terminfo
        None
    }
    
    /// Browse a URL in CLI mode
    pub async fn browse(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        match self.mode {
            CLIMode::Minimal => self.browse_minimal(url).await,
            CLIMode::Standard => self.browse_standard(url).await,
            CLIMode::Full => self.browse_full(url).await,
        }
    }
    
    /// Minimal mode: Extract and display text only
    async fn browse_minimal(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Fetch HTML
        let html = self.fetch_url(url).await?;
        
        // Extract text using minimal parser
        let text = self.extract_text(&html);
        
        // Format for terminal width
        let formatted = self.wrap_text(&text, self.viewport.width as usize);
        
        Ok(formatted)
    }
    
    /// Standard mode: ANSI colors and basic formatting
    #[cfg(feature = "ansi-colors")]
    async fn browse_standard(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let html = self.fetch_url(url).await?;
        
        // Parse with minimal DOM
        let dom = self.parse_minimal_dom(&html)?;
        
        // Render with ANSI codes
        let renderer = ANSIRenderer::new(self.viewport.width as usize);
        let output = renderer.render(&dom);
        
        Ok(output)
    }
    
    #[cfg(not(feature = "ansi-colors"))]
    async fn browse_standard(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Fallback to minimal if ANSI not enabled
        self.browse_minimal(url).await
    }
    
    /// Full mode: Terminal graphics support
    #[cfg(feature = "terminal-graphics")]
    async fn browse_full(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let html = self.fetch_url(url).await?;
        let dom = self.parse_minimal_dom(&html)?;
        
        // Render with graphics protocol
        if let Some(protocol) = self.graphics_protocol {
            let renderer = GraphicsRenderer::new(protocol, self.viewport);
            return Ok(renderer.render(&dom));
        }
        
        // Fallback to standard mode
        self.browse_standard(url).await
    }
    
    #[cfg(not(feature = "terminal-graphics"))]
    async fn browse_full(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.browse_standard(url).await
    }
    
    /// Fetch URL using minimal network stack
    async fn fetch_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Use reqwest for now, can optimize later
        let response = reqwest::get(url).await?;
        let html = response.text().await?;
        Ok(html)
    }
    
    /// Extract plain text from HTML (minimal parsing)
    fn extract_text(&self, html: &str) -> String {
        let mut text = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;
        
        let lower_html = html.to_lowercase();
        
        for (i, ch) in html.chars().enumerate() {
            match ch {
                '<' => {
                    in_tag = true;
                    
                    // Check if entering script or style tag
                    if lower_html[i..].starts_with("<script") {
                        in_script = true;
                    } else if lower_html[i..].starts_with("<style") {
                        in_style = true;
                    }
                }
                '>' => {
                    in_tag = false;
                    
                    // Check if exiting script or style tag
                    if lower_html[..=i].ends_with("</script>") {
                        in_script = false;
                    } else if lower_html[..=i].ends_with("</style>") {
                        in_style = false;
                    }
                }
                _ => {
                    if !in_tag && !in_script && !in_style {
                        text.push(ch);
                    }
                }
            }
        }
        
        // Clean up whitespace
        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Wrap text to terminal width
    fn wrap_text(&self, text: &str, width: usize) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > width {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            result.push_str(&current_line);
            result.push('\n');
        }
        
        result
    }
    
    /// Parse minimal DOM (lightweight structure)
    fn parse_minimal_dom(&self, html: &str) -> Result<MinimalDOM, Box<dyn std::error::Error>> {
        Ok(MinimalDOM::parse(html))
    }
}

// ============================================================================
// Minimal DOM - Lightweight representation for CLI mode
// ============================================================================

#[derive(Debug)]
pub struct MinimalDOM {
    pub nodes: Vec<MinimalNode>,
}

#[derive(Debug, Clone)]
pub struct MinimalNode {
    pub kind: NodeKind,
    pub text: String,
    pub href: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Heading(u8),  // h1-h6
    Paragraph,
    Link,
    ListItem,
    Code,
    BlockQuote,
}

impl MinimalDOM {
    pub fn parse(html: &str) -> Self {
        let mut nodes = Vec::new();
        
        // Very basic extraction - can be enhanced
        let mut current_text = String::new();
        let mut in_tag = false;
        let mut current_tag = String::new();
        
        for ch in html.chars() {
            match ch {
                '<' => {
                    if !current_text.trim().is_empty() {
                        nodes.push(MinimalNode {
                            kind: NodeKind::Paragraph,
                            text: current_text.trim().to_string(),
                            href: None,
                        });
                        current_text.clear();
                    }
                    in_tag = true;
                    current_tag.clear();
                }
                '>' => {
                    in_tag = false;
                }
                _ => {
                    if in_tag {
                        current_tag.push(ch);
                    } else {
                        current_text.push(ch);
                    }
                }
            }
        }
        
        Self { nodes }
    }
}

// ============================================================================
// ANSI Renderer - Colored output for standard CLI mode
// ============================================================================

#[cfg(feature = "ansi-colors")]
pub struct ANSIRenderer {
    width: usize,
}

#[cfg(feature = "ansi-colors")]
impl ANSIRenderer {
    pub fn new(width: usize) -> Self {
        Self { width }
    }
    
    pub fn render(&self, dom: &MinimalDOM) -> String {
        let mut output = String::new();
        
        for node in &dom.nodes {
            match node.kind {
                NodeKind::Heading(level) => {
                    output.push_str(&self.render_heading(&node.text, level));
                }
                NodeKind::Paragraph => {
                    output.push_str(&self.wrap_text(&node.text));
                    output.push_str("\n\n");
                }
                NodeKind::Link => {
                    output.push_str(&self.render_link(&node.text, node.href.as_deref()));
                }
                NodeKind::Code => {
                    output.push_str(&self.render_code(&node.text));
                }
                _ => {
                    output.push_str(&node.text);
                    output.push('\n');
                }
            }
        }
        
        output
    }
    
    fn render_heading(&self, text: &str, level: u8) -> String {
        let marker = match level {
            1 => "━".repeat(self.width),
            2 => "─".repeat(text.len() + 4),
            _ => String::new(),
        };
        
        let color = "\x1b[1;34m"; // Bold blue
        let reset = "\x1b[0m";
        
        format!("\n{}{}\n{}{}{}\n{}\n", color, marker, color, text, reset, marker)
    }
    
    fn render_link(&self, text: &str, href: Option<&str>) -> String {
        let blue = "\x1b[34m";
        let underline = "\x1b[4m";
        let reset = "\x1b[0m";
        
        if let Some(url) = href {
            format!("{}{}{}{} [{}]", blue, underline, text, reset, url)
        } else {
            format!("{}{}{}{}", blue, underline, text, reset)
        }
    }
    
    fn render_code(&self, text: &str) -> String {
        let gray = "\x1b[90m";
        let reset = "\x1b[0m";
        
        format!("{}    {}{}\n", gray, text, reset)
    }
    
    fn wrap_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > self.width {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }
        
        result
    }
}

// ============================================================================
// Graphics Renderer - Terminal graphics protocol support
// ============================================================================

#[cfg(feature = "terminal-graphics")]
pub struct GraphicsRenderer {
    protocol: GraphicsProtocol,
    viewport: TerminalViewport,
}

#[cfg(feature = "terminal-graphics")]
impl GraphicsRenderer {
    pub fn new(protocol: GraphicsProtocol, viewport: TerminalViewport) -> Self {
        Self { protocol, viewport }
    }
    
    pub fn render(&self, dom: &MinimalDOM) -> String {
        // TODO: Implement graphics protocol rendering
        // For now, fallback to text
        format!("Graphics rendering not yet implemented\n")
    }
}

// Helper to detect terminal size (placeholder - would use termion/crossterm in production)
#[cfg(unix)]
mod term_size {
    pub fn dimensions() -> Result<(usize, usize), ()> {
        // Would use libc ioctl or termion crate
        Ok((80, 24))
    }
}
