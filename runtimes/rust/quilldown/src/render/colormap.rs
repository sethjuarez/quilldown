//! Light-mode remap for dark-themed SVG diagrams.
//!
//! Real technical-report diagrams are often authored for a dark UI (near-black backgrounds,
//! light text). Dropped onto a white Word page they read poorly. When
//! [`ConvertOptions::svg_light_mode`](crate::ConvertOptions::svg_light_mode) is set we rewrite
//! the SVG's colors *before* rasterizing so it sits well on paper.
//!
//! ## The transform
//! Each color is converted to HSL and its **lightness is flipped** (`L' = 1 - L`) while hue and
//! saturation are preserved. This is the well-known "invert lightness" dark↔light toggle:
//! - near-black backgrounds (`L≈0`) become near-white (`L≈1`),
//! - near-white text (`L≈1`) becomes near-black (`L≈0`),
//! - saturated accents keep their hue and stay recognizable (a mid-lightness blue stays blue).
//!
//! Only hex (`#rgb` / `#rrggbb`) and functional `rgb()` / `rgba()` colors are rewritten — the
//! forms virtually every diagram tool (mermaid, Figma, Excalidraw, hand-authored) emits. Named
//! colors and `currentColor` are left untouched.

/// Rewrite every hex and `rgb()`/`rgba()` color in `svg` to its lightness-flipped equivalent,
/// producing a print-friendly light-mode variant of a dark-themed diagram.
pub(crate) fn to_light_mode(svg: &str) -> String {
    let bytes = svg.as_bytes();
    let mut out = String::with_capacity(svg.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            if let Some((rgb, len)) = parse_hex(&bytes[i..]) {
                out.push_str(&to_hex(flip_lightness(rgb)));
                i += len;
                continue;
            }
        } else if starts_with_ci(&bytes[i..], b"rgb") {
            if let Some(parsed) = parse_rgb(&svg[i..]) {
                let (r, g, b) = flip_lightness(parsed.rgb);
                match parsed.alpha {
                    Some(a) => out.push_str(&format!("rgba({r}, {g}, {b}, {a})")),
                    None => out.push_str(&format!("rgb({r}, {g}, {b})")),
                }
                i += parsed.len;
                continue;
            }
        }
        // Not a color start: copy this byte through. `svg` is valid UTF-8 and every color
        // token is pure ASCII, so copying a single byte here never splits a code point.
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Case-insensitive ASCII prefix test.
fn starts_with_ci(haystack: &[u8], prefix: &[u8]) -> bool {
    haystack.len() >= prefix.len() && haystack[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Parse a leading `#rgb` or `#rrggbb` color. Returns `(r,g,b)` and the total token length
/// (including the `#`). Rejects other lengths (e.g. `#rrggbbaa` is left untouched).
fn parse_hex(bytes: &[u8]) -> Option<((u8, u8, u8), usize)> {
    debug_assert_eq!(bytes[0], b'#');
    let hex: Vec<u8> = bytes[1..]
        .iter()
        .take_while(|b| b.is_ascii_hexdigit())
        .copied()
        .collect();
    match hex.len() {
        3 => {
            let r = dup_nibble(hex[0]);
            let g = dup_nibble(hex[1]);
            let b = dup_nibble(hex[2]);
            Some(((r, g, b), 4))
        }
        6 => {
            let r = byte_from_hex(hex[0], hex[1]);
            let g = byte_from_hex(hex[2], hex[3]);
            let b = byte_from_hex(hex[4], hex[5]);
            Some(((r, g, b), 7))
        }
        _ => None,
    }
}

/// A parsed `rgb()`/`rgba()` color: its RGB channels, the total token length, and the
/// optional alpha string (passed through verbatim).
struct ParsedRgb {
    rgb: (u8, u8, u8),
    len: usize,
    alpha: Option<String>,
}

/// Parse a leading `rgb(...)` / `rgba(...)`. Only plain integer channels are handled; anything
/// unusual (percentages, `calc()`, ...) returns `None` and is left untouched.
fn parse_rgb(s: &str) -> Option<ParsedRgb> {
    let open = s.find('(')?;
    let close = s.find(')')?;
    if close < open {
        return None;
    }
    let inner = &s[open + 1..close];
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    let alpha = if parts.len() == 4 {
        Some(parts[3].to_string())
    } else {
        None
    };
    Some(ParsedRgb {
        rgb: (r, g, b),
        len: close + 1,
        alpha,
    })
}

/// Expand a single hex nibble into a full byte (`f` -> `0xff`).
fn dup_nibble(c: u8) -> u8 {
    let v = hex_val(c);
    v << 4 | v
}

fn byte_from_hex(hi: u8, lo: u8) -> u8 {
    hex_val(hi) << 4 | hex_val(lo)
}

fn hex_val(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

/// Render an `(r,g,b)` triple as a `#rrggbb` string.
fn to_hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Flip a color's lightness in HSL space, preserving hue and saturation.
fn flip_lightness((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let (h, s, l) = rgb_to_hsl(r, g, b);
    hsl_to_rgb(h, s, 1.0 - l)
}

/// Convert 8-bit RGB to HSL with each component in `[0, 1]` (hue also normalized to `[0, 1]`).
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;
    if delta.abs() < f64::EPSILON {
        return (0.0, 0.0, l); // achromatic
    }
    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let h = if (max - r).abs() < f64::EPSILON {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    let mut h = h / 6.0;
    if h < 0.0 {
        h += 1.0;
    }
    (h, s, l)
}

/// Convert HSL (components in `[0, 1]`) back to 8-bit RGB.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 1.0 / 3.0);
    let g = hue_to_channel(p, q, h);
    let b = hue_to_channel(p, q, h - 1.0 / 3.0);
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_channel(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
        0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
    }

    #[test]
    fn black_and_white_swap() {
        assert_eq!(flip_lightness((0, 0, 0)), (255, 255, 255));
        assert_eq!(flip_lightness((255, 255, 255)), (0, 0, 0));
    }

    #[test]
    fn dark_background_becomes_light() {
        // GitHub dark canvas -> should become clearly light.
        let light = flip_lightness((13, 17, 23));
        assert!(luminance(light) > 220.0, "got {light:?}");
    }

    #[test]
    fn accent_hue_is_preserved() {
        // A mid indigo: hue stays in the blue range, lightness barely moves (it's already ~0.5).
        let (h_in, _, _) = rgb_to_hsl(79, 70, 229);
        let flipped = flip_lightness((79, 70, 229));
        let (h_out, _, _) = rgb_to_hsl(flipped.0, flipped.1, flipped.2);
        assert!(
            (h_in - h_out).abs() < 0.02,
            "hue drifted: {h_in} -> {h_out}"
        );
    }

    #[test]
    fn rewrites_hex_shorthand_and_full() {
        let out = to_light_mode(r##"<rect fill="#000" stroke="#ffffff"/>"##);
        assert!(out.contains("#ffffff"), "shorthand black -> white: {out}");
        assert!(out.contains("#000000"), "full white -> black: {out}");
    }

    #[test]
    fn rewrites_rgb_function() {
        let out = to_light_mode("fill:rgb(0, 0, 0)");
        assert_eq!(out, "fill:rgb(255, 255, 255)");
    }

    #[test]
    fn leaves_non_colors_untouched() {
        // Ids, numbers, and text content must survive verbatim.
        let src = r##"<g id="node1"><text>Cost is #1</text></g>"##;
        assert_eq!(to_light_mode(src), src);
    }
}
