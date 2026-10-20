// Advanced HTTP Client with HTTP/2 and HTTP/3 support
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct HttpClient {
    config: ClientConfig,
    cache: HttpCache,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub user_agent: String,
    pub enable_http2: bool,
    pub enable_http3: bool,
    pub enable_compression: bool,
    pub enable_cookies: bool,
    pub enable_keepalive: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_redirects: 10,
            user_agent: "PronBrow/1.0".to_string(),
            enable_http2: true,
            enable_http3: true,
            enable_compression: true,
            enable_cookies: true,
            enable_keepalive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub url: String,
    pub response: HttpResponse,
    pub expires: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub version: HttpVersion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

#[derive(Debug, Clone, Copy)]
pub enum HttpVersion {
    HTTP1_1,
    HTTP2,
    HTTP3,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub version: HttpVersion,
}

#[derive(Debug)]
pub struct ConnectionPool {
    connections: HashMap<String, Vec<Connection>>,
    max_per_host: usize,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub last_used: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    HTTP1_1,
    HTTP2,
    HTTP3,
}

impl HttpClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            cache: HttpCache::new(100_000_000), // 100MB cache
        }
    }

    pub async fn request(&mut self, request: HttpRequest) -> Result<HttpResponse, String> {
        // Check cache first
        if request.method == HttpMethod::GET {
            if let Some(cached) = self.cache.get(&request.url) {
                return Ok(cached.response.clone());
            }
        }

        // Simple non-recursive implementation for now
        // TODO: Properly handle redirects with Box::pin
        let response = self.send_simple_request(&request).await?;

        // Cache GET responses if cacheable
        if request.method == HttpMethod::GET && self.is_cacheable(&response) {
            if let Err(e) = self.cache.put(&request.url, response.clone()) {
                // Log cache failure but don't fail the request
                log::warn!("Failed to cache response for {}: {}", request.url, e);
            }
        }

        Ok(response)
    }

    async fn send_simple_request(&self, request: &HttpRequest) -> Result<HttpResponse, String> {
        // Select protocol based on configuration and URL
        let protocol = if request.url.starts_with("https://") {
            if self.config.enable_http3 {
                Protocol::HTTP3
            } else if self.config.enable_http2 {
                Protocol::HTTP2
            } else {
                Protocol::HTTP1_1
            }
        } else {
            Protocol::HTTP1_1
        };
        
        // Create a placeholder connection for now
        // In production, would use connection pool
        let connection = Connection {
            host: request.url.split('/').nth(2).unwrap_or("").to_string(),
            port: if request.url.starts_with("https://") { 443 } else { 80 },
            protocol,
            last_used: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Route to appropriate protocol handler
        match protocol {
            Protocol::HTTP1_1 => self.send_http1(request, &connection).await,
            Protocol::HTTP2 => self.send_http2(request, &connection).await,
            Protocol::HTTP3 => self.send_http3(request, &connection).await,
        }
    }

    async fn send_http1(&self, request: &HttpRequest, _connection: &Connection) 
        -> Result<HttpResponse, String> {
        // Production HTTP/1.1 implementation using reqwest
        use reqwest::header::{HeaderName, HeaderValue};
        
        let client = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .user_agent(&self.config.user_agent)
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;
        
        let mut req = match request.method {
            HttpMethod::GET => client.get(&request.url),
            HttpMethod::POST => client.post(&request.url),
            HttpMethod::PUT => client.put(&request.url),
            HttpMethod::DELETE => client.delete(&request.url),
            HttpMethod::HEAD => client.head(&request.url),
            HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &request.url),
            HttpMethod::PATCH => client.patch(&request.url),
            _ => return Err("Unsupported HTTP method".to_string()),
        };
        
        // Add headers from request
        for (key, value) in &request.headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value)
            ) {
                req = req.header(name, val);
            }
        }
        
        // Add body if present
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }
        
        // Execute request
        let response = req.send().await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason()
            .unwrap_or("Unknown").to_string();
        
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(val) = value.to_str() {
                headers.insert(key.as_str().to_string(), val.to_string());
            }
        }
        
        let body = response.bytes().await
            .map_err(|e| format!("Failed to read response body: {}", e))?
            .to_vec();
        
        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
            version: HttpVersion::HTTP1_1,
        })
    }

    async fn send_http2(&self, request: &HttpRequest, _connection: &Connection) 
        -> Result<HttpResponse, String> {
        // HTTP/2 with multiplexing - reqwest handles this automatically
        use reqwest::header::{HeaderName, HeaderValue};
        
        let client = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .user_agent(&self.config.user_agent)
            .http2_prior_knowledge() // Force HTTP/2
            .build()
            .map_err(|e| format!("HTTP/2 client error: {}", e))?;
        
        let mut req = match request.method {
            HttpMethod::GET => client.get(&request.url),
            HttpMethod::POST => client.post(&request.url),
            HttpMethod::PUT => client.put(&request.url),
            HttpMethod::DELETE => client.delete(&request.url),
            HttpMethod::HEAD => client.head(&request.url),
            HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &request.url),
            HttpMethod::PATCH => client.patch(&request.url),
            _ => return Err("Unsupported HTTP method".to_string()),
        };
        
        for (key, value) in &request.headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value)
            ) {
                req = req.header(name, val);
            }
        }
        
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }
        
        let response = req.send().await
            .map_err(|e| format!("HTTP/2 request failed: {}", e))?;
        
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason()
            .unwrap_or("Unknown").to_string();
        
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(val) = value.to_str() {
                headers.insert(key.as_str().to_string(), val.to_string());
            }
        }
        
        let body = response.bytes().await
            .map_err(|e| format!("Failed to read response body: {}", e))?
            .to_vec();
        
        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
            version: HttpVersion::HTTP2,
        })
    }

    async fn send_http3(&self, request: &HttpRequest, connection: &Connection) 
        -> Result<HttpResponse, String> {
        // HTTP/3 over QUIC - For now, fallback to HTTP/2
        // Full HTTP/3 support requires quinn or h3 crate
        // TODO: Implement native HTTP/3 with quinn crate
        log::warn!("HTTP/3 requested but not yet implemented, falling back to HTTP/2");
        self.send_http2(request, connection).await
    }

    fn is_cacheable(&self, response: &HttpResponse) -> bool {
        if let Some(cache_control) = response.headers.get("cache-control") {
            !cache_control.contains("no-cache") && !cache_control.contains("no-store")
        } else {
            true
        }
    }

    pub async fn get(&mut self, url: &str) -> Result<HttpResponse, String> {
        self.request(HttpRequest {
            method: HttpMethod::GET,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            version: if self.config.enable_http3 {
                HttpVersion::HTTP3
            } else if self.config.enable_http2 {
                HttpVersion::HTTP2
            } else {
                HttpVersion::HTTP1_1
            },
        }).await
    }

    pub async fn post(&mut self, url: &str, body: Vec<u8>) -> Result<HttpResponse, String> {
        self.request(HttpRequest {
            method: HttpMethod::POST,
            url: url.to_string(),
            headers: HashMap::new(),
            body: Some(body),
            version: if self.config.enable_http2 {
                HttpVersion::HTTP2
            } else {
                HttpVersion::HTTP1_1
            },
        }).await
    }
}

impl HttpCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            current_size: 0,
        }
    }

    pub fn get(&self, url: &str) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get(url) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now < entry.expires {
                return Some(entry);
            }
        }
        None
    }

    pub fn put(&mut self, url: &str, response: HttpResponse) -> Result<(), String> {
        let size = response.body.len();
        
        // Evict if necessary
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
        }

        let expires = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; // Default 1 hour

        self.entries.insert(url.to_string(), CacheEntry {
            url: url.to_string(),
            response,
            expires,
            etag: None,
            last_modified: None,
        });

        self.current_size += size;
        Ok(())
    }

    fn evict_oldest(&mut self) {
        if let Some((url, _)) = self.entries.iter().next() {
            let url = url.clone();
            if let Some(entry) = self.entries.remove(&url) {
                self.current_size -= entry.response.body.len();
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
    }
}

impl ConnectionPool {
    pub fn new(max_per_host: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_per_host,
        }
    }

    pub fn get_or_create(&mut self, url: &str) -> Result<Connection, String> {
        let (host, port, _use_tls) = Self::parse_url(url)?;
        
        let key = format!("{}:{}", host, port);
        
        if let Some(connections) = self.connections.get_mut(&key) {
            if let Some(conn) = connections.pop() {
                return Ok(conn);
            }
        }

        // Create new connection
        // TODO: Implement TLS support using use_tls flag
        Ok(Connection {
            host: host.to_string(),
            port,
            protocol: Protocol::HTTP2, // Prefer HTTP/2
            last_used: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    pub fn return_connection(&mut self, connection: Connection) {
        let key = format!("{}:{}", connection.host, connection.port);
        
        let connections = self.connections.entry(key).or_insert_with(Vec::new);
        
        if connections.len() < self.max_per_host {
            connections.push(connection);
        }
    }

    fn parse_url(url: &str) -> Result<(String, u16, bool), String> {
        if url.starts_with("https://") {
            let without_scheme = &url[8..];
            let parts: Vec<&str> = without_scheme.split('/').collect();
            let host_port: Vec<&str> = parts[0].split(':').collect();
            let host = host_port[0].to_string();
            let port = if host_port.len() > 1 {
                host_port[1].parse().unwrap_or(443)
            } else {
                443
            };
            Ok((host, port, true))
        } else if url.starts_with("http://") {
            let without_scheme = &url[7..];
            let parts: Vec<&str> = without_scheme.split('/').collect();
            let host_port: Vec<&str> = parts[0].split(':').collect();
            let host = host_port[0].to_string();
            let port = if host_port.len() > 1 {
                host_port[1].parse().unwrap_or(80)
            } else {
                80
            };
            Ok((host, port, false))
        } else {
            Err("Invalid URL scheme".to_string())
        }
    }
}

// Cookie Jar for managing cookies
#[derive(Debug, Clone)]
pub struct CookieJar {
    cookies: HashMap<String, Vec<Cookie>>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    pub fn set(&mut self, cookie: Cookie) {
        let domain_cookies = self.cookies.entry(cookie.domain.clone()).or_insert_with(Vec::new);
        
        // Replace existing cookie with same name
        domain_cookies.retain(|c| c.name != cookie.name);
        domain_cookies.push(cookie);
    }

    pub fn get(&self, domain: &str, name: &str) -> Option<&Cookie> {
        self.cookies.get(domain)?
            .iter()
            .find(|c| c.name == name)
    }

    pub fn get_all_for_domain(&self, domain: &str) -> Vec<&Cookie> {
        self.cookies.get(domain)
            .map(|cookies| cookies.iter().collect())
            .unwrap_or_default()
    }

    pub fn clear_expired(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for cookies in self.cookies.values_mut() {
            cookies.retain(|c| {
                if let Some(expires) = c.expires {
                    expires > now
                } else {
                    true
                }
            });
        }
    }
}
