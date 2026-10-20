// CSS Selector Matching - Matches CSS selectors to DOM elements

use crate::dom::{Node, NodeType, ElementData};
use crate::css::{Selector, Stylesheet, Rule, Value};
use crate::layout::StyledNode;
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, Value>;

/// Matches a selector against a DOM element
pub fn matches(elem: &ElementData, selector: &Selector) -> bool {
    // Check tag name
    if let Some(ref tag_name) = selector.tag_name {
        if elem.tag_name != *tag_name {
            return false;
        }
    }

    // Check ID
    if let Some(ref id) = selector.id {
        if elem.attributes.get("id") != Some(id) {
            return false;
        }
    }

    // Check classes
    if let Some(ref class_list) = elem.attributes.get("class") {
        let elem_classes: Vec<&str> = class_list.split_whitespace().collect();
        for class in &selector.classes {
            if !elem_classes.contains(&class.as_str()) {
                return false;
            }
        }
    } else if !selector.classes.is_empty() {
        return false;
    }

    true
}

/// Calculate specificity of a selector (for cascade resolution)
pub fn specificity(selector: &Selector) -> Specificity {
    let (a, b, c) = selector.specificity();
    Specificity(a, b, c)
}

/// Specificity (a, b, c) where a=IDs, b=classes, c=tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(usize, usize, usize);

type MatchedRule<'a> = (Specificity, &'a Rule);

/// Find all CSS rules that match an element
pub fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    let mut matched = Vec::new();
    
    for rule in &stylesheet.rules {
        for selector in &rule.selectors {
            if matches(elem, selector) {
                matched.push((specificity(selector), rule));
            }
        }
    }
    
    matched
}

/// Apply CSS rules to an element, respecting specificity
pub fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);
    
    // Sort by specificity (lowest to highest)
    rules.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Apply rules in order (higher specificity overwrites lower)
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    
    values
}

/// Build a styled tree by applying CSS to DOM
pub fn style_tree(root: &Node, stylesheet: &Stylesheet) -> StyledNode {
    let properties = match &root.node_type {
        NodeType::Element(elem) => specified_values(elem, stylesheet),
        NodeType::Text(_) | NodeType::Comment(_) => HashMap::new(),
    };
    
    let mut styled_node = StyledNode::new(root.clone(), properties);
    
    // Recursively style children
    for child in &root.children {
        styled_node.children.push(style_tree(child, stylesheet));
    }
    
    styled_node
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_matches_tag() {
        let attrs = HashMap::new();
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: attrs,
        };
        
        let selector = Selector {
            tag_name: Some("div".to_string()),
            id: None,
            classes: Vec::new(),
        };
        
        assert!(matches(&elem, &selector));
    }
    
    #[test]
    fn test_matches_id() {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "main".to_string());
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: attrs,
        };
        
        let selector = Selector {
            tag_name: None,
            id: Some("main".to_string()),
            classes: Vec::new(),
        };
        
        assert!(matches(&elem, &selector));
    }
    
    #[test]
    fn test_matches_class() {
        let mut attrs = HashMap::new();
        attrs.insert("class".to_string(), "container main".to_string());
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: attrs,
        };
        
        let selector = Selector {
            tag_name: None,
            id: None,
            classes: vec!["container".to_string()],
        };
        
        assert!(matches(&elem, &selector));
    }
}
