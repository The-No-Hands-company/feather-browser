// GPU-Accelerated Rendering Pipeline
use std::collections::HashMap;

#[derive(Debug)]
pub struct GPURenderer {
    device: GPUDevice,
    command_queue: Vec<RenderCommand>,
    render_targets: HashMap<usize, RenderTarget>,
    shaders: HashMap<String, Shader>,
    texture_cache: TextureCache,
}

#[derive(Debug)]
pub struct GPUDevice {
    pub vendor: String,
    pub renderer: String,
    pub version: String,
    pub max_texture_size: usize,
    pub max_vertex_attributes: usize,
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    Clear { color: [f32; 4] },
    DrawQuad { x: f32, y: f32, width: f32, height: f32, texture: Option<usize> },
    DrawText { x: f32, y: f32, text: String, font: String, size: f32 },
    SetTransform { matrix: [f32; 16] },
    SetBlendMode { mode: BlendMode },
    SetClipRect { x: f32, y: f32, width: f32, height: f32 },
    ResetClip,
}

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug)]
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub framebuffer: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Shader {
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: HashMap<String, UniformValue>,
}

#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([f32; 16]),
    Texture(usize),
}

#[derive(Debug)]
pub struct TextureCache {
    textures: HashMap<usize, Texture>,
    next_id: usize,
    max_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: usize,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    Alpha8,
    Depth24,
    Depth32F,
}

impl GPURenderer {
    pub fn new() -> Self {
        Self {
            device: GPUDevice {
                vendor: "PronBrow".to_string(),
                renderer: "Software Rasterizer".to_string(),
                version: "1.0".to_string(),
                max_texture_size: 16384,
                max_vertex_attributes: 16,
            },
            command_queue: Vec::new(),
            render_targets: HashMap::new(),
            shaders: HashMap::new(),
            texture_cache: TextureCache::new(512 * 1024 * 1024), // 512MB texture cache
        }
    }

    pub fn create_render_target(&mut self, width: u32, height: u32) -> usize {
        let id = self.render_targets.len();
        self.render_targets.insert(id, RenderTarget {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
            framebuffer: None,
        });
        id
    }

    pub fn submit_command(&mut self, command: RenderCommand) {
        self.command_queue.push(command);
    }

    pub fn flush(&mut self, target: usize) {
        let commands = std::mem::take(&mut self.command_queue);
        let texture_cache = &self.texture_cache;
        
        if let Some(render_target) = self.render_targets.get_mut(&target) {
            for command in &commands {
                Self::execute_command_static(command, render_target, texture_cache);
            }
        }
    }

    fn execute_command_static(command: &RenderCommand, target: &mut RenderTarget, texture_cache: &TextureCache) {
        match command {
            RenderCommand::Clear { color } => {
                let r = (color[0] * 255.0) as u8;
                let g = (color[1] * 255.0) as u8;
                let b = (color[2] * 255.0) as u8;
                let a = (color[3] * 255.0) as u8;
                
                for pixel in target.pixels.chunks_exact_mut(4) {
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                    pixel[3] = a;
                }
            }
            RenderCommand::DrawQuad { x, y, width, height, texture } => {
                Self::draw_quad_static(target, *x, *y, *width, *height, *texture, texture_cache);
            }
            _ => {}
        }
    }

    fn draw_quad_static(target: &mut RenderTarget, x: f32, y: f32, width: f32, height: f32, texture: Option<usize>, texture_cache: &TextureCache) {
        let x1 = x as i32;
        let y1 = y as i32;
        let x2 = (x + width) as i32;
        let y2 = (y + height) as i32;

        for py in y1.max(0)..y2.min(target.height as i32) {
            for px in x1.max(0)..x2.min(target.width as i32) {
                let idx = (py as usize * target.width as usize + px as usize) * 4;
                
                if let Some(tex_id) = texture {
                    if let Some(tex) = texture_cache.get(tex_id) {
                        let u = ((px - x1) as f32 / width).clamp(0.0, 1.0);
                        let v = ((py - y1) as f32 / height).clamp(0.0, 1.0);
                        let tex_x = (u * tex.width as f32) as usize;
                        let tex_y = (v * tex.height as f32) as usize;
                        let tex_idx = (tex_y * tex.width as usize + tex_x) * 4;
                        
                        if tex_idx + 3 < tex.data.len() && idx + 3 < target.pixels.len() {
                            target.pixels[idx..idx+4].copy_from_slice(&tex.data[tex_idx..tex_idx+4]);
                        }
                    }
                } else {
                    if idx + 3 < target.pixels.len() {
                        target.pixels[idx] = 255;
                        target.pixels[idx + 1] = 255;
                        target.pixels[idx + 2] = 255;
                        target.pixels[idx + 3] = 255;
                    }
                }
            }
        }
    }

    pub fn load_shader(&mut self, name: String, vertex: String, fragment: String) {
        self.shaders.insert(name, Shader {
            vertex_source: vertex,
            fragment_source: fragment,
            uniforms: HashMap::new(),
        });
    }

    pub fn upload_texture(&mut self, width: u32, height: u32, format: TextureFormat, data: Vec<u8>) -> usize {
        self.texture_cache.add(Texture {
            id: 0, // Will be set by cache
            width,
            height,
            format,
            data,
        })
    }
}

impl TextureCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            textures: HashMap::new(),
            next_id: 0,
            max_size,
            current_size: 0,
        }
    }

    pub fn add(&mut self, mut texture: Texture) -> usize {
        let size = texture.data.len();
        
        // Evict if necessary
        while self.current_size + size > self.max_size && !self.textures.is_empty() {
            self.evict_lru();
        }

        let id = self.next_id;
        self.next_id += 1;
        texture.id = id;
        
        self.textures.insert(id, texture);
        self.current_size += size;
        
        id
    }

    pub fn get(&self, id: usize) -> Option<&Texture> {
        self.textures.get(&id)
    }

    pub fn remove(&mut self, id: usize) {
        if let Some(texture) = self.textures.remove(&id) {
            self.current_size -= texture.data.len();
        }
    }

    fn evict_lru(&mut self) {
        // Simplified - would track access times
        if let Some((&id, _)) = self.textures.iter().next() {
            self.remove(id);
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.current_size = 0;
    }
}

// Layer Compositor - For smooth scrolling and animations
#[derive(Debug)]
pub struct LayerCompositor {
    layers: Vec<Layer>,
    dirty_rect: Option<Rect>,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: usize,
    pub bounds: Rect,
    pub transform: Transform,
    pub opacity: f32,
    pub content: LayerContent,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub matrix: [f32; 16],
}

#[derive(Debug, Clone)]
pub enum LayerContent {
    Texture(usize),
    Color([u8; 4]),
    Empty,
}

impl LayerCompositor {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            dirty_rect: None,
        }
    }

    pub fn create_layer(&mut self, bounds: Rect, content: LayerContent) -> usize {
        let id = self.layers.len();
        self.layers.push(Layer {
            id,
            bounds,
            transform: Transform::identity(),
            opacity: 1.0,
            content,
            children: Vec::new(),
        });
        id
    }

    pub fn update_layer(&mut self, id: usize, bounds: Option<Rect>, transform: Option<Transform>, opacity: Option<f32>) {
        if let Some(b) = bounds {
            self.mark_dirty(b);
        }
        
        if let Some(layer) = self.layers.get_mut(id) {
            if let Some(b) = bounds {
                layer.bounds = b;
            }
            if let Some(t) = transform {
                layer.transform = t;
            }
            if let Some(o) = opacity {
                layer.opacity = o;
            }
        }
    }

    pub fn composite(&self, target: &mut RenderTarget) {
        // Composite all layers in order
        for layer in &self.layers {
            self.composite_layer(layer, target);
        }
    }

    fn composite_layer(&self, layer: &Layer, target: &mut RenderTarget) {
        // Simplified compositing
        match &layer.content {
            LayerContent::Color(color) => {
                self.draw_color_layer(layer, color, target);
            }
            LayerContent::Texture(_tex_id) => {
                // Draw texture layer
            }
            LayerContent::Empty => {}
        }
    }

    fn draw_color_layer(&self, layer: &Layer, color: &[u8; 4], target: &mut RenderTarget) {
        let x1 = layer.bounds.x as i32;
        let y1 = layer.bounds.y as i32;
        let x2 = (layer.bounds.x + layer.bounds.width) as i32;
        let y2 = (layer.bounds.y + layer.bounds.height) as i32;

        for y in y1.max(0)..y2.min(target.height as i32) {
            for x in x1.max(0)..x2.min(target.width as i32) {
                let idx = (y as usize * target.width as usize + x as usize) * 4;
                if idx + 3 < target.pixels.len() {
                    let alpha = (color[3] as f32 / 255.0) * layer.opacity;
                    target.pixels[idx] = (target.pixels[idx] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8;
                    target.pixels[idx + 1] = (target.pixels[idx + 1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8;
                    target.pixels[idx + 2] = (target.pixels[idx + 2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8;
                    target.pixels[idx + 3] = 255;
                }
            }
        }
    }

    fn mark_dirty(&mut self, rect: Rect) {
        self.dirty_rect = Some(rect);
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                x,   y,   0.0, 1.0,
            ],
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [
                sx,  0.0, 0.0, 0.0,
                0.0, sy,  0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: [
                cos, sin, 0.0, 0.0,
                -sin, cos, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}
