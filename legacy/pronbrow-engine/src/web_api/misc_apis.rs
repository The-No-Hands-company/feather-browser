// Clipboard API for copy/paste operations
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Clipboard {
    text: Arc<Mutex<String>>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            text: Arc::new(Mutex::new(String::new())),
        }
    }
    
    pub async fn read_text(&self) -> Result<String, String> {
        Ok(self.text.lock().unwrap().clone())
    }
    
    pub async fn write_text(&self, text: &str) -> Result<(), String> {
        *self.text.lock().unwrap() = text.to_string();
        Ok(())
    }
    
    pub async fn read(&self) -> Result<Vec<ClipboardItem>, String> {
        let text = self.text.lock().unwrap().clone();
        if text.is_empty() {
            Ok(Vec::new())
        } else {
            let item = ClipboardItem {
                types: vec!["text/plain".to_string()],
                data: vec![("text/plain".to_string(), text.as_bytes().to_vec())].into_iter().collect(),
            };
            Ok(vec![item])
        }
    }
    
    pub async fn write(&self, items: Vec<ClipboardItem>) -> Result<(), String> {
        if let Some(item) = items.first() {
            if let Some(text_data) = item.data.get("text/plain") {
                let text = String::from_utf8_lossy(text_data).to_string();
                *self.text.lock().unwrap() = text;
            }
        }
        Ok(())
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub types: Vec<String>,
    pub data: std::collections::HashMap<String, Vec<u8>>,
}

impl ClipboardItem {
    pub fn new(data: std::collections::HashMap<String, Vec<u8>>) -> Self {
        let types = data.keys().cloned().collect();
        Self { types, data }
    }
    
    pub async fn get_type(&self, mime_type: &str) -> Result<Vec<u8>, String> {
        self.data.get(mime_type)
            .cloned()
            .ok_or_else(|| format!("Type {} not found", mime_type))
    }
}

// Permissions API
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

#[derive(Debug, Clone)]
pub struct PermissionStatus {
    pub state: PermissionState,
    pub name: String,
}

pub struct Permissions {
    permissions: Arc<Mutex<std::collections::HashMap<String, PermissionState>>>,
}

impl Permissions {
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    pub async fn query(&self, descriptor: PermissionDescriptor) -> Result<PermissionStatus, String> {
        let perms = self.permissions.lock().unwrap();
        let state = perms.get(&descriptor.name)
            .cloned()
            .unwrap_or(PermissionState::Prompt);
        
        Ok(PermissionStatus {
            state,
            name: descriptor.name,
        })
    }
    
    pub async fn request(&self, descriptor: PermissionDescriptor) -> Result<PermissionStatus, String> {
        // In a real browser, this would show a UI prompt
        // For now, auto-grant certain permissions
        let state = match descriptor.name.as_str() {
            "clipboard-read" | "clipboard-write" => PermissionState::Granted,
            _ => PermissionState::Prompt,
        };
        
        self.permissions.lock().unwrap().insert(descriptor.name.clone(), state.clone());
        
        Ok(PermissionStatus {
            state,
            name: descriptor.name,
        })
    }
    
    pub async fn revoke(&self, descriptor: PermissionDescriptor) -> Result<PermissionStatus, String> {
        self.permissions.lock().unwrap().insert(descriptor.name.clone(), PermissionState::Denied);
        
        Ok(PermissionStatus {
            state: PermissionState::Denied,
            name: descriptor.name,
        })
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PermissionDescriptor {
    pub name: String,
}

// Notification API
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tag: Option<String>,
    pub data: Option<String>,
    pub require_interaction: bool,
    pub silent: bool,
}

impl Notification {
    pub fn new(title: &str, options: Option<NotificationOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            title: title.to_string(),
            body: opts.body,
            icon: opts.icon,
            badge: opts.badge,
            tag: opts.tag,
            data: opts.data,
            require_interaction: opts.require_interaction,
            silent: opts.silent,
        }
    }
    
    pub fn permission() -> PermissionState {
        // Would check actual permission status
        PermissionState::Prompt
    }
    
    pub async fn request_permission() -> PermissionState {
        // Would show UI prompt
        PermissionState::Granted
    }
    
    pub fn close(&self) {
        // Close the notification
    }
}

#[derive(Debug, Clone, Default)]
pub struct NotificationOptions {
    pub body: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tag: Option<String>,
    pub data: Option<String>,
    pub require_interaction: bool,
    pub silent: bool,
}

// Battery Status API
#[derive(Debug, Clone)]
pub struct BatteryManager {
    pub charging: bool,
    pub charging_time: f64,
    pub discharging_time: f64,
    pub level: f64,
}

impl BatteryManager {
    pub fn new() -> Self {
        Self {
            charging: false,
            charging_time: f64::INFINITY,
            discharging_time: f64::INFINITY,
            level: 1.0,
        }
    }
}

impl Default for BatteryManager {
    fn default() -> Self {
        Self::new()
    }
}

// Screen Orientation API
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrientationType {
    PortraitPrimary,
    PortraitSecondary,
    LandscapePrimary,
    LandscapeSecondary,
}

#[derive(Debug, Clone)]
pub struct ScreenOrientation {
    pub orientation_type: OrientationType,
    pub angle: u16,
}

impl ScreenOrientation {
    pub fn new() -> Self {
        Self {
            orientation_type: OrientationType::LandscapePrimary,
            angle: 0,
        }
    }
    
    pub async fn lock(&mut self, orientation: OrientationType) -> Result<(), String> {
        self.orientation_type = orientation;
        self.angle = match orientation {
            OrientationType::PortraitPrimary => 0,
            OrientationType::LandscapePrimary => 90,
            OrientationType::PortraitSecondary => 180,
            OrientationType::LandscapeSecondary => 270,
        };
        Ok(())
    }
    
    pub fn unlock(&mut self) {
        // Unlock orientation
    }
}

impl Default for ScreenOrientation {
    fn default() -> Self {
        Self::new()
    }
}
