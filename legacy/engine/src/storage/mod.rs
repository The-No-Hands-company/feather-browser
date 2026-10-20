// Web Storage API - localStorage and sessionStorage

use std::collections::HashMap;

pub struct LocalStorage {
    data: HashMap<String, String>,
}

pub struct SessionStorage {
    data: HashMap<String, String>,
}

pub trait Storage {
    fn get_item(&self, key: &str) -> Option<String>;
    fn set_item(&mut self, key: String, value: String);
    fn remove_item(&mut self, key: &str) -> Option<String>;
    fn clear(&mut self);
    fn key(&self, index: usize) -> Option<String>;
    fn length(&self) -> usize;
}

impl LocalStorage {
    pub fn new() -> Self {
        LocalStorage {
            data: HashMap::new(),
        }
    }
}

impl SessionStorage {
    pub fn new() -> Self {
        SessionStorage {
            data: HashMap::new(),
        }
    }
}

impl Storage for LocalStorage {
    fn get_item(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    
    fn set_item(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
    
    fn remove_item(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
    
    fn clear(&mut self) {
        self.data.clear();
    }
    
    fn key(&self, index: usize) -> Option<String> {
        self.data.keys().nth(index).cloned()
    }
    
    fn length(&self) -> usize {
        self.data.len()
    }
}

impl Storage for SessionStorage {
    fn get_item(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    
    fn set_item(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
    
    fn remove_item(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
    
    fn clear(&mut self) {
        self.data.clear();
    }
    
    fn key(&self, index: usize) -> Option<String> {
        self.data.keys().nth(index).cloned()
    }
    
    fn length(&self) -> usize {
        self.data.len()
    }
}
