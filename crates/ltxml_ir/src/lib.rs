// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! LaTeXML-shaped intermediate representation support.
//!
//! This crate provides a small Rust-native representation of LaTeXML-like
//! `ltx:*` XML trees, plus parsing for JSON-backed `tsem:` semantic specials.
//! The representation intentionally preserves element and attribute names as
//! qualified XML names so that package bindings can be oxidized incrementally
//! while retaining compatibility with LaTeXML's observable output shape.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::BTreeMap, fmt};

/// The LaTeXML namespace URI used by serialized `ltx:*` XML.
pub const LATEXML_NAMESPACE: &str = "http://dlmf.nist.gov/LaTeXML";

/// A qualified XML name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QName(String);

impl QName {
    /// Create a qualified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Borrow this name as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the namespace prefix, if present.
    pub fn prefix(&self) -> Option<&str> {
        self.0.split_once(':').map(|(prefix, _)| prefix)
    }

    /// Return the local part of the name.
    pub fn local_name(&self) -> &str {
        self.0
            .split_once(':')
            .map_or(self.0.as_str(), |(_, local)| local)
    }
}

impl From<&str> for QName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for QName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for QName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for QName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for QName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

/// A source location associated with an IR node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Source file name, if known.
    pub file: Option<String>,
    /// One-indexed source line, if known.
    pub line: Option<u32>,
    /// One-indexed source column, if known.
    pub column: Option<u32>,
}

/// Diagnostic severity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Informational diagnostic.
    Info,
    /// Warning diagnostic.
    Warning,
    /// Error diagnostic.
    Error,
}

/// A diagnostic attached to an IR tree.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
}

impl Diagnostic {
    /// Create a warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
        }
    }

    /// Create an error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
        }
    }
}

/// A child of a LaTeXML-shaped node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LtxmlChild {
    /// An XML element child.
    Element {
        /// The child element.
        node: LtxmlNode,
    },
    /// A text child.
    Text {
        /// The text content.
        text: String,
    },
}

/// A LaTeXML-shaped XML element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LtxmlNode {
    /// Element name, normally an `ltx:*` qualified name.
    pub name: QName,
    /// XML attributes, preserving unknown and namespaced attributes.
    #[serde(default)]
    pub attrs: BTreeMap<QName, String>,
    /// Child nodes and text.
    #[serde(default)]
    pub children: Vec<LtxmlChild>,
    /// Source location associated with this node, if known.
    #[serde(default)]
    pub source: Option<SourceSpan>,
    /// TeX command that created this node, if known.
    #[serde(default)]
    pub tex_command: Option<String>,
    /// Diagnostics attached to this node.
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

impl LtxmlNode {
    /// Create an empty node with the given name.
    pub fn new(name: impl Into<QName>) -> Self {
        Self {
            name: name.into(),
            attrs: BTreeMap::new(),
            children: Vec::new(),
            source: None,
            tex_command: None,
            diagnostics: Vec::new(),
        }
    }

    /// Create an empty `ltx:document` node.
    pub fn document() -> Self {
        Self::new("ltx:document")
    }

    /// Add an attribute and return the updated node.
    pub fn with_attr(mut self, name: impl Into<QName>, value: impl Into<String>) -> Self {
        self.attrs.insert(name.into(), value.into());
        self
    }

    /// Add an element child.
    pub fn push_element(&mut self, node: LtxmlNode) {
        self.children.push(LtxmlChild::Element { node });
    }

    /// Add a text child.
    pub fn push_text(&mut self, text: impl Into<String>) {
        self.children.push(LtxmlChild::Text { text: text.into() });
    }

    /// Return this node's `xml:id`, if present.
    pub fn xml_id(&self) -> Option<&str> {
        self.attr("xml:id")
    }

    /// Return this node's `labelref`, if present.
    pub fn labelref(&self) -> Option<&str> {
        self.attr("labelref")
    }

    /// Return an attribute by qualified name.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.get(&QName::from(name)).map(String::as_str)
    }

    /// Iterate over space-separated label tokens.
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.attr("labels").into_iter().flat_map(str::split_whitespace)
    }

    /// Iterate over space-separated CSS class tokens.
    pub fn class_tokens(&self) -> impl Iterator<Item = &str> {
        self.attr("class").into_iter().flat_map(str::split_whitespace)
    }

    /// Serialize this node and descendants as LaTeXML-like XML.
    pub fn to_xml_string(&self) -> String {
        let mut out = String::new();
        self.write_xml(&mut out, true);
        out
    }

    fn write_xml(&self, out: &mut String, is_root: bool) {
        out.push('<');
        out.push_str(self.name.as_str());

        if is_root && self.name.prefix() == Some("ltx") && !self.attrs.contains_key(&QName::from("xmlns:ltx")) {
            out.push_str(" xmlns:ltx=\"");
            escape_attr(LATEXML_NAMESPACE, out);
            out.push('"');
        }

        for (name, value) in &self.attrs {
            out.push(' ');
            out.push_str(name.as_str());
            out.push_str("=\"");
            escape_attr(value, out);
            out.push('"');
        }

        if self.children.is_empty() {
            out.push_str("/>");
            return;
        }

        out.push('>');

        for child in &self.children {
            match child {
                LtxmlChild::Element { node } => node.write_xml(out, false),
                LtxmlChild::Text { text } => escape_text(text, out),
            }
        }

        out.push_str("</");
        out.push_str(self.name.as_str());
        out.push('>');
    }
}

/// A semantic event parsed from a `tsem:` special.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum TsemEvent {
    /// Begin an element and push it onto the builder stack.
    Begin {
        /// Element name.
        element: QName,
        /// Element attributes.
        #[serde(default)]
        attrs: BTreeMap<QName, String>,
    },
    /// End an element and pop it from the builder stack.
    End {
        /// Optional expected element name.
        #[serde(default)]
        element: Option<QName>,
    },
    /// Append text to the current element.
    Text {
        /// Text content.
        text: String,
    },
    /// Append a complete math element to the current element.
    Math {
        /// Element name, defaulting to `ltx:Math`.
        #[serde(default = "default_math_element")]
        element: QName,
        /// Element attributes.
        #[serde(default)]
        attrs: BTreeMap<QName, String>,
        /// Optional text content.
        #[serde(default)]
        text: Option<String>,
    },
}

fn default_math_element() -> QName {
    QName::from("ltx:Math")
}

/// Error returned when parsing a `tsem:` semantic special.
#[derive(Debug)]
pub enum TsemParseError {
    /// The special did not use the `tsem:` prefix.
    MissingPrefix,
    /// The JSON payload could not be parsed.
    Json(serde_json::Error),
}

impl fmt::Display for TsemParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => write!(f, "semantic special does not start with `tsem:`"),
            Self::Json(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for TsemParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingPrefix => None,
            Self::Json(err) => Some(err),
        }
    }
}

/// Parse a JSON-backed `tsem:` special.
pub fn parse_tsem_special(text: &str) -> Result<TsemEvent, TsemParseError> {
    let payload = text
        .strip_prefix("tsem:")
        .ok_or(TsemParseError::MissingPrefix)?;
    serde_json::from_str(payload).map_err(TsemParseError::Json)
}

/// Builder for an `LtxmlNode` tree from semantic events.
#[derive(Debug)]
pub struct LtxmlBuilder {
    stack: Vec<LtxmlNode>,
}

impl LtxmlBuilder {
    /// Create a builder rooted at `ltx:document`.
    pub fn new_document() -> Self {
        Self::new(LtxmlNode::document())
    }

    /// Create a builder rooted at the provided node.
    pub fn new(root: LtxmlNode) -> Self {
        Self { stack: vec![root] }
    }

    /// Apply one semantic event to the tree under construction.
    pub fn apply_event(&mut self, event: TsemEvent) {
        match event {
            TsemEvent::Begin { element, attrs } => {
                let mut node = LtxmlNode::new(element);
                node.attrs = attrs;
                self.stack.push(node);
            }
            TsemEvent::End { element } => self.end_element(element),
            TsemEvent::Text { text } => self.current_mut().push_text(text),
            TsemEvent::Math { element, attrs, text } => {
                let mut node = LtxmlNode::new(element);
                node.attrs = attrs;
                if let Some(text) = text {
                    node.push_text(text);
                }
                self.current_mut().push_element(node);
            }
        }
    }

    /// Parse and apply one `tsem:` special.
    pub fn apply_special(&mut self, special: &str) -> Result<(), TsemParseError> {
        let event = parse_tsem_special(special)?;
        self.apply_event(event);
        Ok(())
    }

    /// Finish building the tree.
    pub fn finish(mut self) -> LtxmlNode {
        while self.stack.len() > 1 {
            let mut node = self.stack.pop().expect("stack length checked");
            node.diagnostics.push(Diagnostic::warning(format!(
                "unclosed element `{}` closed at end of document",
                node.name
            )));
            self.current_mut().push_element(node);
        }

        self.stack.pop().expect("builder always has a root node")
    }

    fn current_mut(&mut self) -> &mut LtxmlNode {
        self.stack.last_mut().expect("builder always has a root node")
    }

    fn end_element(&mut self, expected: Option<QName>) {
        if self.stack.len() <= 1 {
            self.current_mut()
                .diagnostics
                .push(Diagnostic::error("unexpected end event at document root"));
            return;
        }

        let mut node = self.stack.pop().expect("stack length checked");
        if let Some(expected) = expected {
            if node.name != expected {
                node.diagnostics.push(Diagnostic::error(format!(
                    "mismatched end event: expected `{expected}`, found `{}`",
                    node.name
                )));
            }
        }

        self.current_mut().push_element(node);
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

    #[test]
    fn qname_reports_prefix_and_local_name() {
        let name = QName::from("ltx:section");
        assert_eq!(name.prefix(), Some("ltx"));
        assert_eq!(name.local_name(), "section");

        let name = QName::from("href");
        assert_eq!(name.prefix(), None);
        assert_eq!(name.local_name(), "href");
    }

    #[test]
    fn parses_json_tsem_begin_event() {
        let event = parse_tsem_special(
            r#"tsem:{"event":"begin","element":"ltx:section","attrs":{"xml:id":"S1","labels":"LABEL:intro"}}"#,
        )
        .unwrap();

        let TsemEvent::Begin { element, attrs } = event else {
            panic!("expected begin event");
        };

        assert_eq!(element, QName::from("ltx:section"));
        assert_eq!(attrs.get(&QName::from("xml:id")).map(String::as_str), Some("S1"));
        assert_eq!(attrs.get(&QName::from("labels")).map(String::as_str), Some("LABEL:intro"));
    }

    #[test]
    fn builds_ltxml_tree_from_events() {
        let mut builder = LtxmlBuilder::new_document();
        builder
            .apply_special(r#"tsem:{"event":"begin","element":"ltx:section","attrs":{"xml:id":"S1","labels":"LABEL:intro"}}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"begin","element":"ltx:title"}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"text","text":"Intro & Motivation"}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"end","element":"ltx:title"}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"begin","element":"ltx:p"}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"text","text":"See "}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"math","attrs":{"mode":"inline","tex":"x^2 < y"}}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"end","element":"ltx:p"}"#)
            .unwrap();
        builder
            .apply_special(r#"tsem:{"event":"end","element":"ltx:section"}"#)
            .unwrap();

        let doc = builder.finish();
        assert_eq!(doc.name, QName::from("ltx:document"));
        assert!(doc.diagnostics.is_empty());

        let LtxmlChild::Element { node: section } = &doc.children[0] else {
            panic!("expected section");
        };
        assert_eq!(section.name, QName::from("ltx:section"));
        assert_eq!(section.xml_id(), Some("S1"));
        assert_eq!(section.labels().collect::<Vec<_>>(), vec!["LABEL:intro"]);

        let xml = doc.to_xml_string();
        assert!(xml.contains("<ltx:title>Intro &amp; Motivation</ltx:title>"));
        assert!(xml.contains("tex=\"x^2 &lt; y\""));
    }

    #[test]
    fn serializes_unknown_attributes_to_json() {
        let node = LtxmlNode::new("ltx:graphics")
            .with_attr("graphic", "plot")
            .with_attr("data-custom", "kept");

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("data-custom"));

        let round_tripped: LtxmlNode = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped.attr("data-custom"), Some("kept"));
    }

    #[test]
    fn records_mismatched_end_diagnostic() {
        let mut builder = LtxmlBuilder::new_document();
        builder.apply_event(TsemEvent::Begin {
            element: QName::from("ltx:p"),
            attrs: BTreeMap::new(),
        });
        builder.apply_event(TsemEvent::End {
            element: Some(QName::from("ltx:section")),
        });

        let doc = builder.finish();
        let LtxmlChild::Element { node } = &doc.children[0] else {
            panic!("expected element");
        };
        assert_eq!(node.name, QName::from("ltx:p"));
        assert_eq!(node.diagnostics.len(), 1);
        assert_eq!(node.diagnostics[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn records_unclosed_element_diagnostic() {
        let mut builder = LtxmlBuilder::new_document();
        builder.apply_event(TsemEvent::Begin {
            element: QName::from("ltx:p"),
            attrs: BTreeMap::new(),
        });

        let doc = builder.finish();
        let LtxmlChild::Element { node } = &doc.children[0] else {
            panic!("expected element");
        };
        assert_eq!(node.name, QName::from("ltx:p"));
        assert_eq!(node.diagnostics.len(), 1);
        assert_eq!(node.diagnostics[0].severity, DiagnosticSeverity::Warning);
    }
}
