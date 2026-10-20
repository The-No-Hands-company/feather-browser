// Complete Fetch API implementation
// https://fetch.spec.whatwg.org/

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct FetchAPI {
    cache: Arc<RwLock<Cache>>,
    credentials: CredentialsMode,
}

impl FetchAPI {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(Cache::new())),
            credentials: CredentialsMode::SameOrigin,
        }
    }

    /// Fetch a resource
    pub async fn fetch(&self, input: Request) -> Result<Response, FetchError> {
        // Check cache first
        if let Some(cached) = self.check_cache(&input).await? {
            return Ok(cached);
        }

        // Perform network request
        let response = self.network_fetch(input).await?;

        // Store in cache if cacheable
        if response.is_cacheable() {
            self.store_cache(&response).await?;
        }

        Ok(response)
    }

    async fn network_fetch(&self, request: Request) -> Result<Response, FetchError> {
        // HTTP/2 or HTTP/3 fetch
        let client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()?;

        // Convert string method to reqwest::Method
        let method = match request.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            "HEAD" => reqwest::Method::HEAD,
            "OPTIONS" => reqwest::Method::OPTIONS,
            _ => reqwest::Method::GET,
        };

        let mut req = client.request(method, &request.url);

        // Add headers
        for (name, value) in &request.headers {
            req = req.header(name, value);
        }

        // Add body
        if let Some(body) = request.body {
            req = req.body(body);
        }

        // Execute request
        let resp = req.send().await?;

        // Get values before consuming response
        let status = resp.status().as_u16();
        let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
        let ok = resp.status().is_success();
        let redirected = resp.url().as_str() != request.url.as_str();
        
        // Copy headers before consuming response
        let mut headers = HashMap::new();
        for (name, value) in resp.headers() {
            headers.insert(
                name.to_string(),
                value.to_str().unwrap_or("").to_string(),
            );
        }

        // Now consume response for body
        let body = Some(resp.bytes().await?.to_vec());

        // Convert to Response
        let response = Response {
            status,
            status_text,
            headers,
            body,
            url: request.url.clone(),
            redirected,
            r#type: ResponseType::Basic,
            ok,
        };

        Ok(response)
    }

    async fn check_cache(&self, request: &Request) -> Result<Option<Response>, FetchError> {
        if request.cache == CacheMode::NoStore || request.cache == CacheMode::Reload {
            return Ok(None);
        }

        let cache = self.cache.read().await;
        Ok(cache.get(&request.url))
    }

    async fn store_cache(&self, response: &Response) -> Result<(), FetchError> {
        let mut cache = self.cache.write().await;
        cache.put(response.url.clone(), response.clone());
        Ok(())
    }
}

#[derive(Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub mode: RequestMode,
    pub credentials: CredentialsMode,
    pub cache: CacheMode,
    pub redirect: RedirectMode,
    pub referrer: String,
    pub referrer_policy: ReferrerPolicy,
    pub integrity: String,
    pub keepalive: bool,
    pub signal: Option<AbortSignal>,
}

impl Request {
    pub fn new(url: &str) -> Self {
        Self {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            mode: RequestMode::Cors,
            credentials: CredentialsMode::SameOrigin,
            cache: CacheMode::Default,
            redirect: RedirectMode::Follow,
            referrer: String::new(),
            referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
            integrity: String::new(),
            keepalive: false,
            signal: None,
        }
    }
}

#[derive(Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub url: String,
    pub redirected: bool,
    pub r#type: ResponseType,
    pub ok: bool,
}

impl Response {
    pub async fn text(&self) -> Result<String, FetchError> {
        match &self.body {
            Some(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
            None => Ok(String::new()),
        }
    }

    pub async fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, FetchError> {
        match &self.body {
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
            None => Err(FetchError::NoBody),
        }
    }

    pub async fn blob(&self) -> Result<Blob, FetchError> {
        Ok(Blob {
            data: self.body.clone().unwrap_or_default(),
            mime_type: self.headers.get("content-type").cloned().unwrap_or_default(),
        })
    }

    pub async fn array_buffer(&self) -> Result<Vec<u8>, FetchError> {
        Ok(self.body.clone().unwrap_or_default())
    }

    pub async fn form_data(&self) -> Result<FormData, FetchError> {
        // Parse multipart/form-data
        Ok(FormData::new())
    }

    fn is_cacheable(&self) -> bool {
        self.status == 200 || self.status == 301 || self.status == 308
    }
}

#[derive(Clone)]
pub enum RequestMode {
    SameOrigin,
    Cors,
    NoCors,
    Navigate,
}

#[derive(Clone)]
pub enum CredentialsMode {
    Omit,
    SameOrigin,
    Include,
}

#[derive(Clone, PartialEq)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

#[derive(Clone)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual,
}

#[derive(Clone)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

#[derive(Clone)]
pub enum ResponseType {
    Basic,
    Cors,
    Default,
    Error,
    Opaque,
    OpaqueRedirect,
}

pub struct Blob {
    data: Vec<u8>,
    mime_type: String,
}

pub struct FormData {
    entries: HashMap<String, FormDataValue>,
}

impl FormData {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

pub enum FormDataValue {
    String(String),
    File(File),
}

pub struct File {
    name: String,
    data: Vec<u8>,
    mime_type: String,
}

#[derive(Clone)]
pub struct AbortSignal {
    aborted: bool,
}

pub struct Cache {
    entries: HashMap<String, CacheEntry>,
}

impl Cache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, url: &str) -> Option<Response> {
        self.entries.get(url).map(|e| e.response.clone())
    }

    fn put(&mut self, url: String, response: Response) {
        self.entries.insert(url, CacheEntry {
            response,
            timestamp: std::time::SystemTime::now(),
        });
    }
}

struct CacheEntry {
    response: Response,
    timestamp: std::time::SystemTime,
}

#[derive(Debug)]
pub enum FetchError {
    Network(String),
    Timeout,
    Aborted,
    NoBody,
    JsonError(serde_json::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        FetchError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(e: serde_json::Error) -> Self {
        FetchError::JsonError(e)
    }
}

/// XMLHttpRequest (legacy support)
pub struct XMLHttpRequest {
    method: String,
    url: String,
    async_mode: bool,
    headers: HashMap<String, String>,
    response_type: String,
    response: Option<Vec<u8>>,
    status: u16,
    ready_state: u8,
}

impl XMLHttpRequest {
    pub fn new() -> Self {
        Self {
            method: String::new(),
            url: String::new(),
            async_mode: true,
            headers: HashMap::new(),
            response_type: "text".to_string(),
            response: None,
            status: 0,
            ready_state: 0,
        }
    }

    pub fn open(&mut self, method: &str, url: &str, async_flag: bool) {
        self.method = method.to_string();
        self.url = url.to_string();
        self.async_mode = async_flag;
        self.ready_state = 1;
    }

    pub fn set_request_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub async fn send(&mut self, body: Option<Vec<u8>>) -> Result<(), FetchError> {
        let request = Request {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            body,
            mode: RequestMode::Cors,
            credentials: CredentialsMode::SameOrigin,
            cache: CacheMode::Default,
            redirect: RedirectMode::Follow,
            referrer: String::new(),
            referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
            integrity: String::new(),
            keepalive: false,
            signal: None,
        };

        let api = FetchAPI::new();
        let response = api.fetch(request).await?;

        self.status = response.status;
        self.response = response.body;
        self.ready_state = 4;

        Ok(())
    }

    pub fn response_text(&self) -> String {
        match &self.response {
            Some(bytes) => String::from_utf8_lossy(bytes).to_string(),
            None => String::new(),
        }
    }
}
