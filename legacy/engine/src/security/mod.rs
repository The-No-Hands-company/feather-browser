// Security Module - Same-origin policy, CSP, CORS, XSS protection
pub mod cors;
pub mod csp;

use std::collections::HashMap;

pub use cors::{CorsValidator, CorsDecision, CspValidator, parse_origin, is_same_origin};
pub use csp::ContentSecurityPolicy;

#[derive(Debug, Clone)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl Origin {
    pub fn new(scheme: String, host: String, port: u16) -> Self {
        Self { scheme, host, port }
    }
    
    pub fn same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }
}

#[derive(Debug, Clone)]
pub struct LegacyContentSecurityPolicy {
    directives: HashMap<String, Vec<String>>,
}

impl LegacyContentSecurityPolicy {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
        }
    }
    
    pub fn parse(header: &str) -> Self {
        let mut csp = Self::new();
        for directive in header.split(';') {
            let parts: Vec<&str> = directive.trim().split_whitespace().collect();
            if let Some(name) = parts.first() {
                let values: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                csp.directives.insert(name.to_string(), values);
            }
        }
        csp
    }
    
    pub fn allows_script(&self, source: &str) -> bool {
        if let Some(sources) = self.directives.get("script-src") {
            sources.iter().any(|s| s == "'self'" || s == source || s == "'unsafe-inline'")
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct CORSPolicy {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    allow_credentials: bool,
}

impl CORSPolicy {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec![],
            allow_credentials: false,
        }
    }
    
    pub fn allows_origin(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }
}

pub struct SecurityContext {
    origin: Origin,
    csp: LegacyContentSecurityPolicy,
    cors: CORSPolicy,
}

impl SecurityContext {
    pub fn new(origin: Origin) -> Self {
        Self {
            origin,
            csp: LegacyContentSecurityPolicy::new(),
            cors: CORSPolicy::new(),
        }
    }
    
    pub fn can_access(&self, target_origin: &Origin) -> bool {
        self.origin.same_origin(target_origin)
    }
}
