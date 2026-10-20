// CSS Parser - Parses CSS stylesheets and applies styles to DOM

use std::collections::HashMap;

// Feature modules
pub mod flexbox;
pub mod grid;
pub mod animation;
pub mod transform;
pub mod media;
pub mod pseudo;
pub mod values;
pub mod matching;
pub mod animator;

// Re-export matching functionality
pub use matching::{style_tree, matches, specified_values};
pub use animator::{AnimationEngine, RunningAnimation, Transform, EasingFunction};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Selector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

impl Selector {
    pub fn specificity(&self) -> (usize, usize, usize) {
        // (ID, class, tag)
        let id = if self.id.is_some() { 1 } else { 0 };
        let class = self.classes.len();
        let tag = if self.tag_name.is_some() { 1 } else { 0 };
        (id, class, tag)
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct CssParser {
    pos: usize,
    input: String,
}

impl CssParser {
    pub fn new() -> Self {
        CssParser {
            pos: 0,
            input: String::new(),
        }
    }

    pub fn parse(&mut self, css: &str) -> Result<Stylesheet, Box<dyn std::error::Error>> {
        self.pos = 0;
        self.input = css.to_string();
        Ok(Stylesheet {
            rules: self.parse_rules(),
        })
    }

    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(self.parse_selector());
            self.consume_whitespace();
            if self.current_char() == '{' {
                self.consume_char();
                break;
            }
            if self.current_char() == ',' {
                self.consume_char();
            }
        }
        selectors
    }

    fn parse_selector(&mut self) -> Selector {
        let mut selector = Selector {
            tag_name: None,
            id: None,
            classes: Vec::new(),
        };

        self.consume_whitespace();
        
        while !self.eof() && self.current_char() != ',' && self.current_char() != '{' {
            match self.current_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.classes.push(self.parse_identifier());
                }
                c if c.is_alphabetic() => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => {
                    self.consume_char();
                }
            }
        }

        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.current_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        if self.current_char() == ';' {
            self.consume_char();
        }
        Declaration { name, value }
    }

    fn parse_value(&mut self) -> Value {
        self.consume_whitespace();
        match self.current_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_hex_color(),
            _ => {
                let ident = self.parse_identifier();
                // Check if it's rgb() or rgba()
                if ident == "rgb" || ident == "rgba" {
                    self.parse_rgb_color(ident == "rgba")
                } else {
                    Value::Keyword(ident)
                }
            }
        }
    }

    fn parse_length(&mut self) -> Value {
        let num_str = self.consume_while(|c| c.is_numeric() || c == '.');
        let num: f32 = num_str.parse().unwrap();
        let unit = self.parse_unit();
        Value::Length(num, unit)
    }

    fn parse_unit(&mut self) -> Unit {
        let unit_str = self.parse_identifier().to_lowercase();
        match unit_str.as_str() {
            "px" => Unit::Px,
            "em" => Unit::Em,
            "rem" => Unit::Rem,
            "%" => Unit::Percent,
            _ => Unit::Px,
        }
    }

    fn parse_hex_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        let hex = self.consume_while(|c| c.is_ascii_hexdigit());
        
        let (r, g, b) = if hex.len() == 3 {
            // Short form: #RGB -> #RRGGBB
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap();
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap();
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap();
            (r, g, b)
        } else {
            // Full form: #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            (r, g, b)
        };
        
        Value::Color(Color { r, g, b, a: 255 })
    }

    fn parse_rgb_color(&mut self, has_alpha: bool) -> Value {
        self.consume_whitespace();
        assert_eq!(self.consume_char(), '(');
        self.consume_whitespace();
        
        // Parse R value
        let r_str = self.consume_while(|c| c.is_numeric());
        let r = r_str.parse::<u8>().unwrap_or(0);
        self.consume_whitespace();
        if self.current_char() == ',' {
            self.consume_char();
        }
        self.consume_whitespace();
        
        // Parse G value
        let g_str = self.consume_while(|c| c.is_numeric());
        let g = g_str.parse::<u8>().unwrap_or(0);
        self.consume_whitespace();
        if self.current_char() == ',' {
            self.consume_char();
        }
        self.consume_whitespace();
        
        // Parse B value
        let b_str = self.consume_while(|c| c.is_numeric());
        let b = b_str.parse::<u8>().unwrap_or(0);
        self.consume_whitespace();
        
        let a = if has_alpha {
            if self.current_char() == ',' {
                self.consume_char();
            }
            self.consume_whitespace();
            let a_str = self.consume_while(|c| c.is_numeric() || c == '.');
            let alpha_float = a_str.parse::<f32>().unwrap_or(1.0);
            (alpha_float * 255.0) as u8
        } else {
            255
        };
        
        self.consume_whitespace();
        if self.current_char() == ')' {
            self.consume_char();
        }
        
        Value::Color(Color { r, g, b, a })
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.current_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new()
    }
}

pub type PropertyMap = HashMap<String, Value>;

pub fn get_stylesheet_for_dom() -> Stylesheet {
    // Default browser stylesheet (user agent stylesheet)
    let default_css = r#"
        html { display: block; }
        body { display: block; margin: 8px; }
        h1 { display: block; font-size: 2em; font-weight: bold; margin: 0.67em 0; }
        h2 { display: block; font-size: 1.5em; font-weight: bold; margin: 0.83em 0; }
        p { display: block; margin: 1em 0; }
        div { display: block; }
        span { display: inline; }
    "#;
    
    let mut parser = CssParser::new();
    parser.parse(default_css).unwrap()
}
