// Copyright 2025 the Tectonic Project
// Licensed under the MIT License.

//! Compile a line-oriented `tsem:` event stream to post-processed HTML.

use std::{env, fs, path::PathBuf};
use tectonic_engine_spx2ltxml::SpxToLtxmlEngine;
use tectonic_ltxml_html::{render_document_with_options, MathRenderMode, RenderOptions};
use tectonic_ltxml_post::resolve_references;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = args.next().map(PathBuf::from).ok_or("usage: compile_tsem <input.tsem> <output.html>")?;
    let output = args.next().map(PathBuf::from).ok_or("usage: compile_tsem <input.tsem> <output.html>")?;

    let mut engine = SpxToLtxmlEngine::new();
    let contents = fs::read_to_string(&input)?;

    for line in contents.lines() {
        if !line.trim().is_empty() {
            engine.handle_special_text(line)?;
        }
    }

    let mut doc = engine.finish();
    resolve_references(&mut doc);

    let html = render_document_with_options(
        &doc,
        &RenderOptions {
            math_render_mode: MathRenderMode::MathJax,
            include_mathjax_script: true,
            ..RenderOptions::default()
        },
    );

    fs::write(&output, html)?;
    println!("wrote {}", output.display());
    Ok(())
}
