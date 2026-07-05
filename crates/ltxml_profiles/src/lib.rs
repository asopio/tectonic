// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Output profiles for the LaTeXML-shaped HTML pipeline.
//!
//! Profiles bundle renderer defaults and semantic/debug preferences in the same
//! spirit as LaTeXML's `.opt` profiles, but using native Rust data structures.

use tectonic_ltxml_html::{MathRenderMode, RenderOptions};

/// A known built-in profile name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileName {
    /// Conservative default article profile.
    Article,
    /// MathJax-oriented article profile.
    MathJax,
    /// ATLAS-oriented placeholder profile.
    Atlas,
}

impl ProfileName {
    /// Parse a built-in profile name.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "article" | "default" => Some(Self::Article),
            "mathjax" | "modern" => Some(Self::MathJax),
            "atlas" => Some(Self::Atlas),
            _ => None,
        }
    }

    /// Return this profile's canonical name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::MathJax => "mathjax",
            Self::Atlas => "atlas",
        }
    }
}

/// Output splitting behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SplitMode {
    /// Generate a single HTML document.
    None,
    /// Split by section in a later pipeline stage.
    Section,
}

/// A native output profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputProfile {
    /// Profile name.
    pub name: ProfileName,
    /// Template identifier.
    pub template: &'static str,
    /// Split mode.
    pub split: SplitMode,
    /// CSS resources to associate with this profile.
    pub css: &'static [&'static str],
    /// Whether supporting assets should be copied/emitted.
    pub copy_assets: bool,
    /// Whether validation should be requested.
    pub validate: bool,
    /// Whether semantic IR should be retained for debugging.
    pub keep_ltxml_ir: bool,
    /// Whether unsupported constructs should be treated strictly.
    pub strict: bool,
    /// HTML renderer options.
    pub render_options: RenderOptions,
}

impl OutputProfile {
    /// Return the built-in article profile.
    pub fn article() -> Self {
        Self {
            name: ProfileName::Article,
            template: "article",
            split: SplitMode::None,
            css: &["tectonic-article.css"],
            copy_assets: true,
            validate: false,
            keep_ltxml_ir: false,
            strict: false,
            render_options: RenderOptions::default(),
        }
    }

    /// Return the built-in MathJax profile.
    pub fn mathjax() -> Self {
        Self {
            name: ProfileName::MathJax,
            template: "article",
            split: SplitMode::None,
            css: &["tectonic-article.css"],
            copy_assets: true,
            validate: false,
            keep_ltxml_ir: true,
            strict: false,
            render_options: RenderOptions {
                math_render_mode: MathRenderMode::MathJax,
                include_mathjax_script: true,
                ..RenderOptions::default()
            },
        }
    }

    /// Return the built-in ATLAS placeholder profile.
    pub fn atlas() -> Self {
        Self {
            name: ProfileName::Atlas,
            template: "atlas-article",
            split: SplitMode::None,
            css: &["tectonic-article.css", "tectonic-atlas.css"],
            copy_assets: true,
            validate: true,
            keep_ltxml_ir: true,
            strict: false,
            render_options: RenderOptions {
                math_render_mode: MathRenderMode::MathJax,
                include_mathjax_script: true,
                ..RenderOptions::default()
            },
        }
    }
}

/// Return a built-in output profile by name.
pub fn built_in_profile(name: &str) -> Option<OutputProfile> {
    Some(match ProfileName::parse(name)? {
        ProfileName::Article => OutputProfile::article(),
        ProfileName::MathJax => OutputProfile::mathjax(),
        ProfileName::Atlas => OutputProfile::atlas(),
    })
}

/// Iterate over all built-in profiles.
pub fn built_in_profiles() -> impl Iterator<Item = OutputProfile> {
    [OutputProfile::article(), OutputProfile::mathjax(), OutputProfile::atlas()].into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_names() {
        assert_eq!(ProfileName::parse("article"), Some(ProfileName::Article));
        assert_eq!(ProfileName::parse("default"), Some(ProfileName::Article));
        assert_eq!(ProfileName::parse("mathjax"), Some(ProfileName::MathJax));
        assert_eq!(ProfileName::parse("atlas"), Some(ProfileName::Atlas));
        assert_eq!(ProfileName::parse("missing"), None);
    }

    #[test]
    fn mathjax_profile_enables_mathjax_rendering() {
        let profile = built_in_profile("mathjax").unwrap();
        assert_eq!(profile.render_options.math_render_mode, MathRenderMode::MathJax);
        assert!(profile.render_options.include_mathjax_script);
        assert!(profile.keep_ltxml_ir);
    }

    #[test]
    fn atlas_profile_has_atlas_css_and_validation() {
        let profile = built_in_profile("atlas").unwrap();
        assert_eq!(profile.template, "atlas-article");
        assert!(profile.css.contains(&"tectonic-atlas.css"));
        assert!(profile.validate);
    }
}
