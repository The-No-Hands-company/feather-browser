// Blob and File APIs
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Blob {
    pub data: Arc<Vec<u8>>,
    pub mime_type: String,
    pub size: usize,
}

impl Blob {
    pub fn new(data: Vec<u8>, mime_type: Option<&str>) -> Self {
        let size = data.len();
        Self {
            data: Arc::new(data),
            mime_type: mime_type.unwrap_or("").to_string(),
            size,
        }
    }
    
    pub fn from_parts(parts: Vec<BlobPart>, options: Option<BlobPropertyBag>) -> Self {
        let mut data = Vec::new();
        for part in parts {
            match part {
                BlobPart::String(s) => data.extend_from_slice(s.as_bytes()),
                BlobPart::Blob(b) => data.extend_from_slice(&b.data),
                BlobPart::ArrayBuffer(buf) => data.extend_from_slice(&buf),
            }
        }
        
        let mime_type = options.and_then(|o| o.mime_type).unwrap_or_default();
        Self::new(data, Some(&mime_type))
    }
    
    pub fn slice(&self, start: Option<i64>, end: Option<i64>, content_type: Option<&str>) -> Self {
        let len = self.size as i64;
        let start = start.unwrap_or(0).max(0).min(len) as usize;
        let end = end.unwrap_or(len).max(0).min(len) as usize;
        
        let data = self.data[start..end].to_vec();
        Self::new(data, content_type.or(Some(&self.mime_type)))
    }
    
    pub async fn text(&self) -> Result<String, String> {
        String::from_utf8((*self.data).clone())
            .map_err(|e| e.to_string())
    }
    
    pub async fn array_buffer(&self) -> Vec<u8> {
        (*self.data).clone()
    }
    
    pub fn stream(&self) -> ReadableStream {
        ReadableStream::from_bytes((*self.data).clone())
    }
}

#[derive(Debug, Clone)]
pub enum BlobPart {
    String(String),
    Blob(Blob),
    ArrayBuffer(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct BlobPropertyBag {
    pub mime_type: Option<String>,
    pub endings: Option<String>, // "transparent" or "native"
}

#[derive(Debug, Clone)]
pub struct File {
    pub blob: Blob,
    pub name: String,
    pub last_modified: u64,
}

impl File {
    pub fn new(data: Vec<u8>, name: String, options: Option<FilePropertyBag>) -> Self {
        let last_modified = options.as_ref()
            .and_then(|o| o.last_modified)
            .unwrap_or_else(|| std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64);
        
        let mime_type = options.and_then(|o| o.mime_type);
        
        Self {
            blob: Blob::new(data, mime_type.as_deref()),
            name,
            last_modified,
        }
    }
    
    pub fn size(&self) -> usize {
        self.blob.size
    }
    
    pub fn mime_type(&self) -> &str {
        &self.blob.mime_type
    }
    
    pub async fn text(&self) -> Result<String, String> {
        self.blob.text().await
    }
    
    pub async fn array_buffer(&self) -> Vec<u8> {
        self.blob.array_buffer().await
    }
}

#[derive(Debug, Clone)]
pub struct FilePropertyBag {
    pub mime_type: Option<String>,
    pub last_modified: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct FileReader {
    ready_state: FileReaderState,
    result: Option<FileReaderResult>,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileReaderState {
    Empty = 0,
    Loading = 1,
    Done = 2,
}

#[derive(Debug, Clone)]
pub enum FileReaderResult {
    Text(String),
    DataURL(String),
    ArrayBuffer(Vec<u8>),
}

impl FileReader {
    pub fn new() -> Self {
        Self {
            ready_state: FileReaderState::Empty,
            result: None,
            error: None,
        }
    }
    
    pub async fn read_as_text(&mut self, blob: &Blob) -> Result<String, String> {
        self.ready_state = FileReaderState::Loading;
        match blob.text().await {
            Ok(text) => {
                self.ready_state = FileReaderState::Done;
                self.result = Some(FileReaderResult::Text(text.clone()));
                Ok(text)
            }
            Err(e) => {
                self.error = Some(e.clone());
                Err(e)
            }
        }
    }
    
    pub async fn read_as_data_url(&mut self, blob: &Blob) -> Result<String, String> {
        self.ready_state = FileReaderState::Loading;
        let data = blob.array_buffer().await;
        let base64 = base64::encode(&data);
        let data_url = format!("data:{};base64,{}", blob.mime_type, base64);
        
        self.ready_state = FileReaderState::Done;
        self.result = Some(FileReaderResult::DataURL(data_url.clone()));
        Ok(data_url)
    }
    
    pub async fn read_as_array_buffer(&mut self, blob: &Blob) -> Result<Vec<u8>, String> {
        self.ready_state = FileReaderState::Loading;
        let data = blob.array_buffer().await;
        
        self.ready_state = FileReaderState::Done;
        self.result = Some(FileReaderResult::ArrayBuffer(data.clone()));
        Ok(data)
    }
    
    pub fn abort(&mut self) {
        self.ready_state = FileReaderState::Done;
        self.error = Some("AbortError".to_string());
    }
}

impl Default for FileReader {
    fn default() -> Self {
        Self::new()
    }
}

// Minimal ReadableStream implementation
#[derive(Debug, Clone)]
pub struct ReadableStream {
    data: Arc<Vec<u8>>,
    position: usize,
}

impl ReadableStream {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
            position: 0,
        }
    }
    
    pub fn get_reader(&mut self) -> ReadableStreamReader {
        ReadableStreamReader {
            stream: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadableStreamReader {
    stream: ReadableStream,
}

impl ReadableStreamReader {
    pub async fn read(&mut self) -> ReadableStreamReadResult {
        if self.stream.position >= self.stream.data.len() {
            return ReadableStreamReadResult {
                done: true,
                value: None,
            };
        }
        
        let chunk_size = 8192.min(self.stream.data.len() - self.stream.position);
        let end = self.stream.position + chunk_size;
        let chunk = self.stream.data[self.stream.position..end].to_vec();
        self.stream.position = end;
        
        ReadableStreamReadResult {
            done: false,
            value: Some(chunk),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadableStreamReadResult {
    pub done: bool,
    pub value: Option<Vec<u8>>,
}

// Helper for base64 encoding
mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }
    
    pub fn decode(data: &str) -> Result<Vec<u8>, String> {
        general_purpose::STANDARD.decode(data)
            .map_err(|e| e.to_string())
    }
}
