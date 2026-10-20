// Performance API - for measuring performance metrics
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Performance {
    navigation_start: Instant,
    time_origin: f64,
    marks: HashMap<String, PerformanceMark>,
    measures: HashMap<String, PerformanceMeasure>,
}

impl Performance {
    pub fn new() -> Self {
        let time_origin = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64() * 1000.0;
        
        Self {
            navigation_start: Instant::now(),
            time_origin,
            marks: HashMap::new(),
            measures: HashMap::new(),
        }
    }
    
    pub fn now(&self) -> f64 {
        self.navigation_start.elapsed().as_secs_f64() * 1000.0
    }
    
    pub fn time_origin(&self) -> f64 {
        self.time_origin
    }
    
    pub fn mark(&mut self, name: &str) -> PerformanceMark {
        let mark = PerformanceMark {
            name: name.to_string(),
            start_time: self.now(),
            entry_type: "mark".to_string(),
            duration: 0.0,
        };
        self.marks.insert(name.to_string(), mark.clone());
        mark
    }
    
    pub fn measure(&mut self, name: &str, start_mark: Option<&str>, end_mark: Option<&str>) -> Result<PerformanceMeasure, String> {
        let start_time = if let Some(mark_name) = start_mark {
            self.marks.get(mark_name)
                .map(|m| m.start_time)
                .ok_or_else(|| format!("Mark '{}' not found", mark_name))?
        } else {
            0.0
        };
        
        let end_time = if let Some(mark_name) = end_mark {
            self.marks.get(mark_name)
                .map(|m| m.start_time)
                .ok_or_else(|| format!("Mark '{}' not found", mark_name))?
        } else {
            self.now()
        };
        
        let measure = PerformanceMeasure {
            name: name.to_string(),
            start_time,
            entry_type: "measure".to_string(),
            duration: end_time - start_time,
        };
        
        self.measures.insert(name.to_string(), measure.clone());
        Ok(measure)
    }
    
    pub fn clear_marks(&mut self, name: Option<&str>) {
        if let Some(mark_name) = name {
            self.marks.remove(mark_name);
        } else {
            self.marks.clear();
        }
    }
    
    pub fn clear_measures(&mut self, name: Option<&str>) {
        if let Some(measure_name) = name {
            self.measures.remove(measure_name);
        } else {
            self.measures.clear();
        }
    }
    
    pub fn get_entries(&self) -> Vec<PerformanceEntry> {
        let mut entries: Vec<PerformanceEntry> = Vec::new();
        
        for mark in self.marks.values() {
            entries.push(PerformanceEntry::Mark(mark.clone()));
        }
        
        for measure in self.measures.values() {
            entries.push(PerformanceEntry::Measure(measure.clone()));
        }
        
        entries.sort_by(|a, b| a.start_time().partial_cmp(&b.start_time()).unwrap());
        entries
    }
    
    pub fn get_entries_by_type(&self, entry_type: &str) -> Vec<PerformanceEntry> {
        self.get_entries().into_iter()
            .filter(|e| e.entry_type() == entry_type)
            .collect()
    }
    
    pub fn get_entries_by_name(&self, name: &str) -> Vec<PerformanceEntry> {
        self.get_entries().into_iter()
            .filter(|e| e.name() == name)
            .collect()
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum PerformanceEntry {
    Mark(PerformanceMark),
    Measure(PerformanceMeasure),
    Navigation(PerformanceNavigationTiming),
    Resource(PerformanceResourceTiming),
}

impl PerformanceEntry {
    pub fn name(&self) -> &str {
        match self {
            PerformanceEntry::Mark(m) => &m.name,
            PerformanceEntry::Measure(m) => &m.name,
            PerformanceEntry::Navigation(n) => &n.name,
            PerformanceEntry::Resource(r) => &r.name,
        }
    }
    
    pub fn entry_type(&self) -> &str {
        match self {
            PerformanceEntry::Mark(m) => &m.entry_type,
            PerformanceEntry::Measure(m) => &m.entry_type,
            PerformanceEntry::Navigation(n) => &n.entry_type,
            PerformanceEntry::Resource(r) => &r.entry_type,
        }
    }
    
    pub fn start_time(&self) -> f64 {
        match self {
            PerformanceEntry::Mark(m) => m.start_time,
            PerformanceEntry::Measure(m) => m.start_time,
            PerformanceEntry::Navigation(n) => n.start_time,
            PerformanceEntry::Resource(r) => r.start_time,
        }
    }
    
    pub fn duration(&self) -> f64 {
        match self {
            PerformanceEntry::Mark(m) => m.duration,
            PerformanceEntry::Measure(m) => m.duration,
            PerformanceEntry::Navigation(n) => n.duration,
            PerformanceEntry::Resource(r) => r.duration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMark {
    pub name: String,
    pub start_time: f64,
    pub entry_type: String,
    pub duration: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMeasure {
    pub name: String,
    pub start_time: f64,
    pub entry_type: String,
    pub duration: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceNavigationTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceResourceTiming {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub initiator_type: String,
    pub next_hop_protocol: String,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
}

// PerformanceObserver for monitoring performance entries
pub struct PerformanceObserver {
    callback: Box<dyn Fn(Vec<PerformanceEntry>)>,
    entry_types: Vec<String>,
}

impl PerformanceObserver {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(Vec<PerformanceEntry>) + 'static,
    {
        Self {
            callback: Box::new(callback),
            entry_types: Vec::new(),
        }
    }
    
    pub fn observe(&mut self, options: PerformanceObserverInit) {
        if let Some(types) = options.entry_types {
            self.entry_types = types;
        }
        if let Some(entry_type) = options.entry_type {
            self.entry_types.push(entry_type);
        }
    }
    
    pub fn disconnect(&mut self) {
        self.entry_types.clear();
    }
    
    pub fn take_records(&self) -> Vec<PerformanceEntry> {
        Vec::new() // Would return buffered entries
    }
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceObserverInit {
    pub entry_types: Option<Vec<String>>,
    pub entry_type: Option<String>,
    pub buffered: bool,
}
