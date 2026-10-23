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
    /// Block-level internal layout, inline participation in parent flow.
    InlineBlock(StyledNode),
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
    pub text_decoration: TextDecoration,
    pub letter_spacing: f32,
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
    pub text_decoration: TextDecoration,
    /// When true, flush the current line immediately (models a <br> break).
    pub force_break: bool,
    /// Extra space added after each character in px (CSS `letter-spacing`).
    pub letter_spacing: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    LineThrough,
}

/// Lays inline text runs into line boxes using a greedy left-to-right algorithm.
pub struct InlineFormattingContext {
    pub font_size: f32,
    pub available_width: f32,
    pub align: TextAlign,
    /// CSS `line-height` resolved to px. None = use font metrics.
    pub line_height_override: Option<f32>,
    /// `white-space: nowrap` — suppress automatic line wrapping.
    pub nowrap: bool,
    /// `white-space: pre` — preserve whitespace and honour `\n` as hard line breaks.
    pub pre: bool,
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
            if item.force_break {
                flush_line!();
                // Ensure the blank line still occupies vertical space.
                if current_items.is_empty() {
                    let item_font = FontMetrics::load_cached_variant(
                        item.font_size,
                        item.font_bold,
                        item.font_italic,
                    );
                    line_y += self.line_height_override.unwrap_or(item_font.line_height);
                }
                continue;
            }
            let item_font =
                FontMetrics::load_cached_variant(item.font_size, item.font_bold, item.font_italic);
            let effective_lh = self
                .line_height_override
                .unwrap_or(item_font.line_height)
                .max(item_font.line_height);
            if self.pre {
                // white-space:pre — honour \n as hard breaks; place each segment verbatim.
                let segments: Vec<&str> = item.text.split('\n').collect();
                for (i, segment) in segments.iter().enumerate() {
                    if !segment.is_empty() {
                        let ls = item.letter_spacing * segment.chars().count() as f32;
                        let w = item_font.measure_str(segment) + ls;
                        let x = cursor_x;
                        current_items.push(PlacedInlineItem {
                            text: segment.to_string(),
                            x,
                            width: w,
                            font_size: item.font_size,
                            color: item.color,
                            font_bold: item.font_bold,
                            font_italic: item.font_italic,
                            text_decoration: item.text_decoration,
                            letter_spacing: item.letter_spacing,
                        });
                        cursor_x = x + w;
                        cur_line_height = cur_line_height.max(effective_lh);
                        cur_baseline = cur_baseline.max(item_font.ascent);
                    }
                    if i < segments.len() - 1 {
                        flush_line!();
                        if current_items.is_empty() {
                            line_y += effective_lh;
                        }
                    }
                }
            } else {
                let words: Vec<&str> = item.text.split_whitespace().collect();
                if words.is_empty() {
                    continue;
                }
                for word in words.iter() {
                    let ls = item.letter_spacing * word.chars().count() as f32;
                    let w = item_font.measure_str(word) + ls;
                    let gap = if cursor_x == 0.0 { 0.0 } else { item_font.space_advance };
                    if !self.nowrap && cursor_x + gap + w > self.available_width && cursor_x > 0.0 {
                        flush_line!();
                    }
                    let actual_gap = if cursor_x == 0.0 { 0.0 } else { item_font.space_advance };
                    let x = cursor_x + actual_gap;
                    current_items.push(PlacedInlineItem {
                        text: word.to_string(),
                        x,
                        width: w,
                        font_size: item.font_size,
                        color: item.color,
                        font_bold: item.font_bold,
                        font_italic: item.font_italic,
                        text_decoration: item.text_decoration,
                        letter_spacing: item.letter_spacing,
                    });
                    cursor_x = x + w;
                    cur_line_height = cur_line_height.max(effective_lh);
                    cur_baseline = cur_baseline.max(item_font.ascent);
                }
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
    /// 1-based index within a parent <ol>; 0 for non-li or unordered list items.
    pub list_item_index: usize,
}

impl LayoutBox {
    fn new(box_type: BoxType) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            text: None,
            children: Vec::new(),
            line_boxes: Vec::new(),
            list_item_index: 0,
        }
    }

    #[allow(dead_code)]
    fn styled_node(&self) -> Option<&StyledNode> {
        match &self.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => Some(s),
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
        Some("inline-block") => BoxType::InlineBlock(styled.clone()),
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
        BoxType::InlineBlock(_) => layout_inline_block(lb, containing, viewport_height, depth),
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
            .all(|c| matches!(c.box_type, BoxType::Anonymous | BoxType::Inline(_) | BoxType::InlineBlock(_)))
}

fn collect_inline_text(
    children: &[LayoutBox],
    parent_font_size: f32,
    parent_color: CssColor,
    parent_bold: bool,
    parent_italic: bool,
    parent_decoration: TextDecoration,
    parent_letter_spacing: f32,
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
                            text_decoration: parent_decoration,
                            force_break: false,
                            letter_spacing: parent_letter_spacing,
                        });
                    }
                }
            }
            BoxType::Inline(s) => {
                // <br> injects a forced line break.
                if s.node.tag_name() == Some("br") {
                    items.push(InlineText {
                        text: String::new(),
                        color: parent_color,
                        font_size: parent_font_size,
                        font_bold: parent_bold,
                        font_italic: parent_italic,
                        text_decoration: parent_decoration,
                        force_break: true,
                        letter_spacing: parent_letter_spacing,
                    });
                    continue;
                }
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
                let decoration = match s.properties.get("text-decoration") {
                    Some(Value::Keyword(k)) if k == "underline" => TextDecoration::Underline,
                    Some(Value::Keyword(k)) if k == "line-through" => TextDecoration::LineThrough,
                    _ => parent_decoration,
                };
                let letter_spacing = match s.properties.get("letter-spacing") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => parent_letter_spacing,
                };
                collect_text_from_layout_children(
                    &child.children,
                    color,
                    font_size,
                    bold,
                    italic,
                    decoration,
                    letter_spacing,
                    &mut items,
                );
            }
            BoxType::InlineBlock(s) => {
                let color = match s.properties.get("color") {
                    Some(Value::Color(c)) => *c,
                    _ => parent_color,
                };
                let font_size = match s.properties.get("font-size") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => parent_font_size,
                };
                let bold = matches!(
                    s.properties.get("font-weight"),
                    Some(Value::Keyword(k)) if k == "bold"
                );
                let italic = matches!(
                    s.properties.get("font-style"),
                    Some(Value::Keyword(k)) if k == "italic" || k == "oblique"
                );
                let decoration = match s.properties.get("text-decoration") {
                    Some(Value::Keyword(k)) if k == "underline" => TextDecoration::Underline,
                    Some(Value::Keyword(k)) if k == "line-through" => TextDecoration::LineThrough,
                    _ => parent_decoration,
                };
                let letter_spacing = match s.properties.get("letter-spacing") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => parent_letter_spacing,
                };
                collect_text_from_layout_children(
                    &child.children,
                    color,
                    font_size,
                    bold,
                    italic,
                    decoration,
                    letter_spacing,
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
    text_decoration: TextDecoration,
    letter_spacing: f32,
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
                    text_decoration,
                    force_break: false,
                    letter_spacing,
                });
            }
        }
        collect_text_from_layout_children(&child.children, color, font_size, font_bold, font_italic, text_decoration, letter_spacing, out);
    }
}

fn layout_block(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32, depth: usize) {
    calculate_block_width(lb, containing, viewport_height);
    calculate_block_position(lb, containing);
    apply_relative_offset(lb);
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
    if all_inline_children(&lb.children)
        && lb
            .children
            .iter()
            .any(|c| matches!(c.box_type, BoxType::InlineBlock(_)))
    {
        layout_inline_block_row(lb, viewport_height, depth);
        calculate_block_height(lb, viewport_height, containing.content.width);
        return;
    }
    if all_inline_children(&lb.children) {
        let font_size = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                match s.properties.get("font-size") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => 16.0,
                }
            }
            BoxType::Anonymous => 16.0,
        };
        let color = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                match s.properties.get("color") {
                    Some(Value::Color(c)) => *c,
                    _ => CssColor::BLACK,
                }
            }
            BoxType::Anonymous => CssColor::BLACK,
        };
        let bold = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => matches!(
                s.properties.get("font-weight"),
                Some(Value::Keyword(k)) if k == "bold"
            ),
            BoxType::Anonymous => false,
        };
        let italic = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => matches!(
                s.properties.get("font-style"),
                Some(Value::Keyword(k)) if k == "italic" || k == "oblique"
            ),
            BoxType::Anonymous => false,
        };
        let decoration = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                match s.properties.get("text-decoration") {
                    Some(Value::Keyword(k)) if k == "underline" => TextDecoration::Underline,
                    Some(Value::Keyword(k)) if k == "line-through" => TextDecoration::LineThrough,
                    _ => TextDecoration::None,
                }
            }
            BoxType::Anonymous => TextDecoration::None,
        };
        let letter_spacing = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                match s.properties.get("letter-spacing") {
                    Some(Value::Length(n, Unit::Px)) => *n,
                    _ => 0.0,
                }
            }
            BoxType::Anonymous => 0.0,
        };
        let mut items = collect_inline_text(&lb.children, font_size, color, bold, italic, decoration, letter_spacing);
        // Inject list marker for <li> elements with list-style-type.
        let list_style = match &lb.box_type {
            BoxType::Block(s) | BoxType::Inline(s)
                if s.node.tag_name() == Some("li") =>
            {
                s.properties.get("list-style-type").and_then(|v| {
                    if let Value::Keyword(k) = v { Some(k.as_str()) } else { None }
                })
            }
            _ => None,
        };
        if let Some(style) = list_style {
            let marker_text = match style {
                "disc" => "• ".to_string(),
                "decimal" => format!("{}. ", lb.list_item_index),
                _ => String::new(),
            };
            if !marker_text.is_empty() {
                items.insert(0, InlineText {
                    text: marker_text,
                    color,
                    font_size,
                    font_bold: bold,
                    font_italic: italic,
                    text_decoration: TextDecoration::None,
                    force_break: false,
                    letter_spacing: 0.0,
                });
            }
        }
        if !items.is_empty() {
            let align = match &lb.box_type {
                BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                    match s.properties.get("text-align") {
                        Some(Value::Keyword(k)) => match k.as_str() {
                            "center" => TextAlign::Center,
                            "right" => TextAlign::Right,
                            _ => TextAlign::Left,
                        },
                        _ => TextAlign::Left,
                    }
                }
                BoxType::Anonymous => TextAlign::Left,
            };
            let line_height_override = match &lb.box_type {
                BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                    match s.properties.get("line-height") {
                        Some(Value::Length(n, Unit::Px)) => Some(*n),
                        Some(Value::Number(n)) => Some(*n * font_size),
                        _ => None,
                    }
                }
                BoxType::Anonymous => None,
            };
            let (nowrap, pre) = match &lb.box_type {
                BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
                    match s.properties.get("white-space") {
                        Some(Value::Keyword(k)) if k == "nowrap" => (true, false),
                        Some(Value::Keyword(k)) if k == "pre" => (false, true),
                        _ => (false, false),
                    }
                }
                BoxType::Anonymous => (false, false),
            };
            let ifc = InlineFormattingContext {
                font_size,
                available_width: lb.dimensions.content.width.max(1.0),
                align,
                line_height_override,
                nowrap,
                pre,
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

    let border_box = matches!(
        styled.properties.get("box-sizing"),
        Some(Value::Keyword(k)) if k == "border-box"
    );

    let mut width = if let Some(w) = explicit_width {
        if border_box {
            // Specified width is the border-box width; subtract borders + padding for content width.
            (w - border_left - border_right - pad_left - pad_right).max(0.0)
        } else {
            w
        }
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

fn apply_relative_offset(lb: &mut LayoutBox) {
    let styled = match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => s.clone(),
        BoxType::Anonymous => return,
    };
    let is_relative = matches!(
        styled.properties.get("position"),
        Some(Value::Keyword(k)) if k == "relative"
    );
    if !is_relative {
        return;
    }
    let dx = match styled.properties.get("left") {
        Some(Value::Length(n, Unit::Px)) => *n,
        _ => match styled.properties.get("right") {
            Some(Value::Length(n, Unit::Px)) => -n,
            _ => 0.0,
        },
    };
    let dy = match styled.properties.get("top") {
        Some(Value::Length(n, Unit::Px)) => *n,
        _ => match styled.properties.get("bottom") {
            Some(Value::Length(n, Unit::Px)) => -n,
            _ => 0.0,
        },
    };
    lb.dimensions.content.x += dx;
    lb.dimensions.content.y += dy;
}

fn is_tag(lb: &LayoutBox, tag: &str) -> bool {
    match &lb.box_type {
        BoxType::Block(s) | BoxType::Inline(s) | BoxType::InlineBlock(s) => {
            s.node.tag_name() == Some(tag)
        }
        BoxType::Anonymous => false,
    }
}

fn layout_block_children(lb: &mut LayoutBox, viewport_height: f32, depth: usize) {
    let dims = lb.dimensions;
    let is_ordered = is_tag(lb, "ol");
    let mut li_counter: usize = 0;
    for child in &mut lb.children {
        if is_ordered && is_tag(child, "li") {
            li_counter += 1;
            child.list_item_index = li_counter;
        }
        layout_box(child, &dims, viewport_height, depth + 1);
        lb.dimensions.content.height += child.dimensions.margin_box().height;
    }
}

fn calculate_block_height(lb: &mut LayoutBox, viewport_height: f32, containing_width: f32) {
    if let BoxType::Block(s) | BoxType::InlineBlock(s) = &lb.box_type {
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
        // min-height clamps up, max-height clamps down
        let min_h = match s.properties.get("min-height") {
            Some(Value::Length(n, Unit::Px)) => Some(*n),
            Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * viewport_height),
            _ => None,
        };
        let max_h = match s.properties.get("max-height") {
            Some(Value::Length(n, Unit::Px)) => Some(*n),
            Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * viewport_height),
            _ => None,
        };
        if let Some(min) = min_h {
            lb.dimensions.content.height = lb.dimensions.content.height.max(min);
        }
        if let Some(max) = max_h {
            lb.dimensions.content.height = lb.dimensions.content.height.min(max);
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

/// Compute the content width of an inline-block, respecting explicit width or
/// shrink-to-fit (using available container width as maximum).
fn calculate_inline_block_width(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32) {
    let styled = match &lb.box_type {
        BoxType::InlineBlock(s) => s.clone(),
        _ => return,
    };
    let cw = containing.content.width;
    let margin_left = lookup_length_opt(&styled, "margin-left");
    let margin_right = lookup_length_opt(&styled, "margin-right");
    let border_left = lookup_length_opt(&styled, "border-left-width");
    let border_right = lookup_length_opt(&styled, "border-right-width");
    let pad_left = lookup_length_opt(&styled, "padding-left");
    let pad_right = lookup_length_opt(&styled, "padding-right");

    let explicit_width = match styled.properties.get("width") {
        Some(Value::Length(n, Unit::Px)) => Some(*n),
        Some(Value::Length(n, Unit::Percent)) => Some(*n / 100.0 * cw),
        Some(Value::Length(n, Unit::Vw)) => Some(*n / 100.0 * cw),
        Some(Value::Length(n, Unit::Vh)) => Some(*n / 100.0 * viewport_height),
        _ => None,
    };

    let border_box = matches!(
        styled.properties.get("box-sizing"),
        Some(Value::Keyword(k)) if k == "border-box"
    );

    let width = if let Some(w) = explicit_width {
        if border_box {
            (w - border_left - border_right - pad_left - pad_right).max(0.0)
        } else {
            w
        }
    } else {
        // Shrink-to-fit: available width minus all fixed horizontal space.
        (cw - margin_left - margin_right - border_left - border_right - pad_left - pad_right).max(0.0)
    };

    lb.dimensions.content.width = width;
    lb.dimensions.margin.left = margin_left;
    lb.dimensions.margin.right = margin_right;
    lb.dimensions.border.left = border_left;
    lb.dimensions.border.right = border_right;
    lb.dimensions.padding.left = pad_left;
    lb.dimensions.padding.right = pad_right;
}

/// Lay out a single `InlineBlock` element.  Its x/y will be overridden by the
/// parent row layout; this function sets width, vertical edges, and height.
fn layout_inline_block(lb: &mut LayoutBox, containing: &Dimensions, viewport_height: f32, depth: usize) {
    if depth >= 512 {
        return;
    }
    calculate_inline_block_width(lb, containing, viewport_height);

    let s = match &lb.box_type {
        BoxType::InlineBlock(s) => s.clone(),
        _ => return,
    };
    lb.dimensions.margin.top = lookup_length_opt(&s, "margin-top");
    lb.dimensions.margin.bottom = lookup_length_opt(&s, "margin-bottom");
    lb.dimensions.border.top = lookup_length_opt(&s, "border-top-width");
    lb.dimensions.border.bottom = lookup_length_opt(&s, "border-bottom-width");
    lb.dimensions.padding.top = lookup_length_opt(&s, "padding-top");
    lb.dimensions.padding.bottom = lookup_length_opt(&s, "padding-bottom");

    // Default position: bottom of containing block (overridden by row layout).
    lb.dimensions.content.x = containing.content.x
        + lb.dimensions.margin.left
        + lb.dimensions.border.left
        + lb.dimensions.padding.left;
    lb.dimensions.content.y = containing.content.y
        + containing.content.height
        + lb.dimensions.margin.top
        + lb.dimensions.border.top
        + lb.dimensions.padding.top;

    // Internal content layout.
    if all_inline_children(&lb.children) {
        let font_size = match s.properties.get("font-size") {
            Some(Value::Length(n, Unit::Px)) => *n,
            _ => 16.0,
        };
        let color = match s.properties.get("color") {
            Some(Value::Color(c)) => *c,
            _ => CssColor::BLACK,
        };
        let bold = matches!(s.properties.get("font-weight"), Some(Value::Keyword(k)) if k == "bold");
        let italic = matches!(s.properties.get("font-style"), Some(Value::Keyword(k)) if k == "italic" || k == "oblique");
        let decoration = match s.properties.get("text-decoration") {
            Some(Value::Keyword(k)) if k == "underline" => TextDecoration::Underline,
            Some(Value::Keyword(k)) if k == "line-through" => TextDecoration::LineThrough,
            _ => TextDecoration::None,
        };
        let letter_spacing = match s.properties.get("letter-spacing") {
            Some(Value::Length(n, Unit::Px)) => *n,
            _ => 0.0,
        };
        let items = collect_inline_text(&lb.children, font_size, color, bold, italic, decoration, letter_spacing);
        if !items.is_empty() {
            let align = match s.properties.get("text-align") {
                Some(Value::Keyword(k)) => match k.as_str() {
                    "center" => TextAlign::Center,
                    "right" => TextAlign::Right,
                    _ => TextAlign::Left,
                },
                _ => TextAlign::Left,
            };
            let line_height_override = match s.properties.get("line-height") {
                Some(Value::Length(n, Unit::Px)) => Some(*n),
                Some(Value::Number(n)) => Some(*n * font_size),
                _ => None,
            };
            let (nowrap, pre) = match s.properties.get("white-space") {
                Some(Value::Keyword(k)) if k == "nowrap" => (true, false),
                Some(Value::Keyword(k)) if k == "pre" => (false, true),
                _ => (false, false),
            };
            let ifc = InlineFormattingContext {
                font_size,
                available_width: lb.dimensions.content.width.max(1.0),
                align,
                line_height_override,
                nowrap,
                pre,
            };
            let (line_boxes, height) = ifc.layout(&items);
            lb.line_boxes = line_boxes;
            lb.dimensions.content.height = height;
        }
    } else if !lb.children.is_empty() {
        layout_block_children(lb, viewport_height, depth);
    }

    calculate_block_height(lb, viewport_height, containing.content.width);
}

/// Position all inline-block children of `lb` in a horizontal row flow,
/// wrapping to the next line when a child would overflow the container width.
fn layout_inline_block_row(lb: &mut LayoutBox, viewport_height: f32, depth: usize) {
    if depth >= 512 {
        return;
    }
    let cw = lb.dimensions.content.width;
    let ox = lb.dimensions.content.x;
    let oy = lb.dimensions.content.y;
    let mut cx: f32 = 0.0;
    let mut cy: f32 = 0.0;
    let mut row_h: f32 = 0.0;

    // Build a fake containing Dimensions so calculate_inline_block_width has a width.
    let fake_containing = Dimensions {
        content: Rect { x: ox, y: oy, width: cw, height: 0.0 },
        ..Default::default()
    };

    for child in &mut lb.children {
        if !matches!(child.box_type, BoxType::InlineBlock(_)) {
            continue;
        }
        calculate_inline_block_width(child, &fake_containing, viewport_height);

        let s = match &child.box_type {
            BoxType::InlineBlock(s) => s.clone(),
            _ => unreachable!(),
        };
        child.dimensions.margin.top = lookup_length_opt(&s, "margin-top");
        child.dimensions.margin.bottom = lookup_length_opt(&s, "margin-bottom");
        child.dimensions.border.top = lookup_length_opt(&s, "border-top-width");
        child.dimensions.border.bottom = lookup_length_opt(&s, "border-bottom-width");
        child.dimensions.padding.top = lookup_length_opt(&s, "padding-top");
        child.dimensions.padding.bottom = lookup_length_opt(&s, "padding-bottom");

        let mb_w = child.dimensions.margin.left
            + child.dimensions.border.left
            + child.dimensions.padding.left
            + child.dimensions.content.width
            + child.dimensions.padding.right
            + child.dimensions.border.right
            + child.dimensions.margin.right;

        // Wrap when the child doesn't fit and we're not at the start of a row.
        if cx + mb_w > cw + 0.5 && cx > 0.5 {
            cy += row_h;
            cx = 0.0;
            row_h = 0.0;
        }

        // Place the child.
        child.dimensions.content.x = ox
            + cx
            + child.dimensions.margin.left
            + child.dimensions.border.left
            + child.dimensions.padding.left;
        child.dimensions.content.y = oy
            + cy
            + child.dimensions.margin.top
            + child.dimensions.border.top
            + child.dimensions.padding.top;

        // Layout the inline-block's own internal content.
        if all_inline_children(&child.children) {
            let font_size = match s.properties.get("font-size") {
                Some(Value::Length(n, Unit::Px)) => *n,
                _ => 16.0,
            };
            let color = match s.properties.get("color") {
                Some(Value::Color(c)) => *c,
                _ => CssColor::BLACK,
            };
            let bold = matches!(s.properties.get("font-weight"), Some(Value::Keyword(k)) if k == "bold");
            let italic = matches!(s.properties.get("font-style"), Some(Value::Keyword(k)) if k == "italic" || k == "oblique");
            let decoration = match s.properties.get("text-decoration") {
                Some(Value::Keyword(k)) if k == "underline" => TextDecoration::Underline,
                Some(Value::Keyword(k)) if k == "line-through" => TextDecoration::LineThrough,
                _ => TextDecoration::None,
            };
            let letter_spacing = match s.properties.get("letter-spacing") {
                Some(Value::Length(n, Unit::Px)) => *n,
                _ => 0.0,
            };
            let items = collect_inline_text(&child.children, font_size, color, bold, italic, decoration, letter_spacing);
            if !items.is_empty() {
                let align = match s.properties.get("text-align") {
                    Some(Value::Keyword(k)) => match k.as_str() {
                        "center" => TextAlign::Center,
                        "right" => TextAlign::Right,
                        _ => TextAlign::Left,
                    },
                    _ => TextAlign::Left,
                };
                let line_height_override = match s.properties.get("line-height") {
                    Some(Value::Length(n, Unit::Px)) => Some(*n),
                    Some(Value::Number(n)) => Some(*n * font_size),
                    _ => None,
                };
                let (nowrap, pre) = match s.properties.get("white-space") {
                    Some(Value::Keyword(k)) if k == "nowrap" => (true, false),
                    Some(Value::Keyword(k)) if k == "pre" => (false, true),
                    _ => (false, false),
                };
                let ifc = InlineFormattingContext {
                    font_size,
                    available_width: child.dimensions.content.width.max(1.0),
                    align,
                    line_height_override,
                    nowrap,
                    pre,
                };
                let (line_boxes, height) = ifc.layout(&items);
                child.line_boxes = line_boxes;
                child.dimensions.content.height = height;
            }
        } else if !child.children.is_empty() {
            layout_block_children(child, viewport_height, depth + 1);
        }

        calculate_block_height(child, viewport_height, cw);

        let mb_h = child.dimensions.margin.top
            + child.dimensions.border.top
            + child.dimensions.padding.top
            + child.dimensions.content.height
            + child.dimensions.padding.bottom
            + child.dimensions.border.bottom
            + child.dimensions.margin.bottom;

        cx += mb_w;
        row_h = row_h.max(mb_h);
    }

    lb.dimensions.content.height = cy + row_h;
}
