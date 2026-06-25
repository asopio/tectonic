# Tectonic HTML Support Documentation

## Overview

This document provides a comprehensive overview of HTML output support in the Tectonic typesetting system, based on deep analysis of the tectonic source code, GitHub discussions, and example implementations.

## Current Status and Maturity

**Status**: Experimental and in active development  
**Maturity**: Early-stage, requires specialized TeX code and is not production-ready for general LaTeX documents

### Key Findings

1. **Not a drop-in replacement**: Unlike LaTeXML, tectonic's HTML mode cannot simply process arbitrary LaTeX documents and produce HTML output. The input TeX code must be specifically designed to work with the HTML output mode.

2. **Low-level interface**: The HTML support operates at a very low level, requiring cooperation from the TeX code being processed through specialized `\special` commands.

3. **Limited user base**: As of the latest information (2024), the maintainer (Peter Williams) is the primary (and possibly only) user of this feature.

4. **Active development**: The feature is still being developed with the goal of eventually supporting standard LaTeX documents, but this is described as "a ways off while the fundamentals are still being developed."

## Technical Architecture

### Processing Pipeline

Tectonic's HTML output mode follows this processing pipeline:

1. **XeTeX Engine** (configured for HTML mode)
   - Processes TeX input similar to standard mode
   - Overrides paragraph processing to create super-long lines without line-breaking
   - Preserves input text without hyphenation for HTML output
   
2. **SPX Output** (Semantically Paginated XDV)
   - Instead of standard XDV (Extended DVI) format, produces SPX files
   - Note: "Semantically Paginated" is now a misnomer - initial vision to use XDV pagination for separating HTML files was abandoned
   - Contains special markers and commands for HTML generation
   
3. **spx2html Engine**
   - Processes SPX files to generate HTML
   - Uses the [tera](https://docs.rs/tera/latest/tera/) templating engine
   - Responds to `\special` commands in the SPX to control HTML output

### Key Components

#### SPX Format

The SPX format is essentially XDV with modifications:
- Different file extension to indicate HTML-specific processing
- Contains `\special` items that control the HTML generation
- Paragraphs are set as single long lines (no line-breaking)
- Text preserved for direct HTML output without hyphenations

#### spx2html Engine

The spx2html engine provides:

1. **Template System**: Uses tera HTML templating engine
2. **Canvas Rendering**: Special logic for rendering "canvases" (equations, mathematical content)
   - Computes CSS styles to position glyphs correctly
   - Creates on-the-fly font subsets mapping Unicode codepoints to specific glyphs
   - Solves the problem of rendering specific typographic glyphs that don't have direct Unicode representation
3. **Output Control**: `\special` commands control:
   - Template selection
   - Template variable setting
   - Output file writing

#### Font Subsetting Innovation

The core innovation of Tectonic's HTML mode is its approach to rendering mathematics and specialized typography:

- **Problem**: TeX knows what specific *glyphs* need to be rendered, but there isn't always a Unicode *character* that maps to that glyph
- **Example**: A font may have a superscript "2" glyph, but emitting "2" in HTML only produces a regular inline "2"
- **Solution**: Create special fonts on-the-fly that directly map Unicode codepoints to specific glyphs in font files
- **Result**: Precise typographic rendering in HTML/CSS without image conversion

This technique is described in detail in: Williams, 2022 TUGboat 43(2) 120–126 (http://dx.doi.org/10.47397/tb/43-2/tb134williams-tectonic)

## Required TeX Modifications

To use tectonic's HTML mode, TeX documents must include:

### 1. Special Commands

`\special` commands control the HTML output:

```tex
% Select a template
\special{tdux:template template-name.html}

% Set template variables
\special{tdux:set variable-name value}

% Emit content to HTML file
\special{tdux:emit output-file.html}

% Start/end markup elements
\special{tdux:mfs element-name}  % "markup fragment start"
\special{tdux:me element-name}   % "markup end"

% Direct text insertion
\special{tdux:dt text-content}
```

### 2. Document Class

Documents must use a class designed for HTML output (e.g., `tdux` class used in tt-weave)

### 3. Template Files

HTML templates (using tera syntax) must be provided and made available to spx2html

### 4. Custom Macros

Standard LaTeX macros often need HTML-aware replacements that emit appropriate `\special` commands

## Capabilities

### What Works

1. **Mathematical typesetting**: High-precision rendering of equations with correct glyph positioning
2. **Font rendering**: Accurate rendering of special typography through font subsetting
3. **Structured output**: Can generate multiple HTML files with proper linking
4. **Rich formatting**: Supports sophisticated formatting through CSS and template control
5. **Interactive features**: Can integrate with JavaScript for interactive elements

### What Doesn't Work

1. **Standard LaTeX documents**: Cannot process arbitrary LaTeX documents without modification
2. **Standard document classes**: Standard classes (article, book, etc.) don't support HTML mode
3. **Most LaTeX packages**: Package support is very limited and requires HTML-specific implementations
4. **Automatic conversion**: No automatic translation from PDF-oriented commands to HTML equivalents

## Limitations

### Technical Limitations

1. **Template requirement**: Must provide HTML templates; missing templates cause errors
2. **Special command requirement**: Documents must explicitly use `\special` commands
3. **No automatic pagination**: The "Semantically Paginated" concept was abandoned
4. **Limited package ecosystem**: Very few packages support HTML output mode

### Practical Limitations

1. **Steep learning curve**: Requires understanding of both TeX internals and HTML/CSS
2. **Manual markup**: Much of the HTML structure must be manually specified
3. **No migration path**: Existing documents require substantial rewriting
4. **Limited tooling**: No convenient development tools or debugging support
5. **Documentation**: Very limited documentation; must read source code

### Comparison to LaTeXML

| Feature | Tectonic HTML | LaTeXML |
|---------|---------------|---------|
| Maturity | Experimental | Production-ready |
| Ease of use | Very difficult | Moderate |
| Standard LaTeX support | No | Extensive |
| Math rendering | High precision | Good |
| Package support | Very limited | Extensive |
| Learning curve | Very steep | Moderate |
| Performance | Fast (when working) | Slower |
| Customization | Complete control | Template-based |

## Example Implementations

### tt-weave

**Repository**: https://github.com/tectonic-typesetting/tt-weave

**Purpose**: Converts programs in Knuth's WEB language to high-quality HTML

**Status**: Primary example of tectonic HTML usage; needs updating for latest tectonic

**Key Features**:
- Custom `tdux` document class
- Extensive use of `\special` commands
- Integration with Vue.js for interactive web application
- Demonstrates template system and canvas rendering

**Build Process**:
1. WEB source → TeX (via tt-weave)
2. TeX → SPX → HTML (via tectonic)
3. HTML + Vue.js → Interactive web app (via yarn/Parcel)

### tectonopedia

**Repository**: https://github.com/tectonic-typesetting/tectonopedia

**Purpose**: Prototype for the Tectonic reference encyclopedia

**Status**: In development, represents the maintainer's vision for HTML documentation

**Approach**: Targets sophisticated online technical documentation as a rich web application rather than static HTML pages

## Command-Line Usage

### Basic HTML Compilation

```bash
tectonic -X compile --outfmt html input.tex
```

### Common Errors

**Error**: "need to emit HTML content but no template has been loaded"

**Cause**: Document doesn't specify a template via `\special` command

**Solution**: Document must be designed for HTML output with proper template references

## Recommendations

### When to Use Tectonic HTML

- **DO USE** if:
  - You're developing specialized documentation systems
  - You have deep TeX/HTML expertise
  - You need complete control over HTML output
  - You're willing to write custom TeX macros and templates
  - You're contributing to tectonic development

- **DON'T USE** if:
  - You need to convert existing LaTeX documents
  - You want a production-ready solution
  - You have limited TeX expertise
  - You need broad package support
  - You need quick results

### Alternative: LaTeXML

For converting ATLAS LaTeX documents to HTML, LaTeXML remains the recommended approach:

- Much more mature and stable
- Better support for standard LaTeX packages
- Easier to use with existing documents
- Better documentation
- Active community support
- Used by arXiv.org for HTML rendering

See the repository's existing LaTeXML workflow in `latexml_scripts/` for the recommended approach.

## Future Outlook

According to the maintainer (Peter Williams), the development roadmap includes:

1. **Near-term**: Developing HTML support for the Tectonopedia use case
2. **Medium-term**: Creating sophisticated online technical documentation tools
3. **Long-term**: Eventually supporting standard LaTeX documents with:
   - Nice single-page templates
   - Hooks in standard LaTeX classes
   - "Just works" HTML output for standard documents

However, this is described as requiring significant development work with no specific timeline.

## Technical Details for Developers

### Source Code Structure

Key areas in the tectonic source tree:

- `crates/engine_xetex/`: Modified XeTeX engine with HTML mode
- `crates/engine_spx2html/`: SPX to HTML conversion engine
- Template files: Tera templates for HTML output
- `\special` handling: In XeTeX engine modifications

### Enabling HTML Mode

HTML mode is enabled by:
1. Setting output format to HTML: `--outfmt html`
2. XeTeX engine detects this and switches to SPX output
3. spx2html processes SPX instead of xdvipdfmx processing XDV

### Template System

The tera templating engine:
- Standard HTML template syntax
- Variables set via `\special` commands
- Templates can be nested/included
- Full control over HTML structure

## Conclusion

Tectonic's HTML support is a fascinating technical innovation with great potential, but it is **not currently suitable for converting standard ATLAS LaTeX documents to HTML**. The system requires:

- Specialized TeX code designed for HTML output
- Deep technical expertise
- Willingness to work with experimental software
- Custom development of templates and macros

**For ATLAS document HTML conversion, continue using LaTeXML** as documented in this repository's existing workflow. Tectonic HTML may become viable in the future as the project matures.

## References

1. GitHub Discussion: https://github.com/tectonic-typesetting/tectonic/discussions/1140
2. Tectonic Documentation: https://tectonic-typesetting.github.io/book/latest/v2cli/compile.html
3. tt-weave Repository: https://github.com/tectonic-typesetting/tt-weave
4. tectonopedia Repository: https://github.com/tectonic-typesetting/tectonopedia
5. Williams, 2022 TUGboat 43(2) 120–126: http://dx.doi.org/10.47397/tb/43-2/tb134williams-tectonic
6. tera templating engine: https://docs.rs/tera/latest/tera/