// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Graphics and table normalization tests.

use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_html::render_document;
use tectonic_ltxml_ir::{LtxmlChild, LtxmlNode};
use tectonic_ltxml_post::normalize_graphics_and_tables;

fn build_fixture() -> LtxmlNode {
    let mut engine = SpxToLtxmlEngine::new();

    for line in include_str!("fixtures/graphics_table.tsem").lines() {
        if !line.trim().is_empty() {
            engine.handle_special_text(line).unwrap();
        }
    }

    engine.finish()
}

fn element_children(node: &LtxmlNode) -> impl Iterator<Item = &LtxmlNode> {
    node.children.iter().filter_map(|child| match child {
        LtxmlChild::Element { node } => Some(node),
        LtxmlChild::Text { .. } => None,
    })
}

fn find_first<'a>(node: &'a LtxmlNode, name: &str) -> Option<&'a LtxmlNode> {
    if node.name.as_str() == name {
        return Some(node);
    }

    element_children(node).find_map(|child| find_first(child, name))
}

#[test]
fn resolves_graphics_candidates_and_warns_on_missing_alt_text() {
    let mut doc = build_fixture();
    let summary = normalize_graphics_and_tables(&mut doc);

    assert_eq!(summary.graphics_resolved, 1);
    assert_eq!(summary.missing_graphics_descriptions, 1);

    let graphics = find_first(&doc, "ltx:graphics").unwrap();
    assert_eq!(graphics.attr("imagesrc"), Some("figures/result.png"));
    assert_eq!(graphics.diagnostics.len(), 1);
}

#[test]
fn wraps_direct_rows_in_tbody_and_renders_table_headers() {
    let mut doc = build_fixture();
    let summary = normalize_graphics_and_tables(&mut doc);
    assert_eq!(summary.tabular_bodies_added, 1);

    let tabular = find_first(&doc, "ltx:tabular").unwrap();
    let tbody = find_first(tabular, "ltx:tbody").unwrap();
    assert_eq!(tbody.children.len(), 2);

    let html = render_document(&doc);
    assert!(html.contains("<img class=\"ltx_graphics\" src=\"figures/result.png\" alt=\"\">"));
    assert!(html.contains("<tbody class=\"ltx_tbody\"><tr class=\"ltx_tr\"><th class=\"ltx_td\" style=\"text-align:left\">Name</th><th class=\"ltx_td\" style=\"text-align:right\">Value</th></tr>"));
    assert!(html.contains("<td class=\"ltx_td\" style=\"text-align:right\">42</td>"));
}
