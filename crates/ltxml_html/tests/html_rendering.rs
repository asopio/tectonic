// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! HTML rendering tests for the Phase 3 LaTeXML-shaped IR subset.

use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_html::{render_document, render_document_with_options, RenderOptions};
use tectonic_ltxml_ir::LtxmlNode;

fn build_fixture() -> LtxmlNode {
    let mut engine = SpxToLtxmlEngine::new();

    for line in include_str!("../../engine_spx2ltxml/tests/fixtures/simple_article.tsem").lines() {
        if !line.trim().is_empty() {
            engine.handle_special_text(line).unwrap();
        }
    }

    engine.finish()
}

#[test]
fn renders_complete_html_document() {
    let doc = build_fixture();
    let html = render_document(&doc);

    assert!(html.starts_with("<!doctype html><html><head><meta charset=\"utf-8\">"));
    assert!(html.contains("<title>A Simple Article</title>"));
    assert!(html.contains("<main class=\"ltx_document\">"));
    assert!(html.ends_with("</body></html>"));
}

#[test]
fn renders_article_structure() {
    let doc = build_fixture();
    let html = render_document(&doc);

    assert!(html.contains("<h1 class=\"ltx_title\">A Simple Article</h1>"));
    assert!(html.contains("<p class=\"ltx_creator\"><span class=\"ltx_personname\">Ada Lovelace</span></p>"));
    assert!(html.contains("<section class=\"ltx_abstract\"><p class=\"ltx_p\">A short abstract.</p></section>"));
    assert!(html.contains("<section id=\"S1\" class=\"ltx_section\" data-labels=\"LABEL:sec:intro\"><h2 class=\"ltx_title\">Introduction</h2>"));
    assert!(html.contains("This is <em class=\"ltx_emph\">important</em>"));
}

#[test]
fn renders_math_and_equations() {
    let doc = build_fixture();
    let html = render_document(&doc);

    assert!(html.contains("<span class=\"ltx_Math ltx_Math_inline\" data-tex=\"x^2\">x^2</span>"));
    assert!(html.contains("<div id=\"E1\" class=\"ltx_equation\" data-labels=\"LABEL:eq:one\"><div class=\"ltx_Math ltx_Math_display\" data-tex=\"E=mc^2\">E=mc^2</div></div>"));
}

#[test]
fn renders_graphics_and_tables() {
    let doc = build_fixture();
    let html = render_document(&doc);

    assert!(html.contains("<figure id=\"F1\" class=\"ltx_figure\" data-labels=\"LABEL:fig:plot\"><img class=\"ltx_graphics\" src=\"plot.png\" alt=\"A plot\"><figcaption class=\"ltx_caption\">A raster plot.</figcaption></figure>"));
    assert!(html.contains("<figure id=\"T1\" class=\"ltx_table\" data-labels=\"LABEL:tab:values\"><table class=\"ltx_tabular\"><tr class=\"ltx_tr\"><td class=\"ltx_td\" style=\"text-align:left\">A</td><td class=\"ltx_td\" style=\"text-align:right\">1</td></tr></table><figcaption class=\"ltx_caption\">A small table.</figcaption></figure>"));
}

#[test]
fn supports_explicit_document_title_option() {
    let doc = build_fixture();
    let html = render_document_with_options(
        &doc,
        &RenderOptions {
            document_title: Some("Custom".to_owned()),
        },
    );

    assert!(html.contains("<title>Custom</title>"));
}
