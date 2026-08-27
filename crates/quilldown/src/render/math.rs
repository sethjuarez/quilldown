//! Typeset `$...$` / `$$...$$` math to real equations, feature-gated behind `math-render`.
//!
//! The pipeline is LaTeX -> Typst -> SVG: [`mitex`] converts the LaTeX fragment to Typst
//! math markup, the Typst engine typesets it on an auto-sized, transparent page using the
//! bundled "New Computer Modern Math" font, and the result is emitted as SVG. Callers embed
//! that SVG through the ordinary image pipeline, so math benefits from the same rasterisation,
//! sizing, alt-text, and optional `<asvg>` vector-layer handling as any other picture.
//!
//! MiTeX's converter emits Typst that references helper bindings (e.g. `mitexsqrt`) defined in
//! its Typst package's "scope". We vendor that scope (`mitex/prelude.typ`,
//! `mitex/latex/standard.typ`, both Apache-2.0 — see `mitex/NOTICE.md`), register the files
//! in-memory, and evaluate the converted math within the scope, mirroring what the upstream
//! `mitex()` function does. Only the scope is needed, not the WASM converter, because we use the
//! pure-Rust `mitex` crate to convert.
//!
//! A fresh engine is built per equation. Typical documents contain only a handful of equations,
//! so this stays simple and avoids a shared-engine/`sys.inputs` template.

use typst::layout::Abs;
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;
use typst_svg::SvgOptions;

/// Point size math is typeset at. Matches the default body font size so inline equations sit
/// at roughly the surrounding text's scale.
const MATH_PT: f64 = 11.0;

/// Transparent gutter (points) left around the equation on the auto-sized page. Without it,
/// tall glyph ink (fraction bars, superscripts, radicals) sits flush against the page edge and
/// Typst clips the outermost pixels. A hair of margin is invisible once embedded but prevents
/// the numerator/denominator or superscripts from being sheared off.
const MATH_MARGIN_PT: f64 = 1.5;

/// Vendored MiTeX Typst scope files. Registered as in-memory sources so the converted math can
/// resolve `mitexsqrt` and friends. `standard.typ` imports `../prelude.typ`, so the relative
/// layout (a `latex/` subdirectory) must be preserved when registering the virtual paths.
const MITEX_PRELUDE: &str = include_str!("mitex/prelude.typ");
const MITEX_STANDARD: &str = include_str!("mitex/latex/standard.typ");

/// Render a LaTeX math fragment to SVG bytes.
///
/// `display` selects block (centered, display-style) versus inline layout. Returns the SVG
/// source on success, or a human-readable error describing which stage failed so the caller
/// can warn and fall back to the literal source.
pub(crate) fn to_svg(latex: &str, display: bool) -> Result<Vec<u8>, String> {
    let converted =
        mitex::convert_math(latex, None).map_err(|e| format!("LaTeX->Typst failed: {e}"))?;
    let escaped = escape_typst_string(&converted);
    let block = if display { "true" } else { "false" };

    // Shrink-wrap the page to the equation's ink and keep the background transparent so the PNG
    // composites cleanly onto Word's white page. A hair of margin keeps tall glyph ink (fraction
    // bars, superscripts, radicals) off the auto-sized page edge, which Typst would otherwise
    // clip. Evaluate the converted markup inside the MiTeX scope exactly as the upstream
    // `mitex()` function does; `block` selects display vs inline.
    let main = format!(
        "#import \"latex/standard.typ\": package as latex-std\n\
         #let mitex-scope = latex-std.scope\n\
         #set page(width: auto, height: auto, margin: {MATH_MARGIN_PT}pt, fill: none)\n\
         #set text(size: {MATH_PT}pt)\n\
         #math.equation(block: {block}, eval(\"$\" + \"{escaped}\" + \"$\", scope: mitex-scope))\n"
    );

    let engine = TypstEngine::builder()
        .main_file(main)
        .with_static_source_file_resolver([
            ("prelude.typ", MITEX_PRELUDE),
            ("latex/standard.typ", MITEX_STANDARD),
        ])
        .fonts(typst_assets::fonts())
        .build();

    let doc: PagedDocument = engine
        .compile()
        .output
        .map_err(|e| format!("Typst compile failed: {e:?}"))?;

    let svg = typst_svg::svg_merged(&doc, &SvgOptions::default(), Abs::zero());
    Ok(svg.into_bytes())
}

/// Escape a string for embedding inside a Typst double-quoted string literal.
fn escape_typst_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}
