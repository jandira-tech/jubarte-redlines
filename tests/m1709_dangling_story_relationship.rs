// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1709: a broken link cannot hide a header/footer's content changes.
//! Packages and dependencies are entirely in memory.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    StoryDiffError, accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::opc::{OpcError, PartFs};
use jubarte::xmllinq::Dom;
use std::io::{Cursor, Write};

const W_URI: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const P_URI: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

fn paragraph(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

fn package(kind: &str, link: &str, revised: bool) -> Vec<u8> {
    let attr = if link == "anchor" {
        "w:anchor=\"local\""
    } else {
        "r:id=\"rIdL\""
    };
    let mut story = format!(
        "<w:p><w:hyperlink {attr}><w:r><w:t>Head.</w:t></w:r></w:hyperlink></w:p><w:tbl><w:tblPr><w:tblW w:w=\"4800\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2400\"/><w:gridCol w:w=\"2400\"/></w:tblGrid><w:tr>"
    );
    for (text, tail) in [("Alpha.", "Tail."), ("Fixed.", "Tail2.")] {
        story.push_str("<w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/></w:tcPr>");
        story.push_str(&paragraph(text));
        if !revised {
            story.push_str(&paragraph(tail));
        }
        story.push_str("</w:tc>");
    }
    story.push_str("</w:tr></w:tbl>");
    let mut parts = Vec::new();
    let ct = "application/vnd.openxmlformats-officedocument.wordprocessingml.";
    let mut overrides = format!(
        "<Override PartName=\"/word/document.xml\" ContentType=\"{ct}document.main+xml\"/>"
    );
    let mut rels = String::new();
    let body = if kind == "body" {
        format!("{story}<w:sectPr/>")
    } else {
        let root = if kind == "header" { "hdr" } else { "ftr" };
        parts.push((
            format!("word/{kind}1.xml"),
            format!("<w:{root} xmlns:w=\"{W_URI}\" xmlns:r=\"{R_URI}\">{story}</w:{root}>"),
        ));
        overrides.push_str(&format!(
            "<Override PartName=\"/word/{kind}1.xml\" ContentType=\"{ct}{kind}+xml\"/>"
        ));
        rels = format!(
            "<Relationship Id=\"rIdStory\" Type=\"{R_URI}/{kind}\" Target=\"{kind}1.xml\"/>"
        );
        if matches!(link, "resolved" | "wrong-type" | "missing-target") {
            let ty = if matches!(link, "wrong-type" | "missing-target") {
                "image"
            } else {
                "hyperlink"
            };
            let (target, mode) = if link == "missing-target" {
                ("absent.xml", "")
            } else {
                ("https://example.invalid/", " TargetMode=\"External\"")
            };
            parts.push((format!("word/_rels/{kind}1.xml.rels"), format!("<Relationships xmlns=\"{P_URI}\"><Relationship Id=\"rIdL\" Type=\"{R_URI}/{ty}\" Target=\"{target}\"{mode}/></Relationships>")));
        }
        format!(
            "{}<w:sectPr><w:{kind}Reference w:type=\"default\" r:id=\"rIdStory\"/></w:sectPr>",
            paragraph("Shared body.")
        )
    };
    parts.push(("word/document.xml".into(), format!("<w:document xmlns:w=\"{W_URI}\" xmlns:r=\"{R_URI}\"><w:body>{body}</w:body></w:document>")));
    parts.push((
        "word/_rels/document.xml.rels".into(),
        format!("<Relationships xmlns=\"{P_URI}\">{rels}</Relationships>"),
    ));
    parts.push(("_rels/.rels".into(), format!("<Relationships xmlns=\"{P_URI}\"><Relationship Id=\"rIdMain\" Type=\"{R_URI}/officeDocument\" Target=\"word/document.xml\"/></Relationships>")));
    parts.push(("[Content_Types].xml".into(), format!("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/>{overrides}</Types>")));
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, xml) in parts {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn text(bytes: &[u8], part: &str) -> String {
    let xml = PartFs::open(bytes).unwrap().part_string(part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.descendants(dom.root(doc).unwrap(), Some(&W::t()))
        .into_iter()
        .map(|n| dom.value(n))
        .collect()
}

fn settings(word: bool) -> WmlComparerSettings {
    if word {
        WmlComparerSettings::default()
    } else {
        WmlComparerSettings::powertools_faithful()
    }
}

fn roundtrip(kind: &str, link: &str, word: bool) {
    let a = package(kind, link, false);
    let b = package(kind, link, true);
    let out = compare_documents_with_settings(&a, &b, &settings(word)).unwrap();
    let part = if kind == "body" {
        "word/document.xml".into()
    } else {
        format!("word/{kind}1.xml")
    };
    let xml = PartFs::open(&out).unwrap().part_string(&part).unwrap();
    assert!(
        xml.contains("<w:del "),
        "{part}: missing deletions ({link}, word={word})"
    );
    assert_eq!(
        text(&accept_revisions(&out).unwrap(), &part),
        text(&b, &part),
        "accept must yield B's story"
    );
    assert_eq!(
        text(&reject_revisions(&out).unwrap(), &part),
        text(&a, &part),
        "reject must yield A's story"
    );
    if link == "dangling" || link == "missing-target" {
        assert!(
            !xml.contains("r:id="),
            "unresolvable link must be neutralized"
        );
        assert!(
            !xml.contains("rPrChange"),
            "neutralizing both sides must not invent formatting revisions"
        );
    }
}

#[test]
fn dangling_header_word() {
    roundtrip("header", "dangling", true);
}
#[test]
fn dangling_header_powertools() {
    roundtrip("header", "dangling", false);
}
#[test]
fn dangling_footer_word() {
    roundtrip("footer", "dangling", true);
}
#[test]
fn dangling_footer_powertools() {
    roundtrip("footer", "dangling", false);
}

#[test]
fn controls_still_diff() {
    for word in [true, false] {
        for (kind, link) in [
            ("header", "anchor"),
            ("header", "resolved"),
            ("body", "dangling"),
        ] {
            roundtrip(kind, link, word);
        }
    }
}

#[test]
fn missing_internal_target_is_neutralized() {
    for word in [true, false] {
        for kind in ["header", "footer"] {
            roundtrip(kind, "missing-target", word);
        }
    }
}

#[test]
fn uncarryable_story_is_an_error_naming_the_part() {
    for word in [true, false] {
        for kind in ["header", "footer"] {
            let a = package(kind, "dangling", false);
            let b = package(kind, "wrong-type", true);
            let result = compare_documents_with_settings(&a, &b, &settings(word));
            assert!(
                result.is_err(),
                "uncarryable {kind} must fail, never silently emit A"
            );
            assert_story_error(result.unwrap_err(), &format!("word/{kind}1.xml"));
        }
    }
}

#[test]
fn missing_or_wrong_story_root_is_not_silently_copied() {
    for word in [true, false] {
        for kind in ["header", "footer"] {
            let a = package(kind, "anchor", false);
            for bad_xml in [None, Some("<wrong/>")] {
                let mut b = PartFs::open(&package(kind, "anchor", true)).unwrap();
                let part = format!("word/{kind}1.xml");
                if let Some(xml) = bad_xml {
                    b.set_part(&part, xml.as_bytes().to_vec());
                } else {
                    b.remove_part(&part);
                }
                let b = b.to_zip().unwrap();
                for (a, b) in [(&a, &b), (&b, &a)] {
                    let result = compare_documents_with_settings(a, b, &settings(word));
                    assert!(
                        result.is_err(),
                        "unreadable {part} must fail, never silently emit A"
                    );
                    assert_story_error(result.unwrap_err(), &part);
                }
            }
        }
    }
}

fn assert_story_error(err: OpcError, part: &str) {
    let OpcError::Io(io) = err else {
        panic!("expected a typed story refusal, got {err}");
    };
    let typed = io
        .get_ref()
        .unwrap()
        .downcast_ref::<StoryDiffError>()
        .expect("typed StoryDiffError");
    assert_eq!(typed.part, part);
    assert!(!typed.reason.is_empty());
    assert!(io.to_string().contains("STORY_DIFF_FAILED"));
}

#[test]
fn one_sided_dangling_links_use_each_sources_relationships() {
    for word in [true, false] {
        for kind in ["header", "footer"] {
            for (a_link, b_link) in [("dangling", "resolved"), ("resolved", "dangling")] {
                let a = package(kind, a_link, false);
                let b = package(kind, b_link, true);
                let out = compare_documents_with_settings(&a, &b, &settings(word)).unwrap();
                let part = format!("word/{kind}1.xml");
                assert_eq!(
                    text(&accept_revisions(&out).unwrap(), &part),
                    text(&b, &part)
                );
                assert_eq!(
                    text(&reject_revisions(&out).unwrap(), &part),
                    text(&a, &part)
                );
                let pkg = PartFs::open(&out).unwrap();
                // The shared hyperlink text takes B's wrapper. A's valid
                // target must not resurrect B's missing reference, nor may
                // A's missing reference erase B's valid target.
                assert_eq!(
                    pkg.part_string(&part).unwrap().contains("r:id="),
                    b_link == "resolved",
                    "{kind}, word={word}, {a_link} -> {b_link}"
                );
                assert!(
                    pkg.read_rels_for(&part)
                        .unwrap()
                        .items
                        .iter()
                        .any(|r| r.target == "https://example.invalid/")
                );
            }
        }
    }
}
