//! Syntax highlighting for fenced code blocks.
//!
//! Fenced blocks carry an info string (`` ```rust ``); we resolve its first token to a
//! syntect syntax and emit one colored monospace run per highlighted span. syntect (with its
//! Oniguruma regex backend) is already in the tree via comrak, so this adds no new native
//! dependency. The default syntax and theme sets are loaded once per process via `OnceLock`.
//!
//! Highlighting is best-effort: an empty or unrecognized language token yields `None`, and the
//! caller falls back to plain monospace lines — the block still renders, just uncolored.

use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// A single highlighted span: an `RRGGBB` hex color (no `#`) and its text.
pub(crate) type Span = (String, String);

/// The bundled syntect syntax definitions, loaded once.
fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// The bundled syntect theme set, loaded once.
fn theme_set() -> &'static ThemeSet {
    static THEMES: OnceLock<ThemeSet> = OnceLock::new();
    THEMES.get_or_init(ThemeSet::load_defaults)
}

/// Resolve a bundled theme by name, falling back to `InspiredGitHub` (a light theme that reads
/// well on the pale code fill) and finally to any available theme.
fn theme(name: &str) -> &'static Theme {
    let set = theme_set();
    set.themes
        .get(name)
        .or_else(|| set.themes.get("InspiredGitHub"))
        .or_else(|| set.themes.values().next())
        .expect("syntect ships at least one default theme")
}

/// The language token from a fence info string: its first whitespace-delimited word, if any.
/// `` ```rust,ignore `` and `` ```rust ignore `` both yield `rust`.
pub(crate) fn language_token(info: &str) -> Option<&str> {
    info.split([' ', '\t', ','])
        .find(|t| !t.is_empty())
        .filter(|t| !t.is_empty())
}

/// An uppercase display label for the block's language, if the info string names one.
pub(crate) fn display_label(info: &str) -> Option<String> {
    language_token(info).map(|t| t.to_uppercase())
}

/// Convert a syntect color to an `RRGGBB` hex string (alpha dropped).
fn hex(c: syntect::highlighting::Color) -> String {
    format!("{:02X}{:02X}{:02X}", c.r, c.g, c.b)
}

/// Highlight `code` as `lang` using the bundled syntect theme named `theme_name`, returning one
/// `Vec<Span>` per line (newlines stripped), or `None` if the language token resolves to no
/// known syntax. An unknown `theme_name` falls back to a bundled light theme.
pub(crate) fn highlight(code: &str, lang: &str, theme_name: &str) -> Option<Vec<Vec<Span>>> {
    let ss = syntaxes();
    let syntax = ss
        .find_syntax_by_token(lang)
        .or_else(|| ss.find_syntax_by_extension(lang))?;
    let mut hl = HighlightLines::new(syntax, theme(theme_name));

    let mut lines = Vec::new();
    // LinesWithEndings keeps the trailing `\n` so syntect's stateful parser sees line breaks;
    // we strip it back out since Word represents each line as its own paragraph.
    for line in LinesWithEndings::from(code) {
        let ranges = hl.highlight_line(line, ss).ok()?;
        let spans = ranges
            .into_iter()
            .map(|(style, text)| (hex(style.foreground), text.replace('\n', "")))
            .filter(|(_, text)| !text.is_empty())
            .collect();
        lines.push(spans);
    }
    Some(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_token_takes_first_word() {
        assert_eq!(language_token("rust"), Some("rust"));
        assert_eq!(language_token("rust ignore"), Some("rust"));
        assert_eq!(language_token("rust,no_run"), Some("rust"));
        assert_eq!(language_token(""), None);
        assert_eq!(language_token("   "), None);
    }

    #[test]
    fn display_label_uppercases() {
        assert_eq!(display_label("python").as_deref(), Some("PYTHON"));
        assert_eq!(display_label(""), None);
    }

    #[test]
    fn highlight_known_language_yields_colored_spans() {
        let lines =
            highlight("fn main() {}\n", "rust", "InspiredGitHub").expect("rust is a known syntax");
        assert_eq!(lines.len(), 1);
        let spans = &lines[0];
        assert!(!spans.is_empty(), "a line of code should produce spans");
        // Reassembled text must equal the source line (order + content preserved).
        let joined: String = spans.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(joined, "fn main() {}");
        // Every color is a 6-hex-digit RRGGBB string.
        for (color, _) in spans {
            assert_eq!(color.len(), 6, "color {color} should be RRGGBB");
            assert!(color.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn highlight_unknown_language_is_none() {
        assert!(highlight("whatever\n", "definitely-not-a-language", "InspiredGitHub").is_none());
    }

    #[test]
    fn highlight_preserves_line_count() {
        let code = "let a = 1;\nlet b = 2;\nlet c = 3;";
        let lines = highlight(code, "rust", "InspiredGitHub").expect("rust syntax");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn unknown_theme_falls_back_but_still_highlights() {
        // An unknown theme name must not panic; it falls back to a bundled theme.
        let lines = highlight("fn main() {}\n", "rust", "no-such-theme").expect("rust syntax");
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].is_empty());
    }

    #[test]
    fn distinct_themes_can_color_differently() {
        // The same token under two different themes should be resolvable; at least one span
        // color differs between InspiredGitHub and Solarized (light).
        let a = highlight("fn main() {}\n", "rust", "InspiredGitHub").unwrap();
        let b = highlight("fn main() {}\n", "rust", "Solarized (light)").unwrap();
        let colors_a: Vec<&str> = a[0].iter().map(|(c, _)| c.as_str()).collect();
        let colors_b: Vec<&str> = b[0].iter().map(|(c, _)| c.as_str()).collect();
        assert_ne!(colors_a, colors_b, "themes should produce different colors");
    }
}
