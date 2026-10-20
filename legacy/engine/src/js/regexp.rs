// Regular Expression Engine

#[derive(Debug, Clone)]
pub struct RegExp {
    pattern: String,
    flags: RegExpFlags,
    compiled: CompiledRegex,
}

#[derive(Debug, Clone)]
pub struct RegExpFlags {
    pub global: bool,       // g
    pub ignore_case: bool,  // i
    pub multiline: bool,    // m
    pub dot_all: bool,      // s
    pub unicode: bool,      // u
    pub sticky: bool,       // y
}

impl Default for RegExpFlags {
    fn default() -> Self {
        RegExpFlags {
            global: false,
            ignore_case: false,
            multiline: false,
            dot_all: false,
            unicode: false,
            sticky: false,
        }
    }
}

#[derive(Debug, Clone)]
enum RegexNode {
    Literal(char),
    Dot,                    // .
    Start,                  // ^
    End,                    // $
    CharClass(Vec<CharRange>),
    NegatedCharClass(Vec<CharRange>),
    Group(Box<RegexNode>),
    Alternation(Vec<RegexNode>),
    Sequence(Vec<RegexNode>),
    ZeroOrMore(Box<RegexNode>),      // *
    OneOrMore(Box<RegexNode>),       // +
    ZeroOrOne(Box<RegexNode>),       // ?
    Exactly(Box<RegexNode>, usize),  // {n}
    Range(Box<RegexNode>, usize, Option<usize>),  // {n,m}
    Backreference(usize),
    LookaheadPositive(Box<RegexNode>),
    LookaheadNegative(Box<RegexNode>),
    LookbehindPositive(Box<RegexNode>),
    LookbehindNegative(Box<RegexNode>),
}

#[derive(Debug, Clone)]
struct CharRange {
    start: char,
    end: char,
}

#[derive(Debug, Clone)]
struct CompiledRegex {
    root: RegexNode,
}

impl RegExp {
    pub fn new(pattern: &str, flags: &str) -> Result<Self, String> {
        let flags = Self::parse_flags(flags)?;
        let compiled = Self::compile(pattern)?;
        
        Ok(RegExp {
            pattern: pattern.to_string(),
            flags,
            compiled,
        })
    }
    
    fn parse_flags(flags: &str) -> Result<RegExpFlags, String> {
        let mut result = RegExpFlags::default();
        
        for ch in flags.chars() {
            match ch {
                'g' => result.global = true,
                'i' => result.ignore_case = true,
                'm' => result.multiline = true,
                's' => result.dot_all = true,
                'u' => result.unicode = true,
                'y' => result.sticky = true,
                _ => return Err(format!("Invalid flag: {}", ch)),
            }
        }
        
        Ok(result)
    }
    
    fn compile(pattern: &str) -> Result<CompiledRegex, String> {
        let mut parser = RegexParser::new(pattern);
        let root = parser.parse()?;
        Ok(CompiledRegex { root })
    }
    
    pub fn test(&self, text: &str) -> bool {
        self.exec(text).is_some()
    }
    
    pub fn exec(&self, text: &str) -> Option<RegExpMatch> {
        let mut matcher = RegexMatcher::new(&self.compiled.root, text, &self.flags);
        matcher.find_match()
    }
    
    pub fn match_all(&self, text: &str) -> Vec<RegExpMatch> {
        if !self.flags.global {
            return self.exec(text).into_iter().collect();
        }
        
        let mut matches = Vec::new();
        let mut matcher = RegexMatcher::new(&self.compiled.root, text, &self.flags);
        
        while let Some(m) = matcher.find_match() {
            matches.push(m);
        }
        
        matches
    }
    
    pub fn replace(&self, text: &str, replacement: &str) -> String {
        let matches = if self.flags.global {
            self.match_all(text)
        } else {
            self.exec(text).into_iter().collect()
        };
        
        let mut result = text.to_string();
        for m in matches.iter().rev() {
            let replaced = Self::apply_replacement(replacement, &m);
            result.replace_range(m.index..m.index + m.text.len(), &replaced);
        }
        
        result
    }
    
    fn apply_replacement(replacement: &str, matched: &RegExpMatch) -> String {
        let mut result = replacement.to_string();
        
        // Replace $& with full match
        result = result.replace("$&", &matched.text);
        
        // Replace $n with capture groups
        for (i, capture) in matched.captures.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            if let Some(cap) = capture {
                result = result.replace(&placeholder, cap);
            }
        }
        
        result
    }
    
    pub fn split(&self, text: &str) -> Vec<String> {
        let matches = self.match_all(text);
        let mut result = Vec::new();
        let mut last_index = 0;
        
        for m in &matches {
            if m.index > last_index {
                result.push(text[last_index..m.index].to_string());
            }
            last_index = m.index + m.text.len();
        }
        
        if last_index < text.len() {
            result.push(text[last_index..].to_string());
        }
        
        result
    }
}

#[derive(Debug, Clone)]
pub struct RegExpMatch {
    pub text: String,
    pub index: usize,
    pub captures: Vec<Option<String>>,
}

struct RegexParser {
    pattern: Vec<char>,
    pos: usize,
}

impl RegexParser {
    fn new(pattern: &str) -> Self {
        RegexParser {
            pattern: pattern.chars().collect(),
            pos: 0,
        }
    }
    
    fn parse(&mut self) -> Result<RegexNode, String> {
        self.parse_alternation()
    }
    
    fn parse_alternation(&mut self) -> Result<RegexNode, String> {
        let mut alternatives = vec![self.parse_sequence()?];
        
        while self.peek() == Some('|') {
            self.advance();
            alternatives.push(self.parse_sequence()?);
        }
        
        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(RegexNode::Alternation(alternatives))
        }
    }
    
    fn parse_sequence(&mut self) -> Result<RegexNode, String> {
        let mut nodes = Vec::new();
        
        while let Some(ch) = self.peek() {
            if ch == '|' || ch == ')' {
                break;
            }
            nodes.push(self.parse_quantified()?);
        }
        
        if nodes.is_empty() {
            Ok(RegexNode::Sequence(vec![]))
        } else if nodes.len() == 1 {
            Ok(nodes.into_iter().next().unwrap())
        } else {
            Ok(RegexNode::Sequence(nodes))
        }
    }
    
    fn parse_quantified(&mut self) -> Result<RegexNode, String> {
        let node = self.parse_atom()?;
        
        match self.peek() {
            Some('*') => {
                self.advance();
                Ok(RegexNode::ZeroOrMore(Box::new(node)))
            }
            Some('+') => {
                self.advance();
                Ok(RegexNode::OneOrMore(Box::new(node)))
            }
            Some('?') => {
                self.advance();
                Ok(RegexNode::ZeroOrOne(Box::new(node)))
            }
            Some('{') => {
                self.advance();
                self.parse_quantifier_range(node)
            }
            _ => Ok(node),
        }
    }
    
    fn parse_quantifier_range(&mut self, node: RegexNode) -> Result<RegexNode, String> {
        let n = self.parse_number()?;
        
        if self.peek() == Some(',') {
            self.advance();
            let m = if self.peek() == Some('}') {
                None
            } else {
                Some(self.parse_number()?)
            };
            self.expect('}')?;
            Ok(RegexNode::Range(Box::new(node), n, m))
        } else {
            self.expect('}')?;
            Ok(RegexNode::Exactly(Box::new(node), n))
        }
    }
    
    fn parse_atom(&mut self) -> Result<RegexNode, String> {
        match self.peek() {
            Some('.') => {
                self.advance();
                Ok(RegexNode::Dot)
            }
            Some('^') => {
                self.advance();
                Ok(RegexNode::Start)
            }
            Some('$') => {
                self.advance();
                Ok(RegexNode::End)
            }
            Some('[') => {
                self.advance();
                self.parse_char_class()
            }
            Some('(') => {
                self.advance();
                let inner = self.parse_alternation()?;
                self.expect(')')?;
                Ok(RegexNode::Group(Box::new(inner)))
            }
            Some('\\') => {
                self.advance();
                self.parse_escape()
            }
            Some(ch) => {
                let c = ch;
                self.advance();
                Ok(RegexNode::Literal(c))
            }
            None => Err("Unexpected end of pattern".to_string()),
        }
    }
    
    fn parse_char_class(&mut self) -> Result<RegexNode, String> {
        let negated = self.peek() == Some('^');
        if negated {
            self.advance();
        }
        
        let mut ranges = Vec::new();
        
        while self.peek() != Some(']') && self.peek().is_some() {
            let start = self.advance().unwrap();
            
            if self.peek() == Some('-') && self.peek_ahead(1) != Some(']') {
                self.advance(); // consume '-'
                let end = self.advance().ok_or("Expected character after '-'")?;
                ranges.push(CharRange { start, end });
            } else {
                ranges.push(CharRange { start, end: start });
            }
        }
        
        self.expect(']')?;
        
        if negated {
            Ok(RegexNode::NegatedCharClass(ranges))
        } else {
            Ok(RegexNode::CharClass(ranges))
        }
    }
    
    fn parse_escape(&mut self) -> Result<RegexNode, String> {
        match self.peek() {
            Some('d') => {
                self.advance();
                Ok(RegexNode::CharClass(vec![CharRange { start: '0', end: '9' }]))
            }
            Some('w') => {
                self.advance();
                Ok(RegexNode::CharClass(vec![
                    CharRange { start: 'a', end: 'z' },
                    CharRange { start: 'A', end: 'Z' },
                    CharRange { start: '0', end: '9' },
                    CharRange { start: '_', end: '_' },
                ]))
            }
            Some('s') => {
                self.advance();
                Ok(RegexNode::Literal(' ')) // Simplified
            }
            Some(ch) => {
                let c = ch;
                self.advance();
                Ok(RegexNode::Literal(c))
            }
            None => Err("Unexpected end after '\\'".to_string()),
        }
    }
    
    fn parse_number(&mut self) -> Result<usize, String> {
        let mut num = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        num.parse().map_err(|_| "Invalid number".to_string())
    }
    
    fn peek(&self) -> Option<char> {
        self.pattern.get(self.pos).copied()
    }
    
    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.pattern.get(self.pos + offset).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }
    
    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected '{}', got '{:?}'", expected, self.peek()))
        }
    }
}

struct RegexMatcher<'a> {
    regex: &'a RegexNode,
    text: &'a str,
    pos: usize,
    flags: &'a RegExpFlags,
    captures: Vec<Option<String>>,
}

impl<'a> RegexMatcher<'a> {
    fn new(regex: &'a RegexNode, text: &'a str, flags: &'a RegExpFlags) -> Self {
        RegexMatcher {
            regex,
            text,
            pos: 0,
            flags,
            captures: Vec::new(),
        }
    }
    
    fn find_match(&mut self) -> Option<RegExpMatch> {
        while self.pos <= self.text.len() {
            let start = self.pos;
            self.captures.clear();
            
            if let Some(len) = self.match_node(self.regex, self.pos) {
                return Some(RegExpMatch {
                    text: self.text[start..start + len].to_string(),
                    index: start,
                    captures: self.captures.clone(),
                });
            }
            
            if self.flags.sticky {
                return None;
            }
            
            self.pos += 1;
        }
        
        None
    }
    
    fn match_node(&mut self, node: &RegexNode, pos: usize) -> Option<usize> {
        match node {
            RegexNode::Literal(ch) => {
                let text_char = self.text.chars().nth(pos)?;
                if self.flags.ignore_case {
                    if ch.to_lowercase().eq(text_char.to_lowercase()) {
                        Some(1)
                    } else {
                        None
                    }
                } else if *ch == text_char {
                    Some(1)
                } else {
                    None
                }
            }
            RegexNode::Dot => {
                if pos < self.text.len() {
                    Some(1)
                } else {
                    None
                }
            }
            RegexNode::Start => {
                if pos == 0 || (self.flags.multiline && self.text.chars().nth(pos - 1) == Some('\n')) {
                    Some(0)
                } else {
                    None
                }
            }
            RegexNode::End => {
                if pos == self.text.len() || (self.flags.multiline && self.text.chars().nth(pos) == Some('\n')) {
                    Some(0)
                } else {
                    None
                }
            }
            RegexNode::CharClass(ranges) => {
                let ch = self.text.chars().nth(pos)?;
                for range in ranges {
                    if ch >= range.start && ch <= range.end {
                        return Some(1);
                    }
                }
                None
            }
            RegexNode::Sequence(nodes) => {
                let mut total = 0;
                let mut current_pos = pos;
                
                for node in nodes {
                    if let Some(len) = self.match_node(node, current_pos) {
                        total += len;
                        current_pos += len;
                    } else {
                        return None;
                    }
                }
                
                Some(total)
            }
            RegexNode::Alternation(alternatives) => {
                for alt in alternatives {
                    if let Some(len) = self.match_node(alt, pos) {
                        return Some(len);
                    }
                }
                None
            }
            RegexNode::ZeroOrMore(inner) => {
                let mut total = 0;
                let mut current_pos = pos;
                
                while let Some(len) = self.match_node(inner, current_pos) {
                    total += len;
                    current_pos += len;
                }
                
                Some(total)
            }
            RegexNode::OneOrMore(inner) => {
                let first = self.match_node(inner, pos)?;
                let mut total = first;
                let mut current_pos = pos + first;
                
                while let Some(len) = self.match_node(inner, current_pos) {
                    total += len;
                    current_pos += len;
                }
                
                Some(total)
            }
            RegexNode::ZeroOrOne(inner) => {
                Some(self.match_node(inner, pos).unwrap_or(0))
            }
            _ => Some(0), // Simplified for other cases
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regexp_literal() {
        let re = RegExp::new("hello", "").unwrap();
        assert!(re.test("hello world"));
        assert!(!re.test("goodbye"));
    }
    
    #[test]
    fn test_regexp_dot() {
        let re = RegExp::new("h.llo", "").unwrap();
        assert!(re.test("hello"));
        assert!(re.test("hallo"));
    }
    
    #[test]
    fn test_regexp_quantifiers() {
        let re = RegExp::new("a*b", "").unwrap();
        assert!(re.test("b"));
        assert!(re.test("ab"));
        assert!(re.test("aaab"));
    }
}
