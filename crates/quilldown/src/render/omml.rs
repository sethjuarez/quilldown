//! LaTeX -> OMML (native Word math) conversion.
//!
//! docx-rs (0.4.x) has no math type, so this module produces the `<m:oMath>` XML as a string
//! that a post-packing pass ([`super::mathsplice`]) splices into `word/document.xml`. The
//! pipeline is pure-Rust: `latex2mathml` turns LaTeX into MathML, then we walk the MathML with
//! `roxmltree` and translate each node to its OMML equivalent.
//!
//! Native equations are the right target (over a rasterized image) because Word draws them in
//! the document's text color — so they stay legible in dark mode — reflow with the text instead
//! of clipping, and remain vector-crisp and editable.

use latex2mathml::{latex_to_mathml, DisplayStyle};
use roxmltree::Node;

/// The OOXML math namespace. Each `<m:oMath>` self-declares it because the packed
/// `word/document.xml` root does not bind the `m:` prefix.
const MATH_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// Convert a LaTeX equation to an `<m:oMath>` OMML fragment (namespace self-declared).
///
/// `display` selects block vs inline layout, which drives n-ary limit placement (over/under vs
/// sub/sup). Returns `Err` with a human-readable reason when the LaTeX can't be represented, so
/// the caller can fall back to literal source and warn once.
pub(crate) fn latex_to_omml(latex: &str, display: bool) -> Result<String, String> {
    let normalized = normalize_environments(latex);
    let style = if display {
        DisplayStyle::Block
    } else {
        DisplayStyle::Inline
    };
    let mathml = latex_to_mathml(&normalized, style).map_err(|e| format!("{e:?}"))?;
    // latex2mathml never fails hard on an unknown command: it embeds a `[PARSE ERROR: ...]`
    // marker in an <mtext>. Treat that as unrepresentable so we degrade to literal source.
    if mathml.contains("PARSE ERROR") {
        return Err("unsupported LaTeX construct".to_string());
    }
    let mathml = quote_bare_attributes(&mathml);
    let doc =
        roxmltree::Document::parse(&mathml).map_err(|e| format!("mathml parse failed: {e}"))?;
    let conv = Converter { display };
    let inner = conv.children(doc.root_element());
    if inner.trim().is_empty() {
        return Err("empty equation".to_string());
    }
    Ok(format!(r#"<m:oMath xmlns:m="{MATH_NS}">{inner}</m:oMath>"#))
}

/// latex2mathml only knows the `align` environment; remap the common `aligned`/`align*`
/// spellings onto it so `$$\begin{aligned}...\end{aligned}$$` blocks convert.
fn normalize_environments(latex: &str) -> String {
    latex
        .replace("\\begin{aligned}", "\\begin{align}")
        .replace("\\end{aligned}", "\\end{align}")
        .replace("\\begin{align*}", "\\begin{align}")
        .replace("\\end{align*}", "\\end{align}")
}

/// latex2mathml emits some attributes unquoted (e.g. `<mtable columnalign=left>`), which is not
/// well-formed XML, so roxmltree rejects it. Wrap every bare attribute value in quotes. Operates
/// on chars (not bytes) so the unicode math glyphs in text content survive intact.
fn quote_bare_attributes(mathml: &str) -> String {
    let chars: Vec<char> = mathml.chars().collect();
    let mut out = String::with_capacity(mathml.len());
    let mut i = 0;
    let mut in_tag = false;
    while i < chars.len() {
        match chars[i] {
            '<' => {
                in_tag = true;
                out.push('<');
                i += 1;
            }
            '>' => {
                in_tag = false;
                out.push('>');
                i += 1;
            }
            '=' if in_tag => {
                out.push('=');
                i += 1;
                // Leave an already-quoted value untouched.
                if matches!(chars.get(i), Some('"') | Some('\'')) {
                    continue;
                }
                // Quote a bare value: everything up to whitespace, `/`, or the tag's `>`.
                out.push('"');
                while let Some(&d) = chars.get(i) {
                    if d.is_whitespace() || d == '>' || d == '/' {
                        break;
                    }
                    out.push(d);
                    i += 1;
                }
                out.push('"');
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Walks MathML and emits OMML, carrying the display flag needed for n-ary limit placement.
struct Converter {
    display: bool,
}

/// The pieces of an n-ary operator (∑, ∫, ...): its character and converted limit expressions.
struct NaryParts {
    chr: String,
    sub: String,
    sup: String,
    has_sub: bool,
    has_sup: bool,
}

impl Converter {
    /// Convert every element child of `node`, concatenating the OMML.
    fn children(&self, node: Node) -> String {
        let kids = elems(node);
        let mut out = String::new();
        let mut i = 0;
        while i < kids.len() {
            // A big-operator script (∑/∫ with limits) keeps its operand as a following sibling in
            // MathML. Absorb that sibling into the n-ary body so Word doesn't draw an empty
            // placeholder box. Operators (`<mo>`) can't start an operand, so they're left outside.
            if let Some(parts) = self.nary_parts(kids[i]) {
                let (body, step) = match kids.get(i + 1) {
                    Some(n) if n.tag_name().name() != "mo" => (self.convert(*n), 2),
                    _ => (String::new(), 1),
                };
                out.push_str(&self.nary(&parts, &body));
                i += step;
                continue;
            }
            out.push_str(&self.convert(kids[i]));
            i += 1;
        }
        out
    }

    /// Dispatch a single MathML element to its OMML translation.
    fn convert(&self, node: Node) -> String {
        match node.tag_name().name() {
            "mi" | "mn" | "mo" | "mtext" => token(node),
            "mrow" | "mstyle" | "mpadded" => self.children(node),
            "mfrac" => self.frac(node),
            "msqrt" => self.sqrt(node),
            "mroot" => self.root(node),
            "msup" => self.sup(node),
            "msub" => self.sub(node),
            "msubsup" => self.subsup(node),
            "munderover" => self.underover(node),
            "munder" => self.under(node),
            "mover" => self.over(node),
            "mtable" => self.table(node),
            // Thin spaces etc. carry no OMML weight; equations read fine without them.
            "mspace" => String::new(),
            // Unknown wrapper: descend so text content is never dropped.
            _ => self.children(node),
        }
    }

    fn frac(&self, node: Node) -> String {
        let kids = elems(node);
        let num = self.opt(kids.first());
        let den = self.opt(kids.get(1));
        format!("<m:f><m:num><m:e>{num}</m:e></m:num><m:den><m:e>{den}</m:e></m:den></m:f>")
    }

    fn sqrt(&self, node: Node) -> String {
        let inner = self.children(node);
        format!(
            "<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e>{inner}</m:e></m:rad>"
        )
    }

    fn root(&self, node: Node) -> String {
        let kids = elems(node);
        let base = self.opt(kids.first());
        let index = self.opt(kids.get(1));
        format!("<m:rad><m:deg>{index}</m:deg><m:e>{base}</m:e></m:rad>")
    }

    fn sup(&self, node: Node) -> String {
        let kids = elems(node);
        let base = self.opt(kids.first());
        let sup = self.opt(kids.get(1));
        format!("<m:sSup><m:e>{base}</m:e><m:sup>{sup}</m:sup></m:sSup>")
    }

    fn sub(&self, node: Node) -> String {
        let kids = elems(node);
        let base = self.opt(kids.first());
        let sub = self.opt(kids.get(1));
        format!("<m:sSub><m:e>{base}</m:e><m:sub>{sub}</m:sub></m:sSub>")
    }

    fn subsup(&self, node: Node) -> String {
        if let Some(parts) = self.nary_parts(node) {
            return self.nary(&parts, "");
        }
        let kids = elems(node);
        let base = self.opt(kids.first());
        let sub = self.opt(kids.get(1));
        let sup = self.opt(kids.get(2));
        format!("<m:sSubSup><m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>")
    }

    fn underover(&self, node: Node) -> String {
        if let Some(parts) = self.nary_parts(node) {
            return self.nary(&parts, "");
        }
        let kids = elems(node);
        let base = self.opt(kids.first());
        let under = self.opt(kids.get(1));
        let over = self.opt(kids.get(2));
        format!(
            "<m:sSubSup><m:e>{base}</m:e><m:sub>{under}</m:sub><m:sup>{over}</m:sup></m:sSubSup>"
        )
    }

    fn under(&self, node: Node) -> String {
        if let Some(parts) = self.nary_parts(node) {
            return self.nary(&parts, "");
        }
        let kids = elems(node);
        let base = self.opt(kids.first());
        let under = self.opt(kids.get(1));
        format!("<m:limLow><m:e>{base}</m:e><m:lim>{under}</m:lim></m:limLow>")
    }

    fn over(&self, node: Node) -> String {
        if let Some(parts) = self.nary_parts(node) {
            return self.nary(&parts, "");
        }
        let kids = elems(node);
        let base = self.opt(kids.first());
        // An accent (e.g. \vec, \hat, \bar) is a combining mark placed over the base.
        if let Some(over) = kids.get(1) {
            if over.tag_name().name() == "mo" && over.attribute("accent") == Some("true") {
                let chr = escape(over.text().unwrap_or_default());
                return format!(
                    "<m:acc><m:accPr><m:chr m:val=\"{chr}\"/></m:accPr><m:e>{base}</m:e></m:acc>"
                );
            }
        }
        let over = self.opt(kids.get(1));
        format!("<m:limUpp><m:e>{base}</m:e><m:lim>{over}</m:lim></m:limUpp>")
    }

    /// An `<mtable>` (from `align`/`matrix`) becomes an OMML equation array: one stacked row per
    /// `<mtr>`, concatenating each row's `<mtd>` cells. Column alignment at `&` is not preserved.
    fn table(&self, node: Node) -> String {
        let rows: String = node
            .children()
            .filter(|c| c.tag_name().name() == "mtr")
            .map(|tr| {
                let row: String = tr
                    .children()
                    .filter(|c| c.tag_name().name() == "mtd")
                    .map(|td| self.children(td))
                    .collect();
                format!("<m:e>{row}</m:e>")
            })
            .collect();
        format!("<m:eqArr>{rows}</m:eqArr>")
    }

    /// If `node` is a script (`msubsup`/`munderover`/`munder`/`mover`) over a big operator
    /// (∑, ∫, ∏, ...), return its n-ary parts (the operator char plus converted limits). Otherwise
    /// `None`, so the caller falls back to an ordinary script.
    fn nary_parts(&self, node: Node) -> Option<NaryParts> {
        let kids = elems(node);
        let chr = nary_char(*kids.first()?)?;
        let parts = match node.tag_name().name() {
            "msubsup" | "munderover" => NaryParts {
                chr,
                sub: self.opt(kids.get(1)),
                sup: self.opt(kids.get(2)),
                has_sub: true,
                has_sup: true,
            },
            "munder" => NaryParts {
                chr,
                sub: self.opt(kids.get(1)),
                sup: String::new(),
                has_sub: true,
                has_sup: false,
            },
            "mover" => NaryParts {
                chr,
                sub: String::new(),
                sup: self.opt(kids.get(1)),
                has_sub: false,
                has_sup: true,
            },
            _ => return None,
        };
        Some(parts)
    }

    /// Emit an n-ary operator (∑, ∫, ∏, ...) with its limits and operand. In display style the
    /// limits sit over/under the operator; inline they sit as sub/sup to keep the line compact.
    fn nary(&self, p: &NaryParts, body: &str) -> String {
        let lim_loc = if self.display { "undOvr" } else { "subSup" };
        let mut pr = format!(
            "<m:naryPr><m:chr m:val=\"{}\"/><m:limLoc m:val=\"{lim_loc}\"/>",
            escape(&p.chr)
        );
        if !p.has_sub {
            pr.push_str("<m:subHide m:val=\"1\"/>");
        }
        if !p.has_sup {
            pr.push_str("<m:supHide m:val=\"1\"/>");
        }
        pr.push_str("</m:naryPr>");
        format!(
            "<m:nary>{pr}<m:sub>{}</m:sub><m:sup>{}</m:sup><m:e>{body}</m:e></m:nary>",
            p.sub, p.sup
        )
    }

    /// Convert an optional child node, yielding an empty string when absent.
    fn opt(&self, node: Option<&Node>) -> String {
        node.map(|n| self.convert(*n)).unwrap_or_default()
    }
}

/// Convert a leaf token (`<mi>`/`<mn>`/`<mo>`/`<mtext>`) to an OMML run.
///
/// Word auto-italicizes ASCII/Greek letters inside `<m:r>`, which is right for single-letter
/// variables but wrong for multi-letter function names (`sin`, `lim`) and literal text, so those
/// are forced upright with a plain math style.
fn token(node: Node) -> String {
    let text = node.text().unwrap_or_default();
    let name = node.tag_name().name();
    let upright = name == "mtext" || (name == "mi" && text.trim().chars().count() > 1);
    let rpr = if upright {
        "<m:rPr><m:sty m:val=\"p\"/></m:rPr>"
    } else {
        ""
    };
    format!("<m:r>{rpr}<m:t>{}</m:t></m:r>", escape(text))
}

/// If `node` is a lone big-operator `<mo>`, return its character so the caller can build an
/// `<m:nary>` instead of a script; otherwise `None`.
fn nary_char(node: Node) -> Option<String> {
    if node.tag_name().name() != "mo" {
        return None;
    }
    let text = node.text().unwrap_or_default();
    let mut chars = text.trim().chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    is_nary(c).then(|| c.to_string())
}

/// The large operators that take limits and map to OMML `<m:nary>`.
fn is_nary(c: char) -> bool {
    matches!(
        c,
        '∑' | '∏'
            | '∐'
            | '∫'
            | '∬'
            | '∭'
            | '⨌'
            | '∮'
            | '∯'
            | '∰'
            | '⋃'
            | '⋂'
            | '⋁'
            | '⋀'
            | '⨆'
            | '⨅'
            | '⨁'
            | '⨂'
            | '⨀'
            | '⨄'
    )
}

/// Collect a node's element children (skipping whitespace text nodes).
fn elems<'a, 'i>(node: Node<'a, 'i>) -> Vec<Node<'a, 'i>> {
    node.children().filter(Node::is_element).collect()
}

/// Escape text/attribute content for inclusion in OMML XML.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
