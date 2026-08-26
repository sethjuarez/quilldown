//! Per-feature round-trip tests that assert Markdown lands as *native* Word OOXML.
//!
//! Grows one section at a time as roadmap items land. Samples live in `examples/features/`.

mod common;

use common::{
    convert, convert_bytes_with, convert_feature, convert_with, document_rels, document_xml, entry,
    entry_names, features_dir, first_media_png, read_feature,
};
use quilldown::{ConvertOptions, Margins, Orientation, PageSetup, PageSize, Theme};

// ---------------------------------------------------------------------------------------------
// Roadmap #1 — native hyperlink relationships
// ---------------------------------------------------------------------------------------------

#[test]
fn inline_link_becomes_native_hyperlink_with_relationship() {
    let (docx, _stats) = convert("See the [project](https://example.com/project) page.\n");
    let doc = document_xml(&docx);

    // Native hyperlink element (references a relationship id), not just styled text.
    assert!(
        doc.contains("<w:hyperlink") && doc.contains("r:id="),
        "an inline link should emit a native <w:hyperlink r:id=...> element"
    );
    assert!(doc.contains("project"), "link text should survive");

    // The relationship must be registered in document.xml.rels as an external hyperlink.
    let rels = document_rels(&docx).expect("document.xml.rels must exist when links are present");
    assert!(
        rels.contains("relationships/hyperlink") && rels.contains("https://example.com/project"),
        "the hyperlink target must be an external relationship in document.xml.rels\n{rels}"
    );
    assert!(
        rels.contains(r#"TargetMode="External""#),
        "hyperlink relationship should be marked External"
    );
}

#[test]
fn anchor_link_becomes_in_document_anchor() {
    let (docx, _stats) = convert("Jump to [notes](#my-notes).\n\n## my notes\n");
    let doc = document_xml(&docx);
    assert!(
        doc.contains(r#"w:anchor="my-notes""#),
        "a `#fragment` link should become an in-document anchor hyperlink"
    );
    // The target heading must carry a matching bookmark so the anchor actually resolves.
    assert!(
        doc.contains("w:bookmarkStart") && doc.contains(r#"w:name="my-notes""#),
        "the target heading should be bookmarked with the slug so the anchor is not dangling"
    );
}

#[test]
fn autolink_becomes_native_hyperlink() {
    let (docx, _stats) = convert("Bare url https://www.rust-lang.org here.\n");
    let doc = document_xml(&docx);
    let rels = document_rels(&docx).expect("rels must exist");
    assert!(
        doc.contains("<w:hyperlink"),
        "autolink should be a hyperlink"
    );
    assert!(
        rels.contains("https://www.rust-lang.org"),
        "autolink target should be an external relationship"
    );
}

#[test]
fn hyperlinks_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("hyperlinks");
    let doc = document_xml(&docx);
    let rels = document_rels(&docx).expect("rels must exist");

    // External + anchor links, including one inside a table cell.
    assert!(
        doc.contains("<w:hyperlink"),
        "sample should contain hyperlinks"
    );
    assert!(
        doc.contains(r#"w:anchor="anchor-target""#),
        "sample has an anchor link"
    );
    for target in [
        "https://github.com/sethjuarez/quilldown",
        "https://docs.rs/docx-rs",
    ] {
        assert!(
            rels.contains(target),
            "external target {target} should be registered in rels"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Roadmap #2 — clickable endnote reference marks
// ---------------------------------------------------------------------------------------------

#[test]
fn endnote_mark_links_forward_and_note_links_back() {
    let (docx, _stats) = convert("Body text with a citation.[^a]\n\n[^a]: The note body.\n");
    let doc = document_xml(&docx);

    // Body mark: anchor hyperlink to the note, plus a bookmark for the back-link target.
    assert!(
        doc.contains(r#"w:anchor="qd-note-1""#),
        "reference mark should be an anchor hyperlink to the note\n{doc}"
    );
    assert!(
        doc.contains(r#"w:name="qd-noteref-1""#),
        "first reference should be bookmarked so the note can link back"
    );

    // Notes entry: bookmarked as the forward target, number links back to the reference.
    assert!(
        doc.contains(r#"w:name="qd-note-1""#),
        "the Notes entry should be bookmarked as the forward-link target"
    );
    assert!(
        doc.contains(r#"w:anchor="qd-noteref-1""#),
        "the note number should be a back-link to the first reference"
    );
}

#[test]
fn repeated_reference_dedups_to_single_note() {
    let (docx, stats) = convert(
        "First cite[^x] and second cite[^x] to the same note.\n\n[^x]: Only listed once.\n",
    );
    let doc = document_xml(&docx);

    // Two body marks (both link to qd-note-1) but exactly one Notes entry.
    assert_eq!(
        stats.endnotes, 1,
        "a twice-cited note should be listed once"
    );
    assert_eq!(
        doc.matches(r#"w:name="qd-note-1""#).count(),
        1,
        "there should be exactly one Notes bookmark for the deduplicated note"
    );
    // The second reference must not re-emit the noteref bookmark.
    assert_eq!(
        doc.matches(r#"w:name="qd-noteref-1""#).count(),
        1,
        "only the first reference is bookmarked as the back-link target"
    );
}

#[test]
fn endnotes_sample_document_is_wired() {
    let (docx, stats) = convert_feature("endnotes");
    let doc = document_xml(&docx);

    // Sample defines three notes; the attention note is cited twice.
    assert_eq!(stats.endnotes, 3, "sample has three unique notes");
    for n in 1..=3 {
        assert!(
            doc.contains(&format!(r#"w:name="qd-note-{n}""#)),
            "note {n} should have a forward-link bookmark"
        );
        assert!(
            doc.contains(&format!(r#"w:anchor="qd-note-{n}""#)),
            "note {n} should be referenced by an anchor hyperlink"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Roadmap #3 — block-quote styling (indent + left border)
// ---------------------------------------------------------------------------------------------

#[test]
fn blockquote_gets_indent_and_left_border() {
    let (docx, _stats) = convert("> Quoted text.\n\nPlain paragraph.\n");
    let doc = document_xml(&docx);

    // A left paragraph border is the distinctive quote cue — and ONLY the left side (not a box).
    assert!(
        doc.contains("<w:pBdr>") && doc.contains(r#"<w:left w:val="single""#),
        "a block quote should emit a left paragraph border\n{doc}"
    );
    assert!(
        !doc.contains(r#"<w:right w:val="single" w:space="0" w:sz="2""#),
        "a block quote should NOT draw a full box (no default right/top/bottom borders)\n{doc}"
    );
    // And a left indent so the block is set in from the margin.
    assert!(
        doc.contains("<w:ind") && doc.contains(r#"w:left="360""#),
        "a block quote paragraph should be indented from the left margin"
    );
    // Quote body text is tinted with the muted quote color.
    assert!(
        doc.contains(r#"w:val="57606A""#),
        "quote text should use the muted quote color"
    );
}

#[test]
fn plain_paragraph_has_no_quote_border() {
    let (docx, _stats) = convert("Just an ordinary paragraph.\n");
    let doc = document_xml(&docx);
    assert!(
        !doc.contains("<w:pBdr>"),
        "ordinary paragraphs must not get a quote border"
    );
}

#[test]
fn nested_quote_indents_further() {
    let (docx, _stats) = convert("> outer\n>\n> > inner\n");
    let doc = document_xml(&docx);
    // Depth 1 = 360, depth 2 = 720; both indents must be present.
    assert!(
        doc.contains(r#"w:left="360""#) && doc.contains(r#"w:left="720""#),
        "nested quotes should step the indent (360 then 720)\n{doc}"
    );
}

#[test]
fn blockquotes_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("blockquotes");
    let doc = document_xml(&docx);
    assert!(
        doc.contains("<w:pBdr>"),
        "sample should style quotes with borders"
    );
    assert!(
        doc.contains(r#"w:left="720""#),
        "sample includes a nested quote at depth 2"
    );
    // Inline formatting inside a quote should still round-trip (e.g. the hyperlink).
    assert!(
        doc.contains("<w:hyperlink"),
        "quote link should survive styling"
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap #4 — true OOXML superscript (w:vertAlign), incl. endnote marks
// ---------------------------------------------------------------------------------------------

#[test]
fn superscript_uses_true_vertical_alignment() {
    let (docx, _stats) = convert("The area is A = pi r^2^ here.\n");
    let doc = document_xml(&docx);
    assert!(
        doc.contains(r#"<w:vertAlign w:val="superscript" />"#),
        "^..^ should emit a true superscript run, not a Unicode glyph\n{doc}"
    );
    // The superscripted content is a normal digit, not a Unicode superscript character.
    assert!(
        doc.contains("<w:t"),
        "superscript run should carry real text"
    );
    assert!(
        !doc.contains('\u{00b2}'),
        "must not fall back to Unicode superscript ²"
    );
}

#[test]
fn superscript_combines_with_emphasis() {
    let (docx, _stats) = convert("A **bold x^2^** term.\n");
    let doc = document_xml(&docx);
    // The x^2^ run should carry both bold and superscript.
    assert!(
        doc.contains("<w:vertAlign w:val=\"superscript\" />"),
        "superscript inside bold should still align"
    );
    assert!(
        doc.contains("<w:b "),
        "bold should survive alongside superscript"
    );
}

#[test]
fn endnote_mark_is_true_superscript() {
    let (docx, _stats) = convert("Cite it.[^a]\n\n[^a]: Note.\n");
    let doc = document_xml(&docx);
    // The reference mark is now a true superscript digit inside the anchor hyperlink.
    assert!(
        doc.contains(r#"<w:vertAlign w:val="superscript" />"#),
        "endnote reference marks should use true OOXML superscript\n{doc}"
    );
}

#[test]
fn superscript_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("superscript");
    let doc = document_xml(&docx);
    let count = doc
        .matches(r#"<w:vertAlign w:val="superscript" />"#)
        .count();
    assert!(
        count >= 8,
        "sample exercises many superscripts; found {count}"
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap #5 — task-list checkbox marker (no redundant bullet)
// ---------------------------------------------------------------------------------------------

#[test]
fn task_items_render_checkbox_glyphs() {
    let (docx, _stats) = convert("- [x] Done\n- [ ] Todo\n");
    let doc = document_xml(&docx);
    assert!(
        doc.contains('\u{2611}'),
        "a checked task item should render the ballot-box-with-check glyph\n{doc}"
    );
    assert!(
        doc.contains('\u{2610}'),
        "an unchecked task item should render the empty ballot-box glyph\n{doc}"
    );
}

#[test]
fn task_items_suppress_the_list_bullet() {
    // A list of only task items should carry no bullet numbering — the checkbox is the marker.
    let (docx, _stats) = convert("- [x] Done\n- [ ] Todo\n");
    let doc = document_xml(&docx);
    assert!(
        !doc.contains("<w:numId"),
        "task items must not also emit a list bullet (numId)\n{doc}"
    );
    // They align like list items via a hanging indent, and separate marker from text with a tab.
    assert!(
        doc.contains(r#"w:hanging="360""#),
        "task item should use a hanging indent"
    );
    assert!(
        doc.contains("<w:tab"),
        "checkbox marker should be followed by a tab"
    );
}

#[test]
fn plain_bullets_keep_their_numbering_alongside_tasks() {
    // A mixed list: plain bullets still get numbering; the task item does not.
    let (docx, _stats) = convert("- Plain\n- [ ] Task\n");
    let doc = document_xml(&docx);
    assert!(
        doc.contains("<w:numId"),
        "the plain bullet should still be a native list item"
    );
    assert!(
        doc.contains('\u{2610}'),
        "the task item should still render a checkbox"
    );
}

#[test]
fn task_list_sample_document_is_wired() {
    let (docx, _stats) = convert_feature("tasklists");
    let doc = document_xml(&docx);
    let checked = doc.matches('\u{2611}').count();
    let unchecked = doc.matches('\u{2610}').count();
    assert!(
        checked >= 5,
        "sample has many checked items; found {checked}"
    );
    assert!(
        unchecked >= 5,
        "sample has many unchecked items; found {unchecked}"
    );
    // Inline formatting must still work inside a task item.
    assert!(
        doc.contains("<w:b "),
        "bold inside a task item should survive"
    );
    assert!(
        doc.contains("<w:hyperlink"),
        "a link inside a task item should survive"
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap (planned #1) — dual SVG embedding via the Word `<asvg>` extension (opt-in)
// ---------------------------------------------------------------------------------------------

/// Convert the asvg sample with `embed_svg` set, through the bytes path that runs the
/// post-packing `<asvg>` injection.
fn convert_asvg_sample(embed_svg: bool) -> Vec<u8> {
    let md = read_feature("asvg");
    let (docx, _stats) = convert_bytes_with(
        &md,
        &features_dir(),
        ConvertOptions {
            embed_svg,
            ..ConvertOptions::default()
        },
    );
    docx
}

#[test]
fn embed_svg_adds_the_original_vector_as_a_media_part() {
    let docx = convert_asvg_sample(true);
    let names = entry_names(&docx);
    // The PNG fallback is still present...
    assert!(
        names
            .iter()
            .any(|n| n.starts_with("word/media/") && n.ends_with(".png")),
        "the rasterized PNG fallback must still be embedded\n{names:?}"
    );
    // ...and the original SVG is embedded alongside it as its own media part.
    assert!(
        names
            .iter()
            .any(|n| n.starts_with("word/media/") && n.ends_with(".svg")),
        "the original SVG should be embedded as a media part\n{names:?}"
    );
}

#[test]
fn embed_svg_registers_content_type_and_relationship() {
    let docx = convert_asvg_sample(true);
    let ct = entry(&docx, "[Content_Types].xml").expect("content types part");
    assert!(
        ct.contains("image/svg+xml"),
        "an image/svg+xml default must be registered\n{ct}"
    );
    let rels = document_rels(&docx).expect("document rels");
    assert!(
        rels.contains(r#"Target="media/"#) && rels.contains(".svg"),
        "an image relationship targeting the .svg part must exist\n{rels}"
    );
}

#[test]
fn embed_svg_decorates_the_blip_with_the_asvg_extension() {
    let docx = convert_asvg_sample(true);
    let doc = document_xml(&docx);
    assert!(
        doc.contains("asvg:svgBlip"),
        "the picture blip must carry the asvg:svgBlip extension\n{doc}"
    );
    assert!(
        doc.contains("{96DAC541-7B7A-43D3-8B79-37D633B846F1}"),
        "the SVG blip extension GUID must be present\n{doc}"
    );
}

#[test]
fn without_embed_svg_no_vector_layer_is_added() {
    let docx = convert_asvg_sample(false);
    let names = entry_names(&docx);
    assert!(
        !names.iter().any(|n| n.ends_with(".svg")),
        "the default (PNG-only) path must not embed any SVG part\n{names:?}"
    );
    let doc = document_xml(&docx);
    assert!(
        !doc.contains("asvg:svgBlip"),
        "the default path must not decorate the blip with an asvg extension"
    );
    let ct = entry(&docx, "[Content_Types].xml").expect("content types part");
    assert!(
        !ct.contains("image/svg+xml"),
        "the default path must not register an svg content type"
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap (planned #2) — light-mode remap for dark-themed SVG diagrams (opt-in)
// ---------------------------------------------------------------------------------------------

/// Convert the dark-diagram sample, optionally with the light-mode remap, through the bytes
/// path. Returns the packed `.docx` bytes.
fn convert_dark_sample(light_mode: bool, embed_svg: bool) -> Vec<u8> {
    let md = read_feature("svg-light-mode");
    let (docx, _stats) = convert_bytes_with(
        &md,
        &features_dir(),
        ConvertOptions {
            svg_light_mode: light_mode,
            embed_svg,
            ..ConvertOptions::default()
        },
    );
    docx
}

/// Mean luminance (Rec. 601) of a decoded RGBA image's pixels.
fn mean_luminance(png: &[u8]) -> f64 {
    let img = image::load_from_memory(png).expect("media png should decode");
    let rgb = img.to_rgb8();
    let (mut sum, mut n) = (0.0f64, 0u64);
    for p in rgb.pixels() {
        sum += 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
        n += 1;
    }
    sum / n as f64
}

#[test]
fn light_mode_lightens_a_dark_diagram() {
    // The dark sample is mostly a near-black canvas; remapped, it should become mostly light.
    let dark = mean_luminance(&first_media_png(&convert_dark_sample(false, false)).unwrap());
    let light = mean_luminance(&first_media_png(&convert_dark_sample(true, false)).unwrap());
    assert!(
        dark < 90.0,
        "as-authored dark diagram should rasterize dark (got mean luminance {dark:.0})"
    );
    assert!(
        light > 160.0,
        "light-mode diagram should rasterize light (got mean luminance {light:.0})"
    );
    assert!(
        light > dark + 80.0,
        "light mode should clearly lighten the diagram ({dark:.0} -> {light:.0})"
    );
}

#[test]
fn light_mode_remaps_the_embedded_vector_source() {
    // With embed_svg on, the vector layer we keep is the *remapped* SVG, so the original dark
    // color tokens must be gone from it.
    let docx = convert_dark_sample(true, true);
    let svg_name = entry_names(&docx)
        .into_iter()
        .find(|n| n.ends_with(".svg"))
        .expect("an embedded svg part");
    let svg = entry(&docx, &svg_name).expect("svg part readable");
    assert!(
        !svg.contains("#0d1117") && !svg.contains("#e6edf3"),
        "remapped vector must not keep the dark background/text colors\n{svg}"
    );
}

#[test]
fn default_leaves_the_dark_source_untouched() {
    // Without light mode, the embedded vector is the original dark SVG, verbatim.
    let docx = convert_dark_sample(false, true);
    let svg_name = entry_names(&docx)
        .into_iter()
        .find(|n| n.ends_with(".svg"))
        .expect("an embedded svg part");
    let svg = entry(&docx, &svg_name).expect("svg part readable");
    assert!(
        svg.contains("#0d1117"),
        "the as-authored dark background must be preserved when light mode is off\n{svg}"
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap #8 — configurable page setup (size / orientation / margins)
// ---------------------------------------------------------------------------------------------

/// Convert the page-setup sample with a given page setup, from `examples/features` as base dir.
fn convert_page(page: PageSetup) -> Vec<u8> {
    let md = read_feature("page-setup");
    let dir = features_dir();
    let (docx, _stats) = convert_with(
        &md,
        &dir,
        ConvertOptions {
            page,
            ..ConvertOptions::default()
        },
    );
    docx
}

#[test]
fn default_page_setup_is_us_letter_portrait_one_inch() {
    let doc = document_xml(&convert_page(PageSetup::default()));
    assert!(
        doc.contains(r#"<w:pgSz w:w="12240" w:h="15840" />"#),
        "default page should be US Letter portrait (12240x15840)\n{doc}"
    );
    assert!(
        !doc.contains(r#"w:orient="landscape""#),
        "the default portrait page must not carry a landscape orientation flag"
    );
    assert!(
        doc.contains(r#"w:top="1440""#) && doc.contains(r#"w:left="1440""#),
        "default margins should be 1 in (1440 twips) on every side\n{doc}"
    );
}

#[test]
fn a4_page_size_sets_iso_dimensions_and_resizes_content() {
    let doc = document_xml(&convert_page(PageSetup {
        size: PageSize::A4,
        ..PageSetup::default()
    }));
    assert!(
        doc.contains(r#"<w:pgSz w:w="11906" w:h="16838" />"#),
        "A4 should be 11906x16838 twips\n{doc}"
    );
    // Content width = 11906 - 2*1440 = 9026; tables/code/rules must follow.
    assert!(
        doc.contains(r#"<w:tblW w:w="9026" w:type="dxa" />"#),
        "tables must resize to the A4 content width (9026)\n{doc}"
    );
    assert!(
        !doc.contains(r#"w:w="9360""#),
        "no element should keep the Letter content width on an A4 page"
    );
}

#[test]
fn landscape_swaps_dimensions_and_sets_orientation() {
    let doc = document_xml(&convert_page(PageSetup {
        size: PageSize::Letter,
        orientation: Orientation::Landscape,
        margins: Margins::uniform(1440),
    }));
    assert!(
        doc.contains(r#"<w:pgSz w:w="15840" w:h="12240" w:orient="landscape" />"#),
        "landscape Letter should swap to 15840x12240 and flag the orientation\n{doc}"
    );
    // Content width = 15840 - 2*1440 = 12960.
    assert!(
        doc.contains(r#"<w:tblW w:w="12960" w:type="dxa" />"#),
        "content should widen to the landscape text column (12960)\n{doc}"
    );
}

#[test]
fn custom_margins_land_in_page_setup_and_content_width() {
    // 0.5 in uniform margins on Letter -> content width 12240 - 2*720 = 10800.
    let doc = document_xml(&convert_page(PageSetup {
        size: PageSize::Letter,
        orientation: Orientation::Portrait,
        margins: Margins::uniform(720),
    }));
    assert!(
        doc.contains(r#"w:top="720""#)
            && doc.contains(r#"w:right="720""#)
            && doc.contains(r#"w:bottom="720""#)
            && doc.contains(r#"w:left="720""#),
        "every margin side should be the custom 720 twips\n{doc}"
    );
    assert!(
        doc.contains(r#"<w:tblW w:w="10800" w:type="dxa" />"#),
        "content should widen to match the narrower margins (10800)\n{doc}"
    );
}

#[test]
fn page_setup_content_width_helper_matches_geometry() {
    assert_eq!(PageSetup::default().content_width_dxa(), 9360);
    assert_eq!(
        PageSetup {
            size: PageSize::A4,
            ..PageSetup::default()
        }
        .content_width_dxa(),
        9026
    );
    assert_eq!(
        PageSetup {
            size: PageSize::Legal,
            orientation: Orientation::Landscape,
            margins: Margins::uniform(720),
        }
        .content_width_dxa(),
        20160 - 1440
    );
}

// ---------------------------------------------------------------------------------------------
// Roadmap #9 — richer code-block fidelity (syntax highlighting + language label)
// ---------------------------------------------------------------------------------------------

#[test]
fn labeled_code_fence_is_highlighted_with_a_language_label() {
    let (docx, _stats) = convert("```rust\nfn main() {}\n```\n");
    let doc = document_xml(&docx);

    // Uppercase language label above the block.
    assert!(
        doc.contains(">RUST<"),
        "a fenced block should carry an uppercase language label\n{doc}"
    );
    // Highlighted tokens carry explicit run colors. A71D5D is InspiredGitHub's keyword color
    // (the pinned default theme), so `fn` colors that span — proving real highlighting, not
    // just plain monospace.
    assert!(
        doc.contains(r#"<w:color w:val="A71D5D""#),
        "keywords should be colored by the syntax highlighter\n{doc}"
    );
    assert!(doc.contains("main"), "code text must survive highlighting");
}

#[test]
fn unlabeled_code_fence_falls_back_to_plain_monospace() {
    let (docx, _stats) = convert("```\njust text\n```\n");
    let doc = document_xml(&docx);

    assert!(doc.contains("just text"), "code text must survive");
    // No language means no label and no keyword coloring.
    assert!(
        !doc.contains(r#"<w:color w:val="A71D5D""#),
        "an unlabeled fence must not be syntax-highlighted"
    );
}

#[test]
fn highlighting_can_be_disabled() {
    let (docx, _stats) = convert_with(
        "```rust\nfn main() {}\n```\n",
        std::path::Path::new("."),
        ConvertOptions {
            highlight_code: false,
            ..ConvertOptions::default()
        },
    );
    let doc = document_xml(&docx);

    assert!(doc.contains("main"), "code text must survive");
    assert!(
        !doc.contains(">RUST<"),
        "no language label when highlighting is disabled"
    );
    assert!(
        !doc.contains(r#"<w:color w:val="A71D5D""#),
        "no token colors when highlighting is disabled"
    );
}

#[test]
fn code_highlight_sample_document_is_wired() {
    let (docx, stats) = convert_feature("code-highlight");
    let doc = document_xml(&docx);
    assert!(stats.code_blocks >= 3, "sample has three fenced blocks");
    assert!(doc.contains(">RUST<") && doc.contains(">PYTHON<"));
    // The trailing unlabeled fence still renders its text.
    assert!(doc.contains("plain text, no language"));
}

// ---------------------------------------------------------------------------------------------
// Roadmap #10 — swappable style themes (fonts, heading accent, link color, code theme)
// ---------------------------------------------------------------------------------------------

/// Convert `md` with a specific [`Theme`] preset (defaults otherwise), returning the raw docx
/// bytes so callers can inspect both `document.xml` and `styles.xml`.
fn convert_themed(md: &str, theme: Theme) -> Vec<u8> {
    let (docx, _stats) = convert_with(
        md,
        std::path::Path::new("."),
        ConvertOptions {
            theme,
            ..ConvertOptions::default()
        },
    );
    docx
}

fn styles_xml(docx: &[u8]) -> String {
    entry(docx, "word/styles.xml").expect("word/styles.xml present")
}

#[test]
fn default_theme_uses_word_blue_accent_and_calibri() {
    let docx = convert_themed(
        "# Heading\n\n[link](https://example.com)\n\n`code`\n",
        Theme::DEFAULT,
    );
    let styles = styles_xml(&docx);
    let doc = document_xml(&docx);
    // Heading style (in styles.xml) carries the default accent color and Calibri font.
    assert!(
        styles.contains(r#"w:val="2F5496""#),
        "default heading accent 2F5496\n{styles}"
    );
    assert!(
        styles.contains("Calibri"),
        "default body/heading font is Calibri"
    );
    // Hyperlink runs (in document.xml) use the default link blue.
    assert!(
        doc.contains(r#"w:val="0563C1""#),
        "default link color 0563C1"
    );
    // Inline code uses the default monospace font.
    assert!(doc.contains("Consolas"), "default mono font is Consolas");
}

#[test]
fn github_theme_recolors_heading_link_and_code_fill() {
    let docx = convert_themed(
        "# Heading\n\n[link](https://example.com)\n\n```rust\nfn main() {}\n```\n",
        Theme::GITHUB,
    );
    let styles = styles_xml(&docx);
    let doc = document_xml(&docx);
    // GitHub-blue accent replaces the default heading accent (styles.xml) and link (document.xml).
    assert!(
        styles.contains(r#"w:val="0969DA""#),
        "github heading accent 0969DA\n{styles}"
    );
    assert!(
        !styles.contains(r#"w:val="2F5496""#),
        "default heading accent must be gone"
    );
    assert!(
        doc.contains(r#"w:val="0969DA""#),
        "github link color 0969DA"
    );
    // Cooler code fill.
    assert!(
        doc.contains(r#"w:fill="F6F8FA""#),
        "github code fill F6F8FA"
    );
    assert!(
        !doc.contains(r#"w:fill="F2F2F2""#),
        "default code fill must be gone"
    );
}

#[test]
fn solarized_theme_swaps_highlight_palette_and_fill() {
    let default_doc = document_xml(&convert_themed(
        "```rust\nfn main() {}\n```\n",
        Theme::DEFAULT,
    ));
    let solar_doc = document_xml(&convert_themed(
        "```rust\nfn main() {}\n```\n",
        Theme::SOLARIZED,
    ));

    // Warm Solarized code fill.
    assert!(
        solar_doc.contains(r#"w:fill="FDF6E3""#),
        "solarized code fill FDF6E3\n{solar_doc}"
    );
    // The Solarized highlight theme colors keywords differently than InspiredGitHub's A71D5D.
    assert!(
        default_doc.contains(r#"w:color w:val="A71D5D""#),
        "default highlight uses InspiredGitHub keyword color"
    );
    assert!(
        !solar_doc.contains(r#"w:color w:val="A71D5D""#),
        "solarized theme must not reuse the InspiredGitHub keyword color"
    );
}

#[test]
fn theme_from_name_resolves_known_presets() {
    assert_eq!(Theme::from_name("default"), Some(Theme::DEFAULT));
    assert_eq!(Theme::from_name("GitHub"), Some(Theme::GITHUB));
    assert_eq!(Theme::from_name(" solarized "), Some(Theme::SOLARIZED));
    assert_eq!(Theme::from_name("nope"), None);
}

#[test]
fn themes_sample_document_is_wired() {
    let (docx, stats) = convert_feature("themes");
    let styles = styles_xml(&docx);
    let doc = document_xml(&docx);
    assert!(stats.code_blocks >= 1, "sample has a fenced block");
    // Default preset markers are present when the sample converts with default options.
    assert!(
        styles.contains(r#"w:val="2F5496""#),
        "default heading accent"
    );
    assert!(doc.contains(r#"w:val="0563C1""#), "default link color");
    assert!(doc.contains("themed code block"), "code text survives");
}
