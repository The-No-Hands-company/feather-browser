// Event System Module
pub mod event_system;

use std::collections::HashMap;

pub use event_system::{
    EventDispatcher, Event as DOMEvent, EventTarget as DOMEventTarget,
    EventPhase, AddEventListenerOptions, events
};

#[derive(Debug, Clone)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Focus(FocusEvent),
    Input(InputEvent),
    Form(FormEvent),
    Touch(TouchEvent),
    Pointer(PointerEvent),
    Drag(DragEvent),
    Wheel(WheelEvent),
    Animation(AnimationEvent),
    Transition(TransitionEvent),
    Custom(String, HashMap<String, String>),
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub x: f32,
    pub y: f32,
    pub client_x: f32,
    pub client_y: f32,
    pub button: MouseButton,
    pub buttons: u8,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventType {
    Click,
    DblClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseEnter,
    MouseLeave,
    MouseOver,
    MouseOut,
    ContextMenu,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub event_type: KeyboardEventType,
    pub key: String,
    pub code: String,
    pub location: KeyLocation,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyboardEventType {
    KeyDown,
    KeyUp,
    KeyPress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyLocation {
    Standard,
    Left,
    Right,
    Numpad,
}

#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub event_type: FocusEventType,
    pub related_target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusEventType {
    Focus,
    Blur,
    FocusIn,
    FocusOut,
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub data: Option<String>,
    pub input_type: String,
}

#[derive(Debug, Clone)]
pub struct FormEvent {
    pub event_type: FormEventType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormEventType {
    Submit,
    Reset,
    Change,
    Input,
}

#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub event_type: TouchEventType,
    pub touches: Vec<Touch>,
    pub changed_touches: Vec<Touch>,
    pub target_touches: Vec<Touch>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEventType {
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
}

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    pub identifier: i32,
    pub client_x: f32,
    pub client_y: f32,
    pub page_x: f32,
    pub page_y: f32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
    pub force: f32,
}

#[derive(Debug, Clone)]
pub struct PointerEvent {
    pub pointer_id: i32,
    pub width: f32,
    pub height: f32,
    pub pressure: f32,
    pub pointer_type: PointerType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

#[derive(Debug, Clone)]
pub struct DragEvent {
    pub event_type: DragEventType,
    pub data_transfer: Option<DataTransfer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragEventType {
    Drag,
    DragStart,
    DragEnd,
    DragEnter,
    DragOver,
    DragLeave,
    Drop,
}

#[derive(Debug, Clone)]
pub struct DataTransfer {
    pub drop_effect: DropEffect,
    pub effect_allowed: EffectAllowed,
    pub items: Vec<DataTransferItem>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropEffect {
    None,
    Copy,
    Move,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectAllowed {
    None,
    Copy,
    CopyLink,
    CopyMove,
    Link,
    LinkMove,
    Move,
    All,
}

#[derive(Debug, Clone)]
pub struct DataTransferItem {
    pub kind: String,
    pub item_type: String,
}

#[derive(Debug, Clone)]
pub struct WheelEvent {
    pub delta_x: f32,
    pub delta_y: f32,
    pub delta_z: f32,
    pub delta_mode: DeltaMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeltaMode {
    Pixel,
    Line,
    Page,
}

#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub animation_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

#[derive(Debug, Clone)]
pub struct TransitionEvent {
    pub property_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

pub struct EventTarget {
    listeners: HashMap<String, Vec<Box<dyn Fn(&Event)>>>,
}

impl EventTarget {
    pub fn new() -> Self {
        EventTarget {
            listeners: HashMap::new(),
        }
    }
    
    pub fn add_event_listener(&mut self, event_type: &str, callback: Box<dyn Fn(&Event)>) {
        self.listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }
    
    pub fn dispatch_event(&self, event: &Event) {
        let event_type = match event {
            Event::Mouse(me) => format!("{:?}", me.event_type).to_lowercase(),
            Event::Keyboard(ke) => format!("{:?}", ke.event_type).to_lowercase(),
            _ => "unknown".to_string(),
        };
        
        if let Some(listeners) = self.listeners.get(&event_type) {
            for listener in listeners {
                listener(event);
            }
        }
    }
}
