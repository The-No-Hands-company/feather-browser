// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{Unit, Value};
use crate::engine::dom::NodeType;
use crate::engine::style::StyledNode;

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn expanded_by(self, e: &EdgeSizes) -> Rect {
        Rect {
            x: self.x - e.left,
            y: self.y - e.top,
            width: self.width + e.left + e.right,
            height: self.height + e.top + e.bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(&self.padding)
    }
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
}

#[derive(Debug, Clone)]
pub enum BoxType {
    Block(StyledNode),
    Inline(StyledNode),
    Anonymous,
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    /// Non-None for text leaf nodes.
    pub text: Option<String>,
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    fn new(box_type: BoxType) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            text: None,
            children: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn styled_node(&self) -> Option<&StyledNode> {
        match &self.box_type {
            BoxType::Block(s) | BoxType::Inline(s) => Some(s),
            BoxType::Anonymous => None,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Build a layout tree for `root` at `viewport_width` pixels wide.
    pub fn build(root: &StyledNode, viewport_width: f32) -> LayoutBox {
        let mut containing = Dimensions::default();
        containing.content.width = viewport_width;
        let mut tree = build_layout_tree(root);
        layout_box(&mut tree, &containing);
        tree
    }
}

fn build_layout_tree(styled: &StyledNode) -> LayoutBox {
    let display = get_display(styled);
    let box_type = match display.as_deref() {
        Some("none") => {
            return LayoutBox::new(BoxType::Anonymous);
        }
        Some("inline") => BoxType::Inline(styled.clone()),
        _ => BoxType::Block(styled.clone()),
    };

    let mut lb = LayoutBox::new(box_type);

    for child in &styled.children {
        match &child.node.node_type {
            NodeType::Text(s) => {
                let text = s.trim().to_string();
                if !text.is_empty() {
                    let mut text_box = LayoutBox::new(BoxType::Anonymous);
                    text_box.text = Some(text);
                    lb.children.push(text_box);
                }
            }
            NodeType::Element(_) => {
                let child_display = get_display(child);
                if child_display.as_deref() != Some("none") {
                    lb.children.push(build_layout_tree(child));
                }
            }
            NodeType::Comment(_) => {}
        }
    }
    lb
}

fn get_display(styled: &StyledNode) -> Option<String> {
    styled.properties.get("display").and_then(|v| match v {
        Value::Keyword(k) => Some(k.clone()),
        _ => None,
    })
}

fn layout_box(lb: &mut LayoutBox, containing: &Dimensions) {
    match lb.box_type.clone() {
        BoxType::Block(_) => layout_block(lb, containing),
        BoxType::Inline(_) => layout_inline(lb, containing),
        BoxType::Anonymous => {
            lb.dimensions.content.x = containing.content.x;
            lb.dimensions.content.y = containing.content.y;
        }
    }
}

fn layout_block(lb: &mut LayoutBox, containing: &Dimensions) {
    calculate_block_width(lb, containing);
    calculate_block_position(lb, containing);
    layout_block_children(lb);
    calculate_block_height(lb);
}

fn calculate_block_width(lb: &mut LayoutBox, containing: &Dimensions) {
    let styled = match &lb.box_type {
        BoxType::Block(s) => s.clone(),
        _ => return,
    };
    let cw = containing.content.width;

    let margin_left = lookup_length_opt(&styled, "margin-left");
    let margin_right = lookup_length_opt(&styled, "margin-right");
    let border_left = lookup_length_opt(&styled, "border-left-width");
    let border_right = lookup_length_opt(&styled, "border-right-width");
    let pad_left = lookup_length_opt(&styled, "padding-left");
    let pad_right = lookup_length_opt(&styled, "padding-right");

    // Check if width is explicitly set
    let explicit_width = match styled.properties.get("width") {
        Some(Value::Length(n, Unit::Px)) => Some(*n),
        Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * cw),
        _ => None,
    };

    let fixed = margin_left + margin_right + border_left + border_right + pad_left + pad_right;

    let width = if let Some(w) = explicit_width {
        w
    } else {
        // auto: fill available space
        (cw - fixed).max(0.0)
    };

    lb.dimensions.content.width = width;
    lb.dimensions.margin.left = margin_left;
    lb.dimensions.margin.right = margin_right;
    lb.dimensions.border.left = border_left;
    lb.dimensions.border.right = border_right;
    lb.dimensions.padding.left = pad_left;
    lb.dimensions.padding.right = pad_right;
}

fn calculate_block_position(lb: &mut LayoutBox, containing: &Dimensions) {
    let styled = match &lb.box_type {
        BoxType::Block(s) => s.clone(),
        _ => return,
    };
    lb.dimensions.margin.top = lookup_length_opt(&styled, "margin-top");
    lb.dimensions.margin.bottom = lookup_length_opt(&styled, "margin-bottom");
    lb.dimensions.border.top = lookup_length_opt(&styled, "border-top-width");
    lb.dimensions.border.bottom = lookup_length_opt(&styled, "border-bottom-width");
    lb.dimensions.padding.top = lookup_length_opt(&styled, "padding-top");
    lb.dimensions.padding.bottom = lookup_length_opt(&styled, "padding-bottom");

    lb.dimensions.content.x = containing.content.x
        + lb.dimensions.margin.left
        + lb.dimensions.border.left
        + lb.dimensions.padding.left;
    lb.dimensions.content.y = containing.content.y
        + containing.content.height
        + lb.dimensions.margin.top
        + lb.dimensions.border.top
        + lb.dimensions.padding.top;
}

fn layout_block_children(lb: &mut LayoutBox) {
    let dims = lb.dimensions;
    for child in &mut lb.children {
        layout_box(child, &dims);
        lb.dimensions.content.height += child.dimensions.margin_box().height;
    }
}

fn calculate_block_height(lb: &mut LayoutBox) {
    if let BoxType::Block(s) = &lb.box_type {
        if let Some(Value::Length(h, Unit::Px)) = s.properties.get("height") {
            lb.dimensions.content.height = *h;
        }
    }
}

fn layout_inline(lb: &mut LayoutBox, containing: &Dimensions) {
    lb.dimensions.content.x = containing.content.x;
    lb.dimensions.content.y = containing.content.y + containing.content.height;
    lb.dimensions.content.width = containing.content.width;
    // Inline height is set by text rendering; leave zero for now
}

fn lookup_length_opt(styled: &StyledNode, prop: &str) -> f32 {
    match styled.properties.get(prop) {
        Some(Value::Length(n, Unit::Px)) => *n,
        _ => 0.0,
    }
}
