// CSS Grid Layout Module

#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    Fixed(f32),           // px, em, etc.
    Flex(f32),            // fr units
    MinContent,
    MaxContent,
    Auto,
    MinMax(Box<GridTrack>, Box<GridTrack>),
    FitContent(f32),
    Repeat(usize, Vec<GridTrack>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyItems {
    Start,
    End,
    Center,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignContent {
    Start,
    End,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone)]
pub struct GridContainer {
    pub template_rows: Vec<GridTrack>,
    pub template_columns: Vec<GridTrack>,
    pub auto_rows: Vec<GridTrack>,
    pub auto_columns: Vec<GridTrack>,
    pub auto_flow: GridAutoFlow,
    pub row_gap: f32,
    pub column_gap: f32,
    pub justify_items: JustifyItems,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub align_content: AlignContent,
}

#[derive(Debug, Clone, Copy)]
pub struct GridPlacement {
    pub row_start: i32,
    pub row_end: i32,
    pub column_start: i32,
    pub column_end: i32,
}

#[derive(Debug, Clone)]
pub struct GridItem {
    pub placement: GridPlacement,
    pub justify_self: Option<JustifyItems>,
    pub align_self: Option<AlignItems>,
}

impl Default for GridAutoFlow {
    fn default() -> Self {
        GridAutoFlow::Row
    }
}

impl Default for JustifyItems {
    fn default() -> Self {
        JustifyItems::Stretch
    }
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::Start
    }
}

impl Default for AlignContent {
    fn default() -> Self {
        AlignContent::Start
    }
}

impl Default for GridContainer {
    fn default() -> Self {
        GridContainer {
            template_rows: Vec::new(),
            template_columns: Vec::new(),
            auto_rows: Vec::new(),
            auto_columns: Vec::new(),
            auto_flow: GridAutoFlow::default(),
            row_gap: 0.0,
            column_gap: 0.0,
            justify_items: JustifyItems::default(),
            align_items: AlignItems::default(),
            justify_content: JustifyContent::default(),
            align_content: AlignContent::default(),
        }
    }
}

impl Default for GridPlacement {
    fn default() -> Self {
        GridPlacement {
            row_start: 0,
            row_end: 0,
            column_start: 0,
            column_end: 0,
        }
    }
}

impl Default for GridItem {
    fn default() -> Self {
        GridItem {
            placement: GridPlacement::default(),
            justify_self: None,
            align_self: None,
        }
    }
}

// Grid track parsing
pub fn parse_grid_track(value: &str) -> Option<GridTrack> {
    if value.ends_with("fr") {
        value.strip_suffix("fr")?.parse::<f32>().ok().map(GridTrack::Flex)
    } else if value.ends_with("px") {
        value.strip_suffix("px")?.parse::<f32>().ok().map(GridTrack::Fixed)
    } else if value == "min-content" {
        Some(GridTrack::MinContent)
    } else if value == "max-content" {
        Some(GridTrack::MaxContent)
    } else if value == "auto" {
        Some(GridTrack::Auto)
    } else {
        None
    }
}

pub fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlow> {
    match value {
        "row" => Some(GridAutoFlow::Row),
        "column" => Some(GridAutoFlow::Column),
        "row dense" => Some(GridAutoFlow::RowDense),
        "column dense" => Some(GridAutoFlow::ColumnDense),
        _ => None,
    }
}
