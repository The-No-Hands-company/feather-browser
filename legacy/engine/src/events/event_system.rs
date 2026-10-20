// DOM Level 3 Event System - Complete implementation
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct EventTarget {
    listeners: HashMap<String, Vec<EventListener>>,
    element_id: String,
}

pub struct EventListener {
    callback: Rc<RefCell<dyn FnMut(&Event)>>,
    use_capture: bool,
    once: bool,
    passive: bool,
    signal: Option<Rc<AbortSignal>>,
}

pub struct Event {
    pub event_type: String,
    pub target: Option<String>,
    pub current_target: Option<String>,
    pub event_phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub composed: bool,
    pub is_trusted: bool,
    pub timestamp: f64,
    pub data: EventData,
}

#[derive(Clone, PartialEq)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

#[derive(Clone)]
pub enum EventData {
    Mouse(MouseEventData),
    Keyboard(KeyboardEventData),
    Touch(TouchEventData),
    Wheel(WheelEventData),
    Focus(FocusEventData),
    Input(InputEventData),
    Composition(CompositionEventData),
    Pointer(PointerEventData),
    Drag(DragEventData),
    Animation(AnimationEventData),
    Transition(TransitionEventData),
    Custom(HashMap<String, String>),
    None,
}

#[derive(Clone)]
pub struct MouseEventData {
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub page_x: i32,
    pub page_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub button: i16,
    pub buttons: u16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub related_target: Option<String>,
}

#[derive(Clone)]
pub struct KeyboardEventData {
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

#[derive(Clone)]
pub struct TouchEventData {
    pub touches: Vec<Touch>,
    pub target_touches: Vec<Touch>,
    pub changed_touches: Vec<Touch>,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

#[derive(Clone)]
pub struct Touch {
    pub identifier: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub page_x: i32,
    pub page_y: i32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
    pub force: f32,
}

#[derive(Clone)]
pub struct WheelEventData {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: u32,
}

#[derive(Clone)]
pub struct FocusEventData {
    pub related_target: Option<String>,
}

#[derive(Clone)]
pub struct InputEventData {
    pub data: Option<String>,
    pub input_type: String,
    pub is_composing: bool,
}

#[derive(Clone)]
pub struct CompositionEventData {
    pub data: String,
}

#[derive(Clone)]
pub struct PointerEventData {
    pub pointer_id: i32,
    pub width: f32,
    pub height: f32,
    pub pressure: f32,
    pub tangential_pressure: f32,
    pub tilt_x: i32,
    pub tilt_y: i32,
    pub twist: i32,
    pub pointer_type: String,
    pub is_primary: bool,
}

#[derive(Clone)]
pub struct DragEventData {
    pub data_transfer: DataTransfer,
}

#[derive(Clone)]
pub struct DataTransfer {
    pub drop_effect: String,
    pub effect_allowed: String,
    pub files: Vec<String>,
    pub items: Vec<DataTransferItem>,
    pub types: Vec<String>,
}

#[derive(Clone)]
pub struct DataTransferItem {
    pub kind: String,
    pub item_type: String,
}

#[derive(Clone)]
pub struct AnimationEventData {
    pub animation_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

#[derive(Clone)]
pub struct TransitionEventData {
    pub property_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

pub struct AbortSignal {
    aborted: Rc<RefCell<bool>>,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: Rc::new(RefCell::new(false)),
        }
    }
    
    pub fn abort(&self) {
        *self.aborted.borrow_mut() = true;
    }
    
    pub fn is_aborted(&self) -> bool {
        *self.aborted.borrow()
    }
}

impl Event {
    pub fn new(event_type: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            target: None,
            current_target: None,
            event_phase: EventPhase::None,
            bubbles: true,
            cancelable: true,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            composed: false,
            is_trusted: true,
            timestamp: 0.0,
            data: EventData::None,
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

impl EventTarget {
    pub fn new(element_id: String) -> Self {
        Self {
            listeners: HashMap::new(),
            element_id,
        }
    }
    
    pub fn add_event_listener<F>(&mut self, event_type: &str, callback: F, options: AddEventListenerOptions)
    where
        F: FnMut(&Event) + 'static,
    {
        let listener = EventListener {
            callback: Rc::new(RefCell::new(callback)),
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
    
    pub fn remove_event_listener(&mut self, event_type: &str, use_capture: bool) {
        if let Some(listeners) = self.listeners.get_mut(event_type) {
            listeners.retain(|l| l.use_capture != use_capture);
        }
    }
    
    pub fn dispatch_event(&mut self, event: &mut Event) -> bool {
        event.current_target = Some(self.element_id.clone());
        
        if let Some(listeners) = self.listeners.get_mut(&event.event_type) {
            let mut to_remove = Vec::new();
            
            for (i, listener) in listeners.iter_mut().enumerate() {
                // Check if aborted
                if let Some(signal) = &listener.signal {
                    if signal.is_aborted() {
                        to_remove.push(i);
                        continue;
                    }
                }
                
                // Check phase
                let should_invoke = match event.event_phase {
                    EventPhase::Capturing => listener.use_capture,
                    EventPhase::Bubbling => !listener.use_capture,
                    EventPhase::AtTarget => true,
                    EventPhase::None => false,
                };
                
                if should_invoke {
                    listener.callback.borrow_mut()(event);
                    
                    if listener.once {
                        to_remove.push(i);
                    }
                    
                    if event.immediate_propagation_stopped {
                        break;
                    }
                }
            }
            
            // Remove once listeners
            for i in to_remove.iter().rev() {
                listeners.remove(*i);
            }
        }
        
        !event.default_prevented
    }
}

pub struct AddEventListenerOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
    pub signal: Option<Rc<AbortSignal>>,
}

impl Default for AddEventListenerOptions {
    fn default() -> Self {
        Self {
            capture: false,
            once: false,
            passive: false,
            signal: None,
        }
    }
}

// Event dispatcher for DOM tree
pub struct EventDispatcher {
    targets: HashMap<String, EventTarget>,
    dom_tree: HashMap<String, Vec<String>>, // element_id -> ancestors
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
            dom_tree: HashMap::new(),
        }
    }
    
    pub fn register_target(&mut self, element_id: String, ancestors: Vec<String>) {
        self.targets.insert(element_id.clone(), EventTarget::new(element_id.clone()));
        self.dom_tree.insert(element_id, ancestors);
    }
    
    pub fn dispatch(&mut self, target_id: &str, mut event: Event) -> bool {
        event.target = Some(target_id.to_string());
        
        // Get ancestor path
        let ancestors = self.dom_tree.get(target_id).cloned().unwrap_or_default();
        
        // Capturing phase
        event.event_phase = EventPhase::Capturing;
        for ancestor_id in ancestors.iter().rev() {
            if let Some(target) = self.targets.get_mut(ancestor_id) {
                target.dispatch_event(&mut event);
                if event.propagation_stopped {
                    return !event.default_prevented;
                }
            }
        }
        
        // At target phase
        event.event_phase = EventPhase::AtTarget;
        if let Some(target) = self.targets.get_mut(target_id) {
            target.dispatch_event(&mut event);
            if event.propagation_stopped {
                return !event.default_prevented;
            }
        }
        
        // Bubbling phase
        if event.bubbles {
            event.event_phase = EventPhase::Bubbling;
            for ancestor_id in &ancestors {
                if let Some(target) = self.targets.get_mut(ancestor_id) {
                    target.dispatch_event(&mut event);
                    if event.propagation_stopped {
                        return !event.default_prevented;
                    }
                }
            }
        }
        
        !event.default_prevented
    }
    
    pub fn add_listener<F>(&mut self, element_id: &str, event_type: &str, callback: F, options: AddEventListenerOptions)
    where
        F: FnMut(&Event) + 'static,
    {
        if let Some(target) = self.targets.get_mut(element_id) {
            target.add_event_listener(event_type, callback, options);
        }
    }
    
    pub fn remove_listener(&mut self, element_id: &str, event_type: &str, use_capture: bool) {
        if let Some(target) = self.targets.get_mut(element_id) {
            target.remove_event_listener(event_type, use_capture);
        }
    }
}

// Helper to create common events
pub mod events {
    use super::*;
    
    pub fn click(x: i32, y: i32) -> Event {
        let mut event = Event::new("click");
        event.data = EventData::Mouse(MouseEventData {
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
            page_x: x,
            page_y: y,
            offset_x: x,
            offset_y: y,
            button: 0,
            buttons: 1,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            related_target: None,
        });
        event
    }
    
    pub fn keydown(key: &str, code: &str) -> Event {
        let mut event = Event::new("keydown");
        event.data = EventData::Keyboard(KeyboardEventData {
            key: key.to_string(),
            code: code.to_string(),
            location: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
            char_code: 0,
            key_code: 0,
        });
        event
    }
    
    pub fn input(data: &str) -> Event {
        let mut event = Event::new("input");
        event.data = EventData::Input(InputEventData {
            data: Some(data.to_string()),
            input_type: "insertText".to_string(),
            is_composing: false,
        });
        event
    }
}
