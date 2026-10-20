// Service Workers - Offline capabilities and background processing
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ServiceWorker {
    pub script_url: String,
    pub scope: String,
    pub state: ServiceWorkerState,
    pub registration_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceWorkerState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

pub struct ServiceWorkerRegistration {
    pub active: Option<ServiceWorker>,
    pub installing: Option<ServiceWorker>,
    pub waiting: Option<ServiceWorker>,
    pub scope: String,
    pub update_via_cache: UpdateViaCache,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateViaCache {
    Imports,
    All,
    None,
}

pub struct ServiceWorkerContainer {
    registrations: HashMap<String, ServiceWorkerRegistration>,
    controller: Option<ServiceWorker>,
}

impl ServiceWorkerContainer {
    pub fn new() -> Self {
        ServiceWorkerContainer {
            registrations: HashMap::new(),
            controller: None,
        }
    }

    pub fn register(&mut self, script_url: &str, scope: &str) -> Result<String, String> {
        let worker = ServiceWorker {
            script_url: script_url.to_string(),
            scope: scope.to_string(),
            state: ServiceWorkerState::Installing,
            registration_id: format!("sw-{}", self.registrations.len()),
        };

        let registration = ServiceWorkerRegistration {
            active: None,
            installing: Some(worker.clone()),
            waiting: None,
            scope: scope.to_string(),
            update_via_cache: UpdateViaCache::Imports,
        };

        let id = worker.registration_id.clone();
        self.registrations.insert(scope.to_string(), registration);
        Ok(id)
    }

    pub fn unregister(&mut self, scope: &str) -> bool {
        self.registrations.remove(scope).is_some()
    }

    pub fn get_registration(&self, scope: &str) -> Option<&ServiceWorkerRegistration> {
        self.registrations.get(scope)
    }
}

// Cache API for Service Workers
pub struct CacheStorage {
    caches: HashMap<String, Cache>,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub name: String,
    entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub request: Request,
    pub response: Response,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl CacheStorage {
    pub fn new() -> Self {
        CacheStorage {
            caches: HashMap::new(),
        }
    }

    pub fn open(&mut self, name: &str) -> Result<&mut Cache, String> {
        if !self.caches.contains_key(name) {
            self.caches.insert(name.to_string(), Cache {
                name: name.to_string(),
                entries: HashMap::new(),
            });
        }
        Ok(self.caches.get_mut(name).unwrap())
    }

    pub fn delete(&mut self, name: &str) -> bool {
        self.caches.remove(name).is_some()
    }

    pub fn has(&self, name: &str) -> bool {
        self.caches.contains_key(name)
    }

    pub fn keys(&self) -> Vec<String> {
        self.caches.keys().cloned().collect()
    }
}

impl Cache {
    pub fn put(&mut self, request: Request, response: Response) {
        let entry = CacheEntry {
            request: request.clone(),
            response,
            timestamp: 0, // Would use SystemTime in real impl
        };
        self.entries.insert(request.url.clone(), entry);
    }

    pub fn get(&self, url: &str) -> Option<&Response> {
        self.entries.get(url).map(|e| &e.response)
    }

    pub fn delete(&mut self, url: &str) -> bool {
        self.entries.remove(url).is_some()
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

// Background Sync API
pub struct SyncManager {
    registrations: Vec<SyncRegistration>,
}

#[derive(Debug, Clone)]
pub struct SyncRegistration {
    pub tag: String,
    pub last_attempt: Option<u64>,
}

impl SyncManager {
    pub fn new() -> Self {
        SyncManager {
            registrations: Vec::new(),
        }
    }

    pub fn register(&mut self, tag: &str) -> Result<(), String> {
        let registration = SyncRegistration {
            tag: tag.to_string(),
            last_attempt: None,
        };
        self.registrations.push(registration);
        Ok(())
    }

    pub fn get_tags(&self) -> Vec<String> {
        self.registrations.iter().map(|r| r.tag.clone()).collect()
    }
}
