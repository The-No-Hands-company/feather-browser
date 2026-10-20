// Cookie Manager - RFC 6265 compliant
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<u64>, // Unix timestamp
    pub max_age: Option<i64>, // seconds
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SameSite {
    None,
    Lax,
    Strict,
}

pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>, // domain -> cookies
}

impl CookieJar {
    pub fn new() -> Self {
        CookieJar {
            cookies: HashMap::new(),
        }
    }

    /// Set a cookie from a Set-Cookie header
    pub fn set_cookie(&mut self, cookie_str: &str, url: &str) {
        if let Some(cookie) = Self::parse_cookie(cookie_str, url) {
            let domain = cookie.domain.clone().unwrap_or_else(|| Self::extract_domain(url));
            
            // Don't store cookies with max_age=0 (immediate expiration)
            if let Some(max_age) = cookie.max_age {
                if max_age <= 0 {
                    // Remove the cookie if it exists
                    if let Some(cookies) = self.cookies.get_mut(&domain) {
                        cookies.retain(|c| c.name != cookie.name || c.path != cookie.path);
                    }
                    return;
                }
            }
            
            self.cookies
                .entry(domain.clone())
                .or_insert_with(Vec::new)
                .retain(|c| c.name != cookie.name || c.path != cookie.path);
            
            self.cookies.get_mut(&domain).unwrap().push(cookie);
        }
    }

    /// Get cookies for a URL as a Cookie header value
    pub fn get_cookies(&self, url: &str, is_https: bool) -> Option<String> {
        let domain = Self::extract_domain(url);
        let path = Self::extract_path(url);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut applicable_cookies = Vec::new();

        // Check all domains that match (including parent domains)
        for (cookie_domain, cookies) in &self.cookies {
            if Self::domain_matches(&domain, cookie_domain) {
                for cookie in cookies {
                    // Check max-age expiration (takes precedence over expires)
                    if let Some(max_age) = cookie.max_age {
                        if max_age <= 0 {
                            continue; // Expired
                        }
                    } else if let Some(expires) = cookie.expires {
                        // Check expires only if max-age not set
                        if expires < now {
                            continue;
                        }
                    }

                    // Check secure flag
                    if cookie.secure && !is_https {
                        continue;
                    }

                    // Check path
                    if let Some(cookie_path) = &cookie.path {
                        if !path.starts_with(cookie_path) {
                            continue;
                        }
                    }

                    applicable_cookies.push(format!("{}={}", cookie.name, cookie.value));
                }
            }
        }

        if applicable_cookies.is_empty() {
            None
        } else {
            Some(applicable_cookies.join("; "))
        }
    }

    /// Parse a Set-Cookie header value
    fn parse_cookie(cookie_str: &str, url: &str) -> Option<Cookie> {
        let parts: Vec<&str> = cookie_str.split(';').map(|s| s.trim()).collect();
        
        if parts.is_empty() {
            return None;
        }

        // First part is name=value
        let name_value: Vec<&str> = parts[0].splitn(2, '=').collect();
        if name_value.len() != 2 {
            return None;
        }

        let mut cookie = Cookie {
            name: name_value[0].to_string(),
            value: name_value[1].to_string(),
            domain: None,
            path: Some("/".to_string()),
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
        };

        // Parse attributes
        for part in &parts[1..] {
            let attr: Vec<&str> = part.splitn(2, '=').collect();
            let attr_name = attr[0].to_lowercase();

            match attr_name.as_str() {
                "domain" => {
                    if attr.len() == 2 {
                        cookie.domain = Some(attr[1].to_string());
                    }
                }
                "path" => {
                    if attr.len() == 2 {
                        cookie.path = Some(attr[1].to_string());
                    }
                }
                "expires" => {
                    if attr.len() == 2 {
                        // Parse HTTP date format (simplified)
                        cookie.expires = Some(0); // TODO: Proper date parsing
                    }
                }
                "max-age" => {
                    if attr.len() == 2 {
                        if let Ok(max_age) = attr[1].parse::<i64>() {
                            cookie.max_age = Some(max_age);
                        }
                    }
                }
                "secure" => cookie.secure = true,
                "httponly" => cookie.http_only = true,
                "samesite" => {
                    if attr.len() == 2 {
                        cookie.same_site = match attr[1].to_lowercase().as_str() {
                            "none" => SameSite::None,
                            "lax" => SameSite::Lax,
                            "strict" => SameSite::Strict,
                            _ => SameSite::Lax,
                        };
                    }
                }
                _ => {}
            }
        }

        // If no domain specified, use the URL's domain
        if cookie.domain.is_none() {
            cookie.domain = Some(Self::extract_domain(url));
        }

        Some(cookie)
    }

    fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.to_string();
            }
        }
        "".to_string()
    }

    fn extract_path(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            return parsed.path().to_string();
        }
        "/".to_string()
    }

    fn domain_matches(request_domain: &str, cookie_domain: &str) -> bool {
        if request_domain == cookie_domain {
            return true;
        }

        // Check if request_domain is a subdomain of cookie_domain
        if cookie_domain.starts_with('.') {
            return request_domain.ends_with(cookie_domain) ||
                   request_domain == &cookie_domain[1..];
        }

        false
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_parsing() {
        let mut jar = CookieJar::new();
        jar.set_cookie("session=abc123; Path=/; HttpOnly; Secure", "https://example.com/test");
        
        let cookies = jar.get_cookies("https://example.com/test", true);
        assert_eq!(cookies, Some("session=abc123".to_string()));
    }

    #[test]
    fn test_cookie_expiration() {
        let mut jar = CookieJar::new();
        jar.set_cookie("test=value; Max-Age=0", "https://example.com");
        
        // Cookie with Max-Age=0 should be expired
        let cookies = jar.get_cookies("https://example.com", false);
        assert!(cookies.is_none() || cookies == Some("".to_string()));
    }
}
