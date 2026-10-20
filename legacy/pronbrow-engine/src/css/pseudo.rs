// CSS Pseudo-classes and Pseudo-elements Module

#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    // User action pseudo-classes
    Hover,
    Active,
    Focus,
    FocusVisible,
    FocusWithin,
    
    // Input pseudo-classes
    Enabled,
    Disabled,
    ReadOnly,
    ReadWrite,
    PlaceholderShown,
    Default,
    Checked,
    Indeterminate,
    Blank,
    Valid,
    Invalid,
    InRange,
    OutOfRange,
    Required,
    Optional,
    
    // Location pseudo-classes
    Link,
    Visited,
    Target,
    TargetWithin,
    
    // Tree-structural pseudo-classes
    Root,
    Empty,
    FirstChild,
    LastChild,
    OnlyChild,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    NthChild(i32, i32),           // an + b
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    
    // Linguistic pseudo-classes
    Lang(String),
    Dir(TextDirection),
    
    // Negation and matching
    Not(Vec<PseudoClass>),
    Is(Vec<PseudoClass>),
    Where(Vec<PseudoClass>),
    Has(Vec<PseudoClass>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
    Selection,
    Backdrop,
    Placeholder,
    Marker,
    FileSelectorButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone)]
pub struct PseudoState {
    pub hover: bool,
    pub active: bool,
    pub focus: bool,
    pub visited: bool,
    pub checked: bool,
    pub disabled: bool,
    pub enabled: bool,
}

impl Default for PseudoState {
    fn default() -> Self {
        PseudoState {
            hover: false,
            active: false,
            focus: false,
            visited: false,
            checked: false,
            disabled: false,
            enabled: true,
        }
    }
}

impl PseudoState {
    pub fn matches(&self, pseudo_class: &PseudoClass) -> bool {
        match pseudo_class {
            PseudoClass::Hover => self.hover,
            PseudoClass::Active => self.active,
            PseudoClass::Focus => self.focus,
            PseudoClass::Visited => self.visited,
            PseudoClass::Checked => self.checked,
            PseudoClass::Disabled => self.disabled,
            PseudoClass::Enabled => self.enabled,
            _ => false,
        }
    }
}

pub fn parse_pseudo_class(value: &str) -> Option<PseudoClass> {
    match value.trim() {
        "hover" => Some(PseudoClass::Hover),
        "active" => Some(PseudoClass::Active),
        "focus" => Some(PseudoClass::Focus),
        "focus-visible" => Some(PseudoClass::FocusVisible),
        "focus-within" => Some(PseudoClass::FocusWithin),
        "enabled" => Some(PseudoClass::Enabled),
        "disabled" => Some(PseudoClass::Disabled),
        "read-only" => Some(PseudoClass::ReadOnly),
        "read-write" => Some(PseudoClass::ReadWrite),
        "checked" => Some(PseudoClass::Checked),
        "valid" => Some(PseudoClass::Valid),
        "invalid" => Some(PseudoClass::Invalid),
        "required" => Some(PseudoClass::Required),
        "optional" => Some(PseudoClass::Optional),
        "link" => Some(PseudoClass::Link),
        "visited" => Some(PseudoClass::Visited),
        "target" => Some(PseudoClass::Target),
        "root" => Some(PseudoClass::Root),
        "empty" => Some(PseudoClass::Empty),
        "first-child" => Some(PseudoClass::FirstChild),
        "last-child" => Some(PseudoClass::LastChild),
        "only-child" => Some(PseudoClass::OnlyChild),
        "first-of-type" => Some(PseudoClass::FirstOfType),
        "last-of-type" => Some(PseudoClass::LastOfType),
        "only-of-type" => Some(PseudoClass::OnlyOfType),
        _ => None,
    }
}

pub fn parse_pseudo_element(value: &str) -> Option<PseudoElement> {
    match value.trim() {
        "before" => Some(PseudoElement::Before),
        "after" => Some(PseudoElement::After),
        "first-line" => Some(PseudoElement::FirstLine),
        "first-letter" => Some(PseudoElement::FirstLetter),
        "selection" => Some(PseudoElement::Selection),
        "backdrop" => Some(PseudoElement::Backdrop),
        "placeholder" => Some(PseudoElement::Placeholder),
        "marker" => Some(PseudoElement::Marker),
        "file-selector-button" => Some(PseudoElement::FileSelectorButton),
        _ => None,
    }
}

// Parse nth expressions like "2n+1", "odd", "even", etc.
pub fn parse_nth(value: &str) -> Option<(i32, i32)> {
    let value = value.trim();
    
    match value {
        "odd" => Some((2, 1)),
        "even" => Some((2, 0)),
        _ => {
            if let Ok(n) = value.parse::<i32>() {
                Some((0, n))
            } else if value.contains('n') {
                // Parse "an+b" format
                let parts: Vec<&str> = value.split('n').collect();
                let a = if parts[0].is_empty() || parts[0] == "+" {
                    1
                } else if parts[0] == "-" {
                    -1
                } else {
                    parts[0].parse().ok()?
                };
                
                let b = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1].trim().parse().unwrap_or(0)
                } else {
                    0
                };
                
                Some((a, b))
            } else {
                None
            }
        }
    }
}
