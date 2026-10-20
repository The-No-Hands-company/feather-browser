// Compositor - Layer composition, GPU acceleration, smooth scrolling
use crate::render::DisplayCommand;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
    pub transform: Transform,
    pub commands: Vec<DisplayCommand>,
    pub needs_repaint: bool,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub matrix: [f32; 16],
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
    
    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: [
                1.0, 0.0, 0.0, x,
                0.0, 1.0, 0.0, y,
                0.0, 0.0, 1.0, z,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

pub struct Compositor {
    layers: HashMap<usize, Layer>,
    next_id: usize,
    dirty_layers: Vec<usize>,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            next_id: 0,
            dirty_layers: Vec::new(),
        }
    }
    
    pub fn create_layer(&mut self, x: f32, y: f32, width: f32, height: f32) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let layer = Layer {
            id,
            x,
            y,
            width,
            height,
            opacity: 1.0,
            transform: Transform::identity(),
            commands: Vec::new(),
            needs_repaint: true,
        };
        
        self.layers.insert(id, layer);
        self.dirty_layers.push(id);
        id
    }
    
    pub fn update_layer(&mut self, id: usize, commands: Vec<DisplayCommand>) {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.commands = commands;
            layer.needs_repaint = true;
            if !self.dirty_layers.contains(&id) {
                self.dirty_layers.push(id);
            }
        }
    }
    
    pub fn composite(&mut self) -> Vec<DisplayCommand> {
        let mut result = Vec::new();
        let mut sorted_layers: Vec<_> = self.layers.values().collect();
        sorted_layers.sort_by_key(|l| l.id);
        
        for layer in sorted_layers {
            result.extend(layer.commands.clone());
        }
        
        self.dirty_layers.clear();
        result
    }
}
