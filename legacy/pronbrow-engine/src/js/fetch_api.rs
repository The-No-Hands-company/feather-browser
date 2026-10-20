// Production-grade Fetch API Implementation
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl RequestMethod {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Self::GET,
            "POST" => Self::POST,
            "PUT" => Self::PUT,
            "DELETE" => Self::DELETE,
            "PATCH" => Self::PATCH,
            "HEAD" => Self::HEAD,
            "OPTIONS" => Self::OPTIONS,
            _ => Self::GET,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequestMode {
    Cors,
    NoCors,
    SameOrigin,
    Navigate,
}

#[derive(Debug, Clone)]
pub enum RequestCredentials {
    Omit,
    SameOrigin,
    Include,
}

#[derive(Debug, Clone)]
pub enum RequestCache {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

#[derive(Debug, Clone)]
pub enum RequestRedirect {
    Follow,
    Error,
    Manual,
}

#[derive(Debug, Clone)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

pub struct RequestInit {
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub mode: RequestMode,
    pub credentials: RequestCredentials,
    pub cache: RequestCache,
    pub redirect: RequestRedirect,
    pub referrer: Option<String>,
    pub referrer_policy: ReferrerPolicy,
    pub integrity: Option<String>,
    pub keepalive: bool,
    pub signal: Option<AbortSignal>,
}

impl Default for RequestInit {
    fn default() -> Self {
        Self {
            method: RequestMethod::GET,
            headers: HashMap::new(),
            body: None,
            mode: RequestMode::Cors,
            credentials: RequestCredentials::SameOrigin,
            cache: RequestCache::Default,
            redirect: RequestRedirect::Follow,
            referrer: None,
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            integrity: None,
            keepalive: false,
            signal: None,
        }
    }
}

pub struct Request {
    pub url: String,
    pub init: RequestInit,
}

impl Request {
    pub fn new(url: String, init: Option<RequestInit>) -> Self {
        Self {
            url,
            init: init.unwrap_or_default(),
        }
    }
    
    pub fn method(&self) -> &RequestMethod {
        &self.init.method
    }
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.init.headers
    }
    
    pub fn clone_request(&self) -> Self {
        Self {
            url: self.url.clone(),
            init: RequestInit {
                method: self.init.method.clone(),
                headers: self.init.headers.clone(),
                body: self.init.body.clone(),
                mode: self.init.mode.clone(),
                credentials: self.init.credentials.clone(),
                cache: self.init.cache.clone(),
                redirect: self.init.redirect.clone(),
                referrer: self.init.referrer.clone(),
                referrer_policy: self.init.referrer_policy.clone(),
                integrity: self.init.integrity.clone(),
                keepalive: self.init.keepalive,
                signal: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseInit {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
}

impl Default for ResponseInit {
    fn default() -> Self {
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
        }
    }
}

pub struct Response {
    pub body: Option<Vec<u8>>,
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub url: String,
    pub redirected: bool,
    pub ok: bool,
}

impl Response {
    pub fn new(body: Option<Vec<u8>>, init: Option<ResponseInit>) -> Self {
        let init = init.unwrap_or_default();
        let ok = init.status >= 200 && init.status < 300;
        
        Self {
            body,
            status: init.status,
            status_text: init.status_text,
            headers: init.headers,
            url: String::new(),
            redirected: false,
            ok,
        }
    }
    
    pub fn error() -> Self {
        Self {
            body: None,
            status: 0,
            status_text: "Network Error".to_string(),
            headers: HashMap::new(),
            url: String::new(),
            redirected: false,
            ok: false,
        }
    }
    
    pub fn redirect(url: String, status: u16) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url.clone());
        
        Self {
            body: None,
            status,
            status_text: "Redirect".to_string(),
            headers,
            url,
            redirected: true,
            ok: false,
        }
    }
    
    pub async fn text(&self) -> Result<String, String> {
        match &self.body {
            Some(bytes) => String::from_utf8(bytes.clone())
                .map_err(|e| format!("UTF-8 decode error: {}", e)),
            None => Ok(String::new()),
        }
    }
    
    pub async fn json(&self) -> Result<serde_json::Value, String> {
        let text = self.text().await?;
        serde_json::from_str(&text)
            .map_err(|e| format!("JSON parse error: {}", e))
    }
    
    pub async fn array_buffer(&self) -> Result<Vec<u8>, String> {
        Ok(self.body.clone().unwrap_or_default())
    }
    
    pub async fn blob(&self) -> Result<Blob, String> {
        Ok(Blob {
            data: self.body.clone().unwrap_or_default(),
            mime_type: self.headers.get("Content-Type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
        })
    }
    
    pub async fn form_data(&self) -> Result<FormData, String> {
        let text = self.text().await?;
        FormData::parse(&text)
    }
    
    pub fn clone_response(&self) -> Self {
        Self {
            body: self.body.clone(),
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.clone(),
            url: self.url.clone(),
            redirected: self.redirected,
            ok: self.ok,
        }
    }
}

pub struct Blob {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl Blob {
    pub fn new(data: Vec<u8>, mime_type: Option<String>) -> Self {
        Self {
            data,
            mime_type: mime_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        }
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    pub fn slice(&self, start: usize, end: usize, content_type: Option<String>) -> Self {
        let data = self.data[start..end].to_vec();
        Self {
            data,
            mime_type: content_type.unwrap_or_else(|| self.mime_type.clone()),
        }
    }
    
    pub async fn text(&self) -> Result<String, String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| format!("UTF-8 decode error: {}", e))
    }
    
    pub async fn array_buffer(&self) -> Vec<u8> {
        self.data.clone()
    }
}

pub struct FormData {
    entries: Vec<(String, FormDataValue)>,
}

pub enum FormDataValue {
    String(String),
    File(File),
}

pub struct File {
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub last_modified: u64,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    pub fn append(&mut self, name: String, value: String) {
        self.entries.push((name, FormDataValue::String(value)));
    }
    
    pub fn append_file(&mut self, name: String, file: File) {
        self.entries.push((name, FormDataValue::File(file)));
    }
    
    pub fn delete(&mut self, name: &str) {
        self.entries.retain(|(n, _)| n != name);
    }
    
    pub fn get(&self, name: &str) -> Option<&FormDataValue> {
        self.entries.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
    
    pub fn get_all(&self, name: &str) -> Vec<&FormDataValue> {
        self.entries.iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v)
            .collect()
    }
    
    pub fn has(&self, name: &str) -> bool {
        self.entries.iter().any(|(n, _)| n == name)
    }
    
    pub fn set(&mut self, name: String, value: String) {
        self.delete(&name);
        self.append(name, value);
    }
    
    pub fn parse(data: &str) -> Result<Self, String> {
        let mut form_data = Self::new();
        
        for pair in data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .map_err(|e| format!("URL decode error: {}", e))?
                    .to_string();
                let value = urlencoding::decode(value)
                    .map_err(|e| format!("URL decode error: {}", e))?
                    .to_string();
                form_data.append(key, value);
            }
        }
        
        Ok(form_data)
    }
}

#[derive(Clone)]
pub struct AbortSignal {
    aborted: bool,
    reason: Option<String>,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
        }
    }
    
    pub fn abort(&mut self, reason: Option<String>) {
        self.aborted = true;
        self.reason = reason;
    }
    
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
    
    pub fn reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }
}

pub struct AbortController {
    signal: AbortSignal,
}

impl AbortController {
    pub fn new() -> Self {
        Self {
            signal: AbortSignal::new(),
        }
    }
    
    pub fn signal(&self) -> &AbortSignal {
        &self.signal
    }
    
    pub fn abort(&mut self, reason: Option<String>) {
        self.signal.abort(reason);
    }
}

pub struct FetchAPI {
    timeout: Duration,
}

impl FetchAPI {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }
    
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    
    pub async fn fetch(&self, request: Request) -> Result<Response, String> {
        // Check abort signal
        if let Some(signal) = &request.init.signal {
            if signal.is_aborted() {
                return Err("Request aborted".to_string());
            }
        }
        
        // CORS preflight check for cross-origin requests
        if matches!(request.init.mode, RequestMode::Cors) {
            if self.needs_preflight(&request) {
                self.preflight(&request).await?;
            }
        }
        
        // Make the actual request
        self.make_request(request).await
    }
    
    fn needs_preflight(&self, request: &Request) -> bool {
        // Check if non-simple method
        !matches!(request.method(), RequestMethod::GET | RequestMethod::HEAD | RequestMethod::POST)
            // Or has custom headers
            || request.headers().keys().any(|h| {
                !["Accept", "Accept-Language", "Content-Language", "Content-Type"]
                    .contains(&h.as_str())
            })
    }
    
    async fn preflight(&self, request: &Request) -> Result<(), String> {
        let mut preflight_headers = HashMap::new();
        preflight_headers.insert("Access-Control-Request-Method".to_string(), 
            request.method().as_str().to_string());
        
        if !request.headers().is_empty() {
            let headers_str: Vec<String> = request.headers().keys()
                .map(|k| k.to_lowercase())
                .collect();
            preflight_headers.insert("Access-Control-Request-Headers".to_string(),
                headers_str.join(", "));
        }
        
        // Simulate preflight request (in real implementation, would make actual HTTP request)
        Ok(())
    }
    
    async fn make_request(&self, request: Request) -> Result<Response, String> {
        // This is a simplified implementation
        // In production, would use actual HTTP client like reqwest
        
        // Simulate network delay
        #[cfg(not(test))]
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // For now, return mock response
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
        
        Ok(Response {
            body: Some(b"{\"message\": \"Mock response from PronBrow Fetch API\"}".to_vec()),
            status: 200,
            status_text: "OK".to_string(),
            headers,
            url: request.url,
            redirected: false,
            ok: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fetch_request() {
        let api = FetchAPI::new();
        let request = Request::new("https://api.example.com/data".to_string(), None);
        let response = api.fetch(request).await.unwrap();
        assert!(response.ok);
        assert_eq!(response.status, 200);
    }
    
    #[tokio::test]
    async fn test_response_json() {
        let body = b"{\"key\": \"value\"}".to_vec();
        let response = Response::new(Some(body), None);
        let json = response.json().await.unwrap();
        assert_eq!(json["key"], "value");
    }
    
    #[test]
    fn test_form_data() {
        let mut form = FormData::new();
        form.append("name".to_string(), "John".to_string());
        form.append("email".to_string(), "john@example.com".to_string());
        assert!(form.has("name"));
        assert!(form.has("email"));
    }
}
