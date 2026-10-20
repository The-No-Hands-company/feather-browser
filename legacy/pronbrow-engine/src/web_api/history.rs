// History API for navigation and session management
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub state: Option<String>, // Serialized state object
}

#[derive(Debug, Clone)]
pub enum ScrollRestoration {
    Auto,
    Manual,
}

pub struct History {
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
    current_index: Arc<Mutex<usize>>,
    scroll_restoration: Arc<Mutex<ScrollRestoration>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            current_index: Arc::new(Mutex::new(0)),
            scroll_restoration: Arc::new(Mutex::new(ScrollRestoration::Auto)),
        }
    }
    
    pub fn length(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
    
    pub fn state(&self) -> Option<String> {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap();
        entries.get(index).and_then(|e| e.state.clone())
    }
    
    pub fn scroll_restoration(&self) -> ScrollRestoration {
        self.scroll_restoration.lock().unwrap().clone()
    }
    
    pub fn set_scroll_restoration(&self, value: ScrollRestoration) {
        *self.scroll_restoration.lock().unwrap() = value;
    }
    
    pub fn go(&self, delta: i32) -> Result<(), String> {
        let mut index = self.current_index.lock().unwrap();
        let entries = self.entries.lock().unwrap();
        let new_index = (*index as i32 + delta) as usize;
        
        if new_index < entries.len() {
            *index = new_index;
            Ok(())
        } else {
            Err("Invalid history index".to_string())
        }
    }
    
    pub fn back(&self) -> Result<(), String> {
        self.go(-1)
    }
    
    pub fn forward(&self) -> Result<(), String> {
        self.go(1)
    }
    
    pub fn push_state(&self, state: Option<String>, title: &str, url: Option<&str>) -> Result<(), String> {
        let mut entries = self.entries.lock().unwrap();
        let mut index = self.current_index.lock().unwrap();
        
        // Remove forward history when pushing new state
        entries.truncate(*index + 1);
        
        let current_url = if let Some(entry) = entries.last() {
            entry.url.clone()
        } else {
            String::new()
        };
        
        let new_url = url.unwrap_or(&current_url).to_string();
        
        entries.push(HistoryEntry {
            url: new_url,
            title: title.to_string(),
            state,
        });
        
        *index = entries.len() - 1;
        Ok(())
    }
    
    pub fn replace_state(&self, state: Option<String>, title: &str, url: Option<&str>) -> Result<(), String> {
        let mut entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap();
        
        if let Some(entry) = entries.get_mut(index) {
            entry.state = state;
            entry.title = title.to_string();
            if let Some(new_url) = url {
                entry.url = new_url.to_string();
            }
            Ok(())
        } else {
            Err("No current history entry".to_string())
        }
    }
    
    pub fn current_entry(&self) -> Option<HistoryEntry> {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap();
        entries.get(index).cloned()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

// Location API
#[derive(Debug, Clone)]
pub struct Location {
    pub href: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub origin: String,
}

impl Location {
    pub fn from_url(url: &str) -> Result<Self, String> {
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
        
        let origin = if parsed.scheme() == "http" || parsed.scheme() == "https" {
            format!("{}://{}", parsed.scheme(), host)
        } else {
            "null".to_string()
        };
        
        Ok(Self {
            href: url.to_string(),
            protocol,
            host,
            hostname,
            port,
            pathname,
            search,
            hash,
            origin,
        })
    }
    
    pub fn assign(&mut self, url: &str) -> Result<(), String> {
        *self = Self::from_url(url)?;
        Ok(())
    }
    
    pub fn replace(&mut self, url: &str) -> Result<(), String> {
        *self = Self::from_url(url)?;
        Ok(())
    }
    
    pub fn reload(&self) {
        // Trigger page reload
    }
    
    pub fn to_string(&self) -> String {
        self.href.clone()
    }
}
