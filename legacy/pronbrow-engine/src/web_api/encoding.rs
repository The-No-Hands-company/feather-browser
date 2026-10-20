// TextEncoder and TextDecoder APIs
use std::str;

#[derive(Debug, Clone)]
pub struct TextEncoder {
    encoding: String,
}

impl TextEncoder {
    pub fn new() -> Self {
        Self {
            encoding: "utf-8".to_string(),
        }
    }
    
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
    
    pub fn encode(&self, input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }
    
    pub fn encode_into(&self, source: &str, destination: &mut [u8]) -> EncodeIntoResult {
        let bytes = source.as_bytes();
        let written = bytes.len().min(destination.len());
        destination[..written].copy_from_slice(&bytes[..written]);
        
        // Count how many characters were read
        let read = if written < bytes.len() {
            // Find the last complete UTF-8 character boundary
            let mut read = written;
            while read > 0 && (bytes[read] & 0xC0) == 0x80 {
                read -= 1;
            }
            str::from_utf8(&bytes[..read])
                .map(|s| s.chars().count())
                .unwrap_or(0)
        } else {
            source.chars().count()
        };
        
        EncodeIntoResult { read, written }
    }
}

impl Default for TextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EncodeIntoResult {
    pub read: usize,
    pub written: usize,
}

#[derive(Debug, Clone)]
pub struct TextDecoder {
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
}

impl TextDecoder {
    pub fn new(label: Option<&str>, options: Option<TextDecoderOptions>) -> Result<Self, String> {
        let encoding = Self::normalize_encoding(label.unwrap_or("utf-8"))?;
        let opts = options.unwrap_or_default();
        
        Ok(Self {
            encoding,
            fatal: opts.fatal,
            ignore_bom: opts.ignore_bom,
        })
    }
    
    fn normalize_encoding(label: &str) -> Result<String, String> {
        let normalized = label.to_lowercase().replace(&['-', '_'][..], "");
        match normalized.as_str() {
            "unicode" | "unicode11utf8" | "utf8" => Ok("utf-8".to_string()),
            "utf16le" | "utf16" => Ok("utf-16le".to_string()),
            "utf16be" => Ok("utf-16be".to_string()),
            "iso88591" | "latin1" => Ok("iso-8859-1".to_string()),
            _ => Err(format!("Unsupported encoding: {}", label)),
        }
    }
    
    pub fn encoding(&self) -> &str {
        &self.encoding
    }
    
    pub fn fatal(&self) -> bool {
        self.fatal
    }
    
    pub fn ignore_bom(&self) -> bool {
        self.ignore_bom
    }
    
    pub fn decode(&self, input: Option<&[u8]>, options: Option<DecodeOptions>) -> Result<String, String> {
        let bytes = input.unwrap_or(&[]);
        let opts = options.unwrap_or_default();
        
        if bytes.is_empty() && !opts.stream {
            return Ok(String::new());
        }
        
        let mut data = bytes;
        
        // Handle BOM
        if !self.ignore_bom && self.encoding == "utf-8" {
            if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
                data = &data[3..];
            }
        }
        
        match self.encoding.as_str() {
            "utf-8" => {
                if self.fatal {
                    str::from_utf8(data)
                        .map(|s| s.to_string())
                        .map_err(|e| format!("UTF-8 decode error: {}", e))
                } else {
                    Ok(String::from_utf8_lossy(data).to_string())
                }
            }
            "utf-16le" => Self::decode_utf16le(data, self.fatal),
            "utf-16be" => Self::decode_utf16be(data, self.fatal),
            "iso-8859-1" => Ok(data.iter().map(|&b| b as char).collect()),
            _ => Err(format!("Unsupported encoding: {}", self.encoding)),
        }
    }
    
    fn decode_utf16le(data: &[u8], fatal: bool) -> Result<String, String> {
        if data.len() % 2 != 0 {
            if fatal {
                return Err("Invalid UTF-16LE: odd number of bytes".to_string());
            }
        }
        
        let mut units: Vec<u16> = Vec::new();
        for chunk in data.chunks_exact(2) {
            units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        
        String::from_utf16(&units)
            .map_err(|e| if fatal {
                format!("UTF-16LE decode error: {}", e)
            } else {
                String::new()
            })
    }
    
    fn decode_utf16be(data: &[u8], fatal: bool) -> Result<String, String> {
        if data.len() % 2 != 0 {
            if fatal {
                return Err("Invalid UTF-16BE: odd number of bytes".to_string());
            }
        }
        
        let mut units: Vec<u16> = Vec::new();
        for chunk in data.chunks_exact(2) {
            units.push(u16::from_be_bytes([chunk[0], chunk[1]]));
        }
        
        String::from_utf16(&units)
            .map_err(|e| if fatal {
                format!("UTF-16BE decode error: {}", e)
            } else {
                String::new()
            })
    }
}

impl Default for TextDecoder {
    fn default() -> Self {
        Self::new(None, None).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextDecoderOptions {
    pub fatal: bool,
    pub ignore_bom: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DecodeOptions {
    pub stream: bool,
}

// Base64 utilities
pub struct Base64 {
}

impl Base64 {
    pub fn encode(data: &[u8]) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
    }
    
    pub fn decode(data: &str) -> Result<Vec<u8>, String> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)
            .map_err(|e| e.to_string())
    }
}

// atob and btoa functions (legacy base64)
pub fn atob(data: &str) -> Result<String, String> {
    let bytes = Base64::decode(data)?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

pub fn btoa(data: &str) -> String {
    Base64::encode(data.as_bytes())
}
