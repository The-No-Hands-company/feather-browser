// Media Queries Module - Responsive Design

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    All,
    Screen,
    Print,
    Speech,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaFeature {
    Width(MediaLength),
    MinWidth(MediaLength),
    MaxWidth(MediaLength),
    Height(MediaLength),
    MinHeight(MediaLength),
    MaxHeight(MediaLength),
    AspectRatio(u32, u32),
    MinAspectRatio(u32, u32),
    MaxAspectRatio(u32, u32),
    Orientation(Orientation),
    Resolution(f32),
    MinResolution(f32),
    MaxResolution(f32),
    Scan(ScanType),
    Grid(bool),
    Update(UpdateFrequency),
    OverflowBlock(OverflowType),
    OverflowInline(OverflowType),
    Color(u32),
    MinColor(u32),
    MaxColor(u32),
    ColorIndex(u32),
    MinColorIndex(u32),
    MaxColorIndex(u32),
    Monochrome(u32),
    MinMonochrome(u32),
    MaxMonochrome(u32),
    ColorGamut(ColorGamut),
    Pointer(PointerType),
    Hover(HoverCapability),
    AnyPointer(PointerType),
    AnyHover(HoverCapability),
    PrefersReducedMotion(MotionPreference),
    PrefersColorScheme(ColorScheme),
    PrefersContrast(ContrastPreference),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaLength {
    Px(f32),
    Em(f32),
    Rem(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanType {
    Interlace,
    Progressive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdateFrequency {
    None,
    Slow,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverflowType {
    None,
    Scroll,
    Paged,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorGamut {
    Srgb,
    P3,
    Rec2020,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerType {
    None,
    Coarse,
    Fine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoverCapability {
    None,
    Hover,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionPreference {
    NoPreference,
    Reduce,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
    NoPreference,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContrastPreference {
    NoPreference,
    More,
    Less,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaCondition {
    Feature(MediaFeature),
    Not(Box<MediaCondition>),
    And(Box<MediaCondition>, Box<MediaCondition>),
    Or(Box<MediaCondition>, Box<MediaCondition>),
}

#[derive(Debug, Clone)]
pub struct MediaQuery {
    pub media_type: Option<MediaType>,
    pub condition: Option<MediaCondition>,
    pub only: bool,
    pub not: bool,
}

#[derive(Debug, Clone)]
pub struct MediaQueryList {
    pub queries: Vec<MediaQuery>,
}

impl MediaQuery {
    pub fn new() -> Self {
        MediaQuery {
            media_type: None,
            condition: None,
            only: false,
            not: false,
        }
    }
    
    pub fn matches(&self, viewport_width: f32, viewport_height: f32, media_type: &MediaType) -> bool {
        // Check media type
        if let Some(ref query_type) = self.media_type {
            if query_type != media_type && *query_type != MediaType::All {
                return self.not;
            }
        }
        
        // Check conditions
        if let Some(ref condition) = self.condition {
            let result = self.evaluate_condition(condition, viewport_width, viewport_height);
            if self.not { !result } else { result }
        } else {
            !self.not
        }
    }
    
    fn evaluate_condition(&self, condition: &MediaCondition, vw: f32, vh: f32) -> bool {
        match condition {
            MediaCondition::Feature(feature) => self.evaluate_feature(feature, vw, vh),
            MediaCondition::Not(inner) => !self.evaluate_condition(inner, vw, vh),
            MediaCondition::And(left, right) => {
                self.evaluate_condition(left, vw, vh) && self.evaluate_condition(right, vw, vh)
            }
            MediaCondition::Or(left, right) => {
                self.evaluate_condition(left, vw, vh) || self.evaluate_condition(right, vw, vh)
            }
        }
    }
    
    fn evaluate_feature(&self, feature: &MediaFeature, vw: f32, vh: f32) -> bool {
        match feature {
            MediaFeature::Width(length) => vw == self.length_to_px(length, vw, vh),
            MediaFeature::MinWidth(length) => vw >= self.length_to_px(length, vw, vh),
            MediaFeature::MaxWidth(length) => vw <= self.length_to_px(length, vw, vh),
            MediaFeature::Height(length) => vh == self.length_to_px(length, vw, vh),
            MediaFeature::MinHeight(length) => vh >= self.length_to_px(length, vw, vh),
            MediaFeature::MaxHeight(length) => vh <= self.length_to_px(length, vw, vh),
            MediaFeature::Orientation(Orientation::Portrait) => vh > vw,
            MediaFeature::Orientation(Orientation::Landscape) => vw > vh,
            MediaFeature::AspectRatio(w, h) => (vw / vh) == (*w as f32 / *h as f32),
            _ => false, // Other features need device/browser context
        }
    }
    
    fn length_to_px(&self, length: &MediaLength, vw: f32, vh: f32) -> f32 {
        match length {
            MediaLength::Px(val) => *val,
            MediaLength::Em(val) => *val * 16.0, // Assuming 16px base
            MediaLength::Rem(val) => *val * 16.0,
            MediaLength::Vw(val) => *val * vw / 100.0,
            MediaLength::Vh(val) => *val * vh / 100.0,
        }
    }
}

pub fn parse_media_type(value: &str) -> Option<MediaType> {
    match value.trim() {
        "all" => Some(MediaType::All),
        "screen" => Some(MediaType::Screen),
        "print" => Some(MediaType::Print),
        "speech" => Some(MediaType::Speech),
        _ => None,
    }
}

pub fn parse_orientation(value: &str) -> Option<Orientation> {
    match value.trim() {
        "portrait" => Some(Orientation::Portrait),
        "landscape" => Some(Orientation::Landscape),
        _ => None,
    }
}

pub fn parse_color_scheme(value: &str) -> Option<ColorScheme> {
    match value.trim() {
        "light" => Some(ColorScheme::Light),
        "dark" => Some(ColorScheme::Dark),
        "no-preference" => Some(ColorScheme::NoPreference),
        _ => None,
    }
}
