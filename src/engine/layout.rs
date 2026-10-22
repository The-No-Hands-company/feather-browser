// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{CssColor, Unit, Value};
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

/// A word or text fragment placed on a line box.
#[derive(Debug, Clone)]
pub struct PlacedInlineItem {
    pub text: String,
    pub x: f32,
    pub width: f32,
    pub font_size: f32,
    pub color: CssColor,
    pub font_bold: bool,
    pub font_italic: bool,
}

/// One horizontal line in an inline formatting context.
#[derive(Debug, Clone)]
pub struct LineBox {
    pub items: Vec<PlacedInlineItem>,
    pub y: f32,
    pub height: f32,
    pub baseline: f32,
}

/// Input item for the inline formatting context.
#[derive(Debug, Clone)]
pub struct InlineText {
    pub text: String,
    pub color: CssColor,
    pub font_size: f32,
    pub font_bold: bool,
    pub font_italic: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Lays inline text runs into line boxes using a greedy left-to-right algorithm.
pub struct InlineFormattingContext {
    pub font_size: f32,
    pub available_width: f32,
    pub align: TextAlign,
}

#[cfg(feature = "feather-renderer")]
impl InlineFormattingContext {
    pub fn layout(&self, items: &[InlineText]) -> (Vec<LineBox>, f32) {
        use crate::engine::font::FontMetrics;

        if items.is_empty() {
            return (vec![], 0.0);
        }

        let mut lines: Vec<LineBox> = Vec::new();
        let mut current_items: Vec<PlacedInlineItem> = Vec::new();
        let mut cursor_x: f32 = 0.0;
        let mut line_y: f32 = 0.0;
        let mut cur_line_height: f32 = 0.0;
        let mut cur_baseline: f32 = 0.0;

        macro_rules! flush_line {
            () => {
                if !current_items.is_empty() {
                    let content_w = current_items.last().map(|i| i.x + i.width).unwrap_or(0.0);
                    let offset = match self.align {
                        TextAlign::Left => 0.0,
                        TextAlign::Center => ((self.available_width - content_w) / 2.0).max(0.0),
                        TextAlign::Right => (self.available_width - content_w).max(0.0),
                    };
                    if offset > 0.0 {
                        for item in &mut current_items {
                            item.x += offset;
                        }
                    }
                    let lh = if cur_line_height > 0.0 {
                        cur_line_height
                    } else {
                        16.0
                    };
                    let bl = if cur_baseline > 0.0 {
                        cur_baseline
                    } else {
                        12.8
                    };
                    lines.push(LineBox {
                        items: std::mem::take(&mut current_items),
                        y: line_y,
                        height: lh,
                        baseline: bl,
                    });
                    line_y += lh;
                    cursor_x = 0.0;
                    cur_line_height = 0.0;
                    cur_baseline = 0.0;
                }
            };
        }

        for item in items {
            let item_font =
                FontMetrics::load_cached_variant(item.font_size, item.font_bold, item.font_italic);
            let words: Vec<&str> = item.text.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            for word in words.iter() {
                let w = item_font.measure_str(word);
                let gap = if cursor_x == 0.0 {
                    0.0
                } else {
                    item_font.space_advance
                };
                if cursor_x + gap + w > self.available_width && cursor_x > 0.0 {
                    flush_line!();
                }
                let actual_gap = if cursor_x == 0.0 {
                    0.0
                } else {
                    item_font.space_advance
                };
                let x = cursor_x + actual_gap;
                current_items.push(PlacedInlineItem {
                    text: word.to_string(),
                    x,
                    width: w,
                    font_size: item.font_size,
                    color: item.color,
                    font_bold: item.font_bold,
                    font_italic: item.font_italic,
                });
                cursor_x = x + w;
                cur_line_height = cur_line_height.max(item_font.line_height);
                cur_baseline = cur_baseline.max(item_font.ascent);
            }
        }
        flush_line!();

        let total_height = line_y;
        (lines, total_height)
    }
}

#[cfg(not(feature = "feather-renderer"))]
impl InlineFormattingContext {
    pub fn layout(&self, _items: &[InlineText]) -> (Vec<LineBox>, f32) {
        (vec![], 0.0)
    }
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    /// Non-None for text leaf nodes.
    pub text: Option<String>,
    pub children: Vec<LayoutBox>,
    pub line_boxes: Vec<LineBox>,
}

impl LayoutBox {
    fn new(box_type: BoxType) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            text: None,
            children: Vec::new(),
            line_boxes: Vec::new(),
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
    /// Uses 600 px as the default viewport height for `vh` resolution.
    pub fn build(root: &StyledNode, viewport_width: f32) -> LayoutBox {
        Self::build_with_viewport(root, viewport_width, 600.0)
    }

    /// Build a layout tree with explicit viewport dimensions for `vw`/`vh` resolution.
    pub fn build_with_viewport(
        root: &StyledNode,
        containing_width: f32,
        viewport_height: f32,
    ) -> LayoutBox {
        let mut containing = Dimensions::default();
        containing.content.width = containing_width;
        let mut tree = build_layout_tree(root);
        layout_box(&mut tree, &containing, viewport_height, 0);
        tree
    }
}

fn build_layout_tree(styled: &StyledNode) -> LayoutBox {
    build_layout_tree_depth(styled, 0)
}

fn build_layout_tree_depth(styled: &StyledNode, depth: usize) -> LayoutBox {
    // Depth cap: prevent stack overflow on pathological DOM (Fix 1).
    if depth >= 512 {
        return LayoutBox::new(BoxType::Anonymous);
    }

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
                    lb.children.push(build_layout_tree_depth(child, depth + 1));
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

fn layout_box(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32, depth: usize) {
    // Depth cap: prevent stack overflow (Fix 1).
    if depth >= 512 {
        return;
    }
    match lb.box_type.clone() {
        BoxType::Block(_) => layout_block(lb, containing, viewport_height, depth),
        BoxType::Inline(_) => layout_inline(lb, containing),
        BoxType::Anonymous => {
            lb.dimensions.content.x = containing.content.x;
            lb.dimensions.content.y = containing.content.y;
        }
    }
}

fn all_inline_children(children: &[LayoutBox]) -> bool {
    !children.is_empty()
        && children
            .iter()
            .all(|c| matches!(c.box_type, BoxType::Anonymous | BoxType::Inline(_)))
}

fn collect_inline_text(
    children: &[LayoutBox],
    parent_font_size: f32,
    parent_color: CssColor,
    parent_bold: bool,
    parent_italic: bool,
) -> Vec<InlineText> {
    let mut items = Vec::new();
    for child in children {
        match &child.box_type {
            BoxType::Anonymous => {
                if let Some(t) = &child.text {
                    if !t.trim().is_empty() {
                        items.push(InlineText {
                            text: t.clone(),
                            color: parent_color,
                            font_size: parent_font_size,
                            font_bold: parent_bold,
                            font_italic: parent_italic,
                        });
                    }
                }
            }
            BoxType::Inline(s) => {
                let color = match s.properties.get("color") {
                    Some(Value::Color(c)) => *c,
                    _ => CssColor::DARK,
                };
                let font_size = match s.properties.get("font-size") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => 16.0,
                };
                let bold = matches!(
                    s.properties.get("font-weight"),
                    Some(Value::Keyword(k)) if k == "bold"
                );
                let italic = matches!(
                    s.properties.get("font-style"),
                    Some(Value::Keyword(k)) if k == "italic" || k == "oblique"
                );
                collect_text_from_layout_children(
                    &child.children,
                    color,
                    font_size,
                    bold,
                    italic,
                    &mut items,
                );
            }
            BoxType::Block(_) => {}
        }
    }
    items
}

fn collect_text_from_layout_children(
    children: &[LayoutBox],
    color: CssColor,
    font_size: f32,
    font_bold: bool,
    font_italic: bool,
    out: &mut Vec<InlineText>,
) {
    for child in children {
        if let Some(t) = &child.text {
            if !t.trim().is_empty() {
                out.push(InlineText {
                    text: t.clone(),
                    color,
                    font_size,
                    font_bold,
                    font_italic,
                });
            }
        }
        collect_text_from_layout_children(&child.children, color, font_size, font_bold, font_italic, out);
    }
}

fn layout_block(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32, depth: usize) {
    calculate_block_width(lb, containing, viewport_height);
    calculate_block_position(lb, containing);
    // <img> elements: set fixed dimensions from HTML attributes, skip child layout.
    if let BoxType::Block(s) | BoxType::Inline(s) = &lb.box_type {
        if let NodeType::Element(e) = &s.node.node_type {
            if e.tag_name == "img" {
                let w = e
                    .attributes
                    .get("width")
                    .and_then(|v| v.parse::<f32>().ok())
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .unwrap_or(100.0);
                let h = e
                    .attributes
                    .get("height")
                    .and_then(|v| v.parse::<f32>().ok())
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .unwrap_or(50.0);
                lb.dimensions.content.width = w;
                lb.dimensions.content.height = h;
                return;
            }
        }
    }
    if all_inline_children(&lb.children) {
        let font_size = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) => match s.properties.get("font-size") {
                Some(Value::Length(n, Unit::Px)) => *n,
                _ => 16.0,
            },
            BoxType::Anonymous => 16.0,
        };
        let color = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) => match s.properties.get("color") {
                Some(Value::Color(c)) => *c,
                _ => CssColor::BLACK,
            },
            BoxType::Anonymous => CssColor::BLACK,
        };
        let bold = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) => matches!(
                s.properties.get("font-weight"),
                Some(Value::Keyword(k)) if k == "bold"
            ),
            BoxType::Anonymous => false,
        };
        let italic = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) => matches!(
                s.properties.get("font-style"),
                Some(Value::Keyword(k)) if k == "italic" || k == "oblique"
            ),
            BoxType::Anonymous => false,
        };
        let items = collect_inline_text(&lb.children, font_size, color, bold, italic);
        if !items.is_empty() {
            let align = match &lb.box_type {
                BoxType::Block(s) | BoxType::Inline(s) => match s.properties.get("text-align") {
                    Some(Value::Keyword(k)) => match k.as_str() {
                        "center" => TextAlign::Center,
                        "right" => TextAlign::Right,
                        _ => TextAlign::Left,
                    },
                    _ => TextAlign::Left,
                },
                BoxType::Anonymous => TextAlign::Left,
            };
            let ifc = InlineFormattingContext {
                font_size,
                available_width: lb.dimensions.content.width.max(1.0),
                align,
            };
            let (line_boxes, height) = ifc.layout(&items);
            lb.line_boxes = line_boxes;
            lb.dimensions.content.height = height;
        }
        // Explicit height (px/vh/vw/%) overrides IFC-derived height.
        calculate_block_height(lb, viewport_height, containing.content.width);
    } else {
        layout_block_children(lb, viewport_height, depth);
        calculate_block_height(lb, viewport_height, containing.content.width);
    }
}

fn calculate_block_width(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32) {
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
        Some(Value::Length(n, Unit::Vw)) => Some(*n / 100.0 * cw),
        Some(Value::Length(n, Unit::Vh)) => Some(*n / 100.0 * viewport_height),
        _ => None,
    };

    let fixed = margin_left + margin_right + border_left + border_right + pad_left + pad_right;

    let mut width = if let Some(w) = explicit_width {
        w
    } else {
        // auto: fill available space
        (cw - fixed).max(0.0)
    };

    // min-width clamps up, max-width clamps down
    let min_w = match styled.properties.get("min-width") {
        Some(Value::Length(n, Unit::Px)) => Some(*n),
        Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * cw),
        _ => None,
    };
    let max_w = match styled.properties.get("max-width") {
        Some(Value::Length(n, Unit::Px)) => Some(*n),
        Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * cw),
        _ => None,
    };
    if let Some(min) = min_w {
        width = width.max(min);
    }
    if let Some(max) = max_w {
        width = width.min(max);
    }

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

fn layout_block_children(lb: &mut LayoutBox, viewport_height: f32, depth: usize) {
    let dims = lb.dimensions;
    for child in &mut lb.children {
        layout_box(child, &dims, viewport_height, depth + 1);
        lb.dimensions.content.height += child.dimensions.margin_box().height;
    }
}

fn calculate_block_height(lb: &mut LayoutBox, viewport_height: f32, containing_width: f32) {
    if let BoxType::Block(s) = &lb.box_type {
        match s.properties.get("height") {
            Some(Value::Length(h, Unit::Px)) => {
                lb.dimensions.content.height = *h;
            }
            Some(Value::Length(h, Unit::Vh)) => {
                lb.dimensions.content.height = *h / 100.0 * viewport_height;
            }
            Some(Value::Length(h, Unit::Vw)) => {
                lb.dimensions.content.height = *h / 100.0 * containing_width;
            }
            Some(Value::Length(h, Unit::Percent)) => {
                lb.dimensions.content.height = *h / 100.0 * containing_width;
            }
            _ => {}
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
