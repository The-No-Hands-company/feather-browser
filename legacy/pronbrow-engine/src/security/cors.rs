// CORS (Cross-Origin Resource Sharing) - Enforce security policies
use std::collections::HashSet;

/// Parse origin from URL
pub fn parse_origin(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url) {
        let scheme = parsed.scheme();
        if let Some(host) = parsed.host_str() {
            let port = parsed.port();
            if let Some(port) = port {
                return Some(format!("{}://{}:{}", scheme, host, port));
            } else {
                return Some(format!("{}://{}", scheme, host));
            }
        }
    }
    None
}

/// Check if two URLs have the same origin
pub fn is_same_origin(origin1: &str, origin2: &str) -> bool {
    if let (Ok(url1), Ok(url2)) = (url::Url::parse(origin1), url::Url::parse(origin2)) {
        return url1.scheme() == url2.scheme() &&
               url1.host_str() == url2.host_str() &&
               url1.port() == url2.port();
    }
    false
}

pub struct CorsValidator {
    allowed_origins: HashSet<String>,
    allowed_methods: HashSet<String>,
    allowed_headers: HashSet<String>,
    allow_credentials: bool,
    max_age: Option<usize>,
}

impl CorsValidator {
    pub fn new() -> Self {
        CorsValidator {
            allowed_origins: HashSet::new(),
            allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]
                .into_iter()
                .map(String::from)
                .collect(),
            allowed_headers: vec!["Content-Type", "Authorization", "X-Requested-With"]
                .into_iter()
                .map(String::from)
                .collect(),
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
        }
    }

    /// Check if a cross-origin request is allowed
    pub fn validate_request(
        &self,
        origin: &str,
        target_origin: &str,
        method: &str,
        headers: &[String],
    ) -> CorsDecision {
        // Same-origin requests are always allowed
        if is_same_origin(origin, target_origin) {
            return CorsDecision::Allow;
        }

        // Check if origin is allowed
        if !self.is_origin_allowed(origin) {
            return CorsDecision::Deny("Origin not allowed".to_string());
        }

        // Check if method is allowed
        if !self.allowed_methods.contains(method) {
            return CorsDecision::Deny("Method not allowed".to_string());
        }

        // Check if headers are allowed
        for header in headers {
            if !self.is_header_allowed(header) {
                return CorsDecision::Deny(format!("Header not allowed: {}", header));
            }
        }

        CorsDecision::Allow
    }

    /// Check if preflight request is required
    pub fn needs_preflight(&self, method: &str, headers: &[String]) -> bool {
        // Simple requests don't need preflight
        let simple_methods = ["GET", "HEAD", "POST"];
        let simple_headers = ["Accept", "Accept-Language", "Content-Language", "Content-Type"];

        if !simple_methods.contains(&method) {
            return true;
        }

        for header in headers {
            let header_lower = header.to_lowercase();
            if !simple_headers.iter().any(|h| h.to_lowercase() == header_lower) {
                return true;
            }
        }

        false
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.is_empty() || 
        self.allowed_origins.contains(origin) ||
        self.allowed_origins.contains("*")
    }

    fn is_header_allowed(&self, header: &str) -> bool {
        let header_lower = header.to_lowercase();
        self.allowed_headers.iter().any(|h| h.to_lowercase() == header_lower)
    }

    fn is_same_origin(origin1: &str, origin2: &str) -> bool {
        if let (Ok(url1), Ok(url2)) = (url::Url::parse(origin1), url::Url::parse(origin2)) {
            return url1.scheme() == url2.scheme() &&
                   url1.host_str() == url2.host_str() &&
                   url1.port() == url2.port();
        }
        false
    }

    pub fn add_allowed_origin(&mut self, origin: String) {
        self.allowed_origins.insert(origin);
    }

    pub fn add_allowed_method(&mut self, method: String) {
        self.allowed_methods.insert(method);
    }

    pub fn add_allowed_header(&mut self, header: String) {
        self.allowed_headers.insert(header);
    }
}

#[derive(Debug, PartialEq)]
pub enum CorsDecision {
    Allow,
    Deny(String),
}

/// Content Security Policy Parser and Validator
pub struct CspValidator {
    directives: std::collections::HashMap<String, Vec<String>>,
}

impl CspValidator {
    pub fn new() -> Self {
        CspValidator {
            directives: std::collections::HashMap::new(),
        }
    }

    /// Parse CSP header value
    pub fn parse(&mut self, csp_header: &str) {
        for directive in csp_header.split(';') {
            let parts: Vec<&str> = directive.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let directive_name = parts[0].to_string();
            let sources: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            
            self.directives.insert(directive_name, sources);
        }
    }

    /// Check if a resource can be loaded
    pub fn can_load(&self, resource_type: &str, url: &str) -> bool {
        let directive_name = match resource_type {
            "script" => "script-src",
            "style" => "style-src",
            "image" => "img-src",
            "font" => "font-src",
            "connect" => "connect-src",
            "frame" => "frame-src",
            "media" => "media-src",
            "object" => "object-src",
            _ => "default-src",
        };

        if let Some(sources) = self.directives.get(directive_name) {
            return self.matches_source(url, sources);
        }

        // Fallback to default-src
        if let Some(sources) = self.directives.get("default-src") {
            return self.matches_source(url, sources);
        }

        // If no CSP directive, allow by default
        true
    }

    fn matches_source(&self, url: &str, sources: &[String]) -> bool {
        for source in sources {
            match source.as_str() {
                "'none'" => return false,
                "'self'" => {
                    // Check if same origin
                    // TODO: Implement proper same-origin check based on current page origin
                    continue; // Skip for now since we don't have context
                }
                "'unsafe-inline'" | "'unsafe-eval'" => return true,
                _ => {
                    // Check if URL starts with the allowed source
                    // e.g., "https://cdn.example.com" should match "https://cdn.example.com/script.js"
                    if url.starts_with(source) {
                        let after_source = &url[source.len()..];
                        if after_source.is_empty() || after_source.starts_with('/') {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if inline scripts are allowed
    pub fn allows_inline_scripts(&self) -> bool {
        if let Some(sources) = self.directives.get("script-src") {
            return sources.iter().any(|s| s == "'unsafe-inline'" || s.starts_with("'nonce-"));
        }
        true // No restriction
    }

    /// Check if eval is allowed
    pub fn allows_eval(&self) -> bool {
        if let Some(sources) = self.directives.get("script-src") {
            return sources.iter().any(|s| s == "'unsafe-eval'");
        }
        true // No restriction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_origin() {
        let _validator = CorsValidator::new();
        assert!(CorsValidator::is_same_origin(
            "https://example.com",
            "https://example.com/path"
        ));
        assert!(!CorsValidator::is_same_origin(
            "https://example.com",
            "http://example.com"
        ));
    }

    #[test]
    fn test_csp_parsing() {
        let mut csp = CspValidator::new();
        csp.parse("default-src 'self'; script-src 'self' https://cdn.example.com");
        
        assert!(csp.can_load("script", "https://cdn.example.com/script.js"));
        assert!(!csp.can_load("script", "https://evil.com/script.js"));
    }
}
