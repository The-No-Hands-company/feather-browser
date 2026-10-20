// Content Security Policy (CSP) implementation
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CspDirective {
    DefaultSrc,
    ScriptSrc,
    StyleSrc,
    ImgSrc,
    FontSrc,
    ConnectSrc,
    MediaSrc,
    ObjectSrc,
    FrameSrc,
    ChildSrc,
    FormAction,
    FrameAncestors,
    BaseUri,
    ManifestSrc,
    WorkerSrc,
}

impl CspDirective {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "default-src" => Some(Self::DefaultSrc),
            "script-src" => Some(Self::ScriptSrc),
            "style-src" => Some(Self::StyleSrc),
            "img-src" => Some(Self::ImgSrc),
            "font-src" => Some(Self::FontSrc),
            "connect-src" => Some(Self::ConnectSrc),
            "media-src" => Some(Self::MediaSrc),
            "object-src" => Some(Self::ObjectSrc),
            "frame-src" => Some(Self::FrameSrc),
            "child-src" => Some(Self::ChildSrc),
            "form-action" => Some(Self::FormAction),
            "frame-ancestors" => Some(Self::FrameAncestors),
            "base-uri" => Some(Self::BaseUri),
            "manifest-src" => Some(Self::ManifestSrc),
            "worker-src" => Some(Self::WorkerSrc),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::DefaultSrc => "default-src",
            Self::ScriptSrc => "script-src",
            Self::StyleSrc => "style-src",
            Self::ImgSrc => "img-src",
            Self::FontSrc => "font-src",
            Self::ConnectSrc => "connect-src",
            Self::MediaSrc => "media-src",
            Self::ObjectSrc => "object-src",
            Self::FrameSrc => "frame-src",
            Self::ChildSrc => "child-src",
            Self::FormAction => "form-action",
            Self::FrameAncestors => "frame-ancestors",
            Self::BaseUri => "base-uri",
            Self::ManifestSrc => "manifest-src",
            Self::WorkerSrc => "worker-src",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContentSecurityPolicy {
    directives: HashMap<CspDirective, HashSet<String>>,
    report_only: bool,
    report_uri: Option<String>,
}

impl ContentSecurityPolicy {
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            report_only: false,
            report_uri: None,
        }
    }

    /// Parse CSP header
    pub fn from_header(header: &str) -> Self {
        let mut policy = Self::new();
        
        for directive_str in header.split(';') {
            let parts: Vec<&str> = directive_str.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let directive_name = parts[0];
            
            if directive_name == "report-uri" && parts.len() > 1 {
                policy.report_uri = Some(parts[1].to_string());
                continue;
            }

            if let Some(directive) = CspDirective::from_str(directive_name) {
                let sources: HashSet<String> = parts[1..].iter()
                    .map(|s| s.to_string())
                    .collect();
                policy.directives.insert(directive, sources);
            }
        }

        policy
    }

    /// Set report-only mode
    pub fn set_report_only(&mut self, report_only: bool) {
        self.report_only = report_only;
    }

    /// Add directive with sources
    pub fn add_directive(&mut self, directive: CspDirective, sources: Vec<String>) {
        self.directives.insert(directive, sources.into_iter().collect());
    }

    /// Add source to directive
    pub fn add_source(&mut self, directive: CspDirective, source: String) {
        self.directives.entry(directive)
            .or_insert_with(HashSet::new)
            .insert(source);
    }

    /// Check if source is allowed for directive
    pub fn is_allowed(&self, directive: CspDirective, source: &str) -> bool {
        // Check specific directive
        if let Some(sources) = self.directives.get(&directive) {
            return Self::check_source(sources, source);
        }

        // Fall back to default-src
        if let Some(sources) = self.directives.get(&CspDirective::DefaultSrc) {
            return Self::check_source(sources, source);
        }

        // If no policy is set, allow everything
        true
    }

    fn check_source(sources: &HashSet<String>, source: &str) -> bool {
        // 'none' blocks everything
        if sources.contains("'none'") {
            return false;
        }

        // '*' allows everything
        if sources.contains("*") {
            return true;
        }

        // 'self' matches same origin - special handling needed
        if source == "'self'" && sources.contains("'self'") {
            return true;
        }

        // Check for exact match or scheme match
        for allowed in sources {
            // Skip special keywords when matching URLs
            if allowed.starts_with('\'') {
                continue;
            }

            // Exact match
            if allowed == source {
                return true;
            }

            // Allow scheme-only sources (e.g., "https:")
            if allowed.ends_with(':') && source.starts_with(allowed) {
                return true;
            }

            // Check if source URL starts with the allowed origin
            // e.g., "https://cdn.example.com" should match "https://cdn.example.com/script.js"
            if source.starts_with(allowed) {
                // Make sure it's a domain match, not just prefix
                let after_allowed = &source[allowed.len()..];
                if after_allowed.is_empty() || after_allowed.starts_with('/') {
                    return true;
                }
            }

            // Allow wildcard domains - *.example.com matches subdomain.example.com
            if allowed.starts_with("*.") {
                let domain = &allowed[2..];
                // Extract domain from URL if it contains protocol
                if let Some(url_domain) = source.split("://").nth(1) {
                    let host = url_domain.split('/').next().unwrap_or(url_domain);
                    if host.ends_with(domain) || host == domain {
                        return true;
                    }
                } else if source.ends_with(domain) || source == domain {
                    return true;
                }
            }
        }

        false
    }

    /// Generate CSP header string
    pub fn to_header(&self) -> String {
        let mut parts = Vec::new();

        for (directive, sources) in &self.directives {
            let sources_str: Vec<String> = sources.iter().cloned().collect();
            parts.push(format!("{} {}", directive.to_str(), sources_str.join(" ")));
        }

        if let Some(report_uri) = &self.report_uri {
            parts.push(format!("report-uri {}", report_uri));
        }

        parts.join("; ")
    }

    /// Check if report-only mode
    pub fn is_report_only(&self) -> bool {
        self.report_only
    }

    /// Report CSP violation (logging for now)
    pub fn report_violation(&self, directive: &CspDirective, blocked_uri: &str, document_uri: &str) {
        eprintln!(
            "CSP Violation: {} blocked '{}' in document '{}'",
            directive.to_str(),
            blocked_uri,
            document_uri
        );

        // TODO: Send violation report to report-uri if configured
    }
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Strict default policy
impl ContentSecurityPolicy {
    pub fn strict() -> Self {
        let mut policy = Self::new();
        
        policy.add_directive(CspDirective::DefaultSrc, vec!["'self'".to_string()]);
        policy.add_directive(CspDirective::ScriptSrc, vec!["'self'".to_string()]);
        policy.add_directive(CspDirective::StyleSrc, vec!["'self'".to_string()]);
        policy.add_directive(CspDirective::ImgSrc, vec!["'self'".to_string(), "data:".to_string()]);
        policy.add_directive(CspDirective::ObjectSrc, vec!["'none'".to_string()]);
        
        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csp_header() {
        let header = "default-src 'self'; script-src 'self' https://cdn.example.com; style-src 'self' 'unsafe-inline'";
        let policy = ContentSecurityPolicy::from_header(header);
        
        assert!(policy.is_allowed(CspDirective::ScriptSrc, "https://cdn.example.com"));
        assert!(!policy.is_allowed(CspDirective::ScriptSrc, "https://evil.com"));
    }

    #[test]
    fn test_csp_strict_policy() {
        let policy = ContentSecurityPolicy::strict();
        
        // Self should be allowed
        assert!(policy.is_allowed(CspDirective::ScriptSrc, "'self'"));
        
        // External scripts should be blocked
        assert!(!policy.is_allowed(CspDirective::ScriptSrc, "https://evil.com/script.js"));
    }

    #[test]
    fn test_csp_wildcard() {
        let mut policy = ContentSecurityPolicy::new();
        policy.add_directive(CspDirective::ImgSrc, vec!["*.example.com".to_string()]);
        
        assert!(policy.is_allowed(CspDirective::ImgSrc, "https://cdn.example.com/image.png"));
        assert!(!policy.is_allowed(CspDirective::ImgSrc, "https://other.com/image.png"));
    }
}
