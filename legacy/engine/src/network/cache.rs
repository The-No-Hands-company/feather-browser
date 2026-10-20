// HTTP caching implementation
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub url: String,
    pub data: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub cached_at: SystemTime,
    pub max_age: Duration,
}

pub struct HTTPCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    current_size: usize,
}

impl HTTPCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            current_size: 0,
        }
    }
    
    pub fn get(&self, url: &str) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get(url) {
            if SystemTime::now().duration_since(entry.cached_at).ok()? < entry.max_age {
                return Some(entry);
            }
        }
        None
    }
    
    pub fn put(&mut self, url: String, data: Vec<u8>, headers: HashMap<String, String>, max_age: Duration) {
        let size = data.len();
        
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
        }
        
        let entry = CacheEntry {
            url: url.clone(),
            data,
            headers,
            cached_at: SystemTime::now(),
            max_age,
        };
        
        self.entries.insert(url, entry);
        self.current_size += size;
    }
    
    fn evict_oldest(&mut self) {
        if let Some((url, entry)) = self.entries.iter().min_by_key(|(_, e)| e.cached_at).map(|(k, v)| (k.clone(), v.clone())) {
            self.current_size -= entry.data.len();
            self.entries.remove(&url);
        }
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
    }
}
