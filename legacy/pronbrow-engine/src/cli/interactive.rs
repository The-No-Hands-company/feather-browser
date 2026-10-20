// Interactive CLI Browser - Enhanced navigation and rendering
//
// Features:
// - Link numbering and keyboard navigation
// - Better ANSI rendering with tables, lists, code blocks
// - History management
// - Syntax highlighting for code
// - Interactive forms

use super::{CLIBrowser, CLIMode, MinimalDOM, NodeKind, TerminalViewport};
use std::collections::VecDeque;
use std::io::{self, Write, stdin, stdout};

/// Interactive browser session with navigation
pub struct InteractiveCLI {
    browser: CLIBrowser,
    history: NavigationHistory,
    current_page: Option<RenderedPage>,
    config: InteractiveConfig,
}

#[derive(Debug, Clone)]
pub struct InteractiveConfig {
    pub max_history: usize,
    pub enable_images: bool,
    pub enable_colors: bool,
    pub syntax_highlight: bool,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            enable_images: true,
            enable_colors: true,
            syntax_highlight: true,
        }
    }
}

/// Navigation history with back/forward
pub struct NavigationHistory {
    entries: VecDeque<String>,
    current_index: usize,
    max_entries: usize,
}

impl NavigationHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            current_index: 0,
            max_entries,
        }
    }
    
    pub fn push(&mut self, url: String) {
        // Remove forward history when navigating to new page
        self.entries.truncate(self.current_index + 1);
        
        // Add new entry
        self.entries.push_back(url);
        
        // Limit size
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        } else {
            self.current_index += 1;
        }
    }
    
    pub fn back(&mut self) -> Option<&str> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.entries.get(self.current_index).map(|s| s.as_str())
        } else {
            None
        }
    }
    
    pub fn forward(&mut self) -> Option<&str> {
        if self.current_index + 1 < self.entries.len() {
            self.current_index += 1;
            self.entries.get(self.current_index).map(|s| s.as_str())
        } else {
            None
        }
    }
    
    pub fn current(&self) -> Option<&str> {
        self.entries.get(self.current_index).map(|s| s.as_str())
    }
}

/// Rendered page with link indices
#[derive(Debug)]
pub struct RenderedPage {
    pub content: String,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub index: usize,
    pub text: String,
    pub url: String,
}

impl InteractiveCLI {
    pub fn new(mode: CLIMode, config: InteractiveConfig) -> Self {
        Self {
            browser: CLIBrowser::new(mode),
            history: NavigationHistory::new(config.max_history),
            current_page: None,
            config,
        }
    }
    
    /// Start interactive browsing session
    pub async fn start(&mut self, initial_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.navigate(initial_url).await?;
        self.run_command_loop().await
    }
    
    /// Navigate to URL
    pub async fn navigate(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let html = self.browser.browse(url).await?;
        
        // Parse and extract links
        let renderer = EnhancedRenderer::new(
            self.browser.viewport.width as usize,
            &self.config,
        );
        
        let dom = MinimalDOM::parse(&html);
        let page = renderer.render(&dom);
        
        self.current_page = Some(page);
        self.history.push(url.to_string());
        
        self.display_page();
        
        Ok(())
    }
    
    /// Display current page
    fn display_page(&self) {
        if let Some(page) = &self.current_page {
            // Clear screen
            print!("\x1b[2J\x1b[H");
            
            // Show content
            println!("{}", page.content);
            
            // Show link summary
            if !page.links.is_empty() {
                println!("\n{}", "─".repeat(self.browser.viewport.width as usize));
                println!("Links: {} total", page.links.len());
                println!("Commands: [number] to follow link, 'b' back, 'f' forward, 'q' quit");
            }
        }
    }
    
    /// Interactive command loop
    async fn run_command_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            print!("\npronbrow> ");
            stdout().flush()?;
            
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            
            let cmd = input.trim();
            
            match cmd {
                "q" | "quit" | "exit" => break,
                "b" | "back" => {
                    if let Some(url) = self.history.back() {
                        self.navigate(url).await?;
                    } else {
                        println!("No previous page");
                    }
                }
                "f" | "forward" => {
                    if let Some(url) = self.history.forward() {
                        self.navigate(url).await?;
                    } else {
                        println!("No forward page");
                    }
                }
                "h" | "help" => {
                    self.show_help();
                }
                "l" | "links" => {
                    self.show_links();
                }
                cmd if cmd.parse::<usize>().is_ok() => {
                    let index = cmd.parse::<usize>().unwrap();
                    if let Some(page) = &self.current_page {
                        if let Some(link) = page.links.get(index) {
                            self.navigate(&link.url).await?;
                        } else {
                            println!("Link {} not found", index);
                        }
                    }
                }
                url if url.starts_with("http://") || url.starts_with("https://") => {
                    self.navigate(url).await?;
                }
                _ => {
                    println!("Unknown command. Type 'help' for commands.");
                }
            }
        }
        
        Ok(())
    }
    
    fn show_help(&self) {
        println!("\n=== PronBrow Interactive Help ===");
        println!("Navigation:");
        println!("  [number]       Follow numbered link");
        println!("  [url]          Navigate to URL");
        println!("  b, back        Go back in history");
        println!("  f, forward     Go forward in history");
        println!("");
        println!("Commands:");
        println!("  h, help        Show this help");
        println!("  l, links       List all links");
        println!("  q, quit        Exit browser");
    }
    
    fn show_links(&self) {
        if let Some(page) = &self.current_page {
            println!("\n=== Links ===");
            for link in &page.links {
                println!("[{}] {} - {}", link.index, link.text, link.url);
            }
        }
    }
}

// ============================================================================
// Enhanced ANSI Renderer
// ============================================================================

pub struct EnhancedRenderer {
    width: usize,
    config: InteractiveConfig,
    link_counter: usize,
}

impl EnhancedRenderer {
    pub fn new(width: usize, config: &InteractiveConfig) -> Self {
        Self {
            width,
            config: config.clone(),
            link_counter: 0,
        }
    }
    
    pub fn render(&self, dom: &MinimalDOM) -> RenderedPage {
        let mut output = String::new();
        let mut links = Vec::new();
        let mut link_counter = 0;
        
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
                    let link_text = if let Some(ref href) = node.href {
                        let index = link_counter;
                        link_counter += 1;
                        
                        links.push(Link {
                            index,
                            text: node.text.clone(),
                            url: href.clone(),
                        });
                        
                        self.render_link(&node.text, Some(href), index)
                    } else {
                        self.render_link(&node.text, None, 0)
                    };
                    output.push_str(&link_text);
                }
                NodeKind::Code => {
                    output.push_str(&self.render_code(&node.text));
                }
                NodeKind::BlockQuote => {
                    output.push_str(&self.render_blockquote(&node.text));
                }
                NodeKind::ListItem => {
                    output.push_str(&self.render_list_item(&node.text));
                }
            }
        }
        
        RenderedPage { content: output, links }
    }
    
    fn render_heading(&self, text: &str, level: u8) -> String {
        if !self.config.enable_colors {
            return format!("\n{}\n{}\n", text, "=".repeat(text.len()));
        }
        
        let (color, marker) = match level {
            1 => ("\x1b[1;35m", "━"), // Bold magenta
            2 => ("\x1b[1;34m", "─"), // Bold blue
            3 => ("\x1b[1;36m", "─"), // Bold cyan
            _ => ("\x1b[1m", "─"),     // Bold
        };
        
        let reset = "\x1b[0m";
        let marker_line = marker.repeat(self.width.min(text.len() + 4));
        
        format!("\n{}{}\n{}{}{}\n{}\n", 
            color, marker_line,
            color, text, reset,
            marker_line
        )
    }
    
    fn render_link(&self, text: &str, href: Option<&str>, index: usize) -> String {
        if !self.config.enable_colors {
            return if let Some(url) = href {
                format!("[{}] {} ({})", index, text, url)
            } else {
                text.to_string()
            };
        }
        
        let blue = "\x1b[34m";
        let cyan = "\x1b[36m";
        let underline = "\x1b[4m";
        let reset = "\x1b[0m";
        
        if let Some(_) = href {
            format!("{}[{}]{} {}{}{}{}", 
                cyan, index, reset,
                blue, underline, text, reset
            )
        } else {
            format!("{}{}{}{}", blue, underline, text, reset)
        }
    }
    
    fn render_code(&self, text: &str) -> String {
        if !self.config.enable_colors {
            return format!("    {}\n", text);
        }
        
        let bg = "\x1b[48;5;236m";  // Dark gray background
        let fg = "\x1b[38;5;156m";  // Light green text
        let reset = "\x1b[0m";
        
        let lines: Vec<&str> = text.lines().collect();
        let mut output = String::new();
        
        output.push_str(&format!("\n{}╭{}╮{}\n", fg, "─".repeat(self.width - 4), reset));
        
        for line in lines {
            output.push_str(&format!("{}│{} {:<width$} {}│{}\n",
                fg, bg, line, reset, fg, reset,
                width = self.width - 6
            ));
        }
        
        output.push_str(&format!("{}╰{}╯{}\n\n", fg, "─".repeat(self.width - 4), reset));
        
        output
    }
    
    fn render_blockquote(&self, text: &str) -> String {
        if !self.config.enable_colors {
            return format!("  | {}\n", text);
        }
        
        let gray = "\x1b[90m";
        let italic = "\x1b[3m";
        let reset = "\x1b[0m";
        
        let lines: Vec<&str> = text.lines().collect();
        let mut output = String::new();
        
        for line in lines {
            output.push_str(&format!("{}│{} {}{}\n", 
                gray, italic, line, reset
            ));
        }
        
        output.push('\n');
        output
    }
    
    fn render_list_item(&self, text: &str) -> String {
        if !self.config.enable_colors {
            return format!("  • {}\n", text);
        }
        
        let bullet = "\x1b[33m•\x1b[0m"; // Yellow bullet
        format!("  {} {}\n", bullet, text)
    }
    
    fn render_table(&self, rows: &[Vec<String>]) -> String {
        if rows.is_empty() {
            return String::new();
        }
        
        // Calculate column widths
        let col_count = rows[0].len();
        let mut col_widths = vec![0; col_count];
        
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
        
        let mut output = String::new();
        
        if self.config.enable_colors {
            let cyan = "\x1b[36m";
            let reset = "\x1b[0m";
            
            // Top border
            output.push_str(&format!("{}┌", cyan));
            for (i, &width) in col_widths.iter().enumerate() {
                output.push_str(&"─".repeat(width + 2));
                if i < col_widths.len() - 1 {
                    output.push_str("┬");
                }
            }
            output.push_str(&format!("┐{}\n", reset));
            
            // Rows
            for (row_idx, row) in rows.iter().enumerate() {
                output.push_str(&format!("{}│{} ", cyan, reset));
                for (i, cell) in row.iter().enumerate() {
                    output.push_str(&format!("{:<width$} ", cell, width = col_widths[i]));
                    output.push_str(&format!("{}│{} ", cyan, reset));
                }
                output.push('\n');
                
                // Header separator
                if row_idx == 0 {
                    output.push_str(&format!("{}├", cyan));
                    for (i, &width) in col_widths.iter().enumerate() {
                        output.push_str(&"─".repeat(width + 2));
                        if i < col_widths.len() - 1 {
                            output.push_str("┼");
                        }
                    }
                    output.push_str(&format!("┤{}\n", reset));
                }
            }
            
            // Bottom border
            output.push_str(&format!("{}└", cyan));
            for (i, &width) in col_widths.iter().enumerate() {
                output.push_str(&"─".repeat(width + 2));
                if i < col_widths.len() - 1 {
                    output.push_str("┴");
                }
            }
            output.push_str(&format!("┘{}\n", reset));
        } else {
            // Plain ASCII table
            for row in rows {
                output.push_str("| ");
                for cell in row {
                    output.push_str(&format!("{} | ", cell));
                }
                output.push('\n');
            }
        }
        
        output
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
// Syntax Highlighter for Code Blocks
// ============================================================================

pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    pub fn highlight(code: &str, language: &str) -> String {
        match language.to_lowercase().as_str() {
            "rust" => Self::highlight_rust(code),
            "javascript" | "js" => Self::highlight_javascript(code),
            "python" => Self::highlight_python(code),
            "html" => Self::highlight_html(code),
            "css" => Self::highlight_css(code),
            _ => code.to_string(),
        }
    }
    
    fn highlight_rust(code: &str) -> String {
        let keywords = ["fn", "let", "mut", "const", "if", "else", "match", "return", "struct", "impl", "pub"];
        let mut result = code.to_string();
        
        let keyword_color = "\x1b[35m"; // Magenta
        let reset = "\x1b[0m";
        
        for kw in &keywords {
            result = result.replace(kw, &format!("{}{}{}", keyword_color, kw, reset));
        }
        
        result
    }
    
    fn highlight_javascript(code: &str) -> String {
        let keywords = ["function", "const", "let", "var", "if", "else", "return", "class", "async", "await"];
        let mut result = code.to_string();
        
        let keyword_color = "\x1b[33m"; // Yellow
        let reset = "\x1b[0m";
        
        for kw in &keywords {
            result = result.replace(kw, &format!("{}{}{}", keyword_color, kw, reset));
        }
        
        result
    }
    
    fn highlight_python(code: &str) -> String {
        let keywords = ["def", "class", "if", "else", "elif", "return", "import", "from", "for", "while"];
        let mut result = code.to_string();
        
        let keyword_color = "\x1b[34m"; // Blue
        let reset = "\x1b[0m";
        
        for kw in &keywords {
            result = result.replace(kw, &format!("{}{}{}", keyword_color, kw, reset));
        }
        
        result
    }
    
    fn highlight_html(code: &str) -> String {
        // Simple tag highlighting
        let tag_color = "\x1b[32m"; // Green
        let reset = "\x1b[0m";
        
        code.replace("<", &format!("{}<", tag_color))
            .replace(">", &format!(">{}", reset))
    }
    
    fn highlight_css(code: &str) -> String {
        let property_color = "\x1b[36m"; // Cyan
        let reset = "\x1b[0m";
        
        // Highlight properties (simple approach)
        let mut result = String::new();
        for line in code.lines() {
            if line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    result.push_str(&format!("{}{}{}: {}\n", 
                        property_color, parts[0], reset, parts[1]
                    ));
                    continue;
                }
            }
            result.push_str(line);
            result.push('\n');
        }
        
        result
    }
}
