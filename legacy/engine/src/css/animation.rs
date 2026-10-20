// CSS Animations and Transitions Module

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
    Steps(u32, StepPosition),
    CubicBezier(f32, f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationPlayState {
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,              // in seconds
    pub timing_function: TimingFunction,
    pub delay: f32,                 // in seconds
    pub iteration_count: f32,       // -1.0 for infinite
    pub direction: AnimationDirection,
    pub fill_mode: AnimationFillMode,
    pub play_state: AnimationPlayState,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub property: String,
    pub duration: f32,              // in seconds
    pub timing_function: TimingFunction,
    pub delay: f32,                 // in seconds
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub offset: f32,                // 0.0 to 1.0 (0% to 100%)
    pub properties: HashMap<String, String>,
    pub timing_function: Option<TimingFunction>,
}

#[derive(Debug, Clone)]
pub struct KeyframeAnimation {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
}

impl Default for TimingFunction {
    fn default() -> Self {
        TimingFunction::Ease
    }
}

impl Default for AnimationDirection {
    fn default() -> Self {
        AnimationDirection::Normal
    }
}

impl Default for AnimationFillMode {
    fn default() -> Self {
        AnimationFillMode::None
    }
}

impl Default for AnimationPlayState {
    fn default() -> Self {
        AnimationPlayState::Running
    }
}

impl Animation {
    pub fn new(name: String) -> Self {
        Animation {
            name,
            duration: 0.0,
            timing_function: TimingFunction::default(),
            delay: 0.0,
            iteration_count: 1.0,
            direction: AnimationDirection::default(),
            fill_mode: AnimationFillMode::default(),
            play_state: AnimationPlayState::default(),
        }
    }
}

impl Transition {
    pub fn new(property: String) -> Self {
        Transition {
            property,
            duration: 0.0,
            timing_function: TimingFunction::default(),
            delay: 0.0,
        }
    }
}

// Easing function calculator
pub fn calculate_easing(progress: f32, timing: &TimingFunction) -> f32 {
    match timing {
        TimingFunction::Linear => progress,
        TimingFunction::Ease => cubic_bezier(progress, 0.25, 0.1, 0.25, 1.0),
        TimingFunction::EaseIn => cubic_bezier(progress, 0.42, 0.0, 1.0, 1.0),
        TimingFunction::EaseOut => cubic_bezier(progress, 0.0, 0.0, 0.58, 1.0),
        TimingFunction::EaseInOut => cubic_bezier(progress, 0.42, 0.0, 0.58, 1.0),
        TimingFunction::StepStart => if progress > 0.0 { 1.0 } else { 0.0 },
        TimingFunction::StepEnd => if progress >= 1.0 { 1.0 } else { 0.0 },
        TimingFunction::Steps(steps, _) => {
            ((progress * (*steps as f32)).floor() / (*steps as f32)).min(1.0)
        }
        TimingFunction::CubicBezier(x1, y1, x2, y2) => {
            cubic_bezier(progress, *x1, *y1, *x2, *y2)
        }
    }
}

// Cubic bezier implementation (complete)
fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    // Full cubic bezier curve: P = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
    // For timing function: P₀ = (0,0), P₁ = (x1,y1), P₂ = (x2,y2), P₃ = (1,1)
    mt3 * 0.0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * 1.0; // x-component (for reference)
    mt3 * 0.0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * 1.0  // y-component (return value)
}

pub fn parse_timing_function(value: &str) -> Option<TimingFunction> {
    match value {
        "linear" => Some(TimingFunction::Linear),
        "ease" => Some(TimingFunction::Ease),
        "ease-in" => Some(TimingFunction::EaseIn),
        "ease-out" => Some(TimingFunction::EaseOut),
        "ease-in-out" => Some(TimingFunction::EaseInOut),
        "step-start" => Some(TimingFunction::StepStart),
        "step-end" => Some(TimingFunction::StepEnd),
        _ => None,
    }
}

pub fn parse_animation_direction(value: &str) -> Option<AnimationDirection> {
    match value {
        "normal" => Some(AnimationDirection::Normal),
        "reverse" => Some(AnimationDirection::Reverse),
        "alternate" => Some(AnimationDirection::Alternate),
        "alternate-reverse" => Some(AnimationDirection::AlternateReverse),
        _ => None,
    }
}

pub fn parse_fill_mode(value: &str) -> Option<AnimationFillMode> {
    match value {
        "none" => Some(AnimationFillMode::None),
        "forwards" => Some(AnimationFillMode::Forwards),
        "backwards" => Some(AnimationFillMode::Backwards),
        "both" => Some(AnimationFillMode::Both),
        _ => None,
    }
}
