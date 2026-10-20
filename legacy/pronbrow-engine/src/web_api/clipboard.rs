// Production-grade Clipboard API Implementation
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum ClipboardItemType {
    Text(String),
    Html(String),
    Image(Vec<u8>),
    Rtf(String),
    Custom(String, Vec<u8>),
}

#[derive(Clone)]
pub struct ClipboardItem {
    types: Vec<String>,
    data: Vec<ClipboardItemType>,
}

impl ClipboardItem {
    pub fn new(data: Vec<(String, ClipboardItemType)>) -> Self {
        let types = data.iter().map(|(t, _)| t.clone()).collect();
        let item_data = data.into_iter().map(|(_, d)| d).collect();
        
        Self {
            types,
            data: item_data,
        }
    }
    
    pub fn types(&self) -> &[String] {
        &self.types
    }
    
    pub fn get_type(&self, mime_type: &str) -> Option<&ClipboardItemType> {
        self.types.iter()
            .position(|t| t == mime_type)
            .and_then(|idx| self.data.get(idx))
    }
}

pub struct ClipboardAPI {
    clipboard_data: Arc<Mutex<Vec<ClipboardItem>>>,
    permission_granted: Arc<Mutex<bool>>,
}

impl ClipboardAPI {
    pub fn new() -> Self {
        Self {
            clipboard_data: Arc::new(Mutex::new(Vec::new())),
            permission_granted: Arc::new(Mutex::new(true)), // Auto-grant in engine
        }
    }
    
    pub async fn read(&self) -> Result<Vec<ClipboardItem>, String> {
        if !*self.permission_granted.lock().unwrap() {
            return Err("Clipboard permission denied".to_string());
        }
        
        Ok(self.clipboard_data.lock().unwrap().clone())
    }
    
    pub async fn read_text(&self) -> Result<String, String> {
        if !*self.permission_granted.lock().unwrap() {
            return Err("Clipboard permission denied".to_string());
        }
        
        let clipboard = self.clipboard_data.lock().unwrap();
        
        if let Some(item) = clipboard.first() {
            if let Some(ClipboardItemType::Text(text)) = item.get_type("text/plain") {
                return Ok(text.clone());
            }
        }
        
        Ok(String::new())
    }
    
    pub async fn write(&self, items: Vec<ClipboardItem>) -> Result<(), String> {
        if !*self.permission_granted.lock().unwrap() {
            return Err("Clipboard permission denied".to_string());
        }
        
        *self.clipboard_data.lock().unwrap() = items;
        Ok(())
    }
    
    pub async fn write_text(&self, text: String) -> Result<(), String> {
        if !*self.permission_granted.lock().unwrap() {
            return Err("Clipboard permission denied".to_string());
        }
        
        let item = ClipboardItem::new(vec![
            ("text/plain".to_string(), ClipboardItemType::Text(text)),
        ]);
        
        *self.clipboard_data.lock().unwrap() = vec![item];
        Ok(())
    }
    
    pub fn request_permission(&mut self) -> bool {
        *self.permission_granted.lock().unwrap() = true;
        true
    }
}

impl Default for ClipboardAPI {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy ClipboardEvent for compatibility
pub struct ClipboardEvent {
    pub clipboard_data: Option<DataTransfer>,
}

pub struct DataTransfer {
    data: std::collections::HashMap<String, String>,
    files: Vec<File>,
}

impl DataTransfer {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            files: Vec::new(),
        }
    }
    
    pub fn get_data(&self, format: &str) -> Option<&String> {
        self.data.get(format)
    }
    
    pub fn set_data(&mut self, format: String, data: String) {
        self.data.insert(format, data);
    }
    
    pub fn clear_data(&mut self, format: Option<&str>) {
        if let Some(fmt) = format {
            self.data.remove(fmt);
        } else {
            self.data.clear();
        }
    }
    
    pub fn types(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
    
    pub fn files(&self) -> &[File] {
        &self.files
    }
}

impl Default for DataTransfer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct File {
    pub name: String,
    pub size: usize,
    pub mime_type: String,
    pub last_modified: u64,
    pub data: Vec<u8>,
}

impl File {
    pub fn new(name: String, data: Vec<u8>, mime_type: String) -> Self {
        Self {
            size: data.len(),
            name,
            mime_type,
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            data,
        }
    }
    
    pub async fn text(&self) -> Result<String, String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| format!("UTF-8 decode error: {}", e))
    }
    
    pub async fn array_buffer(&self) -> Vec<u8> {
        self.data.clone()
    }
    
    pub fn slice(&self, start: usize, end: usize, content_type: Option<String>) -> Self {
        let data = self.data[start..end].to_vec();
        Self {
            name: self.name.clone(),
            size: data.len(),
            mime_type: content_type.unwrap_or_else(|| self.mime_type.clone()),
            last_modified: self.last_modified,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_clipboard_write_read() {
        let clipboard = ClipboardAPI::new();
        
        clipboard.write_text("Hello, Clipboard!".to_string()).await.unwrap();
        let text = clipboard.read_text().await.unwrap();
        
        assert_eq!(text, "Hello, Clipboard!");
    }
    
    #[tokio::test]
    async fn test_clipboard_items() {
        let clipboard = ClipboardAPI::new();
        
        let item = ClipboardItem::new(vec![
            ("text/plain".to_string(), ClipboardItemType::Text("Plain text".to_string())),
            ("text/html".to_string(), ClipboardItemType::Html("<b>HTML</b>".to_string())),
        ]);
        
        clipboard.write(vec![item]).await.unwrap();
        let items = clipboard.read().await.unwrap();
        
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].types().len(), 2);
    }
    
    #[test]
    fn test_data_transfer() {
        let mut dt = DataTransfer::new();
        dt.set_data("text/plain".to_string(), "Test data".to_string());
        
        assert_eq!(dt.get_data("text/plain"), Some(&"Test data".to_string()));
        assert_eq!(dt.types().len(), 1);
    }
}
