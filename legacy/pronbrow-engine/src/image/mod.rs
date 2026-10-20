// Image Loading and Rendering Module

use image::{DynamicImage, GenericImageView, ImageFormat, ImageError};
use crate::css::Color;

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

#[derive(Debug, Clone)]
pub struct ImageLoader {
    cache: std::collections::HashMap<String, DecodedImage>,
}

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Load image from file path
    pub fn load_from_path(&mut self, path: &str) -> Result<DecodedImage, ImageError> {
        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        // Load and decode image
        let img = image::open(path)?;
        let decoded = Self::decode_image(img);
        
        // Cache it
        self.cache.insert(path.to_string(), decoded.clone());
        
        Ok(decoded)
    }

    /// Load image from bytes (for embedded images or network loaded)
    pub fn load_from_bytes(&mut self, bytes: &[u8], format: ImageFormat) -> Result<DecodedImage, ImageError> {
        let img = image::load_from_memory_with_format(bytes, format)?;
        Ok(Self::decode_image(img))
    }

    /// Load PNG from bytes
    pub fn load_png_from_bytes(&mut self, bytes: &[u8]) -> Result<DecodedImage, ImageError> {
        self.load_from_bytes(bytes, ImageFormat::Png)
    }

    /// Load JPEG from bytes
    pub fn load_jpeg_from_bytes(&mut self, bytes: &[u8]) -> Result<DecodedImage, ImageError> {
        self.load_from_bytes(bytes, ImageFormat::Jpeg)
    }

    /// Decode DynamicImage to our pixel format
    fn decode_image(img: DynamicImage) -> DecodedImage {
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for pixel in rgba.pixels() {
            pixels.push(Color {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
                a: pixel[3],
            });
        }
        
        DecodedImage {
            width,
            height,
            pixels,
        }
    }

    /// Get supported formats
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "gif", "bmp", "webp"]
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for ImageLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Resize image to fit within max dimensions while maintaining aspect ratio
pub fn resize_image(img: &DecodedImage, max_width: u32, max_height: u32) -> DecodedImage {
    let (width, height) = (img.width, img.height);
    
    // Calculate aspect ratio
    let aspect = width as f32 / height as f32;
    
    // Calculate new dimensions
    let (new_width, new_height) = if width > max_width || height > max_height {
        if width as f32 / max_width as f32 > height as f32 / max_height as f32 {
            // Width is the limiting factor
            (max_width, (max_width as f32 / aspect) as u32)
        } else {
            // Height is the limiting factor
            ((max_height as f32 * aspect) as u32, max_height)
        }
    } else {
        // No resize needed
        return img.clone();
    };
    
    // Simple nearest-neighbor resize (can be improved with bilinear/bicubic later)
    let mut new_pixels = Vec::with_capacity((new_width * new_height) as usize);
    
    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = (x as f32 * width as f32 / new_width as f32) as u32;
            let src_y = (y as f32 * height as f32 / new_height as f32) as u32;
            let src_idx = (src_y * width + src_x) as usize;
            
            if src_idx < img.pixels.len() {
                new_pixels.push(img.pixels[src_idx]);
            }
        }
    }
    
    DecodedImage {
        width: new_width,
        height: new_height,
        pixels: new_pixels,
    }
}

/// Blend image onto canvas at specified position
pub fn blend_image_onto_canvas(
    canvas_pixels: &mut [Color],
    canvas_width: usize,
    canvas_height: usize,
    image: &DecodedImage,
    x: i32,
    y: i32,
) {
    for img_y in 0..image.height {
        for img_x in 0..image.width {
            let canvas_x = x + img_x as i32;
            let canvas_y = y + img_y as i32;
            
            // Bounds check
            if canvas_x < 0 || canvas_y < 0 
                || canvas_x >= canvas_width as i32 
                || canvas_y >= canvas_height as i32 {
                continue;
            }
            
            let canvas_idx = canvas_y as usize * canvas_width + canvas_x as usize;
            let img_idx = (img_y * image.width + img_x) as usize;
            
            if canvas_idx >= canvas_pixels.len() || img_idx >= image.pixels.len() {
                continue;
            }
            
            let src = &image.pixels[img_idx];
            let dst = &canvas_pixels[canvas_idx];
            
            // Alpha blending
            let alpha = src.a as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            
            canvas_pixels[canvas_idx] = Color {
                r: ((src.r as f32 * alpha + dst.r as f32 * inv_alpha) as u8),
                g: ((src.g as f32 * alpha + dst.g as f32 * inv_alpha) as u8),
                b: ((src.b as f32 * alpha + dst.b as f32 * inv_alpha) as u8),
                a: 255,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_loader_creation() {
        let loader = ImageLoader::new();
        assert_eq!(loader.cache_size(), 0);
    }

    #[test]
    fn test_supported_formats() {
        let formats = ImageLoader::supported_formats();
        assert!(formats.contains(&"png"));
        assert!(formats.contains(&"jpg"));
        assert!(formats.contains(&"jpeg"));
    }

    #[test]
    fn test_resize_maintains_aspect_ratio() {
        let img = DecodedImage {
            width: 100,
            height: 50,
            pixels: vec![Color { r: 0, g: 0, b: 0, a: 255 }; 5000],
        };
        
        let resized = resize_image(&img, 50, 50);
        
        // Should maintain 2:1 aspect ratio
        assert_eq!(resized.width, 50);
        assert_eq!(resized.height, 25);
    }
}
