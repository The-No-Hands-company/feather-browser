// WebSocket support for real-time bidirectional communication
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct WebSocket {
    pub url: String,
    pub state: WebSocketState,
    pub protocol: Option<String>,
    send_queue: VecDeque<WebSocketMessage>,
    receive_queue: VecDeque<WebSocketMessage>,
    pub buffered_amount: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(u16, String),
}

#[derive(Debug, Clone)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: u8,
    pub mask: bool,
    pub payload: Vec<u8>,
}

impl WebSocket {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            state: WebSocketState::Connecting,
            protocol: None,
            send_queue: VecDeque::new(),
            receive_queue: VecDeque::new(),
            buffered_amount: 0,
        }
    }

    pub fn send(&mut self, message: WebSocketMessage) -> Result<(), String> {
        if self.state != WebSocketState::Open {
            return Err("WebSocket is not open".to_string());
        }

        self.buffered_amount += self.message_size(&message);
        self.send_queue.push_back(message);
        Ok(())
    }

    pub fn send_text(&mut self, text: &str) -> Result<(), String> {
        self.send(WebSocketMessage::Text(text.to_string()))
    }

    pub fn send_binary(&mut self, data: Vec<u8>) -> Result<(), String> {
        self.send(WebSocketMessage::Binary(data))
    }

    pub fn receive(&mut self) -> Option<WebSocketMessage> {
        self.receive_queue.pop_front()
    }

    pub fn close(&mut self, code: u16, reason: &str) {
        self.state = WebSocketState::Closing;
        let _ = self.send(WebSocketMessage::Close(code, reason.to_string()));
    }

    pub fn ping(&mut self, data: Vec<u8>) -> Result<(), String> {
        self.send(WebSocketMessage::Ping(data))
    }

    fn message_size(&self, message: &WebSocketMessage) -> usize {
        match message {
            WebSocketMessage::Text(s) => s.len(),
            WebSocketMessage::Binary(b) => b.len(),
            WebSocketMessage::Ping(p) => p.len(),
            WebSocketMessage::Pong(p) => p.len(),
            WebSocketMessage::Close(_, r) => r.len() + 2,
        }
    }

    pub fn encode_frame(&self, message: &WebSocketMessage) -> Vec<u8> {
        let mut frame = Vec::new();
        
        // FIN bit and opcode
        let (opcode, payload) = match message {
            WebSocketMessage::Text(s) => (1, s.as_bytes().to_vec()),
            WebSocketMessage::Binary(b) => (2, b.clone()),
            WebSocketMessage::Close(code, reason) => {
                let mut p = Vec::new();
                p.extend_from_slice(&code.to_be_bytes());
                p.extend_from_slice(reason.as_bytes());
                (8, p)
            },
            WebSocketMessage::Ping(p) => (9, p.clone()),
            WebSocketMessage::Pong(p) => (10, p.clone()),
        };
        
        frame.push(0x80 | opcode); // FIN=1
        
        // Payload length and masking
        let len = payload.len();
        if len < 126 {
            frame.push(0x80 | len as u8); // MASK=1
        } else if len < 65536 {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
        
        // Masking key (4 random bytes)
        let mask_key = [0x12, 0x34, 0x56, 0x78];
        frame.extend_from_slice(&mask_key);
        
        // Masked payload
        for (i, byte) in payload.iter().enumerate() {
            frame.push(byte ^ mask_key[i % 4]);
        }
        
        frame
    }

    pub fn decode_frame(&self, data: &[u8]) -> Result<WebSocketFrame, String> {
        if data.len() < 2 {
            return Err("Frame too short".to_string());
        }

        let fin = (data[0] & 0x80) != 0;
        let opcode = data[0] & 0x0F;
        let mask = (data[1] & 0x80) != 0;
        let mut payload_len = (data[1] & 0x7F) as usize;
        
        let mut offset = 2;
        
        if payload_len == 126 {
            if data.len() < 4 {
                return Err("Frame too short for extended length".to_string());
            }
            payload_len = u16::from_be_bytes([data[2], data[3]]) as usize;
            offset = 4;
        } else if payload_len == 127 {
            if data.len() < 10 {
                return Err("Frame too short for extended length".to_string());
            }
            payload_len = u64::from_be_bytes([
                data[2], data[3], data[4], data[5],
                data[6], data[7], data[8], data[9],
            ]) as usize;
            offset = 10;
        }
        
        let mask_key = if mask {
            if data.len() < offset + 4 {
                return Err("Frame too short for mask key".to_string());
            }
            let key = [data[offset], data[offset+1], data[offset+2], data[offset+3]];
            offset += 4;
            Some(key)
        } else {
            None
        };
        
        if data.len() < offset + payload_len {
            return Err("Frame too short for payload".to_string());
        }
        
        let mut payload = data[offset..offset+payload_len].to_vec();
        
        if let Some(key) = mask_key {
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= key[i % 4];
            }
        }
        
        Ok(WebSocketFrame {
            fin,
            opcode,
            mask,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket() {
        let mut ws = WebSocket::new("wss://example.com");
        ws.state = WebSocketState::Open;
        
        assert!(ws.send_text("Hello").is_ok());
        assert_eq!(ws.buffered_amount, 5);
    }

    #[test]
    fn test_frame_encoding() {
        let ws = WebSocket::new("wss://example.com");
        let msg = WebSocketMessage::Text("Test".to_string());
        let frame = ws.encode_frame(&msg);
        
        assert!(!frame.is_empty());
        assert_eq!(frame[0] & 0x0F, 1); // Text frame
    }
}
