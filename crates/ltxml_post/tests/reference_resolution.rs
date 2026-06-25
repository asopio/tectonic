// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Reference and bibliography post-processing tests.

use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_html::render_document;
use tectonic_ltxml_ir::{LtxmlChild, LtxmlNode};
use tectonic_ltxml_post::resolve_references;

fn build_fixture() -> LtxmlNode {
    let mut engine = SpxToLtxmlEngine::new();

    for line in include_str!("fixtures/refs_and_bib.tsem").lines() {
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

fn find_first<'a>(node: &'a LtxmlNode, name: &str) -> Option<&'a LtxmlNode> {
    if node.name.as_str() == name {
        return Some(node);
    }

    element_children(node).find_map(|child| find_first(child, name))
}

#[test]
fn resolves_section_refs_and_bibrefs() {
    let mut doc = build_fixture();
    let index = resolve_references(&mut doc);

    assert_eq!(index.labels["sec:intro"].id, "S1");
    assert_eq!(index.bibliography["knuth84"].id, "bib.knuth84");

    let reference = find_first(&doc, "ltx:ref").unwrap();
    assert_eq!(reference.attr("idref"), Some("S1"));
    assert_eq!(reference.attr("href"), Some("#S1"));
    assert_eq!(reference.attr("title"), Some("Introduction"));
    assert_eq!(text_content(reference), "1");

    let bibref = find_first(&doc, "ltx:bibref").unwrap();
    assert_eq!(bibref.attr("idref"), Some("bib.knuth84"));
    assert_eq!(bibref.attr("href"), Some("#bib.knuth84"));
    assert_eq!(text_content(bibref), "[1]");
}

#[test]
fn renders_resolved_refs_and_bibliography_as_links() {
    let mut doc = build_fixture();
    resolve_references(&mut doc);

    let html = render_document(&doc);
    assert!(html.contains("<a class=\"ltx_ref\" href=\"#S1\">1</a>"));
    assert!(html.contains("<a class=\"ltx_bibref\" href=\"#bib.knuth84\">[1]</a>"));
    assert!(html.contains("<section id=\"Bib\" class=\"ltx_bibliography\">"));
    assert!(html.contains("<li id=\"bib.knuth84\" class=\"ltx_bibitem\">Donald Knuth. The TeXbook. 1984.</li>"));
}

#[test]
fn leaves_unresolved_references_with_diagnostics() {
    let mut doc = LtxmlNode::document();
    doc.push_element(LtxmlNode::new("ltx:ref").with_attr("labelref", "missing"));
    doc.push_element(LtxmlNode::new("ltx:bibref").with_attr("bibrefs", "missing-bib"));

    resolve_references(&mut doc);

    let LtxmlChild::Element { node: reference } = &doc.children[0] else {
        panic!("expected ref node");
    };
    let LtxmlChild::Element { node: bibref } = &doc.children[1] else {
        panic!("expected bibref node");
    };

    assert_eq!(reference.diagnostics.len(), 1);
    assert_eq!(bibref.diagnostics.len(), 1);
}
