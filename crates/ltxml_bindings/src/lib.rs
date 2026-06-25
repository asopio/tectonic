// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Registry of oxidized LaTeXML-compatible package bindings.
//!
//! Phase 6 introduces package/class binding oxidation as explicit data. Each
//! entry records a LaTeXML source binding, its current support level, and the
//! Tectonic semantic support file that targets compatible `ltx:*` structures.

/// Current oxidation support level for a binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingSupport {
    /// No support beyond preserving unknown nodes/attributes.
    Planned,
    /// Support file exists with fallback/stub behavior.
    Fallback,
    /// Partial support for commonly used constructs.
    Partial,
    /// Broad support for the binding's normal behavior.
    Broad,
}

/// A known LaTeXML-compatible package/class binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingSpec {
    /// LaTeX class/package name.
    pub name: &'static str,
    /// Whether this entry describes a document class or package.
    pub kind: BindingKind,
    /// Tier from the implementation plan.
    pub tier: u8,
    /// Current support level.
    pub support: BindingSupport,
    /// Original LaTeXML binding file used as the compatibility reference.
    pub latexml_source: &'static str,
    /// Tectonic support file emitted/loaded for this binding, if any.
    pub tectonic_support: Option<&'static str>,
    /// Major `ltx:*` elements targeted by this binding.
    pub ltx_elements: &'static [&'static str],
}

/// The type of binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingKind {
    /// LaTeX document class.
    Class,
    /// LaTeX package/style file.
    Package,
    /// Core LaTeX/kernel constructs.
    Core,
}

/// Registered binding specs.
pub const BINDINGS: &[BindingSpec] = &[
    BindingSpec {
        name: "latex-core",
        kind: BindingKind::Core,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Engine/latex_constructs.pool.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/tectonic-semantic-html.tex"),
        ltx_elements: &[
            "ltx:document",
            "ltx:title",
            "ltx:creator",
            "ltx:personname",
            "ltx:section",
            "ltx:subsection",
            "ltx:p",
            "ltx:emph",
            "ltx:ref",
            "ltx:cite",
            "ltx:bibref",
        ],
    },
    BindingSpec {
        name: "article",
        kind: BindingKind::Class,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/article.cls.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/article.ltxmlir.tex"),
        ltx_elements: &["ltx:document", "ltx:title", "ltx:creator", "ltx:section", "ltx:subsection"],
    },
    BindingSpec {
        name: "amsmath",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/amsmath.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/amsmath.ltxmlir.tex"),
        ltx_elements: &["ltx:equation", "ltx:equationgroup", "ltx:Math"],
    },
    BindingSpec {
        name: "graphicx",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/graphicx.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/graphicx.ltxmlir.tex"),
        ltx_elements: &["ltx:graphics", "ltx:figure", "ltx:caption"],
    },
    BindingSpec {
        name: "hyperref",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/hyperref.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/hyperref.ltxmlir.tex"),
        ltx_elements: &["ltx:ref", "ltx:anchor"],
    },
    BindingSpec {
        name: "natbib",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Partial,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/natbib.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/natbib.ltxmlir.tex"),
        ltx_elements: &["ltx:cite", "ltx:bibref", "ltx:bibliography", "ltx:bibitem"],
    },
    BindingSpec {
        name: "booktabs",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Fallback,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/booktabs.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/tabular.ltxmlir.tex"),
        ltx_elements: &["ltx:tabular", "ltx:tr", "ltx:td"],
    },
    BindingSpec {
        name: "array",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Fallback,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/array.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/tabular.ltxmlir.tex"),
        ltx_elements: &["ltx:tabular", "ltx:tr", "ltx:td"],
    },
    BindingSpec {
        name: "longtable",
        kind: BindingKind::Package,
        tier: 1,
        support: BindingSupport::Fallback,
        latexml_source: "reference_latexml/lib/LaTeXML/Package/longtable.sty.ltxml",
        tectonic_support: Some("crates/engine_xetex/support/ltxml-bindings/tabular.ltxmlir.tex"),
        ltx_elements: &["ltx:table", "ltx:tabular", "ltx:tr", "ltx:td"],
    },
];

/// Look up a binding by class/package name.
pub fn find_binding(name: &str) -> Option<&'static BindingSpec> {
    BINDINGS.iter().find(|binding| binding.name == name)
}

/// Iterate over registered Tier 1 bindings.
pub fn tier_one_bindings() -> impl Iterator<Item = &'static BindingSpec> {
    BINDINGS.iter().filter(|binding| binding.tier == 1)
}

/// Return bindings with a concrete Tectonic support file.
pub fn bindings_with_support_files() -> impl Iterator<Item = &'static BindingSpec> {
    BINDINGS.iter().filter(|binding| binding.tectonic_support.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_registered_bindings() {
        let article = find_binding("article").unwrap();
        assert_eq!(article.kind, BindingKind::Class);
        assert_eq!(article.support, BindingSupport::Partial);
        assert!(article.ltx_elements.contains(&"ltx:section"));

        let amsmath = find_binding("amsmath").unwrap();
        assert!(amsmath.ltx_elements.contains(&"ltx:Math"));
        assert!(find_binding("not-a-package").is_none());
    }

    #[test]
    fn tier_one_registry_covers_initial_vertical_slice() {
        let names = tier_one_bindings().map(|binding| binding.name).collect::<Vec<_>>();
        for expected in ["latex-core", "article", "amsmath", "graphicx", "hyperref", "natbib"] {
            assert!(names.contains(&expected));
        }
    }

    #[test]
    fn concrete_bindings_have_support_files() {
        for binding in bindings_with_support_files() {
            assert!(binding.tectonic_support.unwrap().ends_with(".tex"));
        }
    }

    #[test]
    fn registered_support_files_exist() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for binding in bindings_with_support_files() {
            let path = repo_root.join(binding.tectonic_support.unwrap());
            assert!(path.exists(), "missing support file for {}: {}", binding.name, path.display());
        }
    }
}
