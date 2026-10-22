// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct CssColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl CssColor {
    pub const WHITE: CssColor = CssColor { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: CssColor = CssColor { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: CssColor = CssColor { r: 0, g: 0, b: 0, a: 0 };
    pub const DARK: CssColor = CssColor { r: 30, g: 30, b: 30, a: 255 };

    pub fn from_keyword(s: &str) -> Option<Self> {
        match s {
            "red"     => Some(CssColor { r: 255, g: 0,   b: 0,   a: 255 }),
            "green"   => Some(CssColor { r: 0,   g: 128, b: 0,   a: 255 }),
            "blue"    => Some(CssColor { r: 0,   g: 0,   b: 255, a: 255 }),
            "black"   => Some(CssColor::BLACK),
            "white"   => Some(CssColor::WHITE),
            "gray" | "grey" => Some(CssColor { r: 128, g: 128, b: 128, a: 255 }),
            "yellow"  => Some(CssColor { r: 255, g: 255, b: 0,   a: 255 }),
            "orange"  => Some(CssColor { r: 255, g: 165, b: 0,   a: 255 }),
            "purple"  => Some(CssColor { r: 128, g: 0,   b: 128, a: 255 }),
            "pink"    => Some(CssColor { r: 255, g: 192, b: 203, a: 255 }),
            "transparent" => Some(CssColor::TRANSPARENT),
            _ => None,
        }
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
                Some(CssColor { r, g, b, a: 255 })
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(CssColor { r, g, b, a: 255 })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(CssColor),
    Number(f32),
}

#[derive(Debug, Clone)]
pub struct Selector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub ancestor: Option<(Combinator, Box<Selector>)>,
}

impl Selector {
    /// Returns (id_count, class_count, tag_count) — higher is more specific.
    pub fn specificity(&self) -> (usize, usize, usize) {
        let (ai, ac, at) = self.ancestor.as_ref()
            .map(|(_, a)| a.specificity())
            .unwrap_or((0, 0, 0));
        (
            ai + self.id.is_some() as usize,
            ac + self.classes.len(),
            at + self.tag_name.is_some() as usize,
        )
    }
}

/// A single CSS declaration: (property-name, value).
pub type Declaration = (String, Value);

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// Stateless CSS parser. Handles: tag/class/id selectors, px/em/% lengths,
/// named colors, hex colors, and keyword values. Unknown values → Keyword.
pub struct CssParser {
    pos: usize,
    input: Vec<char>,
}

impl CssParser {
    pub fn parse(css: &str) -> Stylesheet {
        let mut p = CssParser { pos: 0, input: css.chars().collect() };
        let rules = p.parse_rules();
        Stylesheet { rules }
    }

    /// Parses a bare declaration list, as found in an element's `style` attribute.
    pub fn parse_inline_declarations(raw: &str) -> Vec<Declaration> {
        let mut p = CssParser { pos: 0, input: raw.chars().collect() };
        p.parse_declarations()
    }

    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.eof() { break; }
            let pos_before = self.pos;
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
            if self.pos == pos_before && !self.eof() {
                self.consume();
            }
        }
        rules
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selectors();
        if selectors.is_empty() { self.skip_block(); return None; }
        self.skip_whitespace_and_comments();
        if self.eof() || self.current() != '{' { return None; }
        self.consume(); // '{'
        let declarations = self.parse_declarations();
        if !self.eof() && self.current() == '}' { self.consume(); }
        Some(Rule { selectors, declarations })
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.eof() || self.current() == '{' { break; }
            selectors.push(self.parse_compound_selector());
            self.skip_whitespace_and_comments();
            if self.eof() || self.current() != ',' { break; }
            self.consume(); // ','
        }
        selectors
    }

    fn parse_compound_selector(&mut self) -> Selector {
        let mut left = self.parse_simple_selector();
        let mut depth: usize = 0;
        loop {
            if depth >= 32 { break; }
            // Detect combinator: whitespace before a non-special char, or explicit '>'
            let had_whitespace = !self.eof() && self.input[self.pos].is_whitespace();
            self.skip_whitespace_and_comments();
            if self.eof() || self.current() == '{' || self.current() == ',' {
                break;
            }
            let combinator = if self.current() == '>' {
                self.consume(); // '>'
                self.skip_whitespace_and_comments();
                Combinator::Child
            } else if had_whitespace
                && (self.current().is_alphanumeric()
                    || matches!(self.current(), '#' | '.' | '*'))
            {
                Combinator::Descendant
            } else {
                break;
            };
            let right = self.parse_simple_selector();
            left = Selector {
                tag_name: right.tag_name,
                id: right.id,
                classes: right.classes,
                ancestor: Some((combinator, Box::new(left))),
            };
            depth += 1;
        }
        left
    }

    fn parse_simple_selector(&mut self) -> Selector {
        let mut sel = Selector { tag_name: None, id: None, classes: Vec::new(), ancestor: None };
        while !self.eof() {
            let c = self.current();
            if c == '#' {
                self.consume();
                sel.id = Some(self.parse_identifier());
            } else if c == '.' {
                self.consume();
                sel.classes.push(self.parse_identifier());
            } else if c == '*' {
                self.consume(); // universal selector — matches everything
            } else if c.is_alphanumeric() || c == '-' || c == '_' {
                sel.tag_name = Some(self.parse_identifier());
            } else {
                break;
            }
        }
        sel
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.eof() || self.current() == '}' { break; }
            decls.extend(self.parse_declaration());
        }
        decls
    }

    fn parse_declaration(&mut self) -> Vec<Declaration> {
        let name = self.parse_identifier();
        if name.is_empty() { self.skip_to(';'); self.consume_if(';'); return vec![]; }
        self.skip_whitespace_and_comments();
        if self.eof() || self.current() != ':' { self.skip_to(';'); self.consume_if(';'); return vec![]; }
        self.consume(); // ':'
        self.skip_whitespace_and_comments();
        let raw = self.consume_while(|c| c != ';' && c != '}').trim().to_string();
        self.consume_if(';');
        expand_shorthand(&name, &raw)
    }

    pub fn parse_value(raw: &str) -> Value {
        let s = raw.trim();
        // rem must be checked before em (longer suffix first)
        if let Some(rest) = s.strip_suffix("rem") {
            if let Ok(n) = rest.trim().parse::<f32>() { return Value::Length(n, Unit::Rem); }
        }
        if let Some(rest) = s.strip_suffix("px") {
            if let Ok(n) = rest.trim().parse::<f32>() { return Value::Length(n, Unit::Px); }
        }
        if let Some(rest) = s.strip_suffix("em") {
            if let Ok(n) = rest.trim().parse::<f32>() { return Value::Length(n, Unit::Em); }
        }
        if let Some(rest) = s.strip_suffix('%') {
            if let Ok(n) = rest.trim().parse::<f32>() { return Value::Length(n, Unit::Percent); }
        }
        // bare number
        if let Ok(n) = s.parse::<f32>() { return Value::Number(n); }
        // hex color
        if s.starts_with('#') {
            if let Some(c) = CssColor::from_hex(s) { return Value::Color(c); }
        }
        // named color
        if let Some(c) = CssColor::from_keyword(s) { return Value::Color(c); }
        // fallback
        Value::Keyword(s.to_string())
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_').to_lowercase()
    }

    fn skip_block(&mut self) {
        if let Some(open) = self.input[self.pos..].iter().position(|&c| c == '{') {
            self.pos += open + 1;
            self.skip_to('}');
            self.consume_if('}');
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.consume_while(|c| c.is_whitespace());
            if self.starts_with("/*") {
                self.pos += 2;
                while !self.eof() && !self.starts_with("*/") { self.consume(); }
                if !self.eof() { self.pos += 2; }
            } else {
                break;
            }
        }
    }

    fn skip_to(&mut self, target: char) {
        self.consume_while(|c| c != target);
    }

    fn consume_if(&mut self, c: char) {
        if !self.eof() && self.current() == c { self.consume(); }
    }

    fn starts_with(&self, s: &str) -> bool {
        let mut s_chars = s.chars();
        let mut i = self.pos;
        loop {
            match s_chars.next() {
                None => return true,
                Some(c) => {
                    if i >= self.input.len() || self.input[i] != c { return false; }
                    i += 1;
                }
            }
        }
    }

    fn eof(&self) -> bool { self.pos >= self.input.len() }
    fn current(&self) -> char { self.input[self.pos] }
    fn consume(&mut self) -> char { let c = self.input[self.pos]; self.pos += 1; c }
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut s = String::new();
        while !self.eof() && test(self.current()) { s.push(self.consume()); }
        s
    }
}

fn expand_shorthand(prop: &str, raw: &str) -> Vec<(String, Value)> {
    match prop {
        "margin" | "padding" => {
            let values: Vec<Value> = raw.split_whitespace()
                .map(CssParser::parse_value)
                .collect();
            expand_box_model(prop, &values)
        }
        "border-width" => {
            let values: Vec<Value> = raw.split_whitespace()
                .map(CssParser::parse_value)
                .collect();
            expand_box_model("border", &values)
        }
        "font" => expand_font_shorthand(raw),
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let mut decls = Vec::new();
            let sides: &[&str] = match prop {
                "border-top"    => &["top"],
                "border-right"  => &["right"],
                "border-bottom" => &["bottom"],
                "border-left"   => &["left"],
                _               => &["top", "right", "bottom", "left"],
            };
            let mut width: Option<Value> = None;
            let mut color: Option<Value> = None;
            for part in raw.split_whitespace() {
                let v = CssParser::parse_value(part);
                match &v {
                    Value::Length(_, _) if width.is_none() => width = Some(v),
                    Value::Color(_)                        => color = Some(v),
                    _ => {}
                }
            }
            if let Some(w) = width {
                for side in sides {
                    decls.push((format!("border-{}-width", side), w.clone()));
                }
            }
            if let Some(c) = color {
                decls.push(("border-color".to_string(), c));
            }
            decls
        }
        _ => vec![(prop.to_string(), CssParser::parse_value(raw))],
    }
}

fn expand_box_model(prefix: &str, values: &[Value]) -> Vec<(String, Value)> {
    if values.is_empty() { return vec![]; }
    let (top, right, bottom, left) = match values.len() {
        1 => (values[0].clone(), values[0].clone(), values[0].clone(), values[0].clone()),
        2 => (values[0].clone(), values[1].clone(), values[0].clone(), values[1].clone()),
        3 => (values[0].clone(), values[1].clone(), values[2].clone(), values[1].clone()),
        _ => (values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone()),
    };
    let suffix = if prefix == "border" { "-width" } else { "" };
    vec![
        (format!("{}-top{}", prefix, suffix),    top),
        (format!("{}-right{}", prefix, suffix),  right),
        (format!("{}-bottom{}", prefix, suffix), bottom),
        (format!("{}-left{}", prefix, suffix),   left),
    ]
}

fn expand_font_shorthand(raw: &str) -> Vec<(String, Value)> {
    let mut decls = Vec::new();
    let parts: Vec<&str> = raw.split_whitespace().collect();
    for part in &parts {
        if part.ends_with("px") || part.ends_with("em") || part.ends_with("rem") {
            let v = CssParser::parse_value(part);
            if matches!(v, Value::Length(_, _)) {
                decls.push(("font-size".to_string(), v));
                break;
            }
        }
    }
    for part in &parts {
        match *part {
            "bold" => { decls.push(("font-weight".to_string(), Value::Keyword("bold".to_string()))); break; }
            "normal" => { decls.push(("font-weight".to_string(), Value::Keyword("normal".to_string()))); break; }
            _ => {}
        }
    }
    decls
}
