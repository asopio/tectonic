# Plan for Adapting LaTeXML-Style HTML Conversion Functionality in Tectonic

## Executive Summary

Tectonic already has an experimental HTML pipeline based on XeTeX-generated SPX and `spx2html`. That pipeline is excellent for exact glyph placement and TeX-driven canvas rendering, but it currently requires source documents and macros to explicitly emit HTML-oriented `tdux:` specials. LaTeXML takes the opposite approach: it models TeX/LaTeX semantically, digests macros into an intermediate XML document, applies package/class bindings, and post-processes that semantic document into HTML, MathML, SVG, cross-references, bibliographies, indices, and other web outputs.

The right adaptation strategy is **not** to port LaTeXML wholesale into Rust or make Tectonic emulate LaTeXML's Perl engine. Tectonic already has a real XeTeX engine and a working asset/font subsystem. Instead, Tectonic should add a **LaTeXML-shaped semantic conversion layer** that preserves LaTeXML's existing `ltx:*` document structure as closely as possible, then incrementally "oxidizes" LaTeXML functionality package binding by package binding:

1. Keep Tectonic's real TeX execution for compatibility, file resolution, package loading, counters, aux files, BibTeX/Biber workflows, and math layout.
2. Add a structured semantic capture mechanism that standard LaTeX bindings can target without hand-authoring low-level HTML specials.
3. Introduce a Rust-native `LtxmlIr` intermediate representation that is intentionally near-isomorphic to LaTeXML's `ltx:*` XML model, including element names, key attributes, and containment structure.
4. Preserve unknown `ltx:*` elements and attributes so unsupported packages degrade gracefully and can be oxidized later without changing the IR shape.
5. Port LaTeXML class/package bindings incrementally into Rust-side and/or TeX-side bindings, using LaTeXML's own XML output as the compatibility oracle.
6. Add post-processors for document normalization, cross-references, citations, bibliography, table of contents, math conversion, graphics, splitting, and final HTML rendering.
7. Preserve and reuse Tectonic's unique strengths: deterministic bundle management, fast Rust implementation, real XeTeX execution, asset generation, and high-fidelity math/glyph rendering.

The deliverable should be a staged implementation that starts with a LaTeXML-compatible `article` subset and grows toward ATLAS/hep paper support by oxidizing one binding family at a time.

## Source Baseline

### Existing Tectonic HTML Model

The current Tectonic HTML work is centered around:

- `crates/engine_spx2html/src/lib.rs`: SPX-to-HTML engine entry point, asset handling, file emission modes.
- `crates/engine_spx2html/src/specials.rs`: parser for `tdux:` specials such as template selection, direct text, manual element starts/ends, canvas starts/ends, output path setting, and asset provisioning.
- `crates/engine_spx2html/src/emission.rs`: HTML content accumulation, element stack management, auto spacing, font-derived element management, and canvas glyph/rule collection.
- `crates/engine_spx2html/src/templating.rs`: Tera-based template rendering and output path management.
- `crates/engine_spx2html/src/html.rs`: HTML5 element metadata and auto-close rules.

Current limitation: the TeX document must be HTML-aware and emit the required specials. Standard LaTeX documents do not naturally produce semantic HTML structure.

### Relevant LaTeXML Concepts to Adapt

LaTeXML's source shows a stable, mature architecture with several components worth adapting conceptually:

- **Mouth/Gullet/Stomach digestion model**: `LaTeXML::Core::Stomach` digests tokens and invokes definitions, producing boxes/whatsits and tracking state.
- **Document construction model**: `LaTeXML::Core::Document` builds an XML DOM with schema-aware containment, auto-open/auto-close, IDs, labels, and node properties.
- **Binding files**: `.ltxml` files define class/package behavior with `DefMacro`, `DefPrimitive`, `DefConstructor`, counters, resources, schemas, and semantic replacements.
- **Semantic schema**: `LaTeXML/resources/RelaxNG/LaTeXML*.rnc` defines the `ltx:*` document model.
- **Post-processing pipeline**: `LaTeXML::Post::*` modules perform cross-reference resolution, MathML conversion, graphics processing, bibliography/index generation, splitting, writing, and resource management.
- **Profiles/options**: `resources/Profiles/*.opt` bundle conversion choices such as output format, preloads, math formats, and paths.
- **Math representations**: `LaTeXML::Post::MathML`, `Post::TeXMath`, `Post::UnicodeMath`, `Post::SVG`, and XMath support multiple math outputs and parallel markup.

## Guiding Principles

1. **Do not replace XeTeX with a LaTeXML-style TeX interpreter.** Tectonic's advantage is that it runs a real engine. Re-implementing TeX digestion in Rust would be a large, high-risk duplication of LaTeXML.
2. **Move from manual HTML specials to semantic events.** The immediate pain point is that HTML structure must be manually specified. Introduce semantic capture commands that standard classes/packages can emit automatically.
3. **Separate semantic structure from presentation.** LaTeXML succeeds because it first creates a semantic XML document and only later renders HTML/MathML/SVG. Tectonic should adopt the same separation.
4. **Make package support incremental.** Start with core LaTeX, `article`, common math packages, graphics, hyperref, natbib/biblatex workflows, and ATLAS-specific macros. Avoid attempting full package parity upfront.
5. **Make LaTeXML compatibility the primary IR constraint.** The Rust IR should preserve LaTeXML's `ltx:*` output structure closely enough that fixtures can be diffed against LaTeXML before HTML rendering is considered.
6. **Expose debuggable intermediate output.** Provide `--outfmt ltxml-ir`, `--keep-ltxml-ir`, or similar to inspect the LaTeXML-shaped IR before HTML rendering.
7. **Use existing Rust crates where practical.** Use `quick-xml`, `roxmltree`, `markup5ever`, `html5ever`, `serde`, `tera`, `pulldown-cmark` if needed, rather than inventing all infrastructure.
8. **Oxidize incrementally.** Port LaTeXML package/class bindings one at a time, preserving the expected `ltx:*` output shape and adding Rust-native validation/post-processing as coverage grows.

## Proposed Target Architecture

```text
LaTeX source
   |
   v
Tectonic XeTeX engine in HTML-semantic mode
   |
   | emits SPX + semantic events/specials + assets + aux data
   v
Semantic capture layer
   |
   | builds Rust-native LaTeXML-shaped IR (LtxmlIr)
   | serializes/debugs as ltx:* XML/JSON
   v
LaTeXML-compatible normalization/post-processing pipeline
   |-- IDs and labels
   |-- references and links
   |-- table of contents
   |-- bibliography/citation resolution
   |-- figure/table numbering
   |-- math conversion or canvas attachment
   |-- graphics/resource processing
   |-- accessibility metadata
   v
HTML renderer
   |-- single-page HTML
   |-- split HTML tree
   |-- optional MathML/SVG/canvas math modes
   |-- CSS/JS/assets
```

## New Core Components

### 1. LaTeXML-Shaped IR (`LtxmlIr`)

Add a new crate, tentatively:

```text
crates/ltxml_ir/
```

Responsibilities:

- Represent a document tree independent of HTML layout while preserving LaTeXML's `ltx:*` structure as closely as possible.
- Preserve LaTeXML element names, key attributes, text nodes, namespaces, containment expectations, and post-processing metadata.
- Preserve source locators, TeX command provenance, labels, counters, IDs, math source, asset references, and diagnostics.
- Support serialization to LaTeXML-like XML as the primary debug/interchange format, with JSON as a secondary debug format.
- Support schema-like validation against a progressively implemented subset of LaTeXML's RelaxNG schemas.
- Preserve unknown elements and attributes so that unsupported bindings can round-trip through the IR until they are oxidized.

The first milestone should intentionally avoid a Tectonic-native semantic vocabulary. It should model LaTeXML's vocabulary directly enough that a fixture can be processed by LaTeXML and Tectonic and compared at the `ltx:*` tree level.

Suggested node model:

```rust
pub enum LtxmlName {
    Document,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    P,
    Title,
    Toctitle,
    Creator,
    Personname,
    Contact,
    Text,
    Emph,
    Ref,
    Cite,
    Bibref,
    Equation,
    Equationgroup,
    Math,
    XMath,
    XMTok,
    XMApp,
    XMRef,
    XMHint,
    XMText,
    Figure,
    Table,
    Caption,
    Tabular,
    Thead,
    Tbody,
    Tfoot,
    Tr,
    Td,
    Graphics,
    Note,
    Resource,
    Error,
    Other(QName),
}

pub struct LtxmlNode {
    pub name: LtxmlName,
    pub attrs: BTreeMap<QName, String>,
    pub children: Vec<LtxmlChild>,
    pub source: Option<SourceSpan>,
    pub tex_command: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

pub enum LtxmlChild {
    Element(LtxmlNode),
    Text(String),
}
```

Typed helpers can sit on top of the generic model for ergonomics, but the stored representation should remain close to XML:

```rust
impl LtxmlNode {
    pub fn xml_id(&self) -> Option<&str>;
    pub fn labels(&self) -> impl Iterator<Item = &str>;
    pub fn labelref(&self) -> Option<&str>;
    pub fn class_tokens(&self) -> impl Iterator<Item = &str>;
}
```

This approach lets Tectonic use LaTeXML as an oracle: the same test document can be run through LaTeXML, serialized as `ltx:*` XML, and structurally compared with Tectonic's `LtxmlIr` output.

### 2. Semantic Event Protocol

Extend or complement `tdux:` specials with a more structured event protocol, tentatively `tsem:`:

```tex
\special{tsem:{"event":"begin","element":"ltx:section","attrs":{"xml:id":"S1","refnum":"1"}}}
\special{tsem:{"event":"begin","element":"ltx:title"}}
Introduction
\special{tsem:{"event":"end","element":"ltx:title"}}
\special{tsem:{"event":"begin","element":"ltx:p"}}
\special{tsem:{"event":"math","element":"ltx:Math","attrs":{"mode":"inline","tex":"x^2"}}}
\special{tsem:{"event":"end","element":"ltx:p"}}
\special{tsem:{"event":"end","element":"ltx:section"}}
```

The existing `tdux:` specials are low-level HTML controls. `tsem:` should be high-level semantic controls.

Implementation option:

- Add `Special::SemanticEvent(&str)` to `crates/engine_spx2html/src/specials.rs`, or create a sibling crate that consumes SPX before/alongside `spx2html`.
- Prefer a structured encoding that is robust against spaces and braces. Candidates:
  - JSON payload: `\special{tsem:{"event":"begin","kind":"section",...}}`
  - key-value payload with TeX escaping rules.
  - CBOR/base64 for later, but not for early development.

Recommendation: start with line-oriented JSON because Rust parsing is straightforward with `serde_json`, it is easy to debug, and it avoids ad hoc parsing bugs.

### 3. Semantic SPX Processor

Add a processor that reads SPX/XDV events and builds `LtxmlIr` rather than directly producing HTML.

Tentative crate:

```text
crates/engine_spx2sem/
```

Responsibilities:

- Parse XDV/SPX events using `tectonic_xdv` as `spx2html` does.
- Track current semantic node stack.
- Capture text runs as semantic text rather than only raw HTML content.
- Associate low-level canvas fragments with semantic math nodes.
- Preserve asset definitions and font/canvas data for downstream rendering.
- Produce `.ltxml.xml` and `.ltxml-ir.json` debug outputs.

This processor can initially coexist with `engine_spx2html`; later, `spx2html` can become a renderer from `LtxmlIr`.

### 4. LaTeX Semantic Support Package

Create TeX support files that are automatically preloaded in `--outfmt html` or a new `--outfmt semantic-html` mode.

Tentative files:

```text
crates/engine_xetex/support/tectonic-semantic-html.tex
crates/engine_xetex/support/tectonic-semantic-latex.ltx
```

Responsibilities:

- Patch core LaTeX commands to emit `tsem:` events.
- Provide wrappers for document structure, title metadata, sectioning, paragraphs, lists, captions, labels, refs, citations, footnotes, math environments, tabulars, and graphics.
- Be careful not to break PDF output. These patches should be active only in semantic HTML mode.

Initial macro targets:

- `\documentclass`, `\begin{document}`, `\end{document}`
- `\title`, `\author`, `\date`, `\maketitle`, `abstract`
- `\part`, `\chapter`, `\section`, `\subsection`, `\subsubsection`, `\paragraph`
- `\label`, `\ref`, `\pageref`, `\autoref`, `\nameref`
- `\cite`, `\citep`, `\citet`, `\nocite`, bibliography commands
- `equation`, `align`, `gather`, `multline`, inline math, display math
- `figure`, `table`, `caption`, `subcaption`
- `itemize`, `enumerate`, `description`, `\item`
- `tabular`, `array`, `longtable`
- `\includegraphics`
- `verbatim`, `lstlisting`, `minted` fallback behavior

### 5. Package Binding Oxidation Layer

LaTeXML's binding files are a major reason it supports many packages. Tectonic should treat them as the migration map and oxidize them incrementally into Rust/TeX-native equivalents while preserving their `ltx:*` output shape.

The goal is not to execute Perl `.ltxml` files inside Tectonic. The goal is to port their behavior binding by binding, using LaTeXML's output as the oracle.

#### Oxidation workflow

For each class/package binding:

1. Select a small fixture corpus that exercises the binding.
2. Run the fixture through LaTeXML and save the generated `ltx:*` XML.
3. Implement the corresponding Tectonic binding so it emits events or IR nodes that produce the same `ltx:*` structure.
4. Diff Tectonic's serialized `LtxmlIr` against LaTeXML's XML, allowing controlled differences such as source locators, generated IDs, or math rendering metadata.
5. Add the fixture and compatibility diff to CI.
6. Mark unsupported macros/options with explicit diagnostics and preserve unknown data where possible.

#### Binding implementation options

##### Approach A: TeX-native compatibility bindings

For each package/class, ship a corresponding `.tsem.tex` or `.ltxmlir.tex` file that patches macros when the package is loaded and emits LaTeXML-shaped semantic events.

Example:

```text
semantic-bindings/article.ltxmlir.tex
semantic-bindings/amsmath.ltxmlir.tex
semantic-bindings/graphicx.ltxmlir.tex
semantic-bindings/hyperref.ltxmlir.tex
semantic-bindings/natbib.ltxmlir.tex
semantic-bindings/atlasphysics.ltxmlir.tex
```

Pros:

- Works with real XeTeX expansion and package behavior.
- Easier to integrate with Tectonic's current architecture.
- Does not need a separate TeX interpreter.
- Can be compared directly against LaTeXML's emitted `ltx:*` structure.

Cons:

- Harder to express semantic transformations than LaTeXML's `DefConstructor` replacement strings.
- Requires careful LaTeX patching.

##### Approach B: Rust-readable declarative binding DSL

Create a Rust-readable binding format inspired by LaTeXML's `DefConstructor`, but targeting `ltx:*` names directly:

```toml
[[constructor]]
command = "\\emph"
args = ["group"]
output = { element = "ltx:emph", children = ["#1"] }

[[environment]]
name = "abstract"
begin = { element = "ltx:abstract" }
end = true
```

Pros:

- Easier to statically analyze, validate, and test.
- Package support can be data-driven.
- Similar in spirit to `.ltxml` without Perl.
- Can eventually absorb many simple `.ltxml` `DefConstructor` cases semi-mechanically.

Cons:

- Requires a way to intercept macro invocations before or during TeX execution.
- Hard because Tectonic currently lets XeTeX execute macros directly.

##### Approach C: Hybrid generated bindings

For simple `.ltxml` cases, write tooling that reads a constrained subset of LaTeXML binding patterns and generates either TeX-native patches or Rust DSL entries. Complex bindings are manually oxidized.

Recommendation: start with **Approach A** for feasibility, but define `LtxmlIr`, semantic events, tests, and diff tooling around LaTeXML-compatible XML from the beginning. Add Approach B/C later for simple bindings that can be ported more mechanically.

### 6. Post-Processing Pipeline

Add a post-processing crate:

```text
crates/semantic_post/
```

Initial processors:

1. **NormalizeTree**
   - Merge adjacent text nodes.
   - Ensure paragraphs are inside allowed containers.
   - Normalize whitespace.
   - Convert unknown inline/block nodes to safe HTML-compatible fallback nodes.

2. **GenerateIds**
   - Generate stable IDs for sections, equations, figures, tables, footnotes, bibliography items.
   - Use source labels and counters when available.

3. **ResolveLabelsAndRefs**
   - Build label map from `\label` events and aux data.
   - Replace `Reference` nodes with links and displayed text.
   - Preserve unresolved refs with diagnostics.

4. **BuildToc**
   - Generate table of contents from section tree.
   - Support configurable depth.

5. **ResolveCitations**
   - Build bibliography/citation graph from `.aux`, `.bbl`, BibTeX/Biber outputs, or semantic events.
   - Support numeric and author-year rendering initially through simple styles.

6. **ProcessMath**
   - Attach TeX source, display mode, equation number, labels, and rendered representation.
   - Support output modes:
     - Tectonic canvas/high-fidelity HTML.
     - Presentation MathML.
     - TeX source for MathJax fallback.
     - SVG/image fallback for complex cases.

7. **ProcessGraphics**
   - Resolve `\includegraphics` paths through Tectonic's I/O system.
   - Copy/convert assets where needed.
   - Attach dimensions, alt text, captions, and links.

8. **SplitDocument**
   - Optional multi-page HTML split by section/chapter.
   - Generate navigation links and resource-relative paths.

9. **AccessibilityPass**
   - Add landmarks, roles, alt text diagnostics, heading-level checks, table header checks.

10. **WriteHtml**
   - Render final HTML with templates and CSS.

### 7. HTML Renderer

Refactor current `spx2html` concepts into a renderer that can consume `LtxmlIr`:

```text
crates/semantic_html/
```

Responsibilities:

- Map semantic nodes to HTML5 tags.
- Reuse existing `html.rs` element validation/auto-close logic where useful.
- Reuse existing Tera templating, but feed it a richer structured context.
- Reuse asset handling and output path logic from `spx2html`.
- Support multiple templates:
  - plain article
  - arXiv-like scholarly article
  - ATLAS style
  - split documentation/book

HTML mapping examples:

| `LtxmlIr` / `ltx:*` node | HTML output |
|---|---|
| `ltx:document` | `<main class="ltx_document">` |
| `ltx:section` / `ltx:subsection` | `<section><h1/h2>...</h1/h2>...</section>` |
| `ltx:p` | `<p>` |
| `ltx:text` | `<span>` when styling is needed, otherwise text |
| `ltx:emph` | `<em>` |
| `ltx:ref` | `<a>` |
| `ltx:Math` | `<math>` / `<span class="ltx_Math">` / canvas fragment |
| `ltx:equation` | `<figure class="ltx_equation">` or `<div class="ltx_equation">` |
| `ltx:figure` | `<figure>` |
| `ltx:caption` | `<figcaption>` |
| `ltx:cite` / `ltx:bibref` | `<a href="#bib..." class="ltx_cite">[...]</a>` |
| `ltx:bibliography` | `<section class="ltx_bibliography">` |

## Implementation Progress

As of the current `dev-xmlir-phase-8` branch, the first eight implementation phases have been completed as incremental vertical slices:

| Phase | Status | Commit | Main artifacts |
|---|---|---|---|
| Phase 1: LaTeXML-shaped IR and event infrastructure | Complete | `4ab7beea` | `crates/ltxml_ir`, `crates/engine_spx2ltxml` |
| Phase 2: Core article semantic patches/fixtures | Complete | `178fa9c6` | `crates/engine_xetex/support/tectonic-semantic-html.tex`, article `tsem` fixture tests |
| Phase 3: Minimal HTML renderer | Complete | `8bdcd1d1` | `crates/ltxml_html` single-page renderer |
| Phase 4: Math strategy | Complete | `ded04c58` | `MathMetadata`, MathJax-compatible rendering mode |
| Phase 5: References/citations/bibliography | Complete | `91b036a0` | `crates/ltxml_post` reference and bibliography post-processing |
| Phase 6: Package/class binding oxidation registry | Complete | `ac7e9a22` | `crates/ltxml_bindings`, package-specific `.ltxmlir.tex` support files |
| Phase 7: Graphics and tables | Complete | `40665729` | graphics/table normalization in `crates/ltxml_post` |
| Phase 8: Output profiles | Complete | `cd343783` | `crates/ltxml_profiles`, profile-aware `compile_tsem` demo |

The current implementation is still a prototype and does not yet wire these crates into `tectonic -X compile --outfmt html`; instead it demonstrates the pipeline from `tsem:` semantic events through `LtxmlIr`, post-processing, and HTML rendering.

### Current implemented crates

- `tectonic_ltxml_ir`: Rust-native, LaTeXML-shaped IR with `ltx:*` node/attribute preservation, JSON `tsem:` parsing, XML serialization, diagnostics, and math metadata helpers.
- `tectonic_engine_spx2ltxml`: SPX-facing semantic-special extractor that consumes `tsem:` specials into `LtxmlIr`.
- `tectonic_ltxml_html`: single-page HTML renderer for the current `ltx:*` subset, including MathJax-compatible math rendering.
- `tectonic_ltxml_post`: post-processing for labels, references, citations, bibliography items, and unresolved-reference diagnostics.
- `tectonic_ltxml_bindings`: registry of oxidized binding coverage and support-file metadata for Tier 1 bindings.
- `tectonic_ltxml_profiles`: native output profiles (`article`, `mathjax`, `atlas`) that configure templates, CSS resources, MathJax behavior, validation flags, asset behavior, and debug IR retention.

### Current semantic support files

- `crates/engine_xetex/support/tectonic-semantic-html.tex`
- `crates/engine_xetex/support/ltxml-bindings/article.ltxmlir.tex`
- `crates/engine_xetex/support/ltxml-bindings/amsmath.ltxmlir.tex`
- `crates/engine_xetex/support/ltxml-bindings/graphicx.ltxmlir.tex`
- `crates/engine_xetex/support/ltxml-bindings/hyperref.ltxmlir.tex`
- `crates/engine_xetex/support/ltxml-bindings/natbib.ltxmlir.tex`
- `crates/engine_xetex/support/ltxml-bindings/tabular.ltxmlir.tex`

### Current demo artifact

A small binding demonstration was added under:

```text
examples/ltxml_bindings_demo/
```

It contains:

- `binding_demo.tex`: conceptual LaTeX source showing how the oxidized bindings are intended to map from common article/package constructs.
- `binding_demo.tsem`: current hand-authored semantic event stream equivalent.
- `binding_demo.html`: generated HTML output.

The demo was compiled with:

```bash
pixi run cargo run -p tectonic_ltxml_post --example compile_tsem --manifest-path Cargo.toml -- \
  examples/ltxml_bindings_demo/binding_demo.tsem \
  examples/ltxml_bindings_demo/binding_demo.html
```

This validates the current end-to-end prototype path:

```text
tsem events -> LtxmlIr -> resolve refs/citations -> HTML + MathJax-compatible math
```

A richer Phase 7/8 demonstration was also added under:

```text
examples/ltxml_phase7_phase8_demo/
```

It contains:

- `phase7_phase8_demo.tex`: a regular LaTeX document with `amsmath`, `graphicx`, `booktabs`, `hyperref`, section/equation/figure/table references, a raster image, and a table.
- `phase7_plot.png`: image asset used by the LaTeX and semantic demos.
- `phase7_phase8_demo.pdf`: output produced by the prebuilt pixi `tectonic` CLI.
- `phase7_phase8_demo.tsem`: current hand-authored semantic event stream equivalent.
- `phase7_phase8_demo.html`: output generated through the profile-based prototype pipeline using the `atlas` profile.

The LaTeX/PDF side was compiled with:

```bash
pixi run tectonic -X compile --outdir examples/ltxml_phase7_phase8_demo \
  examples/ltxml_phase7_phase8_demo/phase7_phase8_demo.tex
```

The semantic/HTML side was compiled with:

```bash
pixi run cargo run -p tectonic_ltxml_post --example compile_tsem --manifest-path Cargo.toml -- \
  examples/ltxml_phase7_phase8_demo/phase7_phase8_demo.tsem \
  examples/ltxml_phase7_phase8_demo/phase7_phase8_demo.html \
  atlas
```

This richer demo exercises:

- Phase 7 graphics candidate resolution (`phase7_plot`/candidates -> `phase7_plot.png`).
- Preservation of image alt text, width, and height.
- Phase 7 table normalization (`ltx:tr` rows wrapped in `ltx:tbody`).
- Header cells rendered as `<th>` with alignment preserved.
- Resolved section/equation/figure/table references.
- Phase 8 profile selection via the `atlas` profile.
- MathJax-compatible display math output.

### Current test status

The latest Phase 8 test run used:

```bash
pixi run cargo test -p tectonic_ltxml_profiles -p tectonic_ltxml_bindings -p tectonic_ltxml_ir -p tectonic_engine_spx2ltxml -p tectonic_ltxml_html -p tectonic_ltxml_post --manifest-path Cargo.toml
```

Result: `37` tests passed.

## Implementation Roadmap

### Phase 0: Design and Test Corpus

Deliverables:

- Define target scope for the first milestone: simple `article` documents with sections, paragraphs, inline/display math, figures, labels/refs, bibliography, and common HEP macros.
- Collect a minimal test corpus:
  - tiny article
  - math-heavy article
  - figure/caption/reference article
  - bibliography article
  - ATLAS note excerpt
  - failure cases for unsupported packages
- Add expected LaTeXML `ltx:*` XML snapshots, Tectonic `LtxmlIr` snapshots, and later HTML snapshots.
- Define a compatibility diff policy for generated IDs, source locator metadata, whitespace normalization, and math rendering metadata.
- Decide initial math output mode: recommend preserving LaTeXML-like `ltx:Math` with TeX source plus MathJax/Tectonic canvas fallback, then MathML later.

Success criteria:

- A document can be compiled to PDF as before.
- The same source can be compiled to semantic debug output without crashing.
- Unsupported constructs produce diagnostics, not panics.

### Phase 1: LaTeXML-Shaped IR and Event Infrastructure — Complete

Deliverables:

- Add semantic event parser, preferably JSON-backed, with event payloads targeting `ltx:*` element names and attributes directly.
- Add `LtxmlIr` crate with XML-shaped nodes, namespaces, attributes, source spans, diagnostics, unknown element/attribute preservation, and serialization.
- Add `engine_spx2ltxml` or `engine_spx2sem` that reads SPX and builds `LtxmlIr` from semantic specials plus text/math/canvas events.
- Add CLI plumbing:
  - `tectonic -X compile --outfmt ltxml-ir input.tex`
  - or `--keep-ltxml-ir` with `--outfmt html`
- Add debug output files:
  - `input.ltxml-ir.json`
  - `input.ltxml.xml`

Success criteria:

- A hand-authored TeX file emitting `tsem:` specials produces a valid LaTeXML-shaped document tree.
- Serialized XML uses `ltx:*` names and attributes close enough to compare with LaTeXML fixtures.
- Text escaping, Unicode handling, nested nodes, labels, unknown elements/attributes, and diagnostics work.

### Phase 2: Core LaTeX Semantic Patches — Complete as Fixture/Support Slice

Deliverables:

- Auto-load `tectonic-semantic-html.tex` in semantic HTML mode.
- Patch core LaTeX document and sectioning commands.
- Capture paragraphs and text robustly.
- Capture inline/display math events with TeX source and display mode.
- Capture labels and refs.
- Capture figures, captions, and includegraphics at a basic level.

Initial supported subset:

```tex
\documentclass{article}
\title{...}
\author{...}
\begin{document}
\maketitle
\begin{abstract}...\end{abstract}
\section{...}\label{...}
Text \emph{...} \ref{...} $x^2$.
\begin{equation}\label{eq:x} x^2+y^2=z^2 \end{equation}
\begin{figure}\includegraphics{...}\caption{...}\label{...}\end{figure}
\end{document}
```

Success criteria:

- No manual `tsem:` or `tdux:` specials needed in the source document.
- Generated `LtxmlIr` serializes to a coherent `ltx:*` hierarchy.
- The `article` fixture output can be structurally compared against LaTeXML's XML with only documented differences.
- HTML renderer can output readable static HTML from the same IR.

### Phase 3: Minimal HTML Renderer — Complete

Deliverables:

- Add `semantic_html` renderer from LtxmlIr to HTML.
- Use Tera for page-level templates.
- Emit CSS copied/adapted from LaTeXML's conceptual class names only if licensing and attribution are verified, or create fresh CSS with compatible class naming.
- Implement single-page output first.
- Add command:

```bash
tectonic -X compile --outfmt html-semantic input.tex
```

or evolve existing `--outfmt html` behind an experimental feature flag.

Success criteria:

- Simple articles render as browser-readable HTML.
- Section hierarchy, paragraphs, emphasis, links, equations, figures, captions, and bibliography placeholders are present.
- Output does not require document-authored templates.

### Phase 4: Math Strategy — Initial Implementation Complete

Tectonic has a major opportunity to improve on LaTeXML by combining semantic math with high-fidelity layout.

Deliverables:

- Store for every math node:
  - original TeX source
  - display/inline mode
  - equation number and label
  - canvas layout representation if available
  - optional normalized semantic math tree later
- Initial HTML output:
  - `script type="math/tex"` or MathJax-compatible source for reliable display.
  - Tectonic canvas as an optional high-fidelity rendering mode.
- Later output:
  - Presentation MathML generation inspired by LaTeXML's `Post::MathML` and `XMath` model.
  - Parallel markup with MathML plus TeX annotation.

Implementation options:

1. **Short-term**: output TeX source and use MathJax/KaTeX. Fastest path to usable HTML.
2. **Medium-term**: reuse Tectonic canvas rendering for exact visual fidelity, with TeX source as accessibility fallback.
3. **Long-term**: implement a Rust math semantic parser inspired by LaTeXML's XMath and MathML post-processing.

Success criteria:

- Inline/display math renders correctly in the browser.
- Equation references link correctly.
- Math source is preserved for accessibility, search, and fallback.

### Phase 5: Cross-References, Citations, and Bibliographies — Initial Implementation Complete

Deliverables:

- Integrate with Tectonic's existing rerun/aux machinery.
- Parse labels from semantic events and `.aux` files.
- Resolve `\ref`, `\eqref`, `\pageref` fallback, `\autoref`, `\nameref`.
- Support BibTeX-generated `.bbl` first, because it is easier than full BibTeX style emulation.
- Capture bibliography items semantically from `thebibliography` and `.bbl` input.
- Add simple citation renderers:
  - numeric `[1]`
  - compressed numeric `[1–3]`
  - author-year later

Success criteria:

- Documents with labels and citations produce linked HTML.
- Bibliography section is rendered as structured HTML.
- Unresolved references/citations are reported cleanly.

### Phase 6: Package/Class Binding Oxidation

Prioritize package support by real target documents, especially ATLAS/HEP papers. Each package should be treated as an oxidation unit: preserve the LaTeXML binding's observable `ltx:*` output first, then replace the implementation with Rust/TeX-native logic.

Tier 1 packages/classes:

- `article`, `report` where feasible
- `amsmath`, `amssymb`, `amsfonts`, `mathtools`
- `graphicx`, `xcolor`, `color`
- `hyperref`, `url`, `xurl`
- `caption`, `subcaption`
- `booktabs`, `array`, `longtable`, `tabularx`
- `natbib`, basic BibTeX `.bbl`
- common HEP/ATLAS style files and macros

Tier 2 packages:

- `siunitx`
- `cleveref`
- `listings`
- `multirow`
- `enumitem`
- `authblk`
- `lineno`

Tier 3 / fallback packages:

- TikZ/PGF: SVG/image fallback first.
- PSTricks: image fallback.
- minted: pre-rendered/verbatim fallback.
- complex publisher classes: targeted support as needed.

For each package:

1. Identify the relevant `.ltxml` binding files and the `ltx:*` elements/attributes they emit.
2. Build a fixture corpus and save LaTeXML XML output as the compatibility baseline.
3. Add TeX patch/binding files or Rust DSL entries that target the same `ltx:*` structure.
4. Add `LtxmlIr` node mapping or fallback only where the existing core IR lacks coverage.
5. Add structural diff tests against LaTeXML output.
6. Add diagnostic messages for unsupported macros/options.
7. Preserve unknown elements/attributes rather than dropping them, so later oxidation remains possible.

### Phase 7: Graphics and Tables

Deliverables:

- Graphics:
  - resolve graphic paths and extensions
  - copy compatible web formats directly
  - convert EPS/PDF to SVG/PNG when tools are available
  - preserve dimensions and alt text
  - warn on missing alt descriptions
- Tables:
  - semantic capture of `tabular`, `array`, `longtable`
  - alignments and column spans
  - header/body/footer recognition if available
  - fallback image/canvas rendering for extremely complex tables

Success criteria:

- Common scientific figures and tables are readable and accessible in HTML.

### Phase 8: Output Profiles and User Experience — Initial Implementation Complete

Add profiles similar to LaTeXML's `resources/Profiles/*.opt`, but native to Tectonic.

Example profile configuration:

```toml
[html]
template = "article"
split = false
math = "mathjax" # mathjax | canvas | mathml | svg
css = ["tectonic-article.css"]
copy_assets = true
validate = true

[semantic]
keep_ltxml_ir = true
strict = false
```

CLI examples:

```bash
tectonic -X compile --outfmt html input.tex
tectonic -X compile --outfmt html --html-profile atlas input.tex
tectonic -X compile --outfmt ltxml-ir input.tex
tectonic -X compile --outfmt html --html-math mathjax input.tex
tectonic -X compile --outfmt html --html-split section input.tex
```

Success criteria:

- Users can convert ordinary LaTeX without writing templates or specials.
- Debug options make unsupported constructs discoverable.

### Phase 9: Validation, Accessibility, and Quality Gates

Deliverables:

- HTML validation tests.
- LtxmlIr schema validation.
- Link checker for internal refs.
- Accessibility checks:
  - heading order
  - figures with alt text
  - tables with headers where possible
  - math fallback/source preservation
- Snapshot tests for representative documents.
- Differential tests against LaTeXML output for selected examples.

Success criteria:

- Regressions are caught by CI.
- HTML output is stable enough for downstream publication workflows.

## LaTeXML Feature Mapping

| LaTeXML feature | Tectonic adaptation |
|---|---|
| Mouth/Gullet/Stomach TeX interpretation | Keep XeTeX; add semantic TeX patches/events instead of reimplementing digestion. |
| `DefConstructor` semantic replacements | Start with TeX-native semantic bindings; later add declarative DSL if interception becomes feasible. |
| `ltx:*` XML document | Rust `LtxmlIr` with optional `ltx:*` XML serialization. |
| RelaxNG validation | Lightweight Rust validation first; optional XML/RelaxNG compatibility later. |
| Package `.ltxml` files | `.tsem.tex` binding patches plus Rust post-processors. |
| `latexmlpost` | Native Rust `semantic_post` pipeline. |
| MathML/XMath | Preserve TeX source and canvas first; add MathML/XMath-like layer later. |
| CSS/resources/profiles | Native Tectonic HTML profiles and templates. |
| CrossRef/Scan/Split | Native Rust postprocessors. |
| Graphics conversion | Use Tectonic I/O and external converters where available. |

## Risks and Mitigations

### Risk: Macro patching becomes fragile

Mitigation:

- Keep patches narrow and package-specific.
- Use LaTeX hook mechanisms where possible.
- Add tests for each supported package version.
- Fail gracefully with diagnostics.

### Risk: Semantic events lose information available only inside TeX boxes

Mitigation:

- Preserve both semantic events and low-level SPX/canvas data.
- Allow LtxmlIr nodes to attach raw layout fragments.
- Use fallback rendering for unsupported structures.

### Risk: MathML is a very large undertaking

Mitigation:

- Do not make MathML the first milestone.
- Use MathJax/KaTeX-compatible TeX source first.
- Preserve data needed for future MathML generation.

### Risk: Package support scope explodes

Mitigation:

- Prioritize real document corpus coverage.
- Track unsupported macros with metrics.
- Provide useful fallbacks rather than perfect support.

### Risk: Existing `--outfmt html` behavior changes

Mitigation:

- Add a separate experimental mode first, such as `--outfmt html-semantic`.
- Keep low-level `tdux:` pipeline available for tt-weave/tectonopedia users.
- Merge modes only after compatibility is understood.

### Risk: Licensing/copying LaTeXML code or CSS

Mitigation:

- Treat LaTeXML as a reference design, not code to copy.
- Verify license status for any direct resource reuse.
- Prefer fresh Rust implementations and fresh CSS unless reuse is explicitly compatible.

## Updated Near-Term Next Steps

The original near-term tasks have mostly been implemented through Phase 6. The next implementation focus should be:

1. **Wire the prototype into the main processing pipeline**
   - Add an experimental `OutputFormat::LtxmlIr` or internal `--keep-ltxml-ir` path.
   - Route SPX output through `tectonic_engine_spx2ltxml`.
   - Optionally route post-processed `LtxmlIr` through `tectonic_ltxml_html`.

2. **Replace hand-authored `tsem` fixtures with TeX-generated semantic events**
   - Make `tectonic-semantic-html.tex` and package-specific `.ltxmlir.tex` files loadable in an integration test.
   - Exercise real TeX documents instead of only line-oriented `tsem` streams.

3. **Improve binding safety**
   - Add robust JSON/string escaping in TeX support macros.
   - Add tests for special characters in titles, labels, citations, and math source.

4. **Continue Phase 7 graphics/table hardening**
   - Copy/emit image assets through Tectonic's output layer.
   - Add conversion hooks for non-web formats such as PDF/EPS.
   - Improve table row/cell capture from real TeX-generated events.

5. **Wire Phase 8 profiles into the main CLI**
   - Add `--html-profile`, `--html-math`, and `--keep-ltxml-ir` style options.
   - Route profile settings into `tectonic_ltxml_html` and post-processing.

6. **Add LaTeXML differential fixtures**
   - Generate LaTeXML XML for a small fixture corpus.
   - Add structural comparison tests or documented-difference snapshots.

## Milestone Plan Status

### Milestone 1: Hand-authored `LtxmlIr` proof of concept — Complete

Implemented by `crates/ltxml_ir` and `crates/engine_spx2ltxml`.

### Milestone 2: Oxidized `article` baseline — Partially Complete

The current implementation has support files and `tsem` fixtures for article-like documents, but does not yet auto-load these bindings through the main Tectonic compile pipeline.

### Milestone 3: Oxidized figures, equations, tables, and bibliography — Partially Complete

The current prototype supports the relevant `ltx:*` structures in event fixtures, HTML rendering, math metadata, reference/bibliography post-processing, graphics candidate normalization, and table body/header normalization. Real TeX-generated binding coverage still needs integration testing.

### Milestone 4: ATLAS/HEP pilot — Pending

Select a representative ATLAS paper/note after the pipeline can compile real TeX to `LtxmlIr`.

### Milestone 5: Broader package hardening — Pending

The binding registry records Tier 1 packages and support-file stubs; full hardening remains future work.

### Milestone 6: MathML and advanced output — Pending

Current math output preserves TeX source and supports MathJax-compatible HTML. MathML/XMath generation remains future work.

## Definition of Done for a Useful First Release

A first experimental release is useful when this command:

```bash
tectonic -X compile --outfmt html-semantic paper.tex
```

can convert a normal `article`-class scientific LaTeX document with:

- title/author/abstract metadata
- sections/subsections
- paragraphs and inline formatting
- inline and display math with readable browser rendering
- numbered equations with references
- figures with captions
- basic tables
- citations and bibliography from `.bbl`
- internal links
- one generated HTML file plus assets
- clear warnings for unsupported constructs

The output does not need to match LaTeXML feature-for-feature initially. It must be structurally correct, debuggable, and extensible.

## Strategic Recommendation

The practical path is to evolve Tectonic HTML from a **manual, low-level HTML emission system** into a **LaTeXML-compatible semantic LaTeX-to-web pipeline**. LaTeXML provides the target output shape: `ltx:*` bindings, intermediate document model, and post-processing. Tectonic should preserve that structure as closely as possible while relying on its own strengths: real XeTeX execution, reproducible bundles, Rust performance, and high-fidelity rendering assets.

The most important architectural decision is introducing `LtxmlIr` as a Rust-native, LaTeXML-shaped IR rather than inventing a new Tectonic vocabulary first. Once Tectonic can emit and validate LaTeXML-like XML, package support can be oxidized binding by binding, with LaTeXML itself serving as the compatibility oracle. HTML rendering, MathML generation, accessibility checks, and ATLAS-specific customization can then develop incrementally on top of a stable `ltx:*`-compatible foundation.
