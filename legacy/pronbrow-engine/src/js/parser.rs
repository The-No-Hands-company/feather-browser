// JavaScript Parser - Full ES6+ AST Generation
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    
    // Identifiers
    Identifier(String),
    
    // Expressions
    BinaryOp {
        left: Box<ASTNode>,
        operator: BinaryOperator,
        right: Box<ASTNode>,
    },
    UnaryOp {
        operator: UnaryOperator,
        argument: Box<ASTNode>,
    },
    AssignmentExpr {
        left: Box<ASTNode>,
        operator: AssignmentOperator,
        right: Box<ASTNode>,
    },
    ConditionalExpr {
        test: Box<ASTNode>,
        consequent: Box<ASTNode>,
        alternate: Box<ASTNode>,
    },
    CallExpr {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    MemberExpr {
        object: Box<ASTNode>,
        property: Box<ASTNode>,
        computed: bool,
    },
    NewExpr {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    
    // ES6+ Expressions
    ArrowFunction {
        params: Vec<String>,
        body: Box<ASTNode>,
        is_async: bool,
    },
    TemplateLiteral {
        parts: Vec<String>,
        expressions: Vec<ASTNode>,
    },
    SpreadElement {
        argument: Box<ASTNode>,
    },
    AwaitExpression {
        argument: Box<ASTNode>,
    },
    YieldExpression {
        argument: Option<Box<ASTNode>>,
        delegate: bool,  // yield* vs yield
    },
    // ES2020+ Modern Operators
    OptionalMemberExpr {
        object: Box<ASTNode>,
        property: Box<ASTNode>,
        computed: bool,
        optional: bool,  // true for ?. operator
    },
    OptionalCallExpr {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
        optional: bool,  // true for ?.() operator
    },
    
    // Statements
    ExpressionStmt(Box<ASTNode>),
    BlockStmt {
        body: Vec<ASTNode>,
    },
    IfStmt {
        test: Box<ASTNode>,
        consequent: Box<ASTNode>,
        alternate: Option<Box<ASTNode>>,
    },
    WhileStmt {
        test: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    ForStmt {
        init: Option<Box<ASTNode>>,
        test: Option<Box<ASTNode>>,
        update: Option<Box<ASTNode>>,
        body: Box<ASTNode>,
    },
    ReturnStmt {
        argument: Option<Box<ASTNode>>,
    },
    BreakStmt,
    ContinueStmt,
    ThrowStmt {
        argument: Box<ASTNode>,
    },
    TryStmt {
        block: Box<ASTNode>,
        handler: Option<Box<CatchClause>>,
        finalizer: Option<Box<ASTNode>>,
    },
    
    // Declarations
    VariableDeclaration {
        kind: VarKind,
        declarations: Vec<VariableDeclarator>,
    },
    FunctionDeclaration {
        id: String,
        params: Vec<Parameter>,
        body: Box<ASTNode>,
        is_async: bool,
        is_generator: bool,
    },
    FunctionExpression {
        id: Option<String>,  // Optional name (can be anonymous)
        params: Vec<Parameter>,
        body: Box<ASTNode>,
        is_async: bool,
        is_generator: bool,
    },
    ClassDeclaration {
        id: String,
        superclass: Option<String>,
        body: Vec<ClassBodyElement>,
    },
    
    // ES6+ Features
    ObjectLiteral {
        properties: Vec<ObjectProperty>,
    },
    ArrayLiteral {
        elements: Vec<Option<ASTNode>>,
    },
    DestructuringPattern {
        pattern: DestructuringKind,
    },
    
    // Async/Await
    AwaitExpr {
        argument: Box<ASTNode>,
    },
    
    Program {
        body: Vec<ASTNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, StrictEq, StrictNotEq,
    Lt, Lte, Gt, Gte,
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LeftShift, RightShift, UnsignedRightShift,
    In, InstanceOf,
    NullishCoalescing,  // ?? operator (ES2020)
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not, Plus, Minus, BitwiseNot, Typeof, Void, Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
    LogicalOrAssign,   // ||= (ES2021)
    LogicalAndAssign,  // &&= (ES2021)
    NullishAssign,     // ??= (ES2021)
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarKind {
    Var, Let, Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarator {
    pub id: String,
    pub init: Option<ASTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default: Option<ASTNode>,
    pub is_rest: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub param: Option<String>,
    pub body: Box<ASTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethod {
    pub kind: MethodKind,
    pub key: String,
    pub params: Vec<Parameter>,
    pub body: ASTNode,
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassField {
    pub key: String,
    pub value: Option<ASTNode>,
    pub is_static: bool,
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBodyElement {
    Method(ClassMethod),
    Field(ClassField),
    StaticBlock(Box<ASTNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    Constructor, Method, Get, Set,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    pub key: String,
    pub value: ASTNode,
    pub computed: bool,
    pub shorthand: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DestructuringKind {
    Object(Vec<String>),
    Array(Vec<String>),
}

pub struct JSParser {
    tokens: Vec<Token>,
    current: usize,
    /// Symbol table for tracking variable declarations and scopes
    symbol_table: HashMap<String, SymbolInfo>,
    /// Scope stack for tracking nested scopes
    scope_stack: Vec<HashMap<String, SymbolInfo>>,
}

/// Information about a symbol (variable/function) in scope
#[derive(Debug, Clone)]
struct SymbolInfo {
    kind: VarKind,
    defined_at: usize,
    is_function: bool,
    is_hoisted: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    Number(f64),
    String(String),
    True,
    False,
    Null,
    Undefined,
    
    // Keywords
    Let, Const, Var,
    Function, Class, Extends, Static,
    If, Else, While, For, Do,
    Return, Break, Continue,
    Try, Catch, Finally, Throw,
    New, This, Super,
    Async, Await, Yield,
    Typeof, Void, Delete,
    In, InstanceOf,
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, NotEq, StrictEq, StrictNotEq,
    Lt, Lte, Gt, Gte,
    And, Or, Not,
    BitwiseAnd, BitwiseOr, BitwiseXor, BitwiseNot,
    LeftShift, RightShift, UnsignedRightShift,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign,
    OrAssign, AndAssign, NullishAssign,  // ||=, &&=, ??= (ES2021)
    Arrow,
    Spread,
    
    // Punctuation
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Semicolon, Comma, Dot, Colon, Question,
    OptionalChain,  // ?. operator
    NullishCoalesce,  // ?? operator
    
    EOF,
}

impl JSParser {
    pub fn new(input: &str) -> Self {
        let tokens = Self::tokenize(input);
        JSParser {
            tokens,
            current: 0,
            symbol_table: HashMap::new(),
            scope_stack: vec![HashMap::new()],
        }
    }
    
    /// Enter a new scope
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }
    
    /// Exit current scope
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
    
    /// Register a symbol in current scope
    fn register_symbol(&mut self, name: String, kind: VarKind, is_function: bool) {
        let info = SymbolInfo {
            kind: kind.clone(),
            defined_at: self.current,
            is_function,
            is_hoisted: matches!(kind, VarKind::Var) || is_function,
        };
        
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.clone(), info.clone());
        }
        self.symbol_table.insert(name, info);
    }
    
    /// Check if a symbol is defined in any scope
    fn is_symbol_defined(&self, name: &str) -> bool {
        for scope in self.scope_stack.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
    
    fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' | '\n' | '\r' => { i += 1; }
                '+' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        tokens.push(Token::AddAssign);
                        i += 2;
                    } else {
                        tokens.push(Token::Plus);
                        i += 1;
                    }
                }
                '-' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        tokens.push(Token::SubAssign);
                        i += 2;
                    } else {
                        tokens.push(Token::Minus);
                        i += 1;
                    }
                }
                '*' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        tokens.push(Token::MulAssign);
                        i += 2;
                    } else {
                        tokens.push(Token::Star);
                        i += 1;
                    }
                }
                '/' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        tokens.push(Token::DivAssign);
                        i += 2;
                    } else {
                        tokens.push(Token::Slash);
                        i += 1;
                    }
                }
                '%' => { tokens.push(Token::Percent); i += 1; }
                '|' => {
                    if i + 1 < chars.len() && chars[i + 1] == '|' {
                        if i + 2 < chars.len() && chars[i + 2] == '=' {
                            tokens.push(Token::OrAssign);
                            i += 3;
                        } else {
                            tokens.push(Token::Or);
                            i += 2;
                        }
                    } else {
                        tokens.push(Token::BitwiseOr);
                        i += 1;
                    }
                }
                '&' => {
                    if i + 1 < chars.len() && chars[i + 1] == '&' {
                        if i + 2 < chars.len() && chars[i + 2] == '=' {
                            tokens.push(Token::AndAssign);
                            i += 3;
                        } else {
                            tokens.push(Token::And);
                            i += 2;
                        }
                    } else {
                        tokens.push(Token::BitwiseAnd);
                        i += 1;
                    }
                }
                '!' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        if i + 2 < chars.len() && chars[i + 2] == '=' {
                            tokens.push(Token::StrictNotEq);
                            i += 3;
                        } else {
                            tokens.push(Token::NotEq);
                            i += 2;
                        }
                    } else {
                        tokens.push(Token::Not);
                        i += 1;
                    }
                }
                '(' => { tokens.push(Token::LParen); i += 1; }
                ')' => { tokens.push(Token::RParen); i += 1; }
                '{' => { tokens.push(Token::LBrace); i += 1; }
                '}' => { tokens.push(Token::RBrace); i += 1; }
                '[' => { tokens.push(Token::LBracket); i += 1; }
                ']' => { tokens.push(Token::RBracket); i += 1; }
                ';' => { tokens.push(Token::Semicolon); i += 1; }
                ',' => { tokens.push(Token::Comma); i += 1; }
                '.' => { tokens.push(Token::Dot); i += 1; }
                ':' => { tokens.push(Token::Colon); i += 1; }
                '#' => {
                    // Private field/method identifier: #fieldName
                    i += 1;
                    if i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '_') {
                        let start = i;
                        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                            i += 1;
                        }
                        let name: String = chars[start..i].iter().collect();
                        tokens.push(Token::Identifier(format!("#{}", name)));
                    } else {
                        return vec![Token::EOF]; // Error: invalid #
                    }
                }
                '?' => {
                    if i + 1 < chars.len() && chars[i + 1] == '.' {
                        tokens.push(Token::OptionalChain);
                        i += 2;
                    } else if i + 1 < chars.len() && chars[i + 1] == '?' {
                        if i + 2 < chars.len() && chars[i + 2] == '=' {
                            tokens.push(Token::NullishAssign);
                            i += 3;
                        } else {
                            tokens.push(Token::NullishCoalesce);
                            i += 2;
                        }
                    } else {
                        tokens.push(Token::Question);
                        i += 1;
                    }
                }
                '=' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        if i + 2 < chars.len() && chars[i + 2] == '=' {
                            tokens.push(Token::StrictEq);
                            i += 3;
                        } else {
                            tokens.push(Token::Eq);
                            i += 2;
                        }
                    } else if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(Token::Arrow);
                        i += 2;
                    } else {
                        tokens.push(Token::Assign);
                        i += 1;
                    }
                }
                '<' => { tokens.push(Token::Lt); i += 1; }
                '>' => { tokens.push(Token::Gt); i += 1; }
                c if c.is_ascii_digit() => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num: String = chars[start..i].iter().collect();
                    tokens.push(Token::Number(num.parse().unwrap_or(0.0)));
                }
                c if c.is_alphabetic() || c == '_' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    let token = match word.as_str() {
                        "let" => Token::Let,
                        "const" => Token::Const,
                        "var" => Token::Var,
                        "function" => Token::Function,
                        "class" => Token::Class,
                        "extends" => Token::Extends,
                        "static" => Token::Static,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "for" => Token::For,
                        "return" => Token::Return,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "try" => Token::Try,
                        "catch" => Token::Catch,
                        "finally" => Token::Finally,
                        "throw" => Token::Throw,
                        "true" => Token::True,
                        "false" => Token::False,
                        "null" => Token::Null,
                        "undefined" => Token::Undefined,
                        "async" => Token::Async,
                        "await" => Token::Await,
                        "yield" => Token::Yield,
                        "new" => Token::New,
                        "this" => Token::This,
                        "super" => Token::Super,
                        "typeof" => Token::Typeof,
                        "void" => Token::Void,
                        "delete" => Token::Delete,
                        "in" => Token::In,
                        "instanceof" => Token::InstanceOf,
                        "do" => Token::Do,
                        _ => Token::Identifier(word),
                    };
                    tokens.push(token);
                }
                '`' => {
                    // Template literal
                    i += 1; // Skip opening backtick
                    let mut parts = Vec::new();
                    let mut current_part = String::new();
                    let mut expressions_placeholder = Vec::new();

                    while i < chars.len() && chars[i] != '`' {
                        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                            // Found ${expression}
                            parts.push(current_part.clone());
                            current_part.clear();
                            i += 2; // Skip ${

                            // Find matching }
                            let mut brace_depth = 1;
                            let expr_start = i;
                            while i < chars.len() && brace_depth > 0 {
                                if chars[i] == '{' {
                                    brace_depth += 1;
                                } else if chars[i] == '}' {
                                    brace_depth -= 1;
                                }
                                if brace_depth > 0 {
                                    i += 1;
                                }
                            }

                            // Mark that there's an expression here
                            expressions_placeholder.push(expr_start);
                            i += 1; // Skip closing }
                        } else if chars[i] == '\\' && i + 1 < chars.len() {
                            // Escape sequence
                            i += 1;
                            current_part.push(chars[i]);
                            i += 1;
                        } else {
                            current_part.push(chars[i]);
                            i += 1;
                        }
                    }

                    parts.push(current_part);
                    i += 1; // Skip closing backtick

                    // For now, just push as a regular string
                    // Full template literal support requires more complex parsing
                    tokens.push(Token::String(parts.join("")));
                }
                '"' | '\'' => {
                    // String literal
                    let quote_char = chars[i];
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i] != quote_char {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 2; // Skip escaped character
                        } else {
                            i += 1;
                        }
                    }
                    let string_val: String = chars[start..i].iter().collect();
                    tokens.push(Token::String(string_val));
                    if i < chars.len() {
                        i += 1; // Skip closing quote
                    }
                }
                _ => { i += 1; }
            }
        }
        
        tokens.push(Token::EOF);
        tokens
    }
    
    pub fn parse(&mut self) -> Result<ASTNode, String> {
        let mut body = Vec::new();

        while !self.is_at_end() {
            // Skip empty statements (lone semicolons)
            while self.check(&Token::Semicolon) && !self.is_at_end() {
                self.advance();
            }

            if !self.is_at_end() {
                match self.parse_statement() {
                    Ok(stmt) => body.push(stmt),
                    Err(e) => {
                        // Add context to error
                        return Err(format!("Parse error at position {}: {}", self.current, e));
                    }
                }
            }
        }

        // If we have no statements, return a program with undefined
        if body.is_empty() {
            body.push(ASTNode::ExpressionStmt(Box::new(ASTNode::Undefined)));
        }

        Ok(ASTNode::Program { body })
    }
    
    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        // Skip any leading semicolons (empty statements)
        while self.check(&Token::Semicolon) {
            self.advance();
        }

        // Check for end of file
        if self.is_at_end() {
            return Err("Unexpected end of input".to_string());
        }

        match &self.peek() {
            Token::Let | Token::Const | Token::Var => self.parse_variable_declaration(),
            Token::Async => {
                // Could be async function or async arrow function
                // Need to lookahead to determine
                self.parse_function_declaration()
            }
            Token::Function => self.parse_function_declaration(),
            Token::Class => self.parse_class_declaration(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                self.advance();
                self.consume_semicolon();
                Ok(ASTNode::BreakStmt)
            }
            Token::Continue => {
                self.advance();
                self.consume_semicolon();
                Ok(ASTNode::ContinueStmt)
            }
            Token::Try => self.parse_try_statement(),
            Token::Throw => self.parse_throw_statement(),
            Token::LBrace => self.parse_block_statement(),
            Token::Semicolon => {
                // Empty statement
                self.advance();
                Ok(ASTNode::ExpressionStmt(Box::new(ASTNode::Undefined)))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.consume_semicolon();
                Ok(ASTNode::ExpressionStmt(Box::new(expr)))
            }
        }
    }
    
    fn parse_variable_declaration(&mut self) -> Result<ASTNode, String> {
        let kind = match self.advance() {
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            Token::Var => VarKind::Var,
            _ => return Err("Expected variable declaration keyword".to_string()),
        };
        
        let mut declarations = Vec::new();
        
        loop {
            let name = if let Token::Identifier(name) = self.advance() {
                name.clone()
            } else {
                return Err("Expected identifier".to_string());
            };
            
            // Register symbol in symbol table
            self.register_symbol(name.clone(), kind.clone(), false);
            
            let init = if self.check(&Token::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            declarations.push(VariableDeclarator {
                id: name,
                init,
            });
            
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        
        self.consume_semicolon();
        Ok(ASTNode::VariableDeclaration { kind, declarations })
    }
    
    fn parse_function_declaration(&mut self) -> Result<ASTNode, String> {
        // Check for async keyword
        let is_async = if matches!(self.peek(), Token::Async) {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(&Token::Function)?;

        // Check for generator (function*)
        let is_generator = if matches!(self.peek(), Token::Star) {
            self.advance();
            true
        } else {
            false
        };

        let id = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err("Expected function name".to_string());
        };

        // Register function in symbol table
        self.register_symbol(id.clone(), VarKind::Var, true);

        // Enter new scope for function body
        self.push_scope();

        self.expect(&Token::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(&Token::RParen)?;

        let body = Box::new(self.parse_block_statement()?);

        // Exit function scope
        self.pop_scope();

        Ok(ASTNode::FunctionDeclaration {
            id,
            params,
            body,
            is_async,
            is_generator,
        })
    }

    fn parse_function_expression(&mut self) -> Result<ASTNode, String> {
        // Check for async keyword  
        let is_async = if matches!(self.peek(), Token::Async) {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(&Token::Function)?;

        // Check for generator (function*)
        let is_generator = if matches!(self.peek(), Token::Star) {
            self.advance();
            true
        } else {
            false
        };

        // Function name is optional for expressions
        let id = if matches!(self.peek(), Token::Identifier(_)) {
            if let Token::Identifier(name) = self.advance() {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Enter new scope for function body
        self.push_scope();

        // If function has a name, register it in the function's own scope
        if let Some(ref name) = id {
            self.register_symbol(name.clone(), VarKind::Const, false);
        }

        self.expect(&Token::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(&Token::RParen)?;

        let body = Box::new(self.parse_block_statement()?);

        // Exit function scope
        self.pop_scope();

        Ok(ASTNode::FunctionExpression {
            id,
            params,
            body,
            is_async,
            is_generator,
        })
    }
    
    fn parse_class_declaration(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'class'
        
        let id = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err("Expected class name".to_string());
        };
        
        let superclass = if self.check(&Token::Extends) {
            self.advance();
            if let Token::Identifier(name) = self.advance() {
                Some(name.clone())
            } else {
                return Err("Expected superclass name".to_string());
            }
        } else {
            None
        };
        
        self.expect(&Token::LBrace)?;
        let mut body = Vec::new();
        
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for static keyword
            let is_static = if self.check(&Token::Static) {
                self.advance();
                true
            } else {
                false
            };
            
            // Static block: static { ... }
            if is_static && self.check(&Token::LBrace) {
                let block = Box::new(self.parse_block_statement()?);
                body.push(ClassBodyElement::StaticBlock(block));
                continue;
            }
            
            // Get the key (method/field name, private field starts with #)
            let key = if let Token::Identifier(name) = self.advance() {
                name.clone()
            } else {
                continue;
            };
            
            let is_private = key.starts_with('#');
            
            // Check if it's a method (has parentheses) or field (has = or ;)
            if self.check(&Token::LParen) {
                // It's a method
                self.advance(); // consume (
                let params = self.parse_parameters()?;
                self.expect(&Token::RParen)?;
                let method_body = self.parse_block_statement()?;
                
                body.push(ClassBodyElement::Method(ClassMethod {
                    kind: if key == "constructor" {
                        MethodKind::Constructor
                    } else {
                        MethodKind::Method
                    },
                    key,
                    params,
                    body: method_body,
                    is_static,
                }));
            } else if self.check(&Token::Assign) || self.check(&Token::Semicolon) {
                // It's a field
                let value = if self.check(&Token::Assign) {
                    self.advance(); // consume =
                    Some(self.parse_assignment()?)
                } else {
                    None
                };
                
                // Optional semicolon
                if self.check(&Token::Semicolon) {
                    self.advance();
                }
                
                body.push(ClassBodyElement::Field(ClassField {
                    key,
                    value,
                    is_static,
                    is_private,
                }));
            } else {
                return Err(format!("Expected ( or = after class member name '{}'", key));
            }
        }
        
        self.expect(&Token::RBrace)?;
        
        Ok(ASTNode::ClassDeclaration { id, superclass, body })
    }
    
    fn parse_if_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'if'
        self.expect(&Token::LParen)?;
        let test = Box::new(self.parse_expression()?);
        self.expect(&Token::RParen)?;
        
        let consequent = Box::new(self.parse_statement()?);
        
        let alternate = if self.check(&Token::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        
        Ok(ASTNode::IfStmt { test, consequent, alternate })
    }
    
    fn parse_while_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'while'
        self.expect(&Token::LParen)?;
        let test = Box::new(self.parse_expression()?);
        self.expect(&Token::RParen)?;
        let body = Box::new(self.parse_statement()?);
        
        Ok(ASTNode::WhileStmt { test, body })
    }
    
    fn parse_for_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'for'
        self.expect(&Token::LParen)?;
        
        let init = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };
        
        let test = if self.check(&Token::Semicolon) {
            None
        } else {
            let expr = self.parse_expression()?;
            Some(Box::new(expr))
        };
        self.expect(&Token::Semicolon)?;
        
        let update = if self.check(&Token::RParen) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        
        self.expect(&Token::RParen)?;
        let body = Box::new(self.parse_statement()?);
        
        Ok(ASTNode::ForStmt { init, test, update, body })
    }
    
    fn parse_return_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'return'
        
        let argument = if self.check(&Token::Semicolon) || self.is_at_end() {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        
        self.consume_semicolon();
        Ok(ASTNode::ReturnStmt { argument })
    }
    
    fn parse_try_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'try'
        
        let block = Box::new(self.parse_block_statement()?);
        
        let handler = if self.check(&Token::Catch) {
            self.advance(); // consume 'catch'
            
            // Optional catch parameter
            let param = if self.check(&Token::LParen) {
                self.advance();
                let param_name = if let Token::Identifier(name) = self.advance() {
                    Some(name.clone())
                } else {
                    None
                };
                self.expect(&Token::RParen)?;
                param_name
            } else {
                None
            };
            
            let body = Box::new(self.parse_block_statement()?);
            Some(Box::new(CatchClause { param, body }))
        } else {
            None
        };
        
        let finalizer = if self.check(&Token::Finally) {
            self.advance(); // consume 'finally'
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };
        
        Ok(ASTNode::TryStmt { block, handler, finalizer })
    }
    
    fn parse_throw_statement(&mut self) -> Result<ASTNode, String> {
        self.advance(); // consume 'throw'
        
        // Throw requires an expression
        let argument = Box::new(self.parse_expression()?);
        
        self.consume_semicolon();
        Ok(ASTNode::ThrowStmt { argument })
    }
    
    fn parse_block_statement(&mut self) -> Result<ASTNode, String> {
        self.expect(&Token::LBrace)?;
        
        // Enter new scope for block
        self.push_scope();
        
        let mut body = Vec::new();
        
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        
        self.expect(&Token::RBrace)?;
        
        // Exit block scope
        self.pop_scope();
        
        Ok(ASTNode::BlockStmt { body })
    }
    
    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<ASTNode, String> {
        // Check for async arrow function: async () => {} or async param => {}
        if self.check(&Token::Async) {
            let saved_pos = self.current;
            self.advance();
            
            // async param => expr
            if let Token::Identifier(param_name) = self.peek() {
                let param = param_name.clone();
                self.advance();
                
                if self.check(&Token::Arrow) {
                    self.advance();
                    let body = if self.check(&Token::LBrace) {
                        Box::new(self.parse_block_statement()?)
                    } else {
                        Box::new(self.parse_assignment()?)
                    };
                    return Ok(ASTNode::ArrowFunction {
                        params: vec![param],
                        body,
                        is_async: true,
                    });
                }
            }
            
            // async () => expr or async (x, y) => expr
            if self.check(&Token::LParen) {
                self.advance();
                
                let params = if self.check(&Token::RParen) {
                    vec![]
                } else {
                    match self.parse_parameters() {
                        Ok(params) => params.iter().map(|p| p.name.clone()).collect(),
                        Err(_) => {
                            self.current = saved_pos;
                            return self.parse_conditional();
                        }
                    }
                };
                
                if !self.check(&Token::RParen) {
                    self.current = saved_pos;
                    return self.parse_conditional();
                }
                self.advance();
                
                if self.check(&Token::Arrow) {
                    self.advance();
                    let body = if self.check(&Token::LBrace) {
                        Box::new(self.parse_block_statement()?)
                    } else {
                        Box::new(self.parse_assignment()?)
                    };
                    return Ok(ASTNode::ArrowFunction {
                        params,
                        body,
                        is_async: true,
                    });
                }
            }
            
            // Not an async arrow function, restore position
            self.current = saved_pos;
        }
        
        let expr = self.parse_conditional()?;
        
        let operator = if self.check(&Token::Assign) {
            self.advance();
            AssignmentOperator::Assign
        } else if self.check(&Token::AddAssign) {
            self.advance();
            AssignmentOperator::AddAssign
        } else if self.check(&Token::SubAssign) {
            self.advance();
            AssignmentOperator::SubAssign
        } else if self.check(&Token::MulAssign) {
            self.advance();
            AssignmentOperator::MulAssign
        } else if self.check(&Token::DivAssign) {
            self.advance();
            AssignmentOperator::DivAssign
        } else if self.check(&Token::OrAssign) {
            self.advance();
            AssignmentOperator::LogicalOrAssign
        } else if self.check(&Token::AndAssign) {
            self.advance();
            AssignmentOperator::LogicalAndAssign
        } else if self.check(&Token::NullishAssign) {
            self.advance();
            AssignmentOperator::NullishAssign
        } else {
            return Ok(expr);
        };
        
        let right = Box::new(self.parse_assignment()?);
        Ok(ASTNode::AssignmentExpr {
            left: Box::new(expr),
            operator,
            right,
        })
    }
    
    fn parse_conditional(&mut self) -> Result<ASTNode, String> {
        let expr = self.parse_nullish_coalescing()?;
        
        if self.check(&Token::Question) {
            self.advance();
            let consequent = Box::new(self.parse_expression()?);
            self.expect(&Token::Colon)?;
            let alternate = Box::new(self.parse_expression()?);
            return Ok(ASTNode::ConditionalExpr {
                test: Box::new(expr),
                consequent,
                alternate,
            });
        }
        
        Ok(expr)
    }
    
    fn parse_nullish_coalescing(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_logical_or()?;
        
        while self.check(&Token::NullishCoalesce) {
            self.advance();
            let right = self.parse_logical_or()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::NullishCoalescing,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_logical_or(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_logical_and()?;
        
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_logical_and(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_equality()?;
        
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_relational()?;
        
        while let Some(op) = self.match_tokens(&[Token::Eq, Token::NotEq, Token::StrictEq, Token::StrictNotEq]) {
            let operator = match op {
                Token::Eq => BinaryOperator::Eq,
                Token::NotEq => BinaryOperator::NotEq,
                Token::StrictEq => BinaryOperator::StrictEq,
                Token::StrictNotEq => BinaryOperator::StrictNotEq,
                _ => unreachable!(),
            };
            
            let right = self.parse_relational()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_relational(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_additive()?;
        
        while let Some(op) = self.match_tokens(&[Token::Lt, Token::Lte, Token::Gt, Token::Gte]) {
            let operator = match op {
                Token::Lt => BinaryOperator::Lt,
                Token::Lte => BinaryOperator::Lte,
                Token::Gt => BinaryOperator::Gt,
                Token::Gte => BinaryOperator::Gte,
                _ => unreachable!(),
            };
            
            let right = self.parse_additive()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_additive(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_multiplicative()?;
        
        while let Some(op) = self.match_tokens(&[Token::Plus, Token::Minus]) {
            let operator = match op {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            
            let right = self.parse_multiplicative()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative(&mut self) -> Result<ASTNode, String> {
        let mut left = self.parse_unary()?;
        
        while let Some(op) = self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let operator = match op {
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                Token::Percent => BinaryOperator::Mod,
                _ => unreachable!(),
            };
            
            let right = self.parse_unary()?;
            left = ASTNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<ASTNode, String> {
        if let Some(op) = self.match_tokens(&[Token::Not, Token::Plus, Token::Minus, Token::Typeof]) {
            let operator = match op {
                Token::Not => UnaryOperator::Not,
                Token::Plus => UnaryOperator::Plus,
                Token::Minus => UnaryOperator::Minus,
                Token::Typeof => UnaryOperator::Typeof,
                _ => unreachable!(),
            };

            let argument = Box::new(self.parse_unary()?);
            return Ok(ASTNode::UnaryOp { operator, argument });
        }

        // await expression
        if self.check(&Token::Await) {
            self.advance();
            let argument = Box::new(self.parse_unary()?);
            return Ok(ASTNode::AwaitExpression { argument });
        }

        // yield expression  
        if self.check(&Token::Yield) {
            self.advance();
            
            // Check for yield* (delegate)
            let delegate = if self.check(&Token::Star) {
                self.advance();
                true
            } else {
                false
            };
            
            // yield can have no argument
            let argument = if self.is_expression_start() {
                Some(Box::new(self.parse_assignment()?))
            } else {
                None
            };
            
            return Ok(ASTNode::YieldExpression { argument, delegate });
        }

        // Handle new expression
        if self.check(&Token::New) {
            self.advance();
            return self.parse_new_expression();
        }

        self.parse_postfix()
    }
    
    /// Helper to check if current token can start an expression
    fn is_expression_start(&self) -> bool {
        matches!(self.peek(),
            Token::Number(_) | Token::String(_) | Token::True | Token::False |
            Token::Null | Token::Undefined | Token::Identifier(_) |
            Token::LParen | Token::LBracket | Token::LBrace |
            Token::Function | Token::Async | Token::New | Token::This |
            Token::Not | Token::Plus | Token::Minus | Token::Typeof
        )
    }

    fn parse_new_expression(&mut self) -> Result<ASTNode, String> {
        // Parse the callee (just member expressions, not calls)
        let callee = Box::new(self.parse_member_expression()?);

        // Parse arguments if present
        let arguments = if self.check(&Token::LParen) {
            self.advance();
            let args = self.parse_argument_list()?;
            self.expect(&Token::RParen)?;
            args
        } else {
            Vec::new()
        };

        Ok(ASTNode::NewExpr { callee, arguments })
    }

    /// Parse member expressions (property access) but not call expressions
    fn parse_member_expression(&mut self) -> Result<ASTNode, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::OptionalChain) {
                // ?. optional member access
                self.advance();
                if !self.check(&Token::LParen) && !self.check(&Token::LBracket) {
                    // ?.property
                    let name = self.get_property_name()?;
                    expr = ASTNode::OptionalMemberExpr {
                        object: Box::new(expr),
                        property: Box::new(ASTNode::Identifier(name)),
                        computed: false,
                        optional: true,
                    };
                } else if self.check(&Token::LParen) {
                    // ?.() optional call
                    self.advance();
                    let arguments = self.parse_argument_list()?;
                    self.expect(&Token::RParen)?;
                    expr = ASTNode::OptionalCallExpr {
                        callee: Box::new(expr),
                        arguments,
                        optional: true,
                    };
                } else if self.check(&Token::LBracket) {
                    // ?.[...] optional computed member access
                    self.advance();
                    let property = Box::new(self.parse_expression()?);
                    self.expect(&Token::RBracket)?;
                    expr = ASTNode::OptionalMemberExpr {
                        object: Box::new(expr),
                        property,
                        computed: true,
                        optional: true,
                    };
                } else {
                    return Err("Expected property name, call, or computed access after '?.'".to_string());
                }
            } else if self.check(&Token::LBracket) {
                self.advance();
                let property = Box::new(self.parse_expression()?);
                self.expect(&Token::RBracket)?;
                expr = ASTNode::MemberExpr {
                    object: Box::new(expr),
                    property,
                    computed: true,
                };
            } else if self.check(&Token::Dot) {
                self.advance();
                let name = self.get_property_name()?;
                expr = ASTNode::MemberExpr {
                    object: Box::new(expr),
                    property: Box::new(ASTNode::Identifier(name)),
                    computed: false,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }
    
    fn parse_postfix(&mut self) -> Result<ASTNode, String> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.check(&Token::OptionalChain) {
                // ?. optional chaining
                self.advance();
                if self.check(&Token::LParen) {
                    // ?.() optional call
                    self.advance();
                    let arguments = self.parse_argument_list()?;
                    self.expect(&Token::RParen)?;
                    expr = ASTNode::OptionalCallExpr {
                        callee: Box::new(expr),
                        arguments,
                        optional: true,
                    };
                } else if self.check(&Token::LBracket) {
                    // ?.[...] optional computed member access
                    self.advance();
                    let property = Box::new(self.parse_expression()?);
                    self.expect(&Token::RBracket)?;
                    expr = ASTNode::OptionalMemberExpr {
                        object: Box::new(expr),
                        property,
                        computed: true,
                        optional: true,
                    };
                } else if let Token::Identifier(name) = self.peek() {
                    // ?.prop optional property access
                    let name = name.clone();
                    self.advance();
                    expr = ASTNode::OptionalMemberExpr {
                        object: Box::new(expr),
                        property: Box::new(ASTNode::Identifier(name)),
                        computed: false,
                        optional: true,
                    };
                } else {
                    // Try to get property name (allows keywords)
                    match self.get_property_name() {
                        Ok(name) => {
                            expr = ASTNode::OptionalMemberExpr {
                                object: Box::new(expr),
                                property: Box::new(ASTNode::Identifier(name)),
                                computed: false,
                                optional: true,
                            };
                        }
                        Err(_) => {
                            return Err("Expected property name, call, or computed access after '?.'".to_string());
                        }
                    }
                }
            } else if self.check(&Token::LParen) {
                self.advance();
                let arguments = self.parse_argument_list()?;
                self.expect(&Token::RParen)?;
                expr = ASTNode::CallExpr {
                    callee: Box::new(expr),
                    arguments,
                };
            } else if self.check(&Token::LBracket) {
                self.advance();
                let property = Box::new(self.parse_expression()?);
                self.expect(&Token::RBracket)?;
                expr = ASTNode::MemberExpr {
                    object: Box::new(expr),
                    property,
                    computed: true,
                };
            } else if self.check(&Token::Dot) {
                self.advance();
                let name = self.get_property_name()?;
                expr = ASTNode::MemberExpr {
                    object: Box::new(expr),
                    property: Box::new(ASTNode::Identifier(name)),
                    computed: false,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<ASTNode, String> {
        match self.peek() {
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Ok(ASTNode::Number(val))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(ASTNode::String(val))
            }
            Token::True => {
                self.advance();
                Ok(ASTNode::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(ASTNode::Boolean(false))
            }
            Token::Null => {
                self.advance();
                Ok(ASTNode::Null)
            }
            Token::Undefined => {
                self.advance();
                Ok(ASTNode::Undefined)
            }
            Token::This => {
                self.advance();
                Ok(ASTNode::Identifier("this".to_string()))
            }
            Token::Super => {
                self.advance();
                Ok(ASTNode::Identifier("super".to_string()))
            }
            Token::Identifier(name) => {
                let id = name.clone();
                self.advance();
                
                // Arrow function: x => expr or x => { block }
                if self.check(&Token::Arrow) {
                    self.advance();
                    // Body can be either expression or block statement
                    let body = if self.check(&Token::LBrace) {
                        Box::new(self.parse_block_statement()?)
                    } else {
                        Box::new(self.parse_expression()?)
                    };
                    return Ok(ASTNode::ArrowFunction {
                        params: vec![id],
                        body,
                        is_async: false,
                    });
                }
                
                Ok(ASTNode::Identifier(id))
            }
            Token::LParen => {
                self.advance();
                
                // Try to parse as arrow function: (x, y) => expr or () => expr
                let saved_pos = self.current;
                
                // Try parsing parameters
                if self.check(&Token::RParen) {
                    // Empty params: () => expr
                    self.advance();
                    if self.check(&Token::Arrow) {
                        self.advance();
                        // Body can be either expression or block statement
                        let body = if self.check(&Token::LBrace) {
                            Box::new(self.parse_block_statement()?)
                        } else {
                            Box::new(self.parse_expression()?)
                        };
                        return Ok(ASTNode::ArrowFunction {
                            params: vec![],
                            body,
                            is_async: false,
                        });
                    }
                    // Not an arrow function, restore position
                    self.current = saved_pos;
                } else if let Token::Identifier(_) = self.peek() {
                    // Try to parse parameters
                    match self.parse_parameters() {
                        Ok(params) if self.check(&Token::RParen) => {
                            self.advance();
                            if self.check(&Token::Arrow) {
                                self.advance();
                                // Body can be either expression or block statement
                                let body = if self.check(&Token::LBrace) {
                                    Box::new(self.parse_block_statement()?)
                                } else {
                                    Box::new(self.parse_expression()?)
                                };
                                return Ok(ASTNode::ArrowFunction {
                                    params: params.iter().map(|p| p.name.clone()).collect(),
                                    body,
                                    is_async: false,
                                });
                            }
                            // Not an arrow function, restore position
                            self.current = saved_pos;
                        }
                        _ => {
                            // Failed to parse parameters, restore position
                            self.current = saved_pos;
                        }
                    }
                }
                
                // Parse as parenthesized expression
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    elements.push(Some(self.parse_expression()?));
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
                
                self.expect(&Token::RBracket)?;
                Ok(ASTNode::ArrayLiteral { elements })
            }
            Token::LBrace => {
                self.advance();
                let mut properties = Vec::new();

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let key = if let Token::Identifier(key) = self.advance() {
                        key.clone()
                    } else {
                        break;
                    };

                    self.expect(&Token::Colon)?;
                    let value = self.parse_expression()?;
                    properties.push(ObjectProperty {
                        key,
                        value,
                        computed: false,
                        shorthand: false,
                    });

                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }

                self.expect(&Token::RBrace)?;
                Ok(ASTNode::ObjectLiteral { properties })
            }
            Token::Function => {
                // Function expression
                self.parse_function_expression()
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek())),
        }
    }
    
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        
        while !self.check(&Token::RParen) && !self.is_at_end() {
            let is_rest = self.check(&Token::Spread);
            if is_rest {
                self.advance();
            }
            
            let name = if let Token::Identifier(name) = self.advance() {
                name.clone()
            } else {
                break;
            };
            
            let default = if self.check(&Token::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            params.push(Parameter {
                name,
                default,
                is_rest,
            });
            
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(params)
    }
    
    fn parse_argument_list(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut args = Vec::new();
        
        while !self.check(&Token::RParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(args)
    }
    
    // Helper methods
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }
    
    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
    }
    
    fn match_tokens(&mut self, tokens: &[Token]) -> Option<Token> {
        for token in tokens {
            if self.check(token) {
                return Some(self.advance().clone());
            }
        }
        None
    }
    
    fn expect(&mut self, token: &Token) -> Result<(), String> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", token, self.peek()))
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }
    
    fn consume_semicolon(&mut self) {
        if self.check(&Token::Semicolon) {
            self.advance();
        }
    }
    
    /// Get identifier or keyword as property name (for member expressions)
    fn get_property_name(&mut self) -> Result<String, String> {
        let token = self.advance();
        match token {
            Token::Identifier(name) => Ok(name.clone()),
            Token::Catch => Ok("catch".to_string()),
            Token::Finally => Ok("finally".to_string()),
            Token::Try => Ok("try".to_string()),
            Token::Throw => Ok("throw".to_string()),
            Token::New => Ok("new".to_string()),
            Token::Delete => Ok("delete".to_string()),
            Token::Typeof => Ok("typeof".to_string()),
            Token::Void => Ok("void".to_string()),
            Token::In => Ok("in".to_string()),
            Token::InstanceOf => Ok("instanceof".to_string()),
            Token::Class => Ok("class".to_string()),
            Token::Extends => Ok("extends".to_string()),
            Token::Static => Ok("static".to_string()),
            Token::This => Ok("this".to_string()),
            Token::Super => Ok("super".to_string()),
            Token::If => Ok("if".to_string()),
            Token::Else => Ok("else".to_string()),
            Token::While => Ok("while".to_string()),
            Token::For => Ok("for".to_string()),
            Token::Do => Ok("do".to_string()),
            Token::Return => Ok("return".to_string()),
            Token::Break => Ok("break".to_string()),
            Token::Continue => Ok("continue".to_string()),
            Token::Function => Ok("function".to_string()),
            Token::Const => Ok("const".to_string()),
            Token::Let => Ok("let".to_string()),
            Token::Var => Ok("var".to_string()),
            Token::Async => Ok("async".to_string()),
            Token::Await => Ok("await".to_string()),
            Token::Yield => Ok("yield".to_string()),
            _ => Err("Expected property name".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple() {
        let mut parser = JSParser::new("let x = 5;");
        let ast = parser.parse().unwrap();
        // Basic smoke test
        assert!(matches!(ast, ASTNode::Program { .. }));
    }
}
