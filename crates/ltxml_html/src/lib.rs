// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Minimal HTML renderer for LaTeXML-shaped IR.
//!
//! This crate renders a conservative single-page HTML representation of the
//! Phase 1/2 `LtxmlIr` subset. It intentionally maps LaTeXML `ltx:*` element
//! names to HTML while preserving useful `ltx_*` CSS classes, IDs, labels, math
//! TeX source, and raster graphics metadata.

use tectonic_ltxml_ir::{LtxmlChild, LtxmlNode};

/// Options controlling LaTeXML-shaped IR HTML rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Optional document title to use in the HTML `<title>` element.
    pub document_title: Option<String>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { document_title: None }
    }
}

/// Render a LaTeXML-shaped IR tree as a complete single-page HTML document.
pub fn render_document(root: &LtxmlNode) -> String {
    render_document_with_options(root, &RenderOptions::default())
}

/// Render a LaTeXML-shaped IR tree as a complete single-page HTML document.
pub fn render_document_with_options(root: &LtxmlNode, options: &RenderOptions) -> String {
    let mut out = String::new();
    let title = options
        .document_title
        .as_deref()
        .or_else(|| first_child_text(root, "ltx:title"))
        .unwrap_or("Tectonic HTML output");

    out.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>");
    escape_text(title, &mut out);
    out.push_str("</title></head><body>");
    render_node(root, &mut out, RenderContext::default());
    out.push_str("</body></html>");
    out
}

/// Render a LaTeXML-shaped IR node as an HTML fragment.
pub fn render_fragment(node: &LtxmlNode) -> String {
    let mut out = String::new();
    render_node(node, &mut out, RenderContext::default());
    out
}

#[derive(Clone, Copy, Debug, Default)]
struct RenderContext {
    section_level: usize,
}

fn render_node(node: &LtxmlNode, out: &mut String, context: RenderContext) {
    match node.name.as_str() {
        "ltx:document" => render_container(node, out, "main", "ltx_document", context),
        "ltx:title" => render_title(node, out, context),
        "ltx:creator" => render_container(node, out, "p", "ltx_creator", context),
        "ltx:personname" => render_container(node, out, "span", "ltx_personname", context),
        "ltx:contact" => render_container(node, out, "span", "ltx_contact", context),
        "ltx:abstract" => render_container(node, out, "section", "ltx_abstract", context),
        "ltx:section" => render_section(node, out, context, 1),
        "ltx:subsection" => render_section(node, out, context, 2),
        "ltx:subsubsection" => render_section(node, out, context, 3),
        "ltx:paragraph" => render_section(node, out, context, 4),
        "ltx:p" => render_container(node, out, "p", "ltx_p", context),
        "ltx:text" => render_container(node, out, "span", "ltx_text", context),
        "ltx:emph" => render_container(node, out, "em", "ltx_emph", context),
        "ltx:ref" => render_ref(node, out, context),
        "ltx:cite" => render_container(node, out, "span", "ltx_cite", context),
        "ltx:bibref" => render_container(node, out, "span", "ltx_bibref", context),
        "ltx:equation" => render_container(node, out, "div", "ltx_equation", context),
        "ltx:Math" => render_math(node, out),
        "ltx:figure" => render_container(node, out, "figure", "ltx_figure", context),
        "ltx:table" => render_container(node, out, "figure", "ltx_table", context),
        "ltx:caption" => render_container(node, out, "figcaption", "ltx_caption", context),
        "ltx:tabular" => render_container(node, out, "table", "ltx_tabular", context),
        "ltx:thead" => render_container(node, out, "thead", "ltx_thead", context),
        "ltx:tbody" => render_container(node, out, "tbody", "ltx_tbody", context),
        "ltx:tfoot" => render_container(node, out, "tfoot", "ltx_tfoot", context),
        "ltx:tr" => render_container(node, out, "tr", "ltx_tr", context),
        "ltx:td" => render_table_cell(node, out, context),
        "ltx:graphics" => render_graphics(node, out),
        "ltx:note" => render_container(node, out, "aside", "ltx_note", context),
        "ltx:resource" => {}
        "ltx:ERROR" => render_container(node, out, "span", "ltx_ERROR", context),
        _ => render_container(node, out, "span", "ltx_unknown", context),
    }
}

fn render_children(node: &LtxmlNode, out: &mut String, context: RenderContext) {
    for child in &node.children {
        match child {
            LtxmlChild::Element { node } => render_node(node, out, context),
            LtxmlChild::Text { text } => escape_text(text, out),
        }
    }
}

fn render_container(
    node: &LtxmlNode,
    out: &mut String,
    tag: &str,
    base_class: &str,
    context: RenderContext,
) {
    start_tag(node, out, tag, base_class);
    render_children(node, out, context);
    end_tag(out, tag);
}

fn render_section(node: &LtxmlNode, out: &mut String, context: RenderContext, level: usize) {
    start_tag(node, out, "section", section_class(level));

    let heading_level = (level + 1).min(6);
    if let Some(title) = first_named_child(node, "ltx:title") {
        out.push('<');
        out.push_str(heading_tag(heading_level));
        out.push_str(" class=\"ltx_title\">");
        render_children(title, out, RenderContext { section_level: level });
        out.push_str("</");
        out.push_str(heading_tag(heading_level));
        out.push('>');
    }

    let child_context = RenderContext { section_level: level };
    for child in &node.children {
        match child {
            LtxmlChild::Element { node } if node.name.as_str() == "ltx:title" => {}
            LtxmlChild::Element { node } => render_node(node, out, child_context),
            LtxmlChild::Text { text } => escape_text(text, out),
        }
    }

    end_tag(out, "section");

    let _ = context;
}

fn render_title(node: &LtxmlNode, out: &mut String, context: RenderContext) {
    let level = if context.section_level == 0 { 1 } else { (context.section_level + 1).min(6) };
    out.push('<');
    out.push_str(heading_tag(level));
    write_common_attrs(node, out, "ltx_title");
    out.push('>');
    render_children(node, out, context);
    out.push_str("</");
    out.push_str(heading_tag(level));
    out.push('>');
}

fn render_ref(node: &LtxmlNode, out: &mut String, context: RenderContext) {
    out.push_str("<a");
    write_common_attrs(node, out, "ltx_ref");

    if let Some(href) = node.attr("href") {
        out.push_str(" href=\"");
        escape_attr(href, out);
        out.push('"');
    } else if let Some(idref) = node.attr("idref") {
        out.push_str(" href=\"#");
        escape_attr(idref, out);
        out.push('"');
    } else if let Some(labelref) = node.attr("labelref") {
        out.push_str(" data-labelref=\"");
        escape_attr(labelref, out);
        out.push('"');
    }

    out.push('>');
    render_children(node, out, context);
    out.push_str("</a>");
}

fn render_math(node: &LtxmlNode, out: &mut String) {
    let mode = node.attr("mode").unwrap_or("inline");
    let tag = if mode == "display" { "div" } else { "span" };
    let class = if mode == "display" {
        "ltx_Math ltx_Math_display"
    } else {
        "ltx_Math ltx_Math_inline"
    };

    out.push('<');
    out.push_str(tag);
    write_common_attrs(node, out, class);

    if let Some(tex) = node.attr("tex") {
        out.push_str(" data-tex=\"");
        escape_attr(tex, out);
        out.push('"');
    }

    out.push('>');
    if node.children.is_empty() {
        if let Some(tex) = node.attr("tex") {
            escape_text(tex, out);
        }
    } else {
        render_children(node, out, RenderContext::default());
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn render_graphics(node: &LtxmlNode, out: &mut String) {
    out.push_str("<img");
    write_common_attrs(node, out, "ltx_graphics");

    if let Some(src) = node.attr("imagesrc").or_else(|| node.attr("graphic")) {
        out.push_str(" src=\"");
        escape_attr(src, out);
        out.push('"');
    }

    if let Some(alt) = node.attr("description") {
        out.push_str(" alt=\"");
        escape_attr(alt, out);
        out.push('"');
    } else {
        out.push_str(" alt=\"\"");
    }

    if let Some(width) = node.attr("imagewidth") {
        out.push_str(" width=\"");
        escape_attr(width, out);
        out.push('"');
    }

    if let Some(height) = node.attr("imageheight") {
        out.push_str(" height=\"");
        escape_attr(height, out);
        out.push('"');
    }

    out.push('>');
}

fn render_table_cell(node: &LtxmlNode, out: &mut String, context: RenderContext) {
    let tag = if node.attr("thead").is_some() { "th" } else { "td" };
    out.push('<');
    out.push_str(tag);
    write_common_attrs(node, out, "ltx_td");

    for attr in ["colspan", "rowspan"] {
        if let Some(value) = node.attr(attr) {
            out.push(' ');
            out.push_str(attr);
            out.push_str("=\"");
            escape_attr(value, out);
            out.push('"');
        }
    }

    if let Some(align) = node.attr("align") {
        out.push_str(" style=\"text-align:");
        escape_attr(align, out);
        out.push_str("\"");
    }

    out.push('>');
    render_children(node, out, context);
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn start_tag(node: &LtxmlNode, out: &mut String, tag: &str, base_class: &str) {
    out.push('<');
    out.push_str(tag);
    write_common_attrs(node, out, base_class);
    out.push('>');
}

fn end_tag(out: &mut String, tag: &str) {
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn write_common_attrs(node: &LtxmlNode, out: &mut String, base_class: &str) {
    if let Some(id) = node.xml_id() {
        out.push_str(" id=\"");
        escape_attr(id, out);
        out.push('"');
    }

    out.push_str(" class=\"");
    escape_attr(base_class, out);
    if let Some(extra) = node.attr("class") {
        out.push(' ');
        escape_attr(extra, out);
    }
    out.push('"');

    if let Some(labels) = node.attr("labels") {
        out.push_str(" data-labels=\"");
        escape_attr(labels, out);
        out.push('"');
    }

    if let Some(style) = node.attr("cssstyle") {
        out.push_str(" style=\"");
        escape_attr(style, out);
        out.push('"');
    }
}

fn first_named_child<'a>(node: &'a LtxmlNode, name: &str) -> Option<&'a LtxmlNode> {
    node.children.iter().find_map(|child| match child {
        LtxmlChild::Element { node } if node.name.as_str() == name => Some(node),
        _ => None,
    })
}

fn first_child_text<'a>(node: &'a LtxmlNode, name: &str) -> Option<&'a str> {
    let child = first_named_child(node, name)?;
    child.children.iter().find_map(|grandchild| match grandchild {
        LtxmlChild::Text { text } => Some(text.as_str()),
        LtxmlChild::Element { .. } => None,
    })
}

fn heading_tag(level: usize) -> &'static str {
    match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        4 => "h4",
        5 => "h5",
        _ => "h6",
    }
}

fn section_class(level: usize) -> &'static str {
    match level {
        1 => "ltx_section",
        2 => "ltx_subsection",
        3 => "ltx_subsubsection",
        4 => "ltx_paragraph",
        _ => "ltx_sectional_block",
    }
}

fn escape_text(raw: &str, out: &mut String) {
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escape_attr(raw: &str, out: &mut String) {
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tectonic_ltxml_ir::LtxmlNode;

    #[test]
    fn escapes_text_and_attributes() {
        let mut node = LtxmlNode::new("ltx:p").with_attr("xml:id", "a\"b");
        node.push_text("x < y & z");

        let html = render_fragment(&node);
        assert_eq!(html, "<p id=\"a&quot;b\" class=\"ltx_p\">x &lt; y &amp; z</p>");
    }
}
