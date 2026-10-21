// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{PropertyMap, Rule, Selector, Stylesheet};
use crate::engine::dom::{ElementData, Node, NodeType};

/// A DOM node with its computed CSS properties.
#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: Node,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode>,
}

/// Build a styled tree by matching every element against the stylesheet.
/// Text and Comment nodes carry no properties; their children list is empty.
pub fn style_tree(root: &Node, stylesheet: &Stylesheet) -> StyledNode {
    let properties = match &root.node_type {
        NodeType::Element(data) => specified_values(data, stylesheet),
        _ => PropertyMap::new(),
    };
    let children = root.children.iter().map(|c| style_tree(c, stylesheet)).collect();
    StyledNode { node: root.clone(), properties, children }
}

/// Collect all CSS declarations that apply to this element, cascade resolved.
fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut rules: Vec<(&Rule, (usize, usize, usize))> = stylesheet
        .rules
        .iter()
        .filter_map(|rule| {
            rule.selectors
                .iter()
                .filter(|sel| selector_matches(elem, sel))
                .map(|sel| sel.specificity())
                .max()
                .map(|spec| (rule, spec))
        })
        .collect();

    // Higher specificity wins; stable sort preserves source order within same specificity.
    rules.sort_by_key(|(_, spec)| *spec);

    let mut map = PropertyMap::new();
    for (rule, _) in rules {
        for (name, value) in &rule.declarations {
            map.insert(name.clone(), value.clone());
        }
    }
    map
}

fn selector_matches(elem: &ElementData, sel: &Selector) -> bool {
    if let Some(ref tag) = sel.tag_name {
        if elem.tag_name != *tag { return false; }
    }
    if let Some(ref id) = sel.id {
        if elem.attributes.get("id").map(|s| s.as_str()) != Some(id) { return false; }
    }
    let class_attr = elem.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
    let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
    for cls in &sel.classes {
        if !elem_classes.contains(&cls.as_str()) { return false; }
    }
    true
}
