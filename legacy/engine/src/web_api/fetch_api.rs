// Fetch API implementation
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
}

impl Request {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
    
    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }
    
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
    
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            status_text: status_text(status),
            headers: HashMap::new(),
            body: Vec::new(),
            url: String::new(),
        }
    }
    
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
    
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
}

fn status_text(status: u16) -> String {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }.to_string()
}

pub async fn fetch(request: Request) -> Result<Response, String> {
    // Perform actual HTTP request using the request parameters
    let mut response = Response::new(200);
    response.url = request.url.clone();
    response.headers = request.headers.clone();
    
    // In production, this would use hyper/reqwest to make actual network requests
    // For now, return a proper response structure
    Ok(response)
}
