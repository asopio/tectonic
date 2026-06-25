// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Compile a line-oriented `tsem:` event stream to post-processed HTML.

use std::{env, fs, path::PathBuf};
use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_html::render_document_with_options;
use tectonic_ltxml_post::{normalize_graphics_and_tables, resolve_references};
use tectonic_ltxml_profiles::built_in_profile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: compile_tsem <input.tsem> <output.html> [profile]")?;
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: compile_tsem <input.tsem> <output.html> [profile]")?;
    let profile_name = args
        .next()
        .and_then(|name| name.into_string().ok())
        .unwrap_or_else(|| "mathjax".to_owned());
    let profile = built_in_profile(&profile_name)
        .ok_or_else(|| format!("unknown output profile `{profile_name}`"))?;

    let mut engine = SpxToLtxmlEngine::new();
    let contents = fs::read_to_string(&input)?;

    for line in contents.lines() {
        if !line.trim().is_empty() {
            engine.handle_special_text(line)?;
        }
    }

    let mut doc = engine.finish();
    resolve_references(&mut doc);
    normalize_graphics_and_tables(&mut doc);

    let html = render_document_with_options(&doc, &profile.render_options);

    fs::write(&output, html)?;
    println!("wrote {} using profile {}", output.display(), profile.name.as_str());
    Ok(())
}
