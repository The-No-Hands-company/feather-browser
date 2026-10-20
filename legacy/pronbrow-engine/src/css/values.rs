// Advanced CSS Value Types Module

#[derive(Debug, Clone, PartialEq)]
pub enum CalcValue {
    Number(f32),
    Percentage(f32),
    Length(f32, LengthUnit),
    Add(Box<CalcValue>, Box<CalcValue>),
    Subtract(Box<CalcValue>, Box<CalcValue>),
    Multiply(Box<CalcValue>, f32),
    Divide(Box<CalcValue>, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthUnit {
    // Absolute lengths
    Px,
    Cm,
    Mm,
    In,
    Pt,
    Pc,
    
    // Font-relative lengths
    Em,
    Rem,
    Ex,
    Ch,
    
    // Viewport-relative lengths
    Vw,
    Vh,
    Vmin,
    Vmax,
    
    // Container query lengths
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleUnit {
    Deg,
    Grad,
    Rad,
    Turn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeUnit {
    S,      // seconds
    Ms,     // milliseconds
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionUnit {
    Dpi,
    Dpcm,
    Dppx,
}

impl CalcValue {
    pub fn evaluate(&self, context: &CalcContext) -> f32 {
        match self {
            CalcValue::Number(n) => *n,
            CalcValue::Percentage(p) => *p * context.percentage_base / 100.0,
            CalcValue::Length(val, unit) => self.length_to_px(*val, *unit, context),
            CalcValue::Add(left, right) => {
                left.evaluate(context) + right.evaluate(context)
            }
            CalcValue::Subtract(left, right) => {
                left.evaluate(context) - right.evaluate(context)
            }
            CalcValue::Multiply(value, factor) => {
                value.evaluate(context) * factor
            }
            CalcValue::Divide(value, divisor) => {
                value.evaluate(context) / divisor
            }
        }
    }
    
    fn length_to_px(&self, val: f32, unit: LengthUnit, context: &CalcContext) -> f32 {
        match unit {
            LengthUnit::Px => val,
            LengthUnit::Em => val * context.font_size,
            LengthUnit::Rem => val * context.root_font_size,
            LengthUnit::Vw => val * context.viewport_width / 100.0,
            LengthUnit::Vh => val * context.viewport_height / 100.0,
            LengthUnit::Vmin => val * context.viewport_width.min(context.viewport_height) / 100.0,
            LengthUnit::Vmax => val * context.viewport_width.max(context.viewport_height) / 100.0,
            LengthUnit::Cm => val * 37.7952755906,
            LengthUnit::Mm => val * 3.77952755906,
            LengthUnit::In => val * 96.0,
            LengthUnit::Pt => val * 1.333333,
            LengthUnit::Pc => val * 16.0,
            LengthUnit::Ex => val * context.font_size * 0.5, // Approximation
            LengthUnit::Ch => val * context.font_size * 0.5, // Approximation
            _ => val, // Container queries need container context
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CalcContext {
    pub font_size: f32,
    pub root_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub percentage_base: f32,
}

impl Default for CalcContext {
    fn default() -> Self {
        CalcContext {
            font_size: 16.0,
            root_font_size: 16.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
            percentage_base: 0.0,
        }
    }
}

// CSS Filter Effects
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Blur(f32),                      // px
    Brightness(f32),                // percentage or number
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32),                 // degrees
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
    DropShadow(f32, f32, f32, (u8, u8, u8, u8)), // x, y, blur, color
}

// CSS Gradients
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear {
        angle: f32,
        stops: Vec<ColorStop>,
    },
    Radial {
        shape: RadialShape,
        size: RadialSize,
        position: (f32, f32),
        stops: Vec<ColorStop>,
    },
    Conic {
        angle: f32,
        position: (f32, f32),
        stops: Vec<ColorStop>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub color: (u8, u8, u8, u8),
    pub position: Option<f32>,     // percentage 0-100
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadialShape {
    Circle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RadialSize {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
    Explicit(f32, f32),
}

// CSS Box Sizing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

impl Default for BoxSizing {
    fn default() -> Self {
        BoxSizing::ContentBox
    }
}

// CSS Cursor Types
#[derive(Debug, Clone, PartialEq)]
pub enum Cursor {
    Auto,
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
    Custom(String, f32, f32), // URL, hotspot-x, hotspot-y
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::Auto
    }
}

// CSS Blend Modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

pub fn parse_length_unit(unit: &str) -> Option<LengthUnit> {
    match unit {
        "px" => Some(LengthUnit::Px),
        "em" => Some(LengthUnit::Em),
        "rem" => Some(LengthUnit::Rem),
        "vw" => Some(LengthUnit::Vw),
        "vh" => Some(LengthUnit::Vh),
        "vmin" => Some(LengthUnit::Vmin),
        "vmax" => Some(LengthUnit::Vmax),
        "cm" => Some(LengthUnit::Cm),
        "mm" => Some(LengthUnit::Mm),
        "in" => Some(LengthUnit::In),
        "pt" => Some(LengthUnit::Pt),
        "pc" => Some(LengthUnit::Pc),
        "ex" => Some(LengthUnit::Ex),
        "ch" => Some(LengthUnit::Ch),
        _ => None,
    }
}

pub fn convert_angle(value: f32, from: AngleUnit, to: AngleUnit) -> f32 {
    let in_deg = match from {
        AngleUnit::Deg => value,
        AngleUnit::Grad => value * 0.9,
        AngleUnit::Rad => value * 180.0 / std::f32::consts::PI,
        AngleUnit::Turn => value * 360.0,
    };
    
    match to {
        AngleUnit::Deg => in_deg,
        AngleUnit::Grad => in_deg / 0.9,
        AngleUnit::Rad => in_deg * std::f32::consts::PI / 180.0,
        AngleUnit::Turn => in_deg / 360.0,
    }
}
