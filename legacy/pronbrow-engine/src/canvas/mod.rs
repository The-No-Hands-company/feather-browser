// Canvas API Module - 2D Drawing Context

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CanvasContext2D {
    pub fill_style: (u8, u8, u8, u8),
    pub stroke_style: (u8, u8, u8, u8),
    pub line_width: f32,
    pub global_alpha: f32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Canvas {
            width,
            height,
            data: vec![255; (width * height * 4) as usize],
        }
    }
}

impl CanvasContext2D {
    pub fn new() -> Self {
        CanvasContext2D {
            fill_style: (0, 0, 0, 255),
            stroke_style: (0, 0, 0, 255),
            line_width: 1.0,
            global_alpha: 1.0,
        }
    }
}
