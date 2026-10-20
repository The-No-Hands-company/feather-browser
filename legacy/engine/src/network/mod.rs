// Network Engine - Handles HTTP requests and responses
pub mod http2;
pub mod websocket;
pub mod cache;
pub mod client;
pub mod cookies;
pub mod dns;
pub mod fetch;

use std::error::Error;

pub use http2::{Http2Connection, Http2Stream};
pub use websocket::{WebSocket, WebSocketMessage};
pub use cache::HTTPCache;
pub use cookies::{Cookie, CookieJar, SameSite};
pub use dns::DnsResolver;
pub use fetch::{FetchEngine, FetchRequest, FetchResponse, HttpMethod};

#[derive(Debug, Clone)]
pub struct NetworkEngine {
    client: reqwest::blocking::Client,
}

impl NetworkEngine {
    pub fn new() -> Self {
        NetworkEngine {
            client: reqwest::blocking::Client::builder()
                .user_agent("PronBrow/0.1.0")
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .user_agent("PronBrow/0.1.0")
            .build()?;
        
        let response = client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }

    pub fn fetch_blocking(&self, url: &str) -> Result<String, Box<dyn Error>> {
        let response = self.client.get(url).send()?;
        let body = response.text()?;
        Ok(body)
    }

    pub fn parse_url(&self, input: &str) -> String {
        if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else if input.contains('.') && !input.contains(' ') {
            format!("https://{}", input)
        } else {
            format!("https://www.google.com/search?q={}", urlencoding::encode(input))
        }
    }
}

impl Default for NetworkEngine {
    fn default() -> Self {
        Self::new()
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}
