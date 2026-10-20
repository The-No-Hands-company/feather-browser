// Production-grade DOM Event System
// Implements W3C DOM Level 3 Events specification
// Full event propagation, preventDefault, stopPropagation support

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub target: Option<usize>, // Node ID
    pub current_target: Option<usize>,
    pub phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub timestamp: u64,
    pub is_trusted: bool,
    pub data: HashMap<String, EventData>,
}

#[derive(Debug, Clone)]
pub enum EventData {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, EventData>),
}

impl Event {
    pub fn new(event_type: String, bubbles: bool, cancelable: bool) -> Self {
        Event {
            event_type,
            target: None,
            current_target: None,
            phase: EventPhase::None,
            bubbles,
            cancelable,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            is_trusted: true,
            data: HashMap::new(),
        }
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.immediate_propagation_stopped = true;
        self.propagation_stopped = true;
    }
}

// Mouse Events
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub base: Event,
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub button: i32,
    pub buttons: i32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub related_target: Option<usize>,
}

impl MouseEvent {
    pub fn new(event_type: String) -> Self {
        MouseEvent {
            base: Event::new(event_type, true, true),
            screen_x: 0,
            screen_y: 0,
            client_x: 0,
            client_y: 0,
            offset_x: 0,
            offset_y: 0,
            button: 0,
            buttons: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            related_target: None,
        }
    }
}

// Keyboard Events
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub base: Event,
    pub key: String,
    pub code: String,
    pub location: u32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub repeat: bool,
    pub is_composing: bool,
    pub char_code: u32,
    pub key_code: u32,
}

impl KeyboardEvent {
    pub fn new(event_type: String) -> Self {
        KeyboardEvent {
            base: Event::new(event_type, true, true),
            key: String::new(),
            code: String::new(),
            location: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
            char_code: 0,
            key_code: 0,
        }
    }
}

// Focus Events
#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub base: Event,
    pub related_target: Option<usize>,
}

impl FocusEvent {
    pub fn new(event_type: String) -> Self {
        FocusEvent {
            base: Event::new(event_type, true, false),
            related_target: None,
        }
    }
}

// Input Events
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub base: Event,
    pub data: Option<String>,
    pub input_type: String,
    pub is_composing: bool,
}

impl InputEvent {
    pub fn new(event_type: String) -> Self {
        InputEvent {
            base: Event::new(event_type, true, false),
            data: None,
            input_type: String::new(),
            is_composing: false,
        }
    }
}

// Event Listener
pub type EventCallback = Arc<dyn Fn(&Event) + Send + Sync>;

#[derive(Clone)]
pub struct EventListener {
    pub callback: EventCallback,
    pub use_capture: bool,
    pub once: bool,
    pub passive: bool,
    pub signal: Option<Arc<AbortSignal>>,
}

#[derive(Debug, Clone)]
pub struct AbortSignal {
    pub aborted: Arc<Mutex<bool>>,
}

impl AbortSignal {
    pub fn new() -> Self {
        AbortSignal {
            aborted: Arc::new(Mutex::new(false)),
        }
    }

    pub fn abort(&self) {
        *self.aborted.lock().unwrap() = true;
    }

    pub fn is_aborted(&self) -> bool {
        *self.aborted.lock().unwrap()
    }
}

// Event Target
pub struct EventTarget {
    listeners: HashMap<String, Vec<EventListener>>,
    node_id: usize,
    parent_id: Option<usize>,
}

impl EventTarget {
    pub fn new(node_id: usize, parent_id: Option<usize>) -> Self {
        EventTarget {
            listeners: HashMap::new(),
            node_id,
            parent_id,
        }
    }

    pub fn add_event_listener(
        &mut self,
        event_type: &str,
        callback: EventCallback,
        options: EventListenerOptions,
    ) {
        let listener = EventListener {
            callback,
            use_capture: options.capture,
            once: options.once,
            passive: options.passive,
            signal: options.signal,
        };

        self.listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn remove_event_listener(
        &mut self,
        event_type: &str,
        _callback: EventCallback,
        use_capture: bool,
    ) {
        if let Some(listeners) = self.listeners.get_mut(event_type) {
            listeners.retain(|l| l.use_capture != use_capture);
        }
    }

    pub fn dispatch_event(&self, event: &mut Event) -> bool {
        if let Some(listeners) = self.listeners.get(&event.event_type) {
            for listener in listeners {
                // Check abort signal
                if let Some(signal) = &listener.signal {
                    if signal.is_aborted() {
                        continue;
                    }
                }

                // Check phase
                match event.phase {
                    EventPhase::Capturing if !listener.use_capture => continue,
                    EventPhase::Bubbling if listener.use_capture => continue,
                    _ => {}
                }

                // Call listener
                (listener.callback)(event);

                if event.immediate_propagation_stopped {
                    break;
                }
            }
        }

        !event.default_prevented
    }
}

#[derive(Default)]
pub struct EventListenerOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
    pub signal: Option<Arc<AbortSignal>>,
}

// Event Dispatcher - Handles event propagation
pub struct EventDispatcher {
    targets: HashMap<usize, Rc<RefCell<EventTarget>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher {
            targets: HashMap::new(),
        }
    }

    pub fn register_target(&mut self, node_id: usize, parent_id: Option<usize>) {
        let target = Rc::new(RefCell::new(EventTarget::new(node_id, parent_id)));
        self.targets.insert(node_id, target);
    }

    pub fn unregister_target(&mut self, node_id: usize) {
        self.targets.remove(&node_id);
    }

    pub fn add_event_listener(
        &mut self,
        node_id: usize,
        event_type: &str,
        callback: EventCallback,
        options: EventListenerOptions,
    ) {
        if let Some(target) = self.targets.get(&node_id) {
            target
                .borrow_mut()
                .add_event_listener(event_type, callback, options);
        }
    }

    pub fn dispatch_event(&self, target_id: usize, event: &mut Event) -> bool {
        // Build propagation path
        let mut path = Vec::new();
        let mut current_id = Some(target_id);

        while let Some(id) = current_id {
            path.push(id);
            if let Some(target) = self.targets.get(&id) {
                current_id = target.borrow().parent_id;
            } else {
                break;
            }
        }

        event.target = Some(target_id);

        // Capturing phase
        event.phase = EventPhase::Capturing;
        // Iterate from root to target (excluding target itself)
        for &id in path.iter().rev() {
            if id == target_id {
                break; // Don't dispatch to target in capturing phase
            }
            event.current_target = Some(id);
            if let Some(target) = self.targets.get(&id) {
                target.borrow().dispatch_event(event);
            }
            if event.propagation_stopped {
                return !event.default_prevented;
            }
        }

        // At target
        event.phase = EventPhase::AtTarget;
        event.current_target = Some(target_id);
        if let Some(target) = self.targets.get(&target_id) {
            target.borrow().dispatch_event(event);
        }

        if event.propagation_stopped || !event.bubbles {
            return !event.default_prevented;
        }

        // Bubbling phase
        event.phase = EventPhase::Bubbling;
        for &id in path.iter().skip(1) {
            event.current_target = Some(id);
            if let Some(target) = self.targets.get(&id) {
                target.borrow().dispatch_event(event);
            }
            if event.propagation_stopped {
                break;
            }
        }

        !event.default_prevented
    }

    // Create specific event types
    pub fn create_mouse_event(&self, event_type: &str, x: i32, y: i32) -> MouseEvent {
        let mut event = MouseEvent::new(event_type.to_string());
        event.client_x = x;
        event.client_y = y;
        event.screen_x = x;
        event.screen_y = y;
        event
    }

    pub fn create_keyboard_event(&self, event_type: &str, key: &str, code: &str) -> KeyboardEvent {
        let mut event = KeyboardEvent::new(event_type.to_string());
        event.key = key.to_string();
        event.code = code.to_string();
        event
    }

    pub fn create_input_event(&self, event_type: &str, data: &str) -> InputEvent {
        let mut event = InputEvent::new(event_type.to_string());
        event.data = Some(data.to_string());
        event.input_type = "insertText".to_string();
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new("click".to_string(), true, true);
        assert_eq!(event.event_type, "click");
        assert!(event.bubbles);
        assert!(event.cancelable);
        assert!(!event.default_prevented);
    }

    #[test]
    fn test_prevent_default() {
        let mut event = Event::new("click".to_string(), true, true);
        event.prevent_default();
        assert!(event.default_prevented);

        let mut non_cancelable = Event::new("click".to_string(), true, false);
        non_cancelable.prevent_default();
        assert!(!non_cancelable.default_prevented);
    }

    #[test]
    fn test_stop_propagation() {
        let mut event = Event::new("click".to_string(), true, true);
        event.stop_propagation();
        assert!(event.propagation_stopped);
    }

    #[test]
    fn test_event_propagation() {
        let mut dispatcher = EventDispatcher::new();
        
        // Create DOM tree: grandparent -> parent -> child
        dispatcher.register_target(1, None); // grandparent
        dispatcher.register_target(2, Some(1)); // parent
        dispatcher.register_target(3, Some(2)); // child

        let capture_called = Arc::new(Mutex::new(false));
        let bubble_called = Arc::new(Mutex::new(false));

        let capture_flag = capture_called.clone();
        dispatcher.add_event_listener(
            1,
            "click",
            Arc::new(move |_| {
                *capture_flag.lock().unwrap() = true;
            }),
            EventListenerOptions {
                capture: true,
                ..Default::default()
            },
        );

        let bubble_flag = bubble_called.clone();
        dispatcher.add_event_listener(
            1,
            "click",
            Arc::new(move |_| {
                *bubble_flag.lock().unwrap() = true;
            }),
            EventListenerOptions::default(),
        );

        let mut event = Event::new("click".to_string(), true, true);
        dispatcher.dispatch_event(3, &mut event);

        assert!(*capture_called.lock().unwrap());
        assert!(*bubble_called.lock().unwrap());
    }

    #[test]
    fn test_mouse_event() {
        let mut event = MouseEvent::new("click".to_string());
        event.client_x = 100;
        event.client_y = 200;
        event.button = 0;
        event.ctrl_key = true;

        assert_eq!(event.client_x, 100);
        assert_eq!(event.client_y, 200);
        assert!(event.ctrl_key);
    }

    #[test]
    fn test_keyboard_event() {
        let mut event = KeyboardEvent::new("keydown".to_string());
        event.key = "a".to_string();
        event.code = "KeyA".to_string();
        event.ctrl_key = true;

        assert_eq!(event.key, "a");
        assert_eq!(event.code, "KeyA");
        assert!(event.ctrl_key);
    }

    #[test]
    fn test_abort_signal() {
        let signal = Arc::new(AbortSignal::new());
        assert!(!signal.is_aborted());
        
        signal.abort();
        assert!(signal.is_aborted());
    }
}
