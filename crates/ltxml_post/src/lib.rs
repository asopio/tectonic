// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Post-processing for LaTeXML-shaped IR.
//!
//! This crate implements early post-processing for `LtxmlIr`: collect labelled
//! targets and bibliography items, resolve `ltx:ref` and `ltx:bibref` nodes,
//! and normalize graphics/table structures suitable for HTML rendering.

use std::collections::BTreeMap;
use tectonic_ltxml_ir::{Diagnostic, LtxmlChild, LtxmlNode, QName};

/// Summary of one resolved label target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelTarget {
    /// Target XML ID.
    pub id: String,
    /// Target element name.
    pub element: QName,
    /// Target reference number, if available.
    pub refnum: Option<String>,
    /// Target title, if available.
    pub title: Option<String>,
}

/// Summary of one bibliography target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BibliographyTarget {
    /// Bibliography key.
    pub key: String,
    /// Target XML ID.
    pub id: String,
    /// Citation number, if available.
    pub number: Option<String>,
    /// Bibliography item title/text, if available.
    pub title: Option<String>,
}

/// Cross-reference and bibliography index built from an IR tree.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReferenceIndex {
    /// Label lookup table.
    pub labels: BTreeMap<String, LabelTarget>,
    /// Bibliography key lookup table.
    pub bibliography: BTreeMap<String, BibliographyTarget>,
}

/// Run the Phase 5 post-processing pass in place and return the index used.
pub fn resolve_references(root: &mut LtxmlNode) -> ReferenceIndex {
    let mut index = ReferenceIndex::default();
    collect_index(root, &mut index);
    resolve_nodes(root, &index);
    index
}

/// Summary of graphics/table normalization work.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphicsTableSummary {
    /// Number of `ltx:graphics` nodes whose `imagesrc` was filled in.
    pub graphics_resolved: usize,
    /// Number of `ltx:graphics` nodes missing accessible descriptions.
    pub missing_graphics_descriptions: usize,
    /// Number of `ltx:tabular` nodes whose direct rows were wrapped in `ltx:tbody`.
    pub tabular_bodies_added: usize,
}

/// Normalize Phase 7 graphics and table structures in place.
pub fn normalize_graphics_and_tables(root: &mut LtxmlNode) -> GraphicsTableSummary {
    let mut summary = GraphicsTableSummary::default();
    normalize_graphics_and_tables_inner(root, &mut summary);
    summary
}

fn collect_index(node: &LtxmlNode, index: &mut ReferenceIndex) {
    collect_labels(node, index);
    collect_bibliography(node, index);

    for child in &node.children {
        if let LtxmlChild::Element { node } = child {
            collect_index(node, index);
        }
    }
}

fn collect_labels(node: &LtxmlNode, index: &mut ReferenceIndex) {
    let Some(id) = node.xml_id() else {
        return;
    };

    let labels = node.labels().collect::<Vec<_>>();
    if labels.is_empty() {
        return;
    }

    let target = LabelTarget {
        id: id.to_owned(),
        element: node.name.clone(),
        refnum: node.attr("refnum").map(ToOwned::to_owned),
        title: first_title_text(node),
    };

    for label in labels {
        index.labels.insert(normalize_label(label), target.clone());
    }
}

fn collect_bibliography(node: &LtxmlNode, index: &mut ReferenceIndex) {
    if node.name.as_str() != "ltx:bibitem" {
        return;
    }

    let Some(key) = node.attr("key") else {
        return;
    };

    let Some(id) = node.xml_id() else {
        return;
    };

    index.bibliography.insert(
        key.to_owned(),
        BibliographyTarget {
            key: key.to_owned(),
            id: id.to_owned(),
            number: node.attr("refnum").map(ToOwned::to_owned),
            title: first_title_text(node).or_else(|| Some(text_content(node))),
        },
    );
}

fn resolve_nodes(node: &mut LtxmlNode, index: &ReferenceIndex) {
    match node.name.as_str() {
        "ltx:ref" => resolve_ref(node, index),
        "ltx:bibref" => resolve_bibref(node, index),
        _ => {}
    }

    for child in &mut node.children {
        if let LtxmlChild::Element { node } = child {
            resolve_nodes(node, index);
        }
    }
}

fn resolve_ref(node: &mut LtxmlNode, index: &ReferenceIndex) {
    let Some(labelref) = node.attr("labelref").map(normalize_label) else {
        return;
    };

    let Some(target) = index.labels.get(&labelref) else {
        node.diagnostics.push(Diagnostic::warning(format!(
            "unresolved label reference `{labelref}`"
        )));
        return;
    };

    node.attrs.insert(QName::from("idref"), target.id.clone());
    node.attrs.insert(QName::from("href"), format!("#{}", target.id));

    if let Some(title) = &target.title {
        node.attrs.insert(QName::from("title"), title.clone());
    }

    if node.children.is_empty() {
        let text = target
            .refnum
            .as_deref()
            .or(target.title.as_deref())
            .unwrap_or(target.id.as_str());
        node.push_text(text);
    }
}

fn resolve_bibref(node: &mut LtxmlNode, index: &ReferenceIndex) {
    let Some(bibrefs) = node.attr("bibrefs").map(ToOwned::to_owned) else {
        return;
    };

    let keys = bibrefs
        .split(',')
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>();

    if keys.is_empty() {
        return;
    }

    let mut ids = Vec::new();
    let mut labels = Vec::new();
    let mut unresolved = Vec::new();

    for key in keys {
        if let Some(target) = index.bibliography.get(key) {
            ids.push(target.id.clone());
            labels.push(target.number.clone().unwrap_or_else(|| target.key.clone()));
        } else {
            unresolved.push(key.to_owned());
        }
    }

    if !ids.is_empty() {
        node.attrs.insert(QName::from("idref"), ids.join(" "));
        node.attrs.insert(QName::from("href"), format!("#{}", ids[0]));
    }

    if node.children.is_empty() && !labels.is_empty() {
        node.push_text(format!("[{}]", labels.join(", ")));
    }

    for key in unresolved {
        node.diagnostics.push(Diagnostic::warning(format!(
            "unresolved bibliography reference `{key}`"
        )));
    }
}

fn normalize_graphics_and_tables_inner(node: &mut LtxmlNode, summary: &mut GraphicsTableSummary) {
    match node.name.as_str() {
        "ltx:graphics" => normalize_graphics(node, summary),
        "ltx:tabular" => normalize_tabular(node, summary),
        _ => {}
    }

    for child in &mut node.children {
        if let LtxmlChild::Element { node } = child {
            normalize_graphics_and_tables_inner(node, summary);
        }
    }
}

fn normalize_graphics(node: &mut LtxmlNode, summary: &mut GraphicsTableSummary) {
    if node.attr("imagesrc").is_none() {
        if let Some(src) = resolved_graphics_source(node) {
            node.attrs.insert(QName::from("imagesrc"), src);
            summary.graphics_resolved += 1;
        }
    }

    if node.attr("description").is_none() {
        node.diagnostics
            .push(Diagnostic::warning("graphics node is missing an accessible description"));
        summary.missing_graphics_descriptions += 1;
    }
}

fn resolved_graphics_source(node: &LtxmlNode) -> Option<String> {
    if let Some(candidates) = node.attr("candidates") {
        if let Some(candidate) = candidates
            .split(',')
            .map(str::trim)
            .find(|candidate| is_web_graphics_path(candidate))
        {
            return Some(candidate.to_owned());
        }
    }

    let graphic = node.attr("graphic")?;
    if is_web_graphics_path(graphic) || has_extension(graphic) {
        Some(graphic.to_owned())
    } else {
        Some(format!("{graphic}.png"))
    }
}

fn normalize_tabular(node: &mut LtxmlNode, summary: &mut GraphicsTableSummary) {
    let has_direct_rows = node.children.iter().any(|child| match child {
        LtxmlChild::Element { node } => node.name.as_str() == "ltx:tr",
        LtxmlChild::Text { .. } => false,
    });

    let has_sections = node.children.iter().any(|child| match child {
        LtxmlChild::Element { node } => matches!(node.name.as_str(), "ltx:thead" | "ltx:tbody" | "ltx:tfoot"),
        LtxmlChild::Text { .. } => false,
    });

    if !has_direct_rows || has_sections {
        return;
    }

    let mut tbody = LtxmlNode::new("ltx:tbody");
    let mut new_children = Vec::new();

    for child in std::mem::take(&mut node.children) {
        match child {
            LtxmlChild::Element { node } if node.name.as_str() == "ltx:tr" => tbody.push_element(node),
            other => new_children.push(other),
        }
    }

    new_children.push(LtxmlChild::Element { node: tbody });
    node.children = new_children;
    summary.tabular_bodies_added += 1;
}

fn is_web_graphics_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn has_extension(path: &str) -> bool {
    path.rsplit_once('/').map_or(path, |(_, file)| file).contains('.')
}

fn normalize_label(label: impl AsRef<str>) -> String {
    label.as_ref().strip_prefix("LABEL:").unwrap_or(label.as_ref()).to_owned()
}

fn first_title_text(node: &LtxmlNode) -> Option<String> {
    node.children.iter().find_map(|child| match child {
        LtxmlChild::Element { node } if node.name.as_str() == "ltx:title" => Some(text_content(node)),
        _ => None,
    })
}

fn text_content(node: &LtxmlNode) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text
}

fn collect_text(node: &LtxmlNode, text: &mut String) {
    for child in &node.children {
        match child {
            LtxmlChild::Element { node } => collect_text(node, text),
            LtxmlChild::Text { text: child_text } => text.push_str(child_text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_label_ref() {
        let mut root = LtxmlNode::document();
        let mut section = LtxmlNode::new("ltx:section")
            .with_attr("xml:id", "S1")
            .with_attr("labels", "LABEL:sec:intro")
            .with_attr("refnum", "1");
        let mut title = LtxmlNode::new("ltx:title");
        title.push_text("Introduction");
        section.push_element(title);
        root.push_element(section);
        root.push_element(LtxmlNode::new("ltx:ref").with_attr("labelref", "sec:intro"));

        let index = resolve_references(&mut root);
        assert_eq!(index.labels.len(), 1);

        let LtxmlChild::Element { node } = &root.children[1] else {
            panic!("expected ref node");
        };
        assert_eq!(node.attr("idref"), Some("S1"));
        assert_eq!(node.attr("href"), Some("#S1"));
        assert_eq!(text_content(node), "1");
    }

    #[test]
    fn normalizes_graphics_and_tabular() {
        let mut root = LtxmlNode::document();
        root.push_element(LtxmlNode::new("ltx:graphics").with_attr("graphic", "plot"));

        let mut tabular = LtxmlNode::new("ltx:tabular");
        let mut row = LtxmlNode::new("ltx:tr");
        row.push_element(LtxmlNode::new("ltx:td"));
        tabular.push_element(row);
        root.push_element(tabular);

        let summary = normalize_graphics_and_tables(&mut root);
        assert_eq!(summary.graphics_resolved, 1);
        assert_eq!(summary.missing_graphics_descriptions, 1);
        assert_eq!(summary.tabular_bodies_added, 1);

        let LtxmlChild::Element { node: graphics } = &root.children[0] else {
            panic!("expected graphics node");
        };
        assert_eq!(graphics.attr("imagesrc"), Some("plot.png"));
        assert_eq!(graphics.diagnostics.len(), 1);

        let LtxmlChild::Element { node: tabular } = &root.children[1] else {
            panic!("expected tabular node");
        };
        let LtxmlChild::Element { node: tbody } = &tabular.children[0] else {
            panic!("expected tbody node");
        };
        assert_eq!(tbody.name.as_str(), "ltx:tbody");
    }

    #[test]
    fn records_unresolved_ref_diagnostic() {
        let mut root = LtxmlNode::document();
        root.push_element(LtxmlNode::new("ltx:ref").with_attr("labelref", "missing"));

        resolve_references(&mut root);
        let LtxmlChild::Element { node } = &root.children[0] else {
            panic!("expected ref node");
        };
        assert_eq!(node.diagnostics.len(), 1);
    }
}
