// WebGL - Hardware-accelerated 3D graphics
use std::collections::HashMap;

pub struct WebGLContext {
    pub version: WebGLVersion,
    pub programs: Vec<WebGLProgram>,
    pub buffers: Vec<WebGLBuffer>,
    pub textures: Vec<WebGLTexture>,
    pub framebuffers: Vec<WebGLFramebuffer>,
    pub renderbuffers: Vec<WebGLRenderbuffer>,
    pub shaders: Vec<WebGLShader>,
    pub uniforms: HashMap<String, UniformLocation>,
    pub attributes: HashMap<String, u32>,
    pub viewport: (i32, i32, i32, i32),
    pub clear_color: (f32, f32, f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebGLVersion {
    WebGL1,
    WebGL2,
}

#[derive(Debug, Clone)]
pub struct WebGLProgram {
    pub id: usize,
    pub attached_shaders: Vec<usize>,
    pub linked: bool,
}

#[derive(Debug, Clone)]
pub struct WebGLShader {
    pub id: usize,
    pub shader_type: ShaderType,
    pub source: String,
    pub compiled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

#[derive(Debug, Clone)]
pub struct WebGLBuffer {
    pub id: usize,
    pub target: BufferTarget,
    pub data: Vec<u8>,
    pub usage: BufferUsage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
}

#[derive(Debug, Clone)]
pub struct WebGLTexture {
    pub id: usize,
    pub target: TextureTarget,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureTarget {
    Texture2D,
    TextureCubeMap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    RGB,
    RGBA,
    Alpha,
    Luminance,
    LuminanceAlpha,
}

#[derive(Debug, Clone)]
pub struct WebGLFramebuffer {
    pub id: usize,
    pub color_attachment: Option<usize>,
    pub depth_attachment: Option<usize>,
    pub stencil_attachment: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct WebGLRenderbuffer {
    pub id: usize,
    pub width: u32,
    pub height: u32,
    pub format: RenderbufferFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderbufferFormat {
    RGBA4,
    RGB565,
    RGB5A1,
    DepthComponent16,
    StencilIndex8,
}

#[derive(Debug, Clone)]
pub struct UniformLocation {
    pub program_id: usize,
    pub location: i32,
}

impl WebGLContext {
    pub fn new(version: WebGLVersion) -> Self {
        WebGLContext {
            version,
            programs: Vec::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            framebuffers: Vec::new(),
            renderbuffers: Vec::new(),
            shaders: Vec::new(),
            uniforms: HashMap::new(),
            attributes: HashMap::new(),
            viewport: (0, 0, 800, 600),
            clear_color: (0.0, 0.0, 0.0, 1.0),
        }
    }

    // Shader operations
    pub fn create_shader(&mut self, shader_type: ShaderType) -> usize {
        let shader = WebGLShader {
            id: self.shaders.len(),
            shader_type,
            source: String::new(),
            compiled: false,
        };
        self.shaders.push(shader);
        self.shaders.len() - 1
    }

    pub fn shader_source(&mut self, shader_id: usize, source: &str) {
        if let Some(shader) = self.shaders.get_mut(shader_id) {
            shader.source = source.to_string();
        }
    }

    pub fn compile_shader(&mut self, shader_id: usize) -> Result<(), String> {
        if let Some(shader) = self.shaders.get_mut(shader_id) {
            // Simple validation
            if shader.source.is_empty() {
                return Err("Shader source is empty".to_string());
            }
            shader.compiled = true;
            Ok(())
        } else {
            Err("Invalid shader ID".to_string())
        }
    }

    // Program operations
    pub fn create_program(&mut self) -> usize {
        let program = WebGLProgram {
            id: self.programs.len(),
            attached_shaders: Vec::new(),
            linked: false,
        };
        self.programs.push(program);
        self.programs.len() - 1
    }

    pub fn attach_shader(&mut self, program_id: usize, shader_id: usize) {
        if let Some(program) = self.programs.get_mut(program_id) {
            program.attached_shaders.push(shader_id);
        }
    }

    pub fn link_program(&mut self, program_id: usize) -> Result<(), String> {
        if let Some(program) = self.programs.get_mut(program_id) {
            if program.attached_shaders.len() < 2 {
                return Err("Program must have at least 2 shaders".to_string());
            }
            program.linked = true;
            Ok(())
        } else {
            Err("Invalid program ID".to_string())
        }
    }

    // Buffer operations
    pub fn create_buffer(&mut self) -> usize {
        let buffer = WebGLBuffer {
            id: self.buffers.len(),
            target: BufferTarget::ArrayBuffer,
            data: Vec::new(),
            usage: BufferUsage::StaticDraw,
        };
        self.buffers.push(buffer);
        self.buffers.len() - 1
    }

    pub fn buffer_data(&mut self, buffer_id: usize, target: BufferTarget, data: Vec<u8>, usage: BufferUsage) {
        if let Some(buffer) = self.buffers.get_mut(buffer_id) {
            buffer.target = target;
            buffer.data = data;
            buffer.usage = usage;
        }
    }

    // Texture operations
    pub fn create_texture(&mut self) -> usize {
        let texture = WebGLTexture {
            id: self.textures.len(),
            target: TextureTarget::Texture2D,
            width: 0,
            height: 0,
            format: TextureFormat::RGBA,
            data: Vec::new(),
        };
        self.textures.push(texture);
        self.textures.len() - 1
    }

    pub fn tex_image_2d(&mut self, texture_id: usize, width: u32, height: u32, format: TextureFormat, data: Vec<u8>) {
        if let Some(texture) = self.textures.get_mut(texture_id) {
            texture.width = width;
            texture.height = height;
            texture.format = format;
            texture.data = data;
        }
    }

    // Drawing operations
    pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = (r, g, b, a);
    }

    pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.viewport = (x, y, width, height);
    }

    pub fn draw_arrays(&self, _mode: DrawMode, _first: i32, _count: i32) {
        // Rendering implementation
    }

    pub fn draw_elements(&self, _mode: DrawMode, _count: i32, _type_: DataType, _offset: i32) {
        // Rendering implementation
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Float,
}
