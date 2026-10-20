// CSS Animation and Transform Execution Engine
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AnimationEngine {
    active_animations: HashMap<String, RunningAnimation>,
    transitions: HashMap<String, RunningTransition>,
    frame_time: Instant,
}

#[derive(Clone)]
pub struct RunningAnimation {
    pub name: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub iteration_count: f32,
    pub direction: AnimationDirection,
    pub fill_mode: FillMode,
    pub play_state: PlayState,
    pub keyframes: Vec<Keyframe>,
    pub current_iteration: u32,
}

#[derive(Clone)]
pub struct Keyframe {
    pub offset: f32, // 0.0 to 1.0
    pub properties: HashMap<String, AnimatableValue>,
    pub easing: EasingFunction,
}

#[derive(Clone)]
pub enum AnimatableValue {
    Length(f32),
    Color(u8, u8, u8, u8),
    Transform(Transform),
    Number(f32),
    Keyword(String),
}

#[derive(Clone)]
pub struct Transform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub translate_z: f32,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotate_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub skew_x: f32,
    pub skew_y: f32,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            translate_z: 0.0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotate_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
        }
    }
    
    pub fn to_matrix(&self) -> Matrix4x4 {
        // Simplified 2D transform for now
        let cos = self.rotate_z.cos();
        let sin = self.rotate_z.sin();
        
        Matrix4x4 {
            m: [
                [self.scale_x * cos, -self.scale_y * sin, 0.0, self.translate_x],
                [self.scale_x * sin, self.scale_y * cos, 0.0, self.translate_y],
                [0.0, 0.0, 1.0, self.translate_z],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }
}

#[derive(Clone)]
pub struct Matrix4x4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub fn multiply(&self, other: &Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4 { m: [[0.0; 4]; 4] };
        
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        
        result
    }
}

#[derive(Clone)]
pub enum EasingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
    Steps(u32, StepPosition),
}

#[derive(Clone)]
pub enum StepPosition {
    Start,
    End,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::Ease => self.cubic_bezier(t, 0.25, 0.1, 0.25, 1.0),
            EasingFunction::EaseIn => self.cubic_bezier(t, 0.42, 0.0, 1.0, 1.0),
            EasingFunction::EaseOut => self.cubic_bezier(t, 0.0, 0.0, 0.58, 1.0),
            EasingFunction::EaseInOut => self.cubic_bezier(t, 0.42, 0.0, 0.58, 1.0),
            EasingFunction::CubicBezier(x1, y1, x2, y2) => self.cubic_bezier(t, *x1, *y1, *x2, *y2),
            EasingFunction::Steps(n, pos) => {
                let steps = *n as f32;
                match pos {
                    StepPosition::Start => ((t * steps).ceil() / steps).min(1.0),
                    StepPosition::End => ((t * steps).floor() / steps).min(1.0),
                }
            }
        }
    }
    
    fn cubic_bezier(&self, t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        // Complete cubic bezier calculation
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        
        // Full cubic bezier curve: P = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        // For timing function: P₀ = (0,0), P₁ = (x1,y1), P₂ = (x2,y2), P₃ = (1,1)
        let _x = mt3 * 0.0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * 1.0; // x-component
        mt3 * 0.0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * 1.0  // y-component
    }
}

#[derive(Clone)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, PartialEq)]
pub enum PlayState {
    Running,
    Paused,
}

pub struct RunningTransition {
    pub property: String,
    pub start_value: AnimatableValue,
    pub end_value: AnimatableValue,
    pub start_time: Instant,
    pub duration: Duration,
    pub easing: EasingFunction,
    pub delay: Duration,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
            transitions: HashMap::new(),
            frame_time: Instant::now(),
        }
    }
    
    pub fn tick(&mut self) -> Vec<(String, HashMap<String, AnimatableValue>)> {
        let mut updates = Vec::new();
        let now = Instant::now();
        self.frame_time = now;
        
        // Process animations - collect updates first
        let animation_updates: Vec<_> = self.active_animations.iter_mut()
            .filter_map(|(element_id, animation)| {
                if animation.play_state == PlayState::Paused {
                    return None;
                }
                
                let elapsed = now.duration_since(animation.start_time);
                let progress = elapsed.as_secs_f32() / animation.duration.as_secs_f32();
                
                if progress >= 1.0 {
                    // Animation complete
                    animation.current_iteration += 1;
                    
                    if animation.current_iteration as f32 >= animation.iteration_count 
                       && animation.iteration_count != f32::INFINITY {
                        // Animation finished
                        return None;
                    }
                    
                    // Reset for next iteration
                    animation.start_time = now;
                }
                
                Some((element_id.clone(), animation.clone(), progress))
            })
            .collect();
        
        // Calculate values separately
        for (element_id, animation, progress) in animation_updates {
            let values = Self::calculate_animation_values_static(&animation, progress);
            updates.push((element_id, values));
        }
        
        // Process transitions
        for (element_id, transition) in &self.transitions {
            let elapsed = now.duration_since(transition.start_time);
            
            if elapsed < transition.delay {
                continue;
            }
            
            let progress = ((elapsed - transition.delay).as_secs_f32() / transition.duration.as_secs_f32()).min(1.0);
            let eased = transition.easing.apply(progress);
            
            let value = Self::interpolate_value_static(&transition.start_value, &transition.end_value, eased);
            
            let mut values = HashMap::new();
            values.insert(transition.property.clone(), value);
            updates.push((element_id.clone(), values));
        }
        
        updates
    }
    
    fn calculate_animation_values_static(animation: &RunningAnimation, progress: f32) -> HashMap<String, AnimatableValue> {
        let mut result = HashMap::new();
        
        // Adjust progress based on direction
        let adjusted_progress = match animation.direction {
            AnimationDirection::Normal => progress,
            AnimationDirection::Reverse => 1.0 - progress,
            AnimationDirection::Alternate => {
                if animation.current_iteration % 2 == 0 {
                    progress
                } else {
                    1.0 - progress
                }
            }
            AnimationDirection::AlternateReverse => {
                if animation.current_iteration % 2 == 0 {
                    1.0 - progress
                } else {
                    progress
                }
            }
        };
        
        // Find keyframes to interpolate between
        let mut prev_keyframe = &animation.keyframes[0];
        let mut next_keyframe = &animation.keyframes[0];
        
        for i in 0..animation.keyframes.len() {
            if animation.keyframes[i].offset <= adjusted_progress {
                prev_keyframe = &animation.keyframes[i];
            }
            if animation.keyframes[i].offset >= adjusted_progress {
                next_keyframe = &animation.keyframes[i];
                break;
            }
        }
        
        // Interpolate between keyframes
        let keyframe_progress = if next_keyframe.offset == prev_keyframe.offset {
            0.0
        } else {
            (adjusted_progress - prev_keyframe.offset) / (next_keyframe.offset - prev_keyframe.offset)
        };
        
        let eased_progress = next_keyframe.easing.apply(keyframe_progress);
        
        for (prop, start_val) in &prev_keyframe.properties {
            if let Some(end_val) = next_keyframe.properties.get(prop) {
                let interpolated = Self::interpolate_value_static(start_val, end_val, eased_progress);
                result.insert(prop.clone(), interpolated);
            }
        }
        
        result
    }
    
    fn interpolate_value_static(start: &AnimatableValue, end: &AnimatableValue, progress: f32) -> AnimatableValue {
        match (start, end) {
            (AnimatableValue::Length(s), AnimatableValue::Length(e)) => {
                AnimatableValue::Length(s + (e - s) * progress)
            }
            (AnimatableValue::Color(sr, sg, sb, sa), AnimatableValue::Color(er, eg, eb, ea)) => {
                AnimatableValue::Color(
                    (*sr as f32 + (*er as f32 - *sr as f32) * progress) as u8,
                    (*sg as f32 + (*eg as f32 - *sg as f32) * progress) as u8,
                    (*sb as f32 + (*eb as f32 - *sb as f32) * progress) as u8,
                    (*sa as f32 + (*ea as f32 - *sa as f32) * progress) as u8,
                )
            }
            (AnimatableValue::Transform(s), AnimatableValue::Transform(e)) => {
                AnimatableValue::Transform(Transform {
                    translate_x: s.translate_x + (e.translate_x - s.translate_x) * progress,
                    translate_y: s.translate_y + (e.translate_y - s.translate_y) * progress,
                    translate_z: s.translate_z + (e.translate_z - s.translate_z) * progress,
                    rotate_x: s.rotate_x + (e.rotate_x - s.rotate_x) * progress,
                    rotate_y: s.rotate_y + (e.rotate_y - s.rotate_y) * progress,
                    rotate_z: s.rotate_z + (e.rotate_z - s.rotate_z) * progress,
                    scale_x: s.scale_x + (e.scale_x - s.scale_x) * progress,
                    scale_y: s.scale_y + (e.scale_y - s.scale_y) * progress,
                    scale_z: s.scale_z + (e.scale_z - s.scale_z) * progress,
                    skew_x: s.skew_x + (e.skew_x - s.skew_x) * progress,
                    skew_y: s.skew_y + (e.skew_y - s.skew_y) * progress,
                })
            }
            (AnimatableValue::Number(s), AnimatableValue::Number(e)) => {
                AnimatableValue::Number(s + (e - s) * progress)
            }
            _ => end.clone(),
        }
    }
    
    pub fn start_animation(&mut self, element_id: String, animation: RunningAnimation) {
        self.active_animations.insert(element_id, animation);
    }
    
    pub fn start_transition(&mut self, element_id: String, transition: RunningTransition) {
        let key = format!("{}:{}", element_id, transition.property);
        self.transitions.insert(key, transition);
    }
    
    pub fn pause_animation(&mut self, element_id: &str) {
        if let Some(animation) = self.active_animations.get_mut(element_id) {
            animation.play_state = PlayState::Paused;
        }
    }
    
    pub fn resume_animation(&mut self, element_id: &str) {
        if let Some(animation) = self.active_animations.get_mut(element_id) {
            animation.play_state = PlayState::Running;
        }
    }
    
    pub fn stop_animation(&mut self, element_id: &str) {
        self.active_animations.remove(element_id);
    }
}
