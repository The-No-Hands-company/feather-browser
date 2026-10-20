// Production-grade Geolocation API Implementation
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub type PositionCallback = Arc<dyn Fn(Position) + Send + Sync>;
pub type ErrorCallback = Arc<dyn Fn(PositionError) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64, accuracy: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: None,
            accuracy,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub coords: Coordinates,
    pub timestamp: u64,
}

impl Position {
    pub fn new(coords: Coordinates) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self { coords, timestamp }
    }
}

#[derive(Debug, Clone)]
pub enum PositionErrorCode {
    PermissionDenied = 1,
    PositionUnavailable = 2,
    Timeout = 3,
}

#[derive(Debug, Clone)]
pub struct PositionError {
    pub code: PositionErrorCode,
    pub message: String,
}

impl PositionError {
    pub fn permission_denied() -> Self {
        Self {
            code: PositionErrorCode::PermissionDenied,
            message: "User denied geolocation request".to_string(),
        }
    }
    
    pub fn position_unavailable() -> Self {
        Self {
            code: PositionErrorCode::PositionUnavailable,
            message: "Position information is unavailable".to_string(),
        }
    }
    
    pub fn timeout() -> Self {
        Self {
            code: PositionErrorCode::Timeout,
            message: "Position request timed out".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct PositionOptions {
    pub enable_high_accuracy: bool,
    pub timeout: Option<u32>, // milliseconds
    pub maximum_age: Option<u32>, // milliseconds
}

impl Default for PositionOptions {
    fn default() -> Self {
        Self {
            enable_high_accuracy: false,
            timeout: None,
            maximum_age: Some(0),
        }
    }
}

pub struct GeolocationAPI {
    current_position: Arc<Mutex<Option<Position>>>,
    permission_granted: Arc<Mutex<bool>>,
    watchers: Arc<Mutex<std::collections::HashMap<i32, (PositionCallback, Option<ErrorCallback>, PositionOptions)>>>,
    next_watcher_id: Arc<Mutex<i32>>,
}

impl GeolocationAPI {
    pub fn new() -> Self {
        Self {
            current_position: Arc::new(Mutex::new(None)),
            permission_granted: Arc::new(Mutex::new(false)),
            watchers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            next_watcher_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub fn get_current_position<F, E>(
        &self,
        success_callback: F,
        error_callback: Option<E>,
        options: Option<PositionOptions>,
    ) where
        F: Fn(Position) + Send + Sync + 'static,
        E: Fn(PositionError) + Send + Sync + 'static,
    {
        let options = options.unwrap_or_default();
        
        // Check permission
        if !*self.permission_granted.lock().unwrap() {
            if let Some(error_cb) = error_callback {
                error_cb(PositionError::permission_denied());
            }
            return;
        }
        
        // Check cached position
        let current = self.current_position.lock().unwrap();
        if let Some(pos) = current.as_ref() {
            if let Some(max_age) = options.maximum_age {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                
                if now - pos.timestamp <= max_age as u64 {
                    success_callback(pos.clone());
                    return;
                }
            }
        }
        drop(current);
        
        // Simulate getting position (in production, would use actual GPS/WiFi/IP location)
        let position = self.get_simulated_position(options.enable_high_accuracy);
        
        *self.current_position.lock().unwrap() = Some(position.clone());
        success_callback(position);
    }
    
    pub fn watch_position<F, E>(
        &self,
        success_callback: F,
        error_callback: Option<E>,
        options: Option<PositionOptions>,
    ) -> i32
    where
        F: Fn(Position) + Send + Sync + 'static,
        E: Fn(PositionError) + Send + Sync + 'static,
    {
        let options = options.unwrap_or_default();
        
        // Check permission
        if !*self.permission_granted.lock().unwrap() {
            if let Some(error_cb) = error_callback {
                error_cb(PositionError::permission_denied());
            }
            return -1;
        }
        
        let id = {
            let mut next_id = self.next_watcher_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let error_cb = error_callback.map(|cb| Arc::new(cb) as ErrorCallback);
        
        self.watchers.lock().unwrap().insert(
            id,
            (Arc::new(success_callback), error_cb, options),
        );
        
        // Start watching (in production, would set up periodic updates)
        id
    }
    
    pub fn clear_watch(&self, watch_id: i32) {
        self.watchers.lock().unwrap().remove(&watch_id);
    }
    
    pub fn grant_permission(&self) {
        *self.permission_granted.lock().unwrap() = true;
    }
    
    pub fn revoke_permission(&self) {
        *self.permission_granted.lock().unwrap() = false;
    }
    
    pub fn update_position(&self, position: Position) {
        *self.current_position.lock().unwrap() = Some(position.clone());
        
        // Notify all watchers
        let watchers = self.watchers.lock().unwrap();
        for (_, (success_cb, _, _)) in watchers.iter() {
            success_cb(position.clone());
        }
    }
    
    fn get_simulated_position(&self, high_accuracy: bool) -> Position {
        // Simulate a position (San Francisco coordinates)
        let accuracy = if high_accuracy { 10.0 } else { 100.0 };
        
        let coords = Coordinates {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: Some(15.0),
            accuracy,
            altitude_accuracy: Some(10.0),
            heading: Some(45.0),
            speed: Some(0.0),
        };
        
        Position::new(coords)
    }
}

impl Default for GeolocationAPI {
    fn default() -> Self {
        Self::new()
    }
}

// Geolocation Permissions
pub struct GeolocationPermission {
    state: Arc<Mutex<PermissionState>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

impl GeolocationPermission {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PermissionState::Prompt)),
        }
    }
    
    pub fn query(&self) -> PermissionState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn request(&self) -> PermissionState {
        // In production, would show browser permission dialog
        *self.state.lock().unwrap() = PermissionState::Granted;
        PermissionState::Granted
    }
    
    pub fn revoke(&self) {
        *self.state.lock().unwrap() = PermissionState::Denied;
    }
}

impl Default for GeolocationPermission {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_get_current_position() {
        let geo = GeolocationAPI::new();
        geo.grant_permission();
        
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        geo.get_current_position(
            move |pos| {
                assert!(pos.coords.latitude != 0.0);
                assert!(pos.coords.longitude != 0.0);
                called_clone.store(true, Ordering::SeqCst);
            },
            None::<fn(PositionError)>,
            None,
        );
        
        assert!(called.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_permission_denied() {
        let geo = GeolocationAPI::new();
        
        let error_called = Arc::new(AtomicBool::new(false));
        let error_called_clone = error_called.clone();
        
        geo.get_current_position(
            |_| {},
            Some(move |err: PositionError| {
                assert!(matches!(err.code, PositionErrorCode::PermissionDenied));
                error_called_clone.store(true, Ordering::SeqCst);
            }),
            None,
        );
        
        assert!(error_called.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_watch_position() {
        let geo = GeolocationAPI::new();
        geo.grant_permission();
        
        let watch_id = geo.watch_position(
            |_| {},
            None::<fn(PositionError)>,
            None,
        );
        
        assert!(watch_id > 0);
        
        geo.clear_watch(watch_id);
        assert!(geo.watchers.lock().unwrap().is_empty());
    }
}
