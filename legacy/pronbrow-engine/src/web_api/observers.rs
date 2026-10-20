// Mutation Observer - Watch DOM changes
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct MutationObserver {
    callback: Arc<Mutex<Option<Box<dyn Fn(Vec<MutationRecord>) + Send + Sync>>>>,
    observations: Arc<Mutex<Vec<ObservationTarget>>>,
    records: Arc<Mutex<VecDeque<MutationRecord>>>,
}

impl std::fmt::Debug for MutationObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutationObserver").finish()
    }
}

impl Clone for MutationObserver {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
            observations: Arc::clone(&self.observations),
            records: Arc::clone(&self.records),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservationTarget {
    pub target_id: usize,
    pub options: MutationObserverInit,
}

#[derive(Debug, Clone)]
pub struct MutationObserverInit {
    pub child_list: bool,
    pub attributes: bool,
    pub character_data: bool,
    pub subtree: bool,
    pub attribute_old_value: bool,
    pub character_data_old_value: bool,
    pub attribute_filter: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MutationRecord {
    pub mutation_type: MutationType,
    pub target: usize,
    pub added_nodes: Vec<usize>,
    pub removed_nodes: Vec<usize>,
    pub previous_sibling: Option<usize>,
    pub next_sibling: Option<usize>,
    pub attribute_name: Option<String>,
    pub attribute_namespace: Option<String>,
    pub old_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MutationType {
    ChildList,
    Attributes,
    CharacterData,
}

impl MutationObserver {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
            observations: Arc::new(Mutex::new(Vec::new())),
            records: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn observe(&self, target: usize, options: MutationObserverInit) {
        let mut observations = self.observations.lock().unwrap();
        observations.push(ObservationTarget {
            target_id: target,
            options,
        });
    }

    pub fn disconnect(&self) {
        let mut observations = self.observations.lock().unwrap();
        observations.clear();
        
        let mut records = self.records.lock().unwrap();
        records.clear();
    }

    pub fn take_records(&self) -> Vec<MutationRecord> {
        let mut records = self.records.lock().unwrap();
        records.drain(..).collect()
    }

    pub fn record_mutation(&self, record: MutationRecord) {
        let mut records = self.records.lock().unwrap();
        records.push_back(record);
    }

    pub fn flush(&self) {
        let records = self.take_records();
        if !records.is_empty() {
            let callback = self.callback.lock().unwrap();
            if let Some(ref cb) = *callback {
                cb(records);
            }
        }
    }
}

// Intersection Observer - Watch element visibility
pub struct IntersectionObserver {
    callback: Arc<Mutex<Option<Box<dyn Fn(Vec<IntersectionObserverEntry>) + Send + Sync>>>>,
    targets: Arc<Mutex<Vec<usize>>>,
    options: IntersectionObserverInit,
}

impl std::fmt::Debug for IntersectionObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntersectionObserver")
            .field("options", &self.options)
            .finish()
    }
}

impl Clone for IntersectionObserver {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
            targets: Arc::clone(&self.targets),
            options: self.options.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntersectionObserverInit {
    pub root: Option<usize>,
    pub root_margin: String,
    pub threshold: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct IntersectionObserverEntry {
    pub target: usize,
    pub bounding_client_rect: DOMRect,
    pub intersection_rect: DOMRect,
    pub root_bounds: Option<DOMRect>,
    pub intersection_ratio: f64,
    pub is_intersecting: bool,
    pub time: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl IntersectionObserver {
    pub fn new(options: IntersectionObserverInit) -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
            targets: Arc::new(Mutex::new(Vec::new())),
            options,
        }
    }

    pub fn observe(&self, target: usize) {
        let mut targets = self.targets.lock().unwrap();
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    pub fn unobserve(&self, target: usize) {
        let mut targets = self.targets.lock().unwrap();
        targets.retain(|&t| t != target);
    }

    pub fn disconnect(&self) {
        let mut targets = self.targets.lock().unwrap();
        targets.clear();
    }

    pub fn check_intersections(&self) -> Vec<IntersectionObserverEntry> {
        // Simplified - would calculate actual intersections
        Vec::new()
    }
}

// Resize Observer - Watch element size changes
pub struct ResizeObserver {
    callback: Arc<Mutex<Option<Box<dyn Fn(Vec<ResizeObserverEntry>) + Send + Sync>>>>,
    targets: Arc<Mutex<Vec<usize>>>,
}

impl std::fmt::Debug for ResizeObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizeObserver").finish()
    }
}

impl Clone for ResizeObserver {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
            targets: Arc::clone(&self.targets),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResizeObserverEntry {
    pub target: usize,
    pub content_rect: DOMRect,
    pub border_box_size: Vec<ResizeObserverSize>,
    pub content_box_size: Vec<ResizeObserverSize>,
    pub device_pixel_content_box_size: Vec<ResizeObserverSize>,
}

#[derive(Debug, Clone, Copy)]
pub struct ResizeObserverSize {
    pub inline_size: f64,
    pub block_size: f64,
}

impl ResizeObserver {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
            targets: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn observe(&self, target: usize) {
        let mut targets = self.targets.lock().unwrap();
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    pub fn unobserve(&self, target: usize) {
        let mut targets = self.targets.lock().unwrap();
        targets.retain(|&t| t != target);
    }

    pub fn disconnect(&self) {
        let mut targets = self.targets.lock().unwrap();
        targets.clear();
    }
}

// Performance Observer - Monitor performance metrics
pub struct PerformanceObserver {
    callback: Arc<Mutex<Option<Box<dyn Fn(Vec<PerformanceEntry>) + Send + Sync>>>>,
    entry_types: Arc<Mutex<Vec<String>>>,
    buffer: Arc<Mutex<VecDeque<PerformanceEntry>>>,
}

impl std::fmt::Debug for PerformanceObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceObserver").finish()
    }
}

impl Clone for PerformanceObserver {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
            entry_types: Arc::clone(&self.entry_types),
            buffer: Arc::clone(&self.buffer),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceEntry {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
}

impl PerformanceObserver {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
            entry_types: Arc::new(Mutex::new(Vec::new())),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn observe(&self, options: PerformanceObserverInit) {
        let mut entry_types = self.entry_types.lock().unwrap();
        if let Some(types) = options.entry_types {
            entry_types.extend(types);
        }
    }

    pub fn disconnect(&self) {
        let mut entry_types = self.entry_types.lock().unwrap();
        entry_types.clear();
    }

    pub fn take_records(&self) -> Vec<PerformanceEntry> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.drain(..).collect()
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceObserverInit {
    pub entry_types: Option<Vec<String>>,
    pub buffered: bool,
}
