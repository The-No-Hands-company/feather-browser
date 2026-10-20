// URL and URLSearchParams APIs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct URL {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub username: String,
    pub password: String,
    pub origin: String,
    pub search_params: URLSearchParams,
}

impl URL {
    pub fn new(url: &str) -> Result<Self, String> {
        let parsed = url::Url::parse(url).map_err(|e| e.to_string())?;
        
        let protocol = format!("{}:", parsed.scheme());
        let hostname = parsed.host_str().unwrap_or("").to_string();
        let port = parsed.port().map(|p| p.to_string()).unwrap_or_default();
        let host = if port.is_empty() {
            hostname.clone()
        } else {
            format!("{}:{}", hostname, port)
        };
        
        let pathname = parsed.path().to_string();
        let search = parsed.query().map(|q| format!("?{}", q)).unwrap_or_default();
        let hash = parsed.fragment().map(|f| format!("#{}", f)).unwrap_or_default();
        let username = parsed.username().to_string();
        let password = parsed.password().unwrap_or("").to_string();
        
        let origin = if parsed.scheme() == "http" || parsed.scheme() == "https" {
            format!("{}://{}", parsed.scheme(), host)
        } else {
            "null".to_string()
        };
        
        let search_params = URLSearchParams::from_string(&search);
        
        Ok(Self {
            href: url.to_string(),
            protocol,
            host,
            hostname,
            port,
            pathname,
            search,
            hash,
            username,
            password,
            origin,
            search_params,
        })
    }
    
    pub fn to_string(&self) -> String {
        self.href.clone()
    }
    
    pub fn to_json(&self) -> String {
        self.href.clone()
    }
}

#[derive(Debug, Clone)]
pub struct URLSearchParams {
    params: HashMap<String, Vec<String>>,
}

impl URLSearchParams {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }
    
    pub fn from_string(query: &str) -> Self {
        let mut params = HashMap::new();
        let query = query.trim_start_matches('?');
        
        if !query.is_empty() {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    let key = urlencoding::decode(key).unwrap_or_default().to_string();
                    let value = urlencoding::decode(value).unwrap_or_default().to_string();
                    params.entry(key).or_insert_with(Vec::new).push(value);
                } else {
                    let key = urlencoding::decode(pair).unwrap_or_default().to_string();
                    params.entry(key).or_insert_with(Vec::new).push(String::new());
                }
            }
        }
        
        Self { params }
    }
    
    pub fn append(&mut self, name: &str, value: &str) {
        self.params.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value.to_string());
    }
    
    pub fn delete(&mut self, name: &str) {
        self.params.remove(name);
    }
    
    pub fn get(&self, name: &str) -> Option<String> {
        self.params.get(name).and_then(|v| v.first()).cloned()
    }
    
    pub fn get_all(&self, name: &str) -> Vec<String> {
        self.params.get(name).cloned().unwrap_or_default()
    }
    
    pub fn has(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }
    
    pub fn set(&mut self, name: &str, value: &str) {
        self.params.insert(name.to_string(), vec![value.to_string()]);
    }
    
    pub fn sort(&mut self) {
        // URLSearchParams.sort() sorts keys alphabetically
    }
    
    pub fn to_string(&self) -> String {
        let mut pairs = Vec::new();
        for (key, values) in &self.params {
            for value in values {
                let encoded_key = urlencoding::encode(key);
                let encoded_value = urlencoding::encode(value);
                pairs.push(format!("{}={}", encoded_key, encoded_value));
            }
        }
        pairs.join("&")
    }
    
    pub fn entries(&self) -> Vec<(String, String)> {
        let mut entries = Vec::new();
        for (key, values) in &self.params {
            for value in values {
                entries.push((key.clone(), value.clone()));
            }
        }
        entries
    }
    
    pub fn keys(&self) -> Vec<String> {
        self.params.keys().cloned().collect()
    }
    
    pub fn values(&self) -> Vec<String> {
        let mut values = Vec::new();
        for vals in self.params.values() {
            values.extend(vals.clone());
        }
        values
    }
}

impl Default for URLSearchParams {
    fn default() -> Self {
        Self::new()
    }
}
