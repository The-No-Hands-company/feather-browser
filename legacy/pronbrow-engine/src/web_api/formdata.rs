// FormData API for form handling
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FormData {
    fields: Vec<FormDataEntry>,
}

#[derive(Debug, Clone)]
pub struct FormDataEntry {
    pub name: String,
    pub value: FormDataValue,
}

#[derive(Debug, Clone)]
pub enum FormDataValue {
    String(String),
    File(FormDataFile),
    Blob(Vec<u8>, String), // data, mime_type
}

#[derive(Debug, Clone)]
pub struct FormDataFile {
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
        }
    }
    
    pub fn append(&mut self, name: &str, value: FormDataValue) {
        self.fields.push(FormDataEntry {
            name: name.to_string(),
            value,
        });
    }
    
    pub fn append_string(&mut self, name: &str, value: &str) {
        self.append(name, FormDataValue::String(value.to_string()));
    }
    
    pub fn append_file(&mut self, name: &str, file: FormDataFile) {
        self.append(name, FormDataValue::File(file));
    }
    
    pub fn delete(&mut self, name: &str) {
        self.fields.retain(|entry| entry.name != name);
    }
    
    pub fn get(&self, name: &str) -> Option<&FormDataValue> {
        self.fields.iter()
            .find(|entry| entry.name == name)
            .map(|entry| &entry.value)
    }
    
    pub fn get_all(&self, name: &str) -> Vec<&FormDataValue> {
        self.fields.iter()
            .filter(|entry| entry.name == name)
            .map(|entry| &entry.value)
            .collect()
    }
    
    pub fn has(&self, name: &str) -> bool {
        self.fields.iter().any(|entry| entry.name == name)
    }
    
    pub fn set(&mut self, name: &str, value: FormDataValue) {
        self.delete(name);
        self.append(name, value);
    }
    
    pub fn entries(&self) -> Vec<(&String, &FormDataValue)> {
        self.fields.iter()
            .map(|entry| (&entry.name, &entry.value))
            .collect()
    }
    
    pub fn keys(&self) -> Vec<&String> {
        self.fields.iter()
            .map(|entry| &entry.name)
            .collect()
    }
    
    pub fn values(&self) -> Vec<&FormDataValue> {
        self.fields.iter()
            .map(|entry| &entry.value)
            .collect()
    }
    
    pub fn to_multipart(&self, boundary: &str) -> Vec<u8> {
        let mut data = Vec::new();
        
        for entry in &self.fields {
            data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            
            match &entry.value {
                FormDataValue::String(s) => {
                    data.extend_from_slice(
                        format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", entry.name).as_bytes()
                    );
                    data.extend_from_slice(s.as_bytes());
                }
                FormDataValue::File(file) => {
                    data.extend_from_slice(
                        format!(
                            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                            entry.name, file.name
                        ).as_bytes()
                    );
                    data.extend_from_slice(
                        format!("Content-Type: {}\r\n\r\n", file.mime_type).as_bytes()
                    );
                    data.extend_from_slice(&file.data);
                }
                FormDataValue::Blob(blob_data, mime_type) => {
                    data.extend_from_slice(
                        format!("Content-Disposition: form-data; name=\"{}\"\r\n", entry.name).as_bytes()
                    );
                    data.extend_from_slice(
                        format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes()
                    );
                    data.extend_from_slice(blob_data);
                }
            }
            data.extend_from_slice(b"\r\n");
        }
        
        data.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        data
    }
    
    pub fn to_urlencoded(&self) -> String {
        let pairs: Vec<String> = self.fields.iter()
            .filter_map(|entry| {
                if let FormDataValue::String(s) = &entry.value {
                    Some(format!(
                        "{}={}",
                        urlencoding::encode(&entry.name),
                        urlencoding::encode(s)
                    ))
                } else {
                    None
                }
            })
            .collect();
        
        pairs.join("&")
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

// Headers API
#[derive(Debug, Clone)]
pub struct Headers {
    headers: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }
    
    pub fn append(&mut self, name: &str, value: &str) {
        let key = name.to_lowercase();
        self.headers.entry(key)
            .or_insert_with(Vec::new)
            .push(value.to_string());
    }
    
    pub fn delete(&mut self, name: &str) {
        let key = name.to_lowercase();
        self.headers.remove(&key);
    }
    
    pub fn get(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        self.headers.get(&key)
            .and_then(|values| values.first())
            .cloned()
    }
    
    pub fn get_all(&self, name: &str) -> Vec<String> {
        let key = name.to_lowercase();
        self.headers.get(&key).cloned().unwrap_or_default()
    }
    
    pub fn has(&self, name: &str) -> bool {
        let key = name.to_lowercase();
        self.headers.contains_key(&key)
    }
    
    pub fn set(&mut self, name: &str, value: &str) {
        let key = name.to_lowercase();
        self.headers.insert(key, vec![value.to_string()]);
    }
    
    pub fn entries(&self) -> Vec<(String, String)> {
        let mut entries = Vec::new();
        for (name, values) in &self.headers {
            for value in values {
                entries.push((name.clone(), value.clone()));
            }
        }
        entries
    }
    
    pub fn keys(&self) -> Vec<String> {
        self.headers.keys().cloned().collect()
    }
    
    pub fn values(&self) -> Vec<String> {
        let mut values = Vec::new();
        for vals in self.headers.values() {
            values.extend(vals.clone());
        }
        values
    }
    
    pub fn to_map(&self) -> HashMap<String, String> {
        self.headers.iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
            .collect()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(map: HashMap<String, String>) -> Self {
        let mut headers = Headers::new();
        for (k, v) in map {
            headers.set(&k, &v);
        }
        headers
    }
}
