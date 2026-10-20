// Flexbox Layout Module - CSS Flexible Box Layout

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    Nowrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FlexContainer {
    pub direction: Option<FlexDirection>,
    pub wrap: Option<FlexWrap>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub gap: f32,
    pub row_gap: f32,
    pub column_gap: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FlexItem {
    pub grow: f32,
    pub shrink: f32,
    pub basis: f32,
    pub align_self: Option<AlignSelf>,
    pub order: i32,
}

impl Default for FlexDirection {
    fn default() -> Self {
        FlexDirection::Row
    }
}

impl Default for FlexWrap {
    fn default() -> Self {
        FlexWrap::Nowrap
    }
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::FlexStart
    }
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

impl Default for AlignContent {
    fn default() -> Self {
        AlignContent::Stretch
    }
}

impl Default for AlignSelf {
    fn default() -> Self {
        AlignSelf::Auto
    }
}

pub fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
    match value {
        "row" => Some(FlexDirection::Row),
        "row-reverse" => Some(FlexDirection::RowReverse),
        "column" => Some(FlexDirection::Column),
        "column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

pub fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
    match value {
        "nowrap" => Some(FlexWrap::Nowrap),
        "wrap" => Some(FlexWrap::Wrap),
        "wrap-reverse" => Some(FlexWrap::WrapReverse),
        _ => None,
    }
}

pub fn parse_justify_content(value: &str) -> Option<JustifyContent> {
    match value {
        "flex-start" => Some(JustifyContent::FlexStart),
        "flex-end" => Some(JustifyContent::FlexEnd),
        "center" => Some(JustifyContent::Center),
        "space-between" => Some(JustifyContent::SpaceBetween),
        "space-around" => Some(JustifyContent::SpaceAround),
        "space-evenly" => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

pub fn parse_align_items(value: &str) -> Option<AlignItems> {
    match value {
        "flex-start" => Some(AlignItems::FlexStart),
        "flex-end" => Some(AlignItems::FlexEnd),
        "center" => Some(AlignItems::Center),
        "baseline" => Some(AlignItems::Baseline),
        "stretch" => Some(AlignItems::Stretch),
        _ => None,
    }
}

pub fn parse_align_content(value: &str) -> Option<AlignContent> {
    match value {
        "flex-start" => Some(AlignContent::FlexStart),
        "flex-end" => Some(AlignContent::FlexEnd),
        "center" => Some(AlignContent::Center),
        "space-between" => Some(AlignContent::SpaceBetween),
        "space-around" => Some(AlignContent::SpaceAround),
        "stretch" => Some(AlignContent::Stretch),
        _ => None,
    }
}

pub fn parse_align_self(value: &str) -> Option<AlignSelf> {
    match value {
        "auto" => Some(AlignSelf::Auto),
        "flex-start" => Some(AlignSelf::FlexStart),
        "flex-end" => Some(AlignSelf::FlexEnd),
        "center" => Some(AlignSelf::Center),
        "baseline" => Some(AlignSelf::Baseline),
        "stretch" => Some(AlignSelf::Stretch),
        _ => None,
    }
}
