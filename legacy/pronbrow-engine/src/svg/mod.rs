// SVG (Scalable Vector Graphics) Support Module

#[derive(Debug, Clone)]
pub enum SVGElement {
    Svg(SVGAttributes),
    Circle(CircleAttributes),
    Rect(RectAttributes),
    Ellipse(EllipseAttributes),
    Line(LineAttributes),
    Polyline(PolylineAttributes),
    Polygon(PolygonAttributes),
    Path(PathAttributes),
    Text(TextAttributes, String),
    G(GroupAttributes, Vec<SVGElement>),
}

#[derive(Debug, Clone, Default)]
pub struct SVGAttributes {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub viewbox: Option<(f32, f32, f32, f32)>,
}

#[derive(Debug, Clone, Default)]
pub struct CircleAttributes {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
}

#[derive(Debug, Clone, Default)]
pub struct RectAttributes {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct EllipseAttributes {
    pub cx: f32,
    pub cy: f32,
    pub rx: f32,
    pub ry: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LineAttributes {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug, Clone, Default)]
pub struct PolylineAttributes {
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Default)]
pub struct PolygonAttributes {
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Default)]
pub struct PathAttributes {
    pub d: String,
}

#[derive(Debug, Clone, Default)]
pub struct TextAttributes {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default)]
pub struct GroupAttributes {
    pub transform: Option<String>,
}
