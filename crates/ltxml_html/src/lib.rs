// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Minimal HTML renderer for LaTeXML-shaped IR.
//!
//! This crate renders a conservative single-page HTML representation of the
//! Phase 1/2 `LtxmlIr` subset. It intentionally maps LaTeXML `ltx:*` element
//! names to HTML while preserving useful `ltx_*` CSS classes, IDs, labels, math
//! TeX source, and raster graphics metadata.

use tectonic_ltxml_ir::{LtxmlChild, LtxmlNode, MathMetadata, MathMode};

/// Math rendering strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathRenderMode {
    /// Render visible TeX text in `span`/`div` elements.
    Text,
    /// Render MathJax-compatible TeX script tags.
    MathJax,
}

/// Options controlling LaTeXML-shaped IR HTML rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Optional document title to use in the HTML `<title>` element.
    pub document_title: Option<String>,
    /// Math rendering strategy.
    pub math_render_mode: MathRenderMode,
    /// Whether to emit a MathJax loader script when using [`MathRenderMode::MathJax`].
    pub include_mathjax_script: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            document_title: None,
            math_render_mode: MathRenderMode::Text,
            include_mathjax_script: false,
        }
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
    out.push_str("</title>");
    if options.math_render_mode == MathRenderMode::MathJax && options.include_mathjax_script {
        out.push_str("<script src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\" async></script>");
    }
    out.push_str("</head><body>");
    render_node(root, &mut out, RenderContext::new(options));
    out.push_str("</body></html>");
    out
}

/// Render a LaTeXML-shaped IR node as an HTML fragment.
pub fn render_fragment(node: &LtxmlNode) -> String {
    let mut out = String::new();
    let options = RenderOptions::default();
    render_node(node, &mut out, RenderContext::new(&options));
    out
}

#[derive(Clone, Copy, Debug)]
struct RenderContext<'a> {
    section_level: usize,
    options: &'a RenderOptions,
}

impl<'a> RenderContext<'a> {
    fn new(options: &'a RenderOptions) -> Self {
        Self {
            section_level: 0,
            options,
        }
    }

    fn with_section_level(self, section_level: usize) -> Self {
        Self {
            section_level,
            options: self.options,
        }
    }
}

fn render_node(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
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
        "ltx:bibref" => render_bibref(node, out, context),
        "ltx:bibliography" => render_container(node, out, "section", "ltx_bibliography", context),
        "ltx:biblist" => render_container(node, out, "ol", "ltx_biblist", context),
        "ltx:bibitem" => render_container(node, out, "li", "ltx_bibitem", context),
        "ltx:equation" => render_container(node, out, "div", "ltx_equation", context),
        "ltx:Math" => render_math(node, out, context),
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

fn render_children(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
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
    context: RenderContext<'_>,
) {
    start_tag(node, out, tag, base_class);
    render_children(node, out, context);
    end_tag(out, tag);
}

fn render_section(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>, level: usize) {
    start_tag(node, out, "section", section_class(level));

    let heading_level = (level + 1).min(6);
    if let Some(title) = first_named_child(node, "ltx:title") {
        out.push('<');
        out.push_str(heading_tag(heading_level));
        out.push_str(" class=\"ltx_title\">");
        render_children(title, out, context.with_section_level(level));
        out.push_str("</");
        out.push_str(heading_tag(heading_level));
        out.push('>');
    }

    let child_context = context.with_section_level(level);
    for child in &node.children {
        match child {
            LtxmlChild::Element { node } if node.name.as_str() == "ltx:title" => {}
            LtxmlChild::Element { node } => render_node(node, out, child_context),
            LtxmlChild::Text { text } => escape_text(text, out),
        }
    }

    end_tag(out, "section");

}

fn render_title(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
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

fn render_ref(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
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

fn render_bibref(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
    if let Some(href) = node.attr("href") {
        out.push_str("<a");
        write_common_attrs(node, out, "ltx_bibref");
        out.push_str(" href=\"");
        escape_attr(href, out);
        out.push('"');
        out.push('>');
        render_children(node, out, context);
        out.push_str("</a>");
    } else {
        render_container(node, out, "span", "ltx_bibref", context);
    }
}

fn render_math(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
    let metadata = node.math_metadata().unwrap_or(MathMetadata {
        mode: MathMode::Inline,
        tex: node.attr("tex"),
        content_tex: node.attr("content-tex"),
        text: node.attr("text"),
        image_src: node.attr("imagesrc"),
        image_width: node.attr("imagewidth"),
        image_height: node.attr("imageheight"),
        image_depth: node.attr("imagedepth"),
        description: node.attr("description"),
    });

    match context.options.math_render_mode {
        MathRenderMode::Text => render_math_text(node, &metadata, out, context),
        MathRenderMode::MathJax => render_mathjax(node, &metadata, out),
    }
}

fn render_math_text(
    node: &LtxmlNode,
    metadata: &MathMetadata<'_>,
    out: &mut String,
    context: RenderContext<'_>,
) {
    let tag = if metadata.mode == MathMode::Display { "div" } else { "span" };
    let class = if metadata.mode == MathMode::Display {
        "ltx_Math ltx_Math_display"
    } else {
        "ltx_Math ltx_Math_inline"
    };

    out.push('<');
    out.push_str(tag);
    write_math_attrs(node, metadata, out, class);
    out.push('>');

    if node.children.is_empty() {
        escape_text(math_text(metadata), out);
    } else {
        render_children(node, out, context);
    }

    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn render_mathjax(node: &LtxmlNode, metadata: &MathMetadata<'_>, out: &mut String) {
    out.push_str("<script type=\"");
    out.push_str(if metadata.mode == MathMode::Display {
        "math/tex; mode=display"
    } else {
        "math/tex"
    });
    out.push('"');
    write_math_attrs(node, metadata, out, "ltx_Math ltx_Math_mathjax");
    out.push('>');
    escape_script_text(math_text(metadata), out);
    out.push_str("</script>");
}

fn math_text<'a>(metadata: &MathMetadata<'a>) -> &'a str {
    metadata
        .tex
        .or(metadata.content_tex)
        .or(metadata.text)
        .or(metadata.description)
        .unwrap_or("")
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

fn render_table_cell(node: &LtxmlNode, out: &mut String, context: RenderContext<'_>) {
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

fn write_math_attrs(
    node: &LtxmlNode,
    metadata: &MathMetadata<'_>,
    out: &mut String,
    base_class: &str,
) {
    write_common_attrs(node, out, base_class);

    if let Some(tex) = metadata.tex {
        out.push_str(" data-tex=\"");
        escape_attr(tex, out);
        out.push('"');
    }

    if let Some(content_tex) = metadata.content_tex {
        out.push_str(" data-content-tex=\"");
        escape_attr(content_tex, out);
        out.push('"');
    }

    if let Some(text) = metadata.text {
        out.push_str(" data-text=\"");
        escape_attr(text, out);
        out.push('"');
    }

    if let Some(src) = metadata.image_src {
        out.push_str(" data-altimg=\"");
        escape_attr(src, out);
        out.push('"');
    }

    if let Some(width) = metadata.image_width {
        out.push_str(" data-altimg-width=\"");
        escape_attr(width, out);
        out.push('"');
    }

    if let Some(height) = metadata.image_height {
        out.push_str(" data-altimg-height=\"");
        escape_attr(height, out);
        out.push('"');
    }

    if let Some(depth) = metadata.image_depth {
        out.push_str(" data-altimg-depth=\"");
        escape_attr(depth, out);
        out.push('"');
    }

    if let Some(description) = metadata.description {
        out.push_str(" aria-label=\"");
        escape_attr(description, out);
        out.push('"');
    }
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

fn escape_script_text(raw: &str, out: &mut String) {
    for ch in raw.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
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
    fn renders_math_metadata_attributes() {
        let node = LtxmlNode::new("ltx:Math")
            .with_attr("mode", "display")
            .with_attr("tex", "x<y")
            .with_attr("content-tex", "less-than")
            .with_attr("text", "x less than y")
            .with_attr("imagesrc", "math.svg")
            .with_attr("imagewidth", "42")
            .with_attr("imageheight", "10")
            .with_attr("imagedepth", "2")
            .with_attr("description", "inequality");

        let html = render_fragment(&node);
        assert!(html.contains("data-tex=\"x&lt;y\""));
        assert!(html.contains("data-content-tex=\"less-than\""));
        assert!(html.contains("data-text=\"x less than y\""));
        assert!(html.contains("data-altimg=\"math.svg\""));
        assert!(html.contains("data-altimg-width=\"42\""));
        assert!(html.contains("data-altimg-height=\"10\""));
        assert!(html.contains("data-altimg-depth=\"2\""));
        assert!(html.contains("aria-label=\"inequality\""));
    }

    #[test]
    fn escapes_text_and_attributes() {
        let mut node = LtxmlNode::new("ltx:p").with_attr("xml:id", "a\"b");
        node.push_text("x < y & z");

        let html = render_fragment(&node);
        assert_eq!(html, "<p id=\"a&quot;b\" class=\"ltx_p\">x &lt; y &amp; z</p>");
    }
}
