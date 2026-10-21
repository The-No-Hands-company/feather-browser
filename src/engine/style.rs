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
    style_tree_depth(root, stylesheet, 0, &mut Vec::new())
}

fn style_tree_depth(
    root: &Node,
    stylesheet: &Stylesheet,
    depth: usize,
    ancestors: &mut Vec<ElementData>,
) -> StyledNode {
    if depth >= 512 {
        return StyledNode { node: root.clone(), properties: PropertyMap::new(), children: Vec::new() };
    }
    let properties = match &root.node_type {
        NodeType::Element(data) => specified_values(data, stylesheet, ancestors),
        _ => PropertyMap::new(),
    };
    let pushed = if let NodeType::Element(data) = &root.node_type {
        ancestors.push(data.clone());
        true
    } else {
        false
    };
    let mut children = Vec::new();
    for c in &root.children {
        children.push(style_tree_depth(c, stylesheet, depth + 1, ancestors));
    }
    if pushed { ancestors.pop(); }
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

fn simple_matches(elem: &ElementData, sel: &Selector) -> bool {
    if let Some(ref tag) = sel.tag_name {
        if elem.tag_name != *tag { return false; }
    }
    if let Some(ref id) = sel.id {
        if elem.attributes.get("id").map(|s| s.as_str()) != Some(id.as_str()) {
            return false;
        }
    }
    let class_attr = elem.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
    let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
    for cls in &sel.classes {
        if !elem_classes.contains(&cls.as_str()) { return false; }
    }
    true
}

fn selector_matches(elem: &ElementData, sel: &Selector, ancestors: &[ElementData]) -> bool {
    if !simple_matches(elem, sel) { return false; }

    // Flatten the ancestor chain, innermost first. tokens[j] is the j-th compound
    // selector above the subject; rels[j] links tokens[j] to the level beneath it
    // (the subject for j == 0, tokens[j - 1] otherwise).
    let mut tokens: Vec<&Selector> = Vec::new();
    let mut rels: Vec<&Combinator> = Vec::new();
    let mut current: &Selector = sel;
    while let Some((combinator, parent_sel)) = &current.ancestor {
        rels.push(combinator);
        tokens.push(parent_sel.as_ref());
        current = parent_sel.as_ref();
    }
    let n = tokens.len();
    if n == 0 { return true; }

    // Left-to-right DP over ancestors (outermost first is not needed: we walk
    // from the parent outwards). row[j] records whether tokens[j] can be bound to
    // the current ancestor index with a valid chain beneath it. This avoids the
    // greedy nearest-match pitfall for mixed `>` / descendant chains and keeps the
    // cost at O(depth x chain length) per element.
    let depth = ancestors.len();
    let mut prev = vec![false; n]; // bindings at ancestor index i + 1
    let mut row = vec![false; n];  // bindings at ancestor index i
    let mut seen = vec![false; n]; // bindings at any ancestor index > i
    for i in (0..depth).rev() {
        for j in 0..n {
            let linked = if j == 0 {
                match rels[0] {
                    Combinator::Child => i + 1 == depth,
                    Combinator::Descendant => true,
                }
            } else {
                match rels[j] {
                    Combinator::Child => prev[j - 1],
                    Combinator::Descendant => seen[j - 1],
                }
            };
            row[j] = linked && simple_matches(&ancestors[i], tokens[j]);
        }
        for j in 0..n {
            seen[j] |= row[j];
        }
        std::mem::swap(&mut prev, &mut row);
    }
    seen[n - 1]
}
