// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Extract LaTeXML-shaped IR from SPX semantic specials.
//!
//! This crate is the Phase 1 SPX-facing shell for the LaTeXML-shaped IR. It
//! consumes JSON-backed `tsem:` specials and applies them to a
//! [`tectonic_ltxml_ir::LtxmlBuilder`]. Non-`tsem:` specials and ordinary XDV
//! drawing events are ignored for now; later phases can attach canvas, font, and
//! asset data to the generated IR.
//!
//! In addition to the JSON-backed `tsem:` protocol, the following raw-text
//! prefixes are supported to avoid JSON string-escaping issues with
//! user-supplied content such as titles, TeX math source, or file names:
//!
//! - `tsem-text:<content>` — a text event whose content is the raw bytes after
//!   the prefix (no JSON encoding).
//! - `tsem-math:<mode>:<tex>` — a self-contained math event; the first
//!   colon-delimited field is the mode (`inline` or `display`) and everything
//!   after the second colon is the raw TeX source.
//! - `tsem-graphics:<filename>` — a self-contained `ltx:graphics` begin/end
//!   pair whose `graphic` attribute is set to the raw filename.

use std::{
    collections::BTreeMap,
    fmt,
    io::{Error as IoError, Read},
    str,
};
use tectonic_ltxml_ir::{parse_tsem_special, LtxmlBuilder, LtxmlNode, QName, TsemEvent, TsemParseError};
use tectonic_xdv::{FileType, XdvError, XdvEvents, XdvParser};

/// Errors that can occur while extracting `LtxmlIr` from SPX.
#[derive(Debug)]
pub enum SpxToLtxmlError {
    /// XDV/SPX parsing failed.
    Xdv(XdvError),
    /// Input/output failed while reading SPX.
    Io(IoError),
    /// A `\special` payload was not UTF-8.
    Utf8(str::Utf8Error),
    /// A `tsem:` semantic special could not be parsed.
    Tsem(TsemParseError),
    /// The input was not an SPX file.
    WrongFileType(FileType),
}

impl fmt::Display for SpxToLtxmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xdv(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
            Self::Utf8(err) => err.fmt(f),
            Self::Tsem(err) => err.fmt(f),
            Self::WrongFileType(filetype) => write!(f, "file should be SPX format but got {filetype}"),
        }
    }
}

impl std::error::Error for SpxToLtxmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xdv(err) => Some(err),
            Self::Io(err) => Some(err),
            Self::Utf8(err) => Some(err),
            Self::Tsem(err) => Some(err),
            Self::WrongFileType(_) => None,
        }
    }
}

impl From<XdvError> for SpxToLtxmlError {
    fn from(value: XdvError) -> Self {
        Self::Xdv(value)
    }
}

impl From<IoError> for SpxToLtxmlError {
    fn from(value: IoError) -> Self {
        Self::Io(value)
    }
}

impl From<TsemParseError> for SpxToLtxmlError {
    fn from(value: TsemParseError) -> Self {
        Self::Tsem(value)
    }
}

impl From<str::Utf8Error> for SpxToLtxmlError {
    fn from(value: str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}

/// SPX-to-`LtxmlIr` extractor.
#[derive(Debug)]
pub struct SpxToLtxmlEngine {
    builder: LtxmlBuilder,
}

impl Default for SpxToLtxmlEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SpxToLtxmlEngine {
    /// Create a new extractor rooted at `ltx:document`.
    pub fn new() -> Self {
        Self {
            builder: LtxmlBuilder::new_document(),
        }
    }

    /// Consume one raw `\special` payload.
    pub fn handle_special_text(&mut self, contents: &str) -> Result<(), SpxToLtxmlError> {
        // Raw text event: no JSON escaping required.
        if let Some(text) = contents.strip_prefix("tsem-text:") {
            self.builder
                .apply_event(TsemEvent::Text { text: text.to_string() });
            return Ok(());
        }

        // Raw math event: "tsem-math:<mode>:<tex>".  The mode field is always
        // a safe identifier ("inline" or "display"); only the TeX source can
        // contain backslashes or quotes.
        if let Some(payload) = contents.strip_prefix("tsem-math:") {
            if let Some((mode, tex)) = payload.split_once(':') {
                let mut attrs = BTreeMap::new();
                attrs.insert(QName::from("mode"), mode.to_string());
                attrs.insert(QName::from("tex"), tex.to_string());
                self.builder.apply_event(TsemEvent::Math {
                    element: QName::from("ltx:Math"),
                    attrs,
                    text: None,
                });
            }
            return Ok(());
        }

        // Raw graphics event: "tsem-graphics:<filename>".  Creates a
        // self-closing ltx:graphics element with the graphic attribute set.
        if let Some(filename) = contents.strip_prefix("tsem-graphics:") {
            let mut attrs = BTreeMap::new();
            attrs.insert(QName::from("graphic"), filename.to_string());
            self.builder.apply_event(TsemEvent::Begin {
                element: QName::from("ltx:graphics"),
                attrs,
            });
            self.builder.apply_event(TsemEvent::End {
                element: Some(QName::from("ltx:graphics")),
            });
            return Ok(());
        }

        // JSON-backed tsem: event.
        if contents.strip_prefix("tsem:").is_none() {
            return Ok(());
        }

        let event = parse_tsem_special(contents)?;
        self.builder.apply_event(event);
        Ok(())
    }

    /// Finish processing and return the generated IR root.
    pub fn finish(self) -> LtxmlNode {
        self.builder.finish()
    }

    /// Process an SPX stream into a LaTeXML-shaped IR tree.
    pub fn process_spx<R>(input: R) -> Result<LtxmlNode, SpxToLtxmlError>
    where
        R: Read,
    {
        let (engine, _bytes_read) = XdvParser::process(input, Self::new())?;
        Ok(engine.finish())
    }
}

impl XdvEvents for SpxToLtxmlEngine {
    type Error = SpxToLtxmlError;

    fn handle_header(&mut self, filetype: FileType, _comment: &[u8]) -> Result<(), Self::Error> {
        if filetype != FileType::Spx {
            return Err(SpxToLtxmlError::WrongFileType(filetype));
        }

        Ok(())
    }

    fn handle_special(&mut self, _x: i32, _y: i32, contents: &[u8]) -> Result<(), Self::Error> {
        let contents = str::from_utf8(contents)?;
        self.handle_special_text(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tectonic_ltxml_ir::{LtxmlChild, QName};
    use tectonic_xdv::XdvEvents;

    #[test]
    fn ignores_non_tsem_specials() {
        let mut engine = SpxToLtxmlEngine::new();
        engine.handle_special_text("tdux:emit").unwrap();
        let doc = engine.finish();
        assert!(doc.children.is_empty());
    }

    #[test]
    fn builds_ir_from_special_text() {
        let mut engine = SpxToLtxmlEngine::new();
        engine
            .handle_special_text(
                r#"tsem:{"event":"begin","element":"ltx:section","attrs":{"xml:id":"S1"}}"#,
            )
            .unwrap();
        engine
            .handle_special_text(r#"tsem:{"event":"text","text":"Body"}"#)
            .unwrap();
        engine
            .handle_special_text(r#"tsem:{"event":"end","element":"ltx:section"}"#)
            .unwrap();

        let doc = engine.finish();
        let LtxmlChild::Element { node } = &doc.children[0] else {
            panic!("expected section");
        };
        assert_eq!(node.name, QName::from("ltx:section"));
        assert_eq!(node.xml_id(), Some("S1"));
    }

    #[test]
    fn xdv_event_rejects_non_utf8_special() {
        let mut engine = SpxToLtxmlEngine::new();
        let err = engine.handle_special(0, 0, &[0xff]).unwrap_err();
        assert!(matches!(err, SpxToLtxmlError::Utf8(_)));
    }

    #[test]
    fn rejects_xdv_filetype() {
        let mut engine = SpxToLtxmlEngine::new();
        let err = engine.handle_header(FileType::Xdv, b"").unwrap_err();
        assert!(matches!(err, SpxToLtxmlError::WrongFileType(FileType::Xdv)));
    }

    #[test]
    fn raw_text_special_avoids_json_escaping() {
        let mut engine = SpxToLtxmlEngine::new();
        engine
            .handle_special_text(r#"tsem-text:My "quoted" title"#)
            .unwrap();
        let doc = engine.finish();
        let LtxmlChild::Text { text } = &doc.children[0] else {
            panic!("expected text child");
        };
        assert_eq!(text, r#"My "quoted" title"#);
    }

    #[test]
    fn raw_math_special_captures_tex_source_with_backslash() {
        let mut engine = SpxToLtxmlEngine::new();
        engine
            .handle_special_text(r"tsem-math:inline:\frac{a}{b}")
            .unwrap();
        let doc = engine.finish();
        let LtxmlChild::Element { node } = &doc.children[0] else {
            panic!("expected math element");
        };
        assert_eq!(node.name, QName::from("ltx:Math"));
        assert_eq!(node.attr("mode"), Some("inline"));
        assert_eq!(node.attr("tex"), Some(r"\frac{a}{b}"));
    }

    #[test]
    fn raw_graphics_special_creates_graphics_element() {
        let mut engine = SpxToLtxmlEngine::new();
        engine
            .handle_special_text("tsem-graphics:plot.png")
            .unwrap();
        let doc = engine.finish();
        let LtxmlChild::Element { node } = &doc.children[0] else {
            panic!("expected graphics element");
        };
        assert_eq!(node.name, QName::from("ltx:graphics"));
        assert_eq!(node.attr("graphic"), Some("plot.png"));
        assert!(node.children.is_empty());
    }
}
