// Layout Engine - Calculates positions and sizes of elements (Box Model)

use crate::css::{PropertyMap, Value, Unit};
use crate::dom::Node;

#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
    pub position_offset: EdgeSizes, // For relative/absolute positioning
    pub z_index: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(&self.padding)
    }

    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }

    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
}

impl Rect {
    pub fn expanded_by(&self, edge: &EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Display {
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Float {
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: Node,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode>,
}

impl StyledNode {
    pub fn new(node: Node, properties: PropertyMap) -> Self {
        StyledNode {
            node,
            properties,
            children: Vec::new(),
        }
    }

    pub fn value(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn position(&self) -> Position {
        match self.value("position") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "relative" => Position::Relative,
                "absolute" => Position::Absolute,
                "fixed" => Position::Fixed,
                _ => Position::Static,
            },
            _ => Position::Static,
        }
    }

    pub fn overflow(&self) -> Overflow {
        match self.value("overflow") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "hidden" => Overflow::Hidden,
                "scroll" => Overflow::Scroll,
                "auto" => Overflow::Auto,
                _ => Overflow::Visible,
            },
            _ => Overflow::Visible,
        }
    }

    pub fn z_index(&self) -> i32 {
        match self.value("z-index") {
            Some(Value::Length(val, Unit::Px)) => *val as i32,
            Some(Value::Keyword(s)) if s == "auto" => 0,
            _ => 0,
        }
    }

    pub fn float(&self) -> Float {
        match self.value("float") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "left" => Float::Left,
                "right" => Float::Right,
                _ => Float::None,
            },
            _ => Float::None,
        }
    }

    pub fn clear(&self) -> Clear {
        match self.value("clear") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "left" => Clear::Left,
                "right" => Clear::Right,
                "both" => Clear::Both,
                _ => Clear::None,
            },
            _ => Clear::None,
        }
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .or_else(|| self.value(fallback_name))
            .unwrap_or(default)
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
}

#[derive(Debug, Clone)]
pub enum BoxType {
    BlockNode(StyledNode),
    InlineNode(StyledNode),
    AnonymousBlock,
}

impl LayoutBox {
    pub fn new(box_type: BoxType) -> Self {
        LayoutBox {
            dimensions: Default::default(),
            box_type,
            children: Vec::new(),
        }
    }

    pub fn get_styled_node(&self) -> Option<&StyledNode> {
        match &self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => Some(node),
            BoxType::AnonymousBlock => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self {
        LayoutEngine
    }

    pub fn layout_tree(
        &self,
        style_root: &StyledNode,
        mut containing_block: Dimensions,
    ) -> LayoutBox {
        containing_block.content.height = 0.0;

        let mut root_box = self.build_layout_tree(style_root);
        root_box.layout(&mut containing_block);
        root_box
    }

    fn build_layout_tree(&self, style_node: &StyledNode) -> LayoutBox {
        let box_type = match style_node.display() {
            Display::Block => BoxType::BlockNode(style_node.clone()),
            Display::Inline => BoxType::InlineNode(style_node.clone()),
            Display::None => panic!("Root node has display: none"),
        };

        let mut layout_box = LayoutBox::new(box_type);

        for child in &style_node.children {
            match child.display() {
                Display::Block => layout_box.children.push(self.build_layout_tree(child)),
                Display::Inline => layout_box.children.push(self.build_layout_tree(child)),
                Display::None => {}
            }
        }

        layout_box
    }
}

impl LayoutBox {
    pub fn layout(&mut self, containing_block: &mut Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => self.layout_inline(containing_block),
            BoxType::AnonymousBlock => {}
        }
    }

    fn layout_block(&mut self, containing_block: &mut Dimensions) {
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);
        self.layout_block_children();
        self.calculate_block_height();
    }

    fn layout_inline(&mut self, containing_block: &mut Dimensions) {
        // Simplified inline layout
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;
        self.dimensions.content.width = 0.0;
        self.dimensions.content.height = 14.0; // Default line height
    }

    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        let style = self.get_styled_node().unwrap();

        let auto = Value::Keyword("auto".to_string());
        let zero = Value::Length(0.0, Unit::Px);

        let width = style.value("width").unwrap_or(&auto);
        let margin_left = style.lookup("margin-left", "margin", &zero);
        let margin_right = style.lookup("margin-right", "margin", &zero);
        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);
        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let total = sum([
            &margin_left,
            &margin_right,
            &padding_left,
            &padding_right,
            &border_left,
            &border_right,
            width,
        ]);

        let underflow = containing_block.content.width - total;

        let mut calc_width = match width {
            Value::Length(w, Unit::Px) => *w,
            _ => underflow.max(0.0),
        };

        // Apply min-width and max-width constraints
        if let Some(Value::Length(min_w, Unit::Px)) = style.value("min-width") {
            calc_width = calc_width.max(*min_w);
        }
        if let Some(Value::Length(max_w, Unit::Px)) = style.value("max-width") {
            calc_width = calc_width.min(*max_w);
        }
        
        self.dimensions.content.width = calc_width;

        self.dimensions.margin.left = to_px(&margin_left);
        self.dimensions.margin.right = to_px(&margin_right);
        self.dimensions.padding.left = to_px(&padding_left);
        self.dimensions.padding.right = to_px(&padding_right);
        self.dimensions.border.left = to_px(&border_left);
        self.dimensions.border.right = to_px(&border_right);
    }

    fn calculate_block_position(&mut self, containing_block: &mut Dimensions) {
        let zero = Value::Length(0.0, Unit::Px);
        
        let margin_top = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("margin-top", "margin", &zero))
        };
        
        let margin_bottom = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("margin-bottom", "margin", &zero))
        };
        
        let padding_top = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("padding-top", "padding", &zero))
        };
        
        let padding_bottom = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("padding-bottom", "padding", &zero))
        };
        
        let border_top = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("border-top-width", "border-width", &zero))
        };
        
        let border_bottom = {
            let style = self.get_styled_node().unwrap();
            to_px(&style.lookup("border-bottom-width", "border-width", &zero))
        };

        self.dimensions.margin.top = margin_top;
        self.dimensions.margin.bottom = margin_bottom;
        self.dimensions.padding.top = padding_top;
        self.dimensions.padding.bottom = padding_bottom;
        self.dimensions.border.top = border_top;
        self.dimensions.border.bottom = border_bottom;

        self.dimensions.content.x = containing_block.content.x
            + self.dimensions.margin.left
            + self.dimensions.border.left
            + self.dimensions.padding.left;

        self.dimensions.content.y = containing_block.content.y
            + containing_block.content.height
            + self.dimensions.margin.top
            + self.dimensions.border.top
            + self.dimensions.padding.top;

        // Handle positioning
        if let Some(style) = self.get_styled_node() {
            let position = style.position();
            let z_index = style.z_index();
            
            // Get offset values before modifying self
            let top = style.value("top").cloned();
            let right = style.value("right").cloned();
            let bottom = style.value("bottom").cloned();
            let left = style.value("left").cloned();
            
            // Now we can modify self
            self.dimensions.z_index = z_index;

            if position == Position::Relative || position == Position::Absolute {
                if let Some(Value::Length(val, Unit::Px)) = top {
                    self.dimensions.position_offset.top = val;
                    self.dimensions.content.y += val;
                }
                if let Some(Value::Length(val, Unit::Px)) = bottom {
                    self.dimensions.position_offset.bottom = val;
                    self.dimensions.content.y -= val;
                }
                if let Some(Value::Length(val, Unit::Px)) = left {
                    self.dimensions.position_offset.left = val;
                    self.dimensions.content.x += val;
                }
                if let Some(Value::Length(val, Unit::Px)) = right {
                    self.dimensions.position_offset.right = val;
                    self.dimensions.content.x -= val;
                }
            }
        }
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(d);
            d.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        let mut calc_height = if let Some(Value::Length(h, Unit::Px)) = self.get_styled_node().and_then(|s| s.value("height")) {
            *h
        } else {
            self.dimensions.content.height
        };
        
        // Apply min-height and max-height constraints
        if let Some(style) = self.get_styled_node() {
            if let Some(Value::Length(min_h, Unit::Px)) = style.value("min-height") {
                calc_height = calc_height.max(*min_h);
            }
            if let Some(Value::Length(max_h, Unit::Px)) = style.value("max-height") {
                calc_height = calc_height.min(*max_h);
            }
        }
        
        self.dimensions.content.height = calc_height;
    }
}

fn to_px(value: &Value) -> f32 {
    match value {
        Value::Length(f, Unit::Px) => *f,
        _ => 0.0,
    }
}

fn sum<'a, I>(values: I) -> f32
where
    I: IntoIterator<Item = &'a Value>,
{
    values.into_iter().map(|v| to_px(v)).sum()
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
