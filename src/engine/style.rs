// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::engine::css::{Combinator, PropertyMap, Rule, Selector, Stylesheet};
use crate::engine::dom::{ElementData, Node, NodeType};

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: Node,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode>,
}

pub fn style_tree(root: &Node, stylesheet: &Stylesheet) -> StyledNode {
    style_tree_depth(root, stylesheet, 0, &[])
}

fn style_tree_depth(
    root: &Node,
    stylesheet: &Stylesheet,
    depth: usize,
    ancestors: &[ElementData],
) -> StyledNode {
    if depth >= 512 {
        return StyledNode { node: root.clone(), properties: PropertyMap::new(), children: Vec::new() };
    }
    let properties = match &root.node_type {
        NodeType::Element(data) => specified_values(data, stylesheet, ancestors),
        _ => PropertyMap::new(),
    };
    let children = root.children.iter().map(|c| {
        let mut child_ancestors = ancestors.to_vec();
        if let NodeType::Element(data) = &root.node_type {
            child_ancestors.push(data.clone());
        }
        style_tree_depth(c, stylesheet, depth + 1, &child_ancestors)
    }).collect();
    StyledNode { node: root.clone(), properties, children }
}

fn specified_values(
    elem: &ElementData,
    stylesheet: &Stylesheet,
    ancestors: &[ElementData],
) -> PropertyMap {
    let mut rules: Vec<(&Rule, (usize, usize, usize))> = stylesheet
        .rules
        .iter()
        .filter_map(|rule| {
            rule.selectors
                .iter()
                .filter(|sel| selector_matches(elem, sel, ancestors))
                .map(|sel| sel.specificity())
                .max()
                .map(|spec| (rule, spec))
        })
        .collect();
    rules.sort_by_key(|(_, spec)| *spec);
    let mut map = PropertyMap::new();
    for (rule, _) in rules {
        for (name, value) in &rule.declarations {
            map.insert(name.clone(), value.clone());
        }
    }
    map
}

fn selector_matches(elem: &ElementData, sel: &Selector, ancestors: &[ElementData]) -> bool {
    if let Some(ref tag) = sel.tag_name {
        if elem.tag_name != *tag { return false; }
    }
    if let Some(ref id) = sel.id {
        if elem.attributes.get("id").map(|s| s.as_str()) != Some(id.as_str()) { return false; }
    }
    let class_attr = elem.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
    let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
    for cls in &sel.classes {
        if !elem_classes.contains(&cls.as_str()) { return false; }
    }
    if let Some((combinator, ancestor_sel)) = &sel.ancestor {
        match combinator {
            Combinator::Child => {
                match ancestors.last() {
                    Some(parent) => {
                        if !selector_matches(parent, ancestor_sel, &ancestors[..ancestors.len() - 1]) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            Combinator::Descendant => {
                let found = ancestors.iter().enumerate().any(|(i, anc)| {
                    selector_matches(anc, ancestor_sel, &ancestors[..i])
                });
                if !found { return false; }
            }
        }
    }
    true
}
