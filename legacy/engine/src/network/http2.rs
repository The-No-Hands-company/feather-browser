// HTTP/2 Support with multiplexing and server push
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Http2Connection {
    pub streams: HashMap<u32, Http2Stream>,
    next_stream_id: u32,
    window_size: u32,
}

#[derive(Debug, Clone)]
pub struct Http2Stream {
    pub id: u32,
    pub state: StreamState,
    pub headers: Vec<(String, String)>,
    pub data: Vec<u8>,
    pub priority: StreamPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
    Idle,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
}

#[derive(Debug, Clone)]
pub struct StreamPriority {
    pub weight: u8,
    pub depends_on: Option<u32>,
    pub exclusive: bool,
}

#[derive(Debug, Clone)]
pub struct HpackEncoder {
    dynamic_table: Vec<(String, String)>,
    max_table_size: usize,
}

impl Http2Connection {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            next_stream_id: 1,
            window_size: 65535,
        }
    }

    pub fn create_stream(&mut self) -> u32 {
        let stream_id = self.next_stream_id;
        self.next_stream_id += 2;

        let stream = Http2Stream {
            id: stream_id,
            state: StreamState::Idle,
            headers: Vec::new(),
            data: Vec::new(),
            priority: StreamPriority {
                weight: 16,
                depends_on: None,
                exclusive: false,
            },
        };

        self.streams.insert(stream_id, stream);
        stream_id
    }

    pub fn send_headers(&mut self, stream_id: u32, headers: Vec<(String, String)>) {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.headers = headers;
            stream.state = StreamState::Open;
        }
    }

    pub fn send_data(&mut self, stream_id: u32, data: Vec<u8>) {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.data.extend(data);
        }
    }

    pub fn close_stream(&mut self, stream_id: u32) {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.state = StreamState::Closed;
        }
    }

    pub fn update_window(&mut self, increment: u32) {
        self.window_size = self.window_size.saturating_add(increment);
    }
}

impl HpackEncoder {
    pub fn new() -> Self {
        Self {
            dynamic_table: Vec::new(),
            max_table_size: 4096,
        }
    }

    pub fn encode(&mut self, headers: &[(String, String)]) -> Vec<u8> {
        let mut encoded = Vec::new();
        
        for (name, value) in headers {
            // Simplified HPACK encoding
            let name_bytes = name.as_bytes();
            let value_bytes = value.as_bytes();
            
            encoded.push(0x00); // Literal without indexing
            encoded.push(name_bytes.len() as u8);
            encoded.extend_from_slice(name_bytes);
            encoded.push(value_bytes.len() as u8);
            encoded.extend_from_slice(value_bytes);
        }
        
        encoded
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<(String, String)>, String> {
        let mut headers = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            if data[i] == 0x00 {
                i += 1;
                if i >= data.len() {
                    break;
                }
                
                let name_len = data[i] as usize;
                i += 1;
                
                if i + name_len > data.len() {
                    return Err("Invalid HPACK data".to_string());
                }
                
                let name = String::from_utf8_lossy(&data[i..i + name_len]).to_string();
                i += name_len;
                
                if i >= data.len() {
                    break;
                }
                
                let value_len = data[i] as usize;
                i += 1;
                
                if i + value_len > data.len() {
                    return Err("Invalid HPACK data".to_string());
                }
                
                let value = String::from_utf8_lossy(&data[i..i + value_len]).to_string();
                i += value_len;
                
                headers.push((name, value));
            } else {
                i += 1;
            }
        }
        
        Ok(headers)
    }

    pub fn add_to_table(&mut self, name: String, value: String) {
        self.dynamic_table.push((name, value));
        
        // Evict old entries if table is too large
        while self.table_size() > self.max_table_size {
            self.dynamic_table.remove(0);
        }
    }

    fn table_size(&self) -> usize {
        self.dynamic_table.iter()
            .map(|(n, v)| n.len() + v.len() + 32)
            .sum()
    }
}

impl Default for Http2Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HpackEncoder {
    fn default() -> Self {
        Self::new()
    }
}
