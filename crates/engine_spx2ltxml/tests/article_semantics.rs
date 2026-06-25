// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Article-like semantic event fixture tests.

use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_ir::{LtxmlChild, LtxmlNode};

fn build_fixture() -> LtxmlNode {
    let mut engine = SpxToLtxmlEngine::new();

    for line in include_str!("fixtures/simple_article.tsem").lines() {
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

fn find_all<'a>(node: &'a LtxmlNode, name: &str, matches: &mut Vec<&'a LtxmlNode>) {
    if node.name.as_str() == name {
        matches.push(node);
    }

    for child in element_children(node) {
        find_all(child, name, matches);
    }
}

#[test]
fn fixture_builds_article_frontmatter_and_section() {
    let doc = build_fixture();
    assert_eq!(doc.name.as_str(), "ltx:document");
    assert_eq!(doc.children.len(), 4);

    let mut titles = Vec::new();
    find_all(&doc, "ltx:title", &mut titles);
    assert_eq!(text_content(titles[0]), "A Simple Article");
    assert_eq!(text_content(titles[1]), "Introduction");

    let creator = find_first(&doc, "ltx:creator").unwrap();
    assert_eq!(creator.attr("role"), Some("author"));
    assert_eq!(text_content(creator), "Ada Lovelace");

    let section = find_first(&doc, "ltx:section").unwrap();
    assert_eq!(section.xml_id(), Some("S1"));
    assert_eq!(section.labels().collect::<Vec<_>>(), vec!["LABEL:sec:intro"]);
}

#[test]
fn fixture_preserves_equation_and_math_metadata() {
    let doc = build_fixture();
    let equation = find_first(&doc, "ltx:equation").unwrap();
    assert_eq!(equation.xml_id(), Some("E1"));
    assert_eq!(equation.labels().collect::<Vec<_>>(), vec!["LABEL:eq:one"]);

    let mut math = Vec::new();
    find_all(&doc, "ltx:Math", &mut math);
    assert_eq!(math.len(), 2);
    assert_eq!(math[0].attr("mode"), Some("inline"));
    assert_eq!(math[0].attr("tex"), Some("x^2"));
    assert_eq!(math[1].attr("mode"), Some("display"));
    assert_eq!(math[1].attr("tex"), Some("E=mc^2"));
}

#[test]
fn fixture_preserves_raster_graphics_and_table_structure() {
    let doc = build_fixture();
    let figure = find_first(&doc, "ltx:figure").unwrap();
    assert_eq!(figure.xml_id(), Some("F1"));

    let graphics = find_first(figure, "ltx:graphics").unwrap();
    assert_eq!(graphics.attr("graphic"), Some("plot.png"));
    assert_eq!(graphics.attr("imagesrc"), Some("plot.png"));
    assert_eq!(graphics.attr("description"), Some("A plot"));

    let table = find_first(&doc, "ltx:table").unwrap();
    assert_eq!(table.xml_id(), Some("T1"));

    let tabular = find_first(table, "ltx:tabular").unwrap();
    let row = find_first(tabular, "ltx:tr").unwrap();
    let cells = element_children(row).collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].name.as_str(), "ltx:td");
    assert_eq!(cells[0].attr("align"), Some("left"));
    assert_eq!(text_content(cells[0]), "A");
    assert_eq!(cells[1].attr("align"), Some("right"));
    assert_eq!(text_content(cells[1]), "1");
}

#[test]
fn fixture_serializes_as_latexml_like_xml() {
    let doc = build_fixture();
    let xml = doc.to_xml_string();

    assert!(xml.starts_with("<ltx:document xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\""));
    assert!(xml.contains("<ltx:section labels=\"LABEL:sec:intro\" xml:id=\"S1\">"));
    assert!(xml.contains("<ltx:Math mode=\"display\" tex=\"E=mc^2\"/>"));
    assert!(xml.contains("<ltx:graphics description=\"A plot\" graphic=\"plot.png\" imagesrc=\"plot.png\"/>"));
    assert!(xml.contains("<ltx:td align=\"right\">1</ltx:td>"));
}
