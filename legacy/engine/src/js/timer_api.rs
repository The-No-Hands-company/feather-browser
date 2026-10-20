// Production-grade Timer API Implementation
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type TimerId = u32;
pub type TimerCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub enum TimerType {
    Timeout,
    Interval,
    Immediate,
    AnimationFrame,
    IdleCallback,
}

pub struct Timer {
    pub id: TimerId,
    pub timer_type: TimerType,
    pub callback: TimerCallback,
    pub delay: Duration,
    pub start_time: Instant,
    pub last_fire: Option<Instant>,
    pub repeat: bool,
    pub cancelled: bool,
}

impl Timer {
    pub fn should_fire(&self, now: Instant) -> bool {
        if self.cancelled {
            return false;
        }
        
        match self.last_fire {
            Some(last) => now.duration_since(last) >= self.delay,
            None => now.duration_since(self.start_time) >= self.delay,
        }
    }
    
    pub fn fire(&mut self) {
        (self.callback)();
        self.last_fire = Some(Instant::now());
        
        if !self.repeat {
            self.cancelled = true;
        }
    }
}

pub struct TimerAPI {
    timers: Arc<Mutex<HashMap<TimerId, Timer>>>,
    next_id: Arc<Mutex<TimerId>>,
    animation_frame_callbacks: Arc<Mutex<HashMap<TimerId, TimerCallback>>>,
    idle_callbacks: Arc<Mutex<HashMap<TimerId, TimerCallback>>>,
}

impl TimerAPI {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            animation_frame_callbacks: Arc::new(Mutex::new(HashMap::new())),
            idle_callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn set_timeout<F>(&self, callback: F, delay: u32) -> TimerId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.get_next_id();
        let timer = Timer {
            id,
            timer_type: TimerType::Timeout,
            callback: Arc::new(callback),
            delay: Duration::from_millis(delay as u64),
            start_time: Instant::now(),
            last_fire: None,
            repeat: false,
            cancelled: false,
        };
        
        self.timers.lock().unwrap().insert(id, timer);
        id
    }
    
    pub fn clear_timeout(&self, id: TimerId) {
        if let Some(timer) = self.timers.lock().unwrap().get_mut(&id) {
            timer.cancelled = true;
        }
    }
    
    pub fn set_interval<F>(&self, callback: F, delay: u32) -> TimerId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.get_next_id();
        let timer = Timer {
            id,
            timer_type: TimerType::Interval,
            callback: Arc::new(callback),
            delay: Duration::from_millis(delay as u64),
            start_time: Instant::now(),
            last_fire: None,
            repeat: true,
            cancelled: false,
        };
        
        self.timers.lock().unwrap().insert(id, timer);
        id
    }
    
    pub fn clear_interval(&self, id: TimerId) {
        if let Some(timer) = self.timers.lock().unwrap().get_mut(&id) {
            timer.cancelled = true;
        }
    }
    
    pub fn set_immediate<F>(&self, callback: F) -> TimerId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.get_next_id();
        let timer = Timer {
            id,
            timer_type: TimerType::Immediate,
            callback: Arc::new(callback),
            delay: Duration::from_millis(0),
            start_time: Instant::now(),
            last_fire: None,
            repeat: false,
            cancelled: false,
        };
        
        self.timers.lock().unwrap().insert(id, timer);
        id
    }
    
    pub fn clear_immediate(&self, id: TimerId) {
        if let Some(timer) = self.timers.lock().unwrap().get_mut(&id) {
            timer.cancelled = true;
        }
    }
    
    pub fn request_animation_frame<F>(&self, callback: F) -> TimerId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.get_next_id();
        self.animation_frame_callbacks.lock().unwrap()
            .insert(id, Arc::new(callback));
        id
    }
    
    pub fn cancel_animation_frame(&self, id: TimerId) {
        self.animation_frame_callbacks.lock().unwrap().remove(&id);
    }
    
    pub fn request_idle_callback<F>(&self, callback: F, timeout: Option<u32>) -> TimerId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.get_next_id();
        
        if let Some(timeout_ms) = timeout {
            // Set a timeout as fallback
            let idle_callback = Arc::new(callback);
            let callback_clone = idle_callback.clone();
            
            self.set_timeout(move || {
                callback_clone();
            }, timeout_ms);
            
            self.idle_callbacks.lock().unwrap()
                .insert(id, idle_callback);
        } else {
            self.idle_callbacks.lock().unwrap()
                .insert(id, Arc::new(callback));
        }
        
        id
    }
    
    pub fn cancel_idle_callback(&self, id: TimerId) {
        self.idle_callbacks.lock().unwrap().remove(&id);
    }
    
    pub fn tick(&self) {
        let now = Instant::now();
        let timers = self.timers.lock().unwrap();
        
        let mut to_fire = Vec::new();
        
        for (id, timer) in timers.iter() {
            if timer.should_fire(now) {
                to_fire.push(*id);
            }
        }
        
        // Fire timers outside the lock to avoid deadlock
        drop(timers);
        
        for id in to_fire {
            if let Some(timer) = self.timers.lock().unwrap().get_mut(&id) {
                timer.fire();
            }
        }
        
        // Clean up cancelled timers
        self.timers.lock().unwrap().retain(|_, timer| !timer.cancelled);
    }
    
    pub fn fire_animation_frame(&self) {
        let callbacks = self.animation_frame_callbacks.lock().unwrap().clone();
        
        for (_, callback) in callbacks {
            callback();
        }
        
        self.animation_frame_callbacks.lock().unwrap().clear();
    }
    
    pub fn fire_idle_callbacks(&self, deadline: IdleDeadline) {
        let callbacks = self.idle_callbacks.lock().unwrap().clone();
        
        for (id, callback) in callbacks {
            if deadline.time_remaining() > 0.0 {
                callback();
                self.idle_callbacks.lock().unwrap().remove(&id);
            } else {
                break;
            }
        }
    }
    
    fn get_next_id(&self) -> TimerId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    }
    
    pub fn clear_all(&self) {
        self.timers.lock().unwrap().clear();
        self.animation_frame_callbacks.lock().unwrap().clear();
        self.idle_callbacks.lock().unwrap().clear();
    }
}

impl Default for TimerAPI {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IdleDeadline {
    deadline: Instant,
}

impl IdleDeadline {
    pub fn new(deadline: Instant) -> Self {
        Self { deadline }
    }
    
    pub fn time_remaining(&self) -> f64 {
        let now = Instant::now();
        if now < self.deadline {
            self.deadline.duration_since(now).as_secs_f64() * 1000.0
        } else {
            0.0
        }
    }
    
    pub fn did_timeout(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

pub struct Performance {
    navigation_start: Instant,
    marks: HashMap<String, Instant>,
    measures: HashMap<String, (Instant, Instant)>,
}

impl Performance {
    pub fn new() -> Self {
        Self {
            navigation_start: Instant::now(),
            marks: HashMap::new(),
            measures: HashMap::new(),
        }
    }
    
    pub fn now(&self) -> f64 {
        self.navigation_start.elapsed().as_secs_f64() * 1000.0
    }
    
    pub fn mark(&mut self, name: String) {
        self.marks.insert(name, Instant::now());
    }
    
    pub fn measure(&mut self, name: String, start_mark: Option<String>, end_mark: Option<String>) -> Result<f64, String> {
        let start_time = if let Some(start) = start_mark {
            *self.marks.get(&start)
                .ok_or_else(|| format!("Start mark '{}' not found", start))?
        } else {
            self.navigation_start
        };
        
        let end_time = if let Some(end) = end_mark {
            *self.marks.get(&end)
                .ok_or_else(|| format!("End mark '{}' not found", end))?
        } else {
            Instant::now()
        };
        
        self.measures.insert(name, (start_time, end_time));
        
        Ok(end_time.duration_since(start_time).as_secs_f64() * 1000.0)
    }
    
    pub fn clear_marks(&mut self) {
        self.marks.clear();
    }
    
    pub fn clear_measures(&mut self) {
        self.measures.clear();
    }
    
    pub fn get_entries(&self) -> Vec<PerformanceEntry> {
        let mut entries = Vec::new();
        
        for (name, instant) in &self.marks {
            entries.push(PerformanceEntry {
                name: name.clone(),
                entry_type: "mark".to_string(),
                start_time: instant.duration_since(self.navigation_start).as_secs_f64() * 1000.0,
                duration: 0.0,
            });
        }
        
        for (name, (start, end)) in &self.measures {
            entries.push(PerformanceEntry {
                name: name.clone(),
                entry_type: "measure".to_string(),
                start_time: start.duration_since(self.navigation_start).as_secs_f64() * 1000.0,
                duration: end.duration_since(*start).as_secs_f64() * 1000.0,
            });
        }
        
        entries
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceEntry {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_set_timeout() {
        let api = TimerAPI::new();
        let fired = Arc::new(AtomicBool::new(false));
        let fired_clone = fired.clone();
        
        api.set_timeout(move || {
            fired_clone.store(true, Ordering::SeqCst);
        }, 10);
        
        std::thread::sleep(Duration::from_millis(20));
        api.tick();
        
        assert!(fired.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_set_interval() {
        let api = TimerAPI::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        let id = api.set_interval(move || {
            *counter_clone.lock().unwrap() += 1;
        }, 10);
        
        std::thread::sleep(Duration::from_millis(35));
        api.tick();
        std::thread::sleep(Duration::from_millis(10));
        api.tick();
        
        api.clear_interval(id);
        
        let count = *counter.lock().unwrap();
        assert!(count >= 2);
    }
    
    #[test]
    fn test_performance() {
        let mut perf = Performance::new();
        
        perf.mark("start".to_string());
        std::thread::sleep(Duration::from_millis(10));
        perf.mark("end".to_string());
        
        let duration = perf.measure("test".to_string(), 
            Some("start".to_string()), 
            Some("end".to_string())).unwrap();
        
        assert!(duration >= 10.0);
    }
}
