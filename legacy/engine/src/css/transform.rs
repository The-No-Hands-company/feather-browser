// CSS Transform Module - 2D and 3D Transformations

#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    // 2D Transforms
    Translate(f32, f32),
    TranslateX(f32),
    TranslateY(f32),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32),                    // in degrees
    SkewX(f32),                     // in degrees
    SkewY(f32),                     // in degrees
    Matrix(f32, f32, f32, f32, f32, f32),
    
    // 3D Transforms
    Translate3d(f32, f32, f32),
    TranslateZ(f32),
    Scale3d(f32, f32, f32),
    ScaleZ(f32),
    Rotate3d(f32, f32, f32, f32),   // x, y, z, angle
    RotateX(f32),
    RotateY(f32),
    RotateZ(f32),
    Perspective(f32),
    Matrix3d([f32; 16]),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformOrigin {
    Position(f32, f32),
    PositionZ(f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformStyle {
    Flat,
    Preserve3d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackfaceVisibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct TransformList {
    pub transforms: Vec<Transform>,
    pub origin: TransformOrigin,
    pub style: TransformStyle,
    pub backface_visibility: BackfaceVisibility,
}

impl Default for TransformOrigin {
    fn default() -> Self {
        TransformOrigin::Position(50.0, 50.0) // 50% 50%
    }
}

impl Default for TransformStyle {
    fn default() -> Self {
        TransformStyle::Flat
    }
}

impl Default for BackfaceVisibility {
    fn default() -> Self {
        BackfaceVisibility::Visible
    }
}

impl Default for TransformList {
    fn default() -> Self {
        TransformList {
            transforms: Vec::new(),
            origin: TransformOrigin::default(),
            style: TransformStyle::default(),
            backface_visibility: BackfaceVisibility::default(),
        }
    }
}

// Matrix operations for transforms
#[derive(Debug, Clone, Copy)]
pub struct Matrix2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix2D {
    pub fn identity() -> Self {
        Matrix2D {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            e: 0.0, f: 0.0,
        }
    }
    
    pub fn translate(x: f32, y: f32) -> Self {
        Matrix2D {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            e: x,   f: y,
        }
    }
    
    pub fn scale(sx: f32, sy: f32) -> Self {
        Matrix2D {
            a: sx,  b: 0.0,
            c: 0.0, d: sy,
            e: 0.0, f: 0.0,
        }
    }
    
    pub fn rotate(angle_deg: f32) -> Self {
        let angle_rad = angle_deg.to_radians();
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix2D {
            a: cos,  b: sin,
            c: -sin, d: cos,
            e: 0.0,  f: 0.0,
        }
    }
    
    pub fn multiply(&self, other: &Matrix2D) -> Matrix2D {
        Matrix2D {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }
    
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

pub fn apply_transform(transform: &Transform) -> Matrix2D {
    match transform {
        Transform::Translate(x, y) => Matrix2D::translate(*x, *y),
        Transform::TranslateX(x) => Matrix2D::translate(*x, 0.0),
        Transform::TranslateY(y) => Matrix2D::translate(0.0, *y),
        Transform::Scale(sx, sy) => Matrix2D::scale(*sx, *sy),
        Transform::ScaleX(sx) => Matrix2D::scale(*sx, 1.0),
        Transform::ScaleY(sy) => Matrix2D::scale(1.0, *sy),
        Transform::Rotate(angle) => Matrix2D::rotate(*angle),
        Transform::SkewX(angle) => {
            let tan = angle.to_radians().tan();
            Matrix2D {
                a: 1.0, b: 0.0,
                c: tan, d: 1.0,
                e: 0.0, f: 0.0,
            }
        }
        Transform::SkewY(angle) => {
            let tan = angle.to_radians().tan();
            Matrix2D {
                a: 1.0, b: tan,
                c: 0.0, d: 1.0,
                e: 0.0, f: 0.0,
            }
        }
        Transform::Matrix(a, b, c, d, e, f) => Matrix2D {
            a: *a, b: *b, c: *c, d: *d, e: *e, f: *f,
        },
        _ => Matrix2D::identity(), // 3D transforms need separate handling
    }
}

pub fn parse_transform(value: &str) -> Option<Transform> {
    let value = value.trim();
    
    if let Some(rest) = value.strip_prefix("translate(") {
        let coords: Vec<f32> = rest.trim_end_matches(')')
            .split(',')
            .filter_map(|s| s.trim().trim_end_matches("px").parse().ok())
            .collect();
        if coords.len() == 2 {
            return Some(Transform::Translate(coords[0], coords[1]));
        }
    } else if let Some(rest) = value.strip_prefix("rotate(") {
        let angle: f32 = rest.trim_end_matches(')')
            .trim_end_matches("deg")
            .parse().ok()?;
        return Some(Transform::Rotate(angle));
    } else if let Some(rest) = value.strip_prefix("scale(") {
        let scales: Vec<f32> = rest.trim_end_matches(')')
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if scales.len() == 1 {
            return Some(Transform::Scale(scales[0], scales[0]));
        } else if scales.len() == 2 {
            return Some(Transform::Scale(scales[0], scales[1]));
        }
    }
    
    None
}
