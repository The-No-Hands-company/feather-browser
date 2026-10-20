// Timer APIs: setTimeout, setInterval, requestAnimationFrame
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

pub struct TimerManager {
    timers: Arc<Mutex<HashMap<u32, Timer>>>,
    next_id: Arc<Mutex<u32>>,
    animation_frames: Arc<Mutex<Vec<AnimationFrameCallback>>>,
}

pub struct Timer {
    pub id: u32,
    pub callback: Box<dyn Fn() + Send>,
    pub interval: Option<Duration>,
    pub next_fire: Instant,
    pub repeat: bool,
}

pub type AnimationFrameCallback = Box<dyn Fn(f64) + Send>;

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            animation_frames: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_timeout<F>(&self, callback: F, delay_ms: u32) -> u32
    where
        F: Fn() + Send + 'static,
    {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let timer = Timer {
            id,
            callback: Box::new(callback),
            interval: None,
            next_fire: Instant::now() + Duration::from_millis(delay_ms as u64),
            repeat: false,
        };

        self.timers.lock().unwrap().insert(id, timer);
        id
    }

    pub fn set_interval<F>(&self, callback: F, delay_ms: u32) -> u32
    where
        F: Fn() + Send + 'static,
    {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let timer = Timer {
            id,
            callback: Box::new(callback),
            interval: Some(Duration::from_millis(delay_ms as u64)),
            next_fire: Instant::now() + Duration::from_millis(delay_ms as u64),
            repeat: true,
        };

        self.timers.lock().unwrap().insert(id, timer);
        id
    }

    pub fn clear_timeout(&self, id: u32) {
        self.timers.lock().unwrap().remove(&id);
    }

    pub fn clear_interval(&self, id: u32) {
        self.timers.lock().unwrap().remove(&id);
    }

    pub fn request_animation_frame<F>(&self, callback: F) -> u32
    where
        F: Fn(f64) + Send + 'static,
    {
        let mut frames = self.animation_frames.lock().unwrap();
        frames.push(Box::new(callback));
        frames.len() as u32
    }

    pub fn cancel_animation_frame(&self, id: u32) {
        let mut frames = self.animation_frames.lock().unwrap();
        if id > 0 && (id as usize) <= frames.len() {
            frames.remove(id as usize - 1);
        }
    }

    pub fn tick(&self) {
        let now = Instant::now();
        let mut timers = self.timers.lock().unwrap();
        let mut to_fire = Vec::new();

        for (id, timer) in timers.iter() {
            if now >= timer.next_fire {
                to_fire.push(*id);
            }
        }

        for id in to_fire {
            if let Some(timer) = timers.get(&id) {
                (timer.callback)();
                
                if timer.repeat {
                    if let Some(interval) = timer.interval {
                        if let Some(timer) = timers.get_mut(&id) {
                            timer.next_fire = now + interval;
                        }
                    }
                } else {
                    timers.remove(&id);
                }
            }
        }
    }

    pub fn tick_animation_frames(&self, timestamp: f64) {
        let mut frames = self.animation_frames.lock().unwrap();
        for callback in frames.drain(..) {
            callback(timestamp);
        }
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}
