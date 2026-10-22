// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::dom::{Dom, Node};
use std::collections::HashMap;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Stateless HTML5 parser. The only public entry point is [`HtmlParser::parse`].
pub struct HtmlParser {
    pos: usize,
    input: Vec<char>,
}

impl HtmlParser {
    /// Parse an HTML string and return a [`Dom`] rooted at an implicit `<html>` element.
    pub fn parse(html: &str) -> Dom {
        let mut p = HtmlParser {
            pos: 0,
            input: html.chars().collect(),
        };
        let nodes = p.parse_nodes(0, false);
        let root = p.ensure_html_root(nodes);
        Dom::new(root)
    }

    // -------------------------------------------------------------------------
    // Root wrapping
    // -------------------------------------------------------------------------

    fn ensure_html_root(&self, mut nodes: Vec<Node>) -> Node {
        // If the document already has a single <html> root, use it directly.
        if nodes.len() == 1 && nodes[0].tag_name() == Some("html") {
            return nodes.remove(0);
        }
        // Separate <head> and <body>-level content.
        let mut head_node: Option<Node> = None;
        let mut body_children: Vec<Node> = Vec::new();
        for node in nodes {
            match node.tag_name() {
                Some("head") => head_node = Some(node),
                Some("body") => {
                    // Flatten any explicit <body> into body_children.
                    for child in node.children {
                        body_children.push(child);
                    }
                }
                _ => body_children.push(node),
            }
        }
        let head = head_node.unwrap_or_else(|| Node::element("head".into(), HashMap::new()));
        let mut body = Node::element("body".into(), HashMap::new());
        body.children = body_children;
        let mut html = Node::element("html".into(), HashMap::new());
        html.children.push(head);
        html.children.push(body);
        html
    }

    // -------------------------------------------------------------------------
    // Node list parsing
    // -------------------------------------------------------------------------

    /// `in_element`: true when parsing children of an open tag; false at document top-level.
    /// Depth cap prevents stack overflows on pathological input (Fix 1).
    /// Stale closing tags at the top level are skipped rather than terminating parsing (Fix 8).
    fn parse_nodes(&mut self, depth: usize, in_element: bool) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.eof() {
                break;
            }
            if self.starts_with("</") {
                if in_element {
                    break; // Normal end-of-children for the enclosing element.
                } else {
                    // Top-level stale closing tag: consume and continue.
                    self.skip_to('>');
                    if !self.eof() {
                        self.consume();
                    }
                    continue;
                }
            }
            if let Some(node) = self.parse_node(depth) {
                nodes.push(node);
            }
        }
        nodes
    }

    fn parse_node(&mut self, depth: usize) -> Option<Node> {
        if self.starts_with("<!--") {
            return Some(self.parse_comment());
        }
        if self.starts_with("<!") {
            // DOCTYPE or similar — skip to '>'
            self.skip_to('>');
            if !self.eof() {
                self.consume(); // consume '>'
            }
            return None;
        }
        if self.current_is('<') {
            return Some(self.parse_element(depth));
        }
        Some(self.parse_text())
    }

    // -------------------------------------------------------------------------
    // Element parsing
    // -------------------------------------------------------------------------

    fn parse_element(&mut self, depth: usize) -> Node {
        // Depth cap: avoid stack overflow on pathological input (Fix 1).
        if depth >= 512 {
            self.skip_to('>');
            if !self.eof() {
                self.consume();
            }
            return Node::text(String::new());
        }

        self.consume(); // '<'
        let tag = self.parse_tag_name().to_lowercase();
        let attrs = self.parse_attributes();

        // Consume '>' or '/>'
        if self.starts_with("/>") {
            self.pos += 2;
        } else {
            self.skip_to('>');
            if !self.eof() {
                self.consume(); // '>'
            }
        }

        if VOID_ELEMENTS.contains(&tag.as_str()) {
            return Node::element(tag, attrs);
        }

        let children = self.parse_nodes(depth + 1, true);

        // Consume closing tag if present
        if self.starts_with("</") {
            self.skip_to('>');
            if !self.eof() {
                self.consume(); // '>'
            }
        }

        let mut node = Node::element(tag, attrs);
        node.children = children;
        node
    }

    // -------------------------------------------------------------------------
    // Attribute parsing
    // -------------------------------------------------------------------------

    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        loop {
            self.skip_whitespace();
            if self.eof() || self.current_is('>') || self.starts_with("/>") {
                break;
            }
            let name = self.parse_tag_name().to_lowercase();
            if name.is_empty() {
                self.consume(); // skip unknown char
                continue;
            }
            self.skip_whitespace();
            let value = if !self.eof() && self.current_is('=') {
                self.consume(); // '='
                self.parse_attr_value()
            } else {
                name.clone() // boolean attribute
            };
            map.insert(name, value);
        }
        map
    }

    fn parse_attr_value(&mut self) -> String {
        self.skip_whitespace();
        if self.eof() {
            return String::new();
        }
        let quote = self.input[self.pos];
        if quote == '"' || quote == '\'' {
            self.consume();
            let v = self.consume_while(|c| c != quote);
            if !self.eof() {
                self.consume(); // closing quote
            }
            v
        } else {
            self.consume_while(|c| !c.is_whitespace() && c != '>')
        }
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    // -------------------------------------------------------------------------
    // Text and comment
    // -------------------------------------------------------------------------

    fn parse_text(&mut self) -> Node {
        let text = self.consume_while(|c| c != '<');
        Node::text(text)
    }

    fn parse_comment(&mut self) -> Node {
        self.pos += 4; // skip "<!--"
        let mut content = String::new();
        while !self.eof() && !self.starts_with("-->") {
            content.push(self.consume());
        }
        if !self.eof() {
            self.pos += 3; // skip "-->"
        }
        Node::comment(content)
    }

    // -------------------------------------------------------------------------
    // Low-level helpers
    // -------------------------------------------------------------------------

    fn skip_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn skip_to(&mut self, target: char) {
        while !self.eof() && self.input[self.pos] != target {
            self.pos += 1;
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let mut s_chars = s.chars();
        let mut i = self.pos;
        loop {
            match s_chars.next() {
                None => return true,
                Some(c) => {
                    if i >= self.input.len() || self.input[i] != c {
                        return false;
                    }
                    i += 1;
                }
            }
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn current_is(&self, c: char) -> bool {
        !self.eof() && self.input[self.pos] == c
    }

    fn consume(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.input[self.pos]) {
            result.push(self.consume());
        }
        result
    }
}
