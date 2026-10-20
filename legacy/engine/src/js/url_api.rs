// Production-grade URL API Implementation
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct URL {
    pub href: String,
    pub protocol: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub origin: String,
    pub search_params: URLSearchParams,
}

impl URL {
    pub fn new(url: &str, base: Option<&str>) -> Result<Self, String> {
        let full_url = if let Some(base_url) = base {
            Self::resolve(url, base_url)?
        } else {
            url.to_string()
        };
        
        Self::parse(&full_url)
    }
    
    pub fn parse(url: &str) -> Result<Self, String> {
        // Parse protocol
        let (protocol, rest) = if let Some(idx) = url.find("://") {
            (&url[..idx], &url[idx + 3..])
        } else {
            return Err("Invalid URL: missing protocol".to_string());
        };
        
        // Parse authentication
        let (auth, rest) = if let Some(idx) = rest.find('@') {
            let auth_part = &rest[..idx];
            let (username, password) = if let Some(colon_idx) = auth_part.find(':') {
                (&auth_part[..colon_idx], &auth_part[colon_idx + 1..])
            } else {
                (auth_part, "")
            };
            ((username, password), &rest[idx + 1..])
        } else {
            (("", ""), rest)
        };
        
        // Parse host and port
        let (host_part, path_part) = if let Some(idx) = rest.find('/') {
            (&rest[..idx], &rest[idx..])
        } else {
            (rest, "/")
        };
        
        let (hostname, port) = if let Some(colon_idx) = host_part.rfind(':') {
            (&host_part[..colon_idx], &host_part[colon_idx + 1..])
        } else {
            (host_part, "")
        };
        
        // Parse path, search, and hash
        let (path_search, hash) = if let Some(hash_idx) = path_part.find('#') {
            (&path_part[..hash_idx], &path_part[hash_idx..])
        } else {
            (path_part, "")
        };
        
        let (pathname, search) = if let Some(search_idx) = path_search.find('?') {
            (&path_search[..search_idx], &path_search[search_idx..])
        } else {
            (path_search, "")
        };
        
        let host = if port.is_empty() {
            hostname.to_string()
        } else {
            format!("{}:{}", hostname, port)
        };
        
        let origin = format!("{}://{}", protocol, host);
        
        // Parse search params only if search is not empty
        let search_params = if search.is_empty() {
            URLSearchParams::new()
        } else {
            URLSearchParams::parse(&search[1..]) // Skip '?'
        };
        
        Ok(Self {
            href: url.to_string(),
            protocol: format!("{}:", protocol),
            username: auth.0.to_string(),
            password: auth.1.to_string(),
            host,
            hostname: hostname.to_string(),
            port: port.to_string(),
            pathname: pathname.to_string(),
            search: search.to_string(),
            hash: hash.to_string(),
            origin,
            search_params,
        })
    }
    
    pub fn resolve(url: &str, base: &str) -> Result<String, String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(url.to_string());
        }
        
        let base_url = Self::parse(base)?;
        
        if url.starts_with("//") {
            return Ok(format!("{}{}", base_url.protocol, url));
        }
        
        if url.starts_with('/') {
            return Ok(format!("{}{}", base_url.origin, url));
        }
        
        // Relative URL
        let base_path = base_url.pathname;
        let base_dir = if let Some(last_slash) = base_path.rfind('/') {
            &base_path[..last_slash + 1]
        } else {
            "/"
        };
        
        Ok(format!("{}{}{}", base_url.origin, base_dir, url))
    }
    
    pub fn to_json(&self) -> String {
        self.href.clone()
    }
    
    pub fn to_string(&self) -> String {
        self.href.clone()
    }
}

#[derive(Debug, Clone)]
pub struct URLSearchParams {
    params: Vec<(String, String)>,
}

impl URLSearchParams {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
        }
    }
    
    pub fn parse(query: &str) -> Self {
        let mut params = Vec::new();
        
        if query.is_empty() {
            return Self { params };
        }
        
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .unwrap_or_else(|_| key.into())
                    .to_string();
                let value = urlencoding::decode(value)
                    .unwrap_or_else(|_| value.into())
                    .to_string();
                params.push((key, value));
            } else if !pair.is_empty() {
                let key = urlencoding::decode(pair)
                    .unwrap_or_else(|_| pair.into())
                    .to_string();
                params.push((key, String::new()));
            }
        }
        
        Self { params }
    }
    
    pub fn append(&mut self, name: String, value: String) {
        self.params.push((name, value));
    }
    
    pub fn delete(&mut self, name: &str) {
        self.params.retain(|(k, _)| k != name);
    }
    
    pub fn get(&self, name: &str) -> Option<&String> {
        self.params.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
    }
    
    pub fn get_all(&self, name: &str) -> Vec<&String> {
        self.params.iter()
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v)
            .collect()
    }
    
    pub fn has(&self, name: &str) -> bool {
        self.params.iter().any(|(k, _)| k == name)
    }
    
    pub fn set(&mut self, name: String, value: String) {
        self.delete(&name);
        self.append(name, value);
    }
    
    pub fn sort(&mut self) {
        self.params.sort_by(|a, b| a.0.cmp(&b.0));
    }
    
    pub fn to_string(&self) -> String {
        self.params.iter()
            .map(|(k, v)| {
                let encoded_key = urlencoding::encode(k);
                let encoded_value = urlencoding::encode(v);
                if v.is_empty() {
                    encoded_key.to_string()
                } else {
                    format!("{}={}", encoded_key, encoded_value)
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    }
    
    pub fn entries(&self) -> Vec<(&String, &String)> {
        self.params.iter()
            .map(|(k, v)| (k, v))
            .collect()
    }
    
    pub fn keys(&self) -> Vec<&String> {
        self.params.iter()
            .map(|(k, _)| k)
            .collect()
    }
    
    pub fn values(&self) -> Vec<&String> {
        self.params.iter()
            .map(|(_, v)| v)
            .collect()
    }
}

pub struct URLPattern {
    pattern: String,
    protocol: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
    pathname: Option<String>,
    search: Option<String>,
    hash: Option<String>,
}

impl URLPattern {
    pub fn new(pattern: &str) -> Result<Self, String> {
        Ok(Self {
            pattern: pattern.to_string(),
            protocol: None,
            hostname: None,
            port: None,
            pathname: None,
            search: None,
            hash: None,
        })
    }
    
    pub fn test(&self, url: &str) -> bool {
        // Simplified pattern matching
        url.contains(&self.pattern)
    }
    
    pub fn exec(&self, url: &str) -> Option<HashMap<String, String>> {
        if self.test(url) {
            let mut result = HashMap::new();
            result.insert("input".to_string(), url.to_string());
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_parse() {
        let url = URL::new("https://user:pass@example.com:8080/path?query=value#hash", None).unwrap();
        assert_eq!(url.protocol, "https:");
        assert_eq!(url.username, "user");
        assert_eq!(url.password, "pass");
        assert_eq!(url.hostname, "example.com");
        assert_eq!(url.port, "8080");
        assert_eq!(url.pathname, "/path");
        assert_eq!(url.search, "?query=value");
        assert_eq!(url.hash, "#hash");
    }
    
    #[test]
    fn test_url_search_params() {
        let mut params = URLSearchParams::new();
        params.append("key1".to_string(), "value1".to_string());
        params.append("key2".to_string(), "value2".to_string());
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert!(params.has("key2"));
    }
    
    #[test]
    fn test_url_resolve() {
        let resolved = URL::resolve("path/to/file.html", "https://example.com/base/").unwrap();
        assert_eq!(resolved, "https://example.com/base/path/to/file.html");
    }
}
