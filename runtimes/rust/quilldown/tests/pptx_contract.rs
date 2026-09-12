use std::collections::HashSet;
use std::io::{Cursor, Read};

use quilldown::typra_spec::{LoadContext, Presentation, SaveContext};
use roxmltree::Document;
use serde_json::json;
use zip::ZipArchive;

fn sample_presentation() -> Presentation {
    Presentation::try_load_from_value(
        &json!({
            "slides": [
                {
                    "title": "Roadmap",
                    "elements": [
                        {
                            "kind": "text_box",
                            "text": "Open docs",
                            "hyperlink": "https://example.com/docs",
                            "bullets": ["Word", "PowerPoint"]
                        },
                        {
                            "kind": "text_box",
                            "text": "Open repo",
                            "hyperlink": "https://example.com/repo"
                        }
                    ],
                    "speakerNotes": ["Use generated typra models.", "Inspect emitted package."]
                }
            ],
            "metadata": {"title": "Roadmap", "creator": "Quilldown"}
        }),
        &LoadContext::default(),
    )
    .unwrap()
}

#[test]
fn emit_powerpoint_reports_manifest_and_invariants() {
    let result = quilldown::typra_runtime::emit_powerpoint(&sample_presentation(), &None).unwrap();
    let value = result.to_value(&SaveContext::default());

    assert_eq!(value["stats"]["slides"], 1);
    assert_eq!(value["stats"]["textBoxes"], 2);
    assert_eq!(value["stats"]["bulletItems"], 2);
    assert_eq!(value["stats"]["links"], 2);
    assert_eq!(value["stats"]["speakerNotes"], 2);
    assert_eq!(value["manifest"]["kind"], "pptx");
    assert_eq!(value["manifest"]["coreProperties"]["title"], "Roadmap");
    assert_eq!(value["manifest"]["coreProperties"]["creator"], "Quilldown");
    for (_, invariant) in value["invariants"].as_object().unwrap() {
        assert_eq!(invariant, true);
    }
    assert!(value["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn render_pptx_bytes_contains_required_openable_parts_and_relationships() {
    let bytes = quilldown::typra_runtime::render_pptx_bytes(&sample_presentation()).unwrap();
    let mut package = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let names = package
        .file_names()
        .map(|name| name.to_string())
        .collect::<HashSet<_>>();
    for name in [
        "[Content_Types].xml",
        "_rels/.rels",
        "docProps/core.xml",
        "ppt/presentation.xml",
        "ppt/_rels/presentation.xml.rels",
        "ppt/slides/slide1.xml",
        "ppt/slides/_rels/slide1.xml.rels",
        "ppt/slideLayouts/slideLayout1.xml",
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        "ppt/slideMasters/slideMaster1.xml",
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        "ppt/notesSlides/notesSlide1.xml",
        "ppt/notesSlides/_rels/notesSlide1.xml.rels",
        "ppt/notesMasters/notesMaster1.xml",
        "ppt/notesMasters/_rels/notesMaster1.xml.rels",
        "ppt/theme/theme1.xml",
    ] {
        assert!(names.contains(name), "missing package part {name}");
    }

    let presentation_xml = read_part(&mut package, "ppt/presentation.xml");
    let presentation = Document::parse(&presentation_xml).unwrap();
    assert!(presentation.descendants().any(|node| node.has_tag_name((
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "sldMasterIdLst"
    ))));
    assert!(presentation.descendants().any(|node| node.has_tag_name((
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "notesMasterIdLst"
    ))));
    assert!(presentation.descendants().any(|node| node.has_tag_name((
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "notesSz"
    ))));

    let presentation_rels_xml = read_part(&mut package, "ppt/_rels/presentation.xml.rels");
    assert!(presentation_rels_xml.contains(r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster" Target="notesMasters/notesMaster1.xml""#));

    let slide_master_xml = read_part(&mut package, "ppt/slideMasters/slideMaster1.xml");
    let slide_master = Document::parse(&slide_master_xml).unwrap();
    assert!(slide_master.descendants().any(|node| node.has_tag_name((
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "clrMap"
    ))));
    let notes_master_xml = read_part(&mut package, "ppt/notesMasters/notesMaster1.xml");
    let notes_master = Document::parse(&notes_master_xml).unwrap();
    assert!(notes_master.descendants().any(|node| node.has_tag_name((
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "clrMap"
    ))));
    assert!(notes_master.descendants().any(|node| {
        node.has_tag_name((
            "http://schemas.openxmlformats.org/presentationml/2006/main",
            "ph",
        )) && node.attribute("type") == Some("body")
    }));

    let theme_xml = read_part(&mut package, "ppt/theme/theme1.xml");
    let theme = Document::parse(&theme_xml).unwrap();
    for tag in ["ea", "cs"] {
        assert!(theme.descendants().any(|node| node
            .has_tag_name(("http://schemas.openxmlformats.org/drawingml/2006/main", tag))));
    }

    let slide_rels_xml = read_part(&mut package, "ppt/slides/_rels/slide1.xml.rels");
    assert!(slide_rels_xml.contains(r#"Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml""#));
    assert!(slide_rels_xml.contains(r#"Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/docs" TargetMode="External""#));
    assert!(slide_rels_xml.contains(r#"Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/repo" TargetMode="External""#));
    assert!(slide_rels_xml.contains(r#"Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml""#));

    let notes_xml = read_part(&mut package, "ppt/notesSlides/notesSlide1.xml");
    let notes = Document::parse(&notes_xml).unwrap();
    let notes_text = notes
        .descendants()
        .filter(|node| {
            node.has_tag_name(("http://schemas.openxmlformats.org/drawingml/2006/main", "t"))
        })
        .filter(|node| {
            node.ancestors().any(|ancestor| {
                ancestor.has_tag_name((
                    "http://schemas.openxmlformats.org/presentationml/2006/main",
                    "sp",
                )) && ancestor.descendants().any(|descendant| {
                    descendant.has_tag_name((
                        "http://schemas.openxmlformats.org/presentationml/2006/main",
                        "ph",
                    )) && descendant.attribute("type") == Some("body")
                })
            })
        })
        .map(|node| node.text().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        notes_text,
        ["Use generated typra models.", "Inspect emitted package."]
    );
}

fn read_part(package: &mut ZipArchive<Cursor<Vec<u8>>>, name: &str) -> String {
    let mut file = package.by_name(name).unwrap();
    let mut value = String::new();
    file.read_to_string(&mut value).unwrap();
    value
}
