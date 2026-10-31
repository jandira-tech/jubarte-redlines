// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Ring 1 probes: each check must fire on intentionally broken packages,
//! and a healthy compare output must pass.

mod common;

use std::io::{Cursor, Write};

use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const ORIG: &[u8] = include_bytes!("fixtures/redline/original.docx");
const MOD: &[u8] = include_bytes!("fixtures/redline/modified.docx");

// Synthetic versions of the A–H shapes probed in Word on 2026-10-18.
fn nested_box_probe(probe: char) -> String {
    let story = |inner: &str| {
        format!("<w:txbxContent><w:p><w:r><w:t>Alice</w:t>{inner}</w:r></w:p></w:txbxContent>")
    };
    let drawing = |inner: &str, frame: &str| {
        format!(
            "<w:drawing><wp:{frame}><wps:wsp><wps:txbx>{}</wps:txbx></wps:wsp></wp:{frame}></w:drawing>",
            story(inner)
        )
    };
    let vml = |inner: &str| {
        format!(
            "<w:pict><v:shape><v:textbox>{}</v:textbox></v:shape></w:pict>",
            story(inner)
        )
    };
    let inner_drawing = drawing("", if probe == 'E' { "inline" } else { "anchor" });
    let inner_fallback = vml("");
    let inner = format!(
        "<mc:AlternateContent><mc:Choice Requires=\"wps\">{inner_drawing}</mc:Choice><mc:Fallback>{inner_fallback}</mc:Fallback></mc:AlternateContent>"
    );
    match probe {
        'F' => vml(&vml("")),
        'G' => vml(""),
        'H' => drawing(&vml(""), "anchor"),
        'I' => drawing(&inner_drawing, "anchor"),
        _ => {
            let choice = drawing(
                if matches!(probe, 'A' | 'C' | 'E') {
                    &inner
                } else {
                    ""
                },
                "anchor",
            );
            let inner_vml = vml("");
            let fallback = vml(if matches!(probe, 'A' | 'D') {
                &inner_vml
            } else {
                ""
            });
            format!(
                "<mc:AlternateContent><mc:Choice Requires=\"wps\">{choice}</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>"
            )
        }
    }
}

fn nested_box_findings(probe: char, part: &str) -> Vec<jubarte::validate::Finding> {
    let content = nested_box_probe(probe);
    let (root, content) = match part {
        "word/header1.xml" => ("hdr", format!("<w:p><w:r>{content}</w:r></w:p>")),
        "word/footer1.xml" => ("ftr", format!("<w:p><w:r>{content}</w:r></w:p>")),
        "word/footnotes.xml" => (
            "footnotes",
            format!("<w:footnote w:id=\"1\"><w:p><w:r>{content}</w:r></w:p></w:footnote>"),
        ),
        "word/endnotes.xml" => (
            "endnotes",
            format!("<w:endnote w:id=\"1\"><w:p><w:r>{content}</w:r></w:p></w:endnote>"),
        ),
        "word/comments.xml" => (
            "comments",
            format!(
                "<w:comment w:id=\"0\" w:author=\"Alice\"><w:p><w:r>{content}</w:r></w:p></w:comment>"
            ),
        ),
        _ => (
            "document",
            format!("<w:body><w:p><w:r>{content}</w:r></w:p></w:body>"),
        ),
    };
    let xml = format!(
        r#"<w:{root} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml">{content}</w:{root}>"#
    );
    let bytes = zip_with_parts(&[(part, &xml)]);
    jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "NESTED_TEXT_BOX")
        .collect()
}

macro_rules! fatal_nested_box_probe {
    ($name:ident, $probe:literal, $part:literal) => {
        #[test]
        fn $name() {
            let findings = nested_box_findings($probe, $part);
            assert_eq!(
                findings.len(),
                1,
                "probe {} in {}: {findings:?}",
                $probe,
                $part
            );
            let finding = &findings[0];
            assert!(finding.word_fatal && !finding.repairable, "{finding:?}");
            assert_eq!(finding.severity, "error");
            assert_eq!(finding.part, $part);
            assert!(finding.location.contains("w:txbxContent[0]"), "{finding:?}");
            assert_eq!(finding.r#ref, finding.location);
        }
    };
}

fatal_nested_box_probe!(nested_box_a_anchor_is_word_fatal, 'A', "word/document.xml");
fatal_nested_box_probe!(nested_box_c_choice_is_word_fatal, 'C', "word/document.xml");
fatal_nested_box_probe!(nested_box_e_inline_is_word_fatal, 'E', "word/document.xml");
fatal_nested_box_probe!(nested_box_f_vml_is_word_fatal, 'F', "word/document.xml");
fatal_nested_box_probe!(nested_box_h_mixed_is_word_fatal, 'H', "word/document.xml");
fatal_nested_box_probe!(
    nested_box_bare_drawing_is_word_fatal,
    'I',
    "word/document.xml"
);
fatal_nested_box_probe!(nested_box_header_is_word_fatal, 'A', "word/header1.xml");
fatal_nested_box_probe!(nested_box_footer_is_word_fatal, 'A', "word/footer1.xml");
fatal_nested_box_probe!(nested_box_footnote_is_word_fatal, 'A', "word/footnotes.xml");
fatal_nested_box_probe!(nested_box_endnote_is_word_fatal, 'A', "word/endnotes.xml");
fatal_nested_box_probe!(nested_box_comment_is_word_fatal, 'A', "word/comments.xml");

#[test]
fn nested_box_b_d_g_are_not_word_fatal() {
    for probe in ['B', 'D', 'G'] {
        assert!(
            nested_box_findings(probe, "word/document.xml").is_empty(),
            "probe {probe}"
        );
    }
}

fn zip_with_parts(parts: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )
        .unwrap();
        for (name, body) in parts {
            z.start_file(*name, opts).unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn package_with_comment_graph(
    document: &str,
    comments: &str,
    comments_extended: &str,
    comments_ids: &str,
    comments_extensible: &str,
) -> Vec<u8> {
    let bytes = zip_with_parts(&[("word/document.xml", document)]);
    let mut pkg = PartFs::open(&bytes).expect("open package");
    for (part, xml, content_type, relationship_type) in [
        (
            "word/comments.xml",
            comments,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        ),
        (
            "word/commentsExtended.xml",
            comments_extended,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
            "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
        ),
        (
            "word/commentsIds.xml",
            comments_ids,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
            "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
        ),
        (
            "word/commentsExtensible.xml",
            comments_extensible,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
            "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
        ),
    ] {
        pkg.set_part(part, xml.as_bytes().to_vec());
        pkg.add_content_type_override(&format!("/{part}"), content_type);
        pkg.add_document_relationship(
            "word/document.xml",
            relationship_type,
            part.strip_prefix("word/").expect("word part"),
        );
    }
    pkg.to_zip().expect("serialize package")
}

const MINIMAL_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:r><w:t>hello</w:t></w:r></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;

const COMMENT_DOCUMENT: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:commentRangeStart w:id="0"/><w:r><w:t>x</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p><w:sectPr/></w:body></w:document>"#;
const COMMENTS: &str = r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:comment w:id="0" w:author="A"><w:p w14:paraId="11111111"><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#;
const COMMENTS_EXTENDED: &str = r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="11111111"/></w15:commentsEx>"#;
const COMMENTS_IDS: &str = r#"<w16cid:commentsIds xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid"><w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000001"/></w16cid:commentsIds>"#;
const COMMENTS_EXTENSIBLE: &str = r#"<w16cex:commentsExtensible xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex"><w16cex:commentExtensible w16cex:durableId="10000001"/></w16cex:commentsExtensible>"#;

#[test]
fn healthy_compare_output_passes_ring1() {
    let out = compare_documents(ORIG, MOD, "Ring1").expect("compare");
    assert_word_valid_package(&out);
}

#[test]
fn probe_dangling_rid_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:hyperlink r:id="rId999"><w:r><w:t>x</w:t></w:r></w:hyperlink></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", doc),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
        ),
    ]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("dangling") || e.contains("rId999")),
        "expected dangling rId error, got: {:?}",
        report.errors
    );
}

fn assert_dangling_relationship_probe(markup: &str, count: usize, fatal: bool) {
    let bytes = splice_first_paragraph_end(&format!("<w:r>{markup}</w:r>"));
    let report = check_word_valid_package(&bytes);
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "DANGLING_RELATIONSHIP")
        .collect();
    assert_eq!(findings.len(), count, "{markup}: {:?}", report.findings);
    for finding in findings {
        assert_eq!(finding.part, "word/document.xml");
        assert_eq!(finding.word_fatal, fatal, "{finding:?}");
        assert!(finding.repairable, "{finding:?}");
        if !fatal {
            assert_eq!(finding.severity, "warning");
        }
        assert_eq!(
            finding.message,
            "dangling relationship id 'rId99' referenced from 'word/document.xml'"
        );
    }
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(
        repaired
            .repaired
            .iter()
            .filter(|f| f.code == "DANGLING_RELATIONSHIP")
            .count(),
        count
    );
    assert!(repaired.remaining.is_empty(), "{:?}", repaired.remaining);
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    assert!(
        !xml.contains("rId99"),
        "dangling attribute survived repair: {xml}"
    );
    assert_word_valid_package(&repaired.docx);
}

#[test]
fn relationship_probe_diagram_attributes() {
    assert_dangling_relationship_probe(
        r#"<dgm:relIds xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" r:dm="rId99" r:lo="rId99" r:qs="rId99" r:cs="rId99"/>"#,
        4,
        false,
    );
}

#[test]
fn relationship_probe_rebound_prefix() {
    assert_dangling_relationship_probe(
        r#"<w:drawing xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" rel:embed="rId99"/>"#,
        1,
        true,
    );
}

#[test]
fn relationship_probe_single_quotes() {
    assert_dangling_relationship_probe("<w:pict r:link='rId99'/>", 1, true);
}

#[test]
fn relationship_probe_every_attribute_local_name() {
    for local in [
        "pict",
        "href",
        "target",
        "top",
        "bottom",
        "left",
        "right",
        "origin",
        "align",
        "futureAttribute",
    ] {
        assert_dangling_relationship_probe(&format!("<w:pict r:{local} = 'rId99'/>"), 1, false);
    }
}

#[test]
fn relationship_probe_ignores_nonattributes_and_other_namespaces() {
    let bytes = splice_first_paragraph_end(
        r#"<!-- r:id="rId99" r:embed="rId99" r:link="rId99" -->
<w:r><w:t><![CDATA[r:id="rId99"]]></w:t></w:r>
<w:r><w:pict xmlns:r="urn:application" r:id="rId99" embed="rId99" data='r:embed="rId99"'/></w:r>"#,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.code != "DANGLING_RELATIONSHIP" && f.code != "MISSING_REL_TARGET"),
        "{:?}",
        report.findings
    );
}

#[test]
fn relationship_probe_known_and_empty_ids_pass() {
    let bytes = splice_first_paragraph_end(
        r#"<w:r><w:pict xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" rel:dm='rId1' r:lo="" r:qs='rId&#49;'/></w:r>"#,
    );
    assert_word_valid_package(&bytes);
}

#[test]
fn relationship_probe_leftover_header_uses_its_own_rels() {
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:p><w:r><w:pict rel:pict='rId1'/></w:r></w:p></w:hdr>"#;
    let mut pkg = PartFs::open(NAMELESS_MOVE_BASE).unwrap();
    pkg.set_part("word/header9.xml", header.as_bytes().to_vec());
    assert!(pkg.read_rels_for("word/header9.xml").is_none());
    let bytes = pkg.to_zip().unwrap();
    assert_eq!(
        common::docx::part_string(&bytes, "word/header9.xml").as_deref(),
        Some(header)
    );
    let report = check_word_valid_package(&bytes);
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "DANGLING_RELATIONSHIP")
        .collect();
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].part, "word/header9.xml");
    assert!(findings[0].message.contains("rId1"));
}

#[test]
fn relationship_probe_missing_target_keeps_existing_code() {
    let bytes = splice_first_paragraph_end(
        r#"<w:r><w:pict xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" rel:dm='rId1'/></w:r>"#,
    );
    let mut pkg = PartFs::open(&bytes).unwrap();
    let target = pkg.read_rels_for("word/document.xml").unwrap().items[0]
        .target
        .clone();
    let resolved = pkg.resolve_rel_target("word/document.xml", &target);
    pkg.remove_part(&resolved);
    let report = check_word_valid_package(&pkg.to_zip().unwrap());
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "MISSING_REL_TARGET")
        .collect();
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(findings[0].message.contains(&target));
}

fn assert_missing_target_probe(bytes: &[u8], owner: &str, target: &str, used: bool) {
    let report = check_word_valid_package(bytes);
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.code != "OVERRIDE_WITHOUT_PART"),
        "{:?}",
        report.findings
    );
    // The public validator must not add the triage copy of this Ring-1 finding.
    for findings in [report.findings, jubarte::validate::validate(bytes).unwrap()] {
        let missing: Vec<_> = findings
            .iter()
            .filter(|f| f.code == "MISSING_REL_TARGET")
            .collect();
        assert_eq!(
            missing.len(),
            1,
            "expected one missing target: {findings:?}"
        );
        let finding = missing[0];
        assert_eq!(finding.part, owner);
        assert!(finding.message.contains(target), "{finding:?}");
        assert_eq!(finding.word_fatal, used, "{finding:?}");
        assert!(!finding.repairable, "{finding:?}");
        assert_eq!(finding.severity, if used { "error" } else { "warning" });
    }
}

#[test]
fn relationship_probe_missing_target_unused_styles() {
    let mut pkg = PartFs::open(NAMELESS_MOVE_BASE).unwrap();
    assert!(jubarte::validate::ring1(&pkg).is_empty());
    pkg.remove_part("word/styles.xml");
    pkg.remove_content_type_override("/word/styles.xml");
    assert_missing_target_probe(
        &pkg.to_zip().unwrap(),
        "word/_rels/document.xml.rels",
        "styles.xml",
        false,
    );
}

#[test]
fn relationship_probe_missing_target_package_app_properties() {
    let mut pkg = PartFs::open(NAMELESS_MOVE_BASE).unwrap();
    pkg.remove_part("docProps/app.xml");
    pkg.remove_content_type_override("/docProps/app.xml");
    assert_missing_target_probe(
        &pkg.to_zip().unwrap(),
        "_rels/.rels",
        "docProps/app.xml",
        false,
    );
}

#[test]
fn relationship_probe_missing_target_used_once_and_word_fatal() {
    // Two uses of the same relationship still describe just one missing target.
    let bytes = splice_first_paragraph_end(
        r#"<w:hyperlink r:id="rId1"><w:r><w:t>x</w:t></w:r></w:hyperlink><w:r><w:pict xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" rel:embed='rId1'/></w:r>"#,
    );
    let mut pkg = PartFs::open(&bytes).unwrap();
    pkg.remove_part("word/styles.xml");
    pkg.remove_content_type_override("/word/styles.xml");
    assert_missing_target_probe(
        &pkg.to_zip().unwrap(),
        "word/document.xml",
        "styles.xml",
        true,
    );
}

#[test]
fn relationship_probe_missing_target_in_non_xml_and_absent_owners() {
    for (owner, rels) in [
        ("word/payload.bin", "word/_rels/payload.bin.rels"),
        ("word/absent.xml", "word/_rels/absent.xml.rels"),
    ] {
        let mut pkg = PartFs::open(NAMELESS_MOVE_BASE).unwrap();
        if owner.ends_with(".bin") {
            pkg.set_part(owner, vec![1, 2, 3]);
            pkg.add_content_type_override(owner, "application/octet-stream");
        }
        pkg.add_document_relationship(owner, "urn:test:missing", "../missing.xml");
        assert_missing_target_probe(&pkg.to_zip().unwrap(), rels, "../missing.xml", false);
    }
}

#[test]
fn relationship_probe_unused_targets_resolve_and_skip_external() {
    for owner in ["word/document.xml", ""] {
        let mut pkg = PartFs::open(NAMELESS_MOVE_BASE).unwrap();
        let target = if owner.is_empty() {
            "word/../word/styles.xml"
        } else {
            "../word/styles.xml"
        };
        let xml = format!(
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="relative" Type="urn:test" Target="{target}"/>
<Relationship Id="absolute" Type="urn:test" Target="/word/styles.xml"/>
<Relationship Id="external" Type="urn:test" Target="missing.xml" TargetMode="External"/>
<Relationship Id="http" Type="urn:test" Target="http://example.invalid/missing"/>
<Relationship Id="https" Type="urn:test" Target="https://example.invalid/missing"/>
<Relationship Id="mailto" Type="urn:test" Target="mailto:owner@example.invalid"/>
</Relationships>"#
        );
        pkg.set_part(
            if owner.is_empty() {
                "_rels/.rels"
            } else {
                "word/_rels/document.xml.rels"
            },
            xml.into_bytes(),
        );
        let report = check_word_valid_package(&pkg.to_zip().unwrap());
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.code != "MISSING_REL_TARGET"),
            "owner {owner}: {:?}",
            report.findings
        );
    }
}

/// Word refuses a package that repeats a relationship Id (docxide case8's
/// fontTable rels: "unreadable content" in Word 16). Opening keeps one copy
/// of an identical repeat, but the finding stays, so `validate` still says
/// the source is not Word-valid.
#[test]
fn probe_duplicate_rid_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:hyperlink r:id="rId7"><w:r><w:t>x</w:t></w:r></w:hyperlink></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let link = r#"<Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://a.example/" TargetMode="External"/>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", doc),
        (
            "word/_rels/document.xml.rels",
            &format!(
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{link}{link}</Relationships>"#
            ),
        ),
    ]);
    let findings = jubarte::validate::validate(&bytes).expect("an identical repeat opens");
    assert!(
        findings.iter().any(|f| f.code == "DUPLICATE_RID"),
        "expected DUPLICATE_RID, got: {findings:?}"
    );
    // `repair` writes the one kept copy: the finding moves to `repaired`.
    let repaired = jubarte::validate::repair(&bytes).expect("repairable");
    assert!(repaired.repaired.iter().any(|f| f.code == "DUPLICATE_RID"));
    assert!(!repaired.remaining.iter().any(|f| f.code == "DUPLICATE_RID"));
    assert_word_valid_package(&repaired.docx);
}

#[test]
fn probe_cross_kind_revision_ids_are_valid() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:ins w:id="1" w:author="A"><w:r><w:t>a</w:t></w:r></w:ins>
      <w:del w:id="1" w:author="A"><w:r><w:delText>b</w:delText></w:r></w:del>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(report.ok(), "accepted ins/del pair: {:?}", report.errors);
}

#[test]
fn probe_wt_under_del_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:del w:id="1" w:author="A"><w:r><w:t>should be delText</w:t></w:r></w:del>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|e| e.contains("w:t under w:del")),
        "expected w:t under w:del error, got: {:?}",
        report.errors
    );
}

/// A comment's reference run written before its paragraph, straight into
/// the body: Word loads the document empty (2026-10-10).
#[test]
fn probe_run_outside_paragraph_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:r><w:t>stray</w:t></w:r>
    <w:p><w:r><w:t>kept</w:t></w:r></w:p>
    <w:tbl><w:tr><w:tc><w:ins w:id="1" w:author="A"><w:r><w:t>stray too</w:t></w:r></w:ins><w:p/></w:tc></w:tr></w:tbl>
    <w:p/>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    let stray: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("a run outside any paragraph"))
        .collect();
    assert_eq!(stray.len(), 2, "{:?}", report.errors);
}

/// KNOWN ISSUE 1 settled: Word requires `w:t` under `w:moveFrom` — `delText` fails open.
#[test]
fn probe_deltext_under_movefrom_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:r><w:delText>moved text</w:delText></w:r></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("w:delText under w:moveFrom")),
        "expected delText-under-moveFrom error, got: {:?}",
        report.errors
    );
}

/// Word refuses `delText` in an insertion inside a move source too (probe M1,
/// 2026-10-18, opened alone): the insertion does not make it deleted text.
#[test]
fn probe_deltext_in_an_insertion_under_movefrom_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:ins w:id="2" w:author="B"><w:r><w:delText>moved text</w:delText></w:r></w:ins></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("w:delText under w:moveFrom")),
        "expected delText-under-moveFrom error, got: {:?}",
        report.errors
    );
}

/// The nearest deletion owns the text: Word opens `delText` in a deletion
/// inside a move source (probe M5, 2026-10-18, opened alone).
#[test]
fn probe_deltext_in_a_deletion_under_movefrom_passes_ring1() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:ins w:id="2" w:author="B"><w:del w:id="3" w:author="C"><w:r><w:delText>moved text</w:delText></w:r></w:del></w:ins></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        !report
            .errors
            .iter()
            .any(|e| e.contains("moveFrom") || e.contains("w:del")),
        "delText in a deletion under moveFrom must be accepted: {:?}",
        report.errors
    );
}

#[test]
fn probe_wt_under_movefrom_passes_ring1() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:r><w:t>moved text</w:t></w:r></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        !report.errors.iter().any(|e| e.contains("moveFrom")),
        "w:t under moveFrom must be accepted: {:?}",
        report.errors
    );
}

#[test]
fn probe_orphan_comment_ref_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:commentReference w:id="42"/></w:r>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("commentReference") && e.contains("42")),
        "expected orphan commentReference error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_dangling_comment_parent_is_a_lead() {
    let comments_extended = r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="11111111" w15:paraIdParent="22222222"/></w15:commentsEx>"#;
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        COMMENTS,
        comments_extended,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("paraIdParent") && error.contains("22222222")),
        "expected dangling paraIdParent error, got: {:?}",
        report.errors
    );
    // Word opens a reply to a missing parent and shows it (h10, 2026-10-10).
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "COMMENT_METADATA_MISMATCH" && !f.word_fatal),
        "{:?}",
        report.findings
    );
}

#[test]
fn healthy_comment_graph_passes_ring1() {
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report.ok(),
        "healthy comment graph should pass: {:?}",
        report.errors
    );
}

#[test]
fn probe_dangling_comment_durable_id_is_a_lead() {
    let comments_extensible = COMMENTS_EXTENSIBLE.replace("10000001", "20000002");
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        &comments_extensible,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("commentsExtensible durableId") && error.contains("20000002")
        }),
        "expected dangling extensible durableId error, got: {:?}",
        report.errors
    );
    // Word opens durableIds that disagree across parts (h03, h05, 2026-10-10).
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "COMMENT_METADATA_MISMATCH" && !f.word_fatal),
        "{:?}",
        report.findings
    );
}

#[test]
fn probe_unanchored_comment_definition_fails() {
    let comments = COMMENTS.replace(
        "</w:comments>",
        r#"<w:comment w:id="1" w:author="B"><w:p w14:paraId="22222222"><w:r><w:t>orphan</w:t></w:r></w:p></w:comment></w:comments>"#,
    );
    let comments_extended = COMMENTS_EXTENDED.replace(
        "</w15:commentsEx>",
        r#"<w15:commentEx w15:paraId="22222222"/></w15:commentsEx>"#,
    );
    let comments_ids = COMMENTS_IDS.replace(
        "</w16cid:commentsIds>",
        r#"<w16cid:commentId w16cid:paraId="22222222" w16cid:durableId="10000002"/></w16cid:commentsIds>"#,
    );
    let comments_extensible = COMMENTS_EXTENSIBLE.replace(
        "</w16cex:commentsExtensible>",
        r#"<w16cex:commentExtensible w16cex:durableId="10000002"/></w16cex:commentsExtensible>"#,
    );
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        &comments,
        &comments_extended,
        &comments_ids,
        &comments_extensible,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("comment definition id '1'") && error.contains("no commentReference")
        }),
        "expected unanchored comment definition error, got: {:?}",
        report.errors
    );
    // Word opens a comment with no reference as it is and does not show it
    // (2026-10-10, d33): the finding is a lead, which the test gate still
    // refuses.
    let unanchored = report
        .findings
        .iter()
        .find(|f| f.code == "COMMENT_UNANCHORED")
        .expect("COMMENT_UNANCHORED");
    assert!(!unanchored.word_fatal, "{unanchored:?}");
}

#[test]
fn probe_unresolved_comment_qname_prefix_fails() {
    let comments = COMMENTS.replacen(
        "<w:comments ",
        r#"<w:comments xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="w16" "#,
        1,
    );
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        &comments,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("unresolved namespace prefix 'w16'")
                && error.contains("word/comments.xml")
        }),
        "expected unresolved QName prefix error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_duplicate_comment_para_id_is_a_lead() {
    let document = COMMENT_DOCUMENT.replace(
        "<w:sectPr/>",
        r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>y</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p><w:sectPr/>"#,
    );
    let comments = COMMENTS.replace(
        "</w:comments>",
        r#"<w:comment w:id="1" w:author="B"><w:p w14:paraId="11111111"><w:r><w:t>second</w:t></w:r></w:p></w:comment></w:comments>"#,
    );
    let bytes = package_with_comment_graph(
        &document,
        &comments,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("duplicate comment paraId") && error.contains("11111111")
        }),
        "expected duplicate comment paraId error, got: {:?}",
        report.errors
    );
    // Word opens a paraId repeated in word/comments.xml (j09, 2026-10-10).
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "COMMENT_METADATA_MISMATCH" && !f.word_fatal),
        "{:?}",
        report.findings
    );
}

#[test]
fn probe_comment_parent_cycle_is_a_lead() {
    let document = COMMENT_DOCUMENT.replace(
        "<w:sectPr/>",
        r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>y</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p><w:sectPr/>"#,
    );
    let comments = COMMENTS.replace(
        "</w:comments>",
        r#"<w:comment w:id="1" w:author="B"><w:p w14:paraId="22222222"><w:r><w:t>second</w:t></w:r></w:p></w:comment></w:comments>"#,
    );
    let comments_extended = r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="11111111" w15:paraIdParent="22222222"/><w15:commentEx w15:paraId="22222222" w15:paraIdParent="11111111"/></w15:commentsEx>"#;
    let comments_ids = COMMENTS_IDS.replace(
        "</w16cid:commentsIds>",
        r#"<w16cid:commentId w16cid:paraId="22222222" w16cid:durableId="10000002"/></w16cid:commentsIds>"#,
    );
    let comments_extensible = COMMENTS_EXTENSIBLE.replace(
        "</w16cex:commentsExtensible>",
        r#"<w16cex:commentExtensible w16cex:durableId="10000002"/></w16cex:commentsExtensible>"#,
    );
    let bytes = package_with_comment_graph(
        &document,
        &comments,
        comments_extended,
        &comments_ids,
        &comments_extensible,
    );
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("paraIdParent cycle")),
        "expected comment parent cycle error, got: {:?}",
        report.errors
    );
    // Word opens two comments that are each other's parent and shows both (j12, 2026-10-10).
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "COMMENT_PARENT_CYCLE" && !f.word_fatal),
        "{:?}",
        report.findings
    );
}

#[test]
fn probe_wrong_comment_relationship_target_fails() {
    let bytes = package_with_comment_graph(
        COMMENT_DOCUMENT,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let mut pkg = PartFs::open(&bytes).expect("open package");
    let rel_type = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
    pkg.remove_relationships_by_type("word/document.xml", rel_type);
    pkg.add_document_relationship("word/document.xml", rel_type, "wrong-comments.xml");
    let broken = pkg.to_zip().expect("serialize package");
    let report = check_word_valid_package(&broken);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("comment relationship") && error.contains("wrong-comments.xml")
        }),
        "expected wrong comment relationship target error, got: {:?}",
        report.errors
    );
    // Word refuses a comment-family relationship to the wrong part (j14, j15, 2026-10-10).
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "COMMENT_PARTS_INCONSISTENT" && f.word_fatal),
        "{:?}",
        report.findings
    );
}

#[test]
fn probe_malformed_xml_part_fails() {
    // Clearly illegal: bare ampersand + mismatched end tag.
    let bytes = zip_with_parts(&[(
        "word/document.xml",
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>& not entity</w:wrong></w:document>"#,
    )]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("well-formed") || e.contains("not a readable")),
        "expected XML parse error, got: {:?}",
        report.errors
    );
}

/// Word refuses a package whose numbering level names a picture bullet the
/// numbering part never defines ("document loaded empty"), repair offer or not.
#[test]
fn probe_undefined_picture_bullet_fails() {
    let numbering = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/><w:lvlPicBulletId w:val="0"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
</w:numbering>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("word/numbering.xml", numbering),
    ]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|e| e.contains("lvlPicBulletId")),
        "expected undefined picture bullet error, got: {:?}",
        report.errors
    );
}

/// `w:name` may not hold a comma (`ST_String` with pattern `[^,]*`;
/// Word reads a comma there as the start of an alias); `w:styleId` may.
#[test]
fn probe_style_name_with_a_comma_fails() {
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="List,Para"><w:name w:val="List,Para"/></w:style>
  <w:style w:type="paragraph" w:styleId="Fine,Id"><w:name w:val="Fine Id"/></w:style>
</w:styles>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("word/styles.xml", styles),
    ]);
    let report = check_word_valid_package(&bytes);
    let named: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "STYLE_NAME_COMMA")
        .collect();
    assert_eq!(named.len(), 1, "{:?}", report.findings);
    assert!(named[0].message.contains("List,Para"), "{named:?}");
}

/// `w:pPr` takes no `w:pPr` of its own: `CT_PPr`'s children are
/// `CT_PPrBase`'s, then `w:rPr`, `w:sectPr` and `w:pPrChange`
/// (`Sch_InvalidElementContentExpectingComplex` from `tools/validate-docx`).
/// The previous properties of a `w:pPrChange` sit under the change, and are
/// Word's own shape.
#[test]
fn probe_ppr_inside_a_ppr_fails() {
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/>
    <w:pPr><w:keepNext/><w:pPr><w:pPrChange w:id="9" w:author="Ana"><w:pPr/></w:pPrChange></w:pPr>
    <w:rPr><w:i/></w:rPr></w:pPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Fine"><w:name w:val="Fine"/>
    <w:pPr><w:keepNext/><w:pPrChange w:id="10" w:author="Ana"><w:pPr><w:keepNext/></w:pPr></w:pPrChange></w:pPr>
  </w:style>
</w:styles>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("word/styles.xml", styles),
    ]);
    let nested: Vec<_> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "NESTED_PPR")
        .collect();
    assert_eq!(nested.len(), 1, "{nested:?}");
    let finding = &nested[0];
    assert_eq!(finding.part, "word/styles.xml");
    assert_eq!(finding.r#ref, finding.location);
    // OOXML-invalid, but no Word probe is recorded: a lead, not Word-fatal.
    assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
}

#[test]
fn probe_paraid_overflow_is_a_repairable_lead() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
  <w:body>
    <w:p w14:paraId="80000000"><w:r><w:t>x</w:t></w:r></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("0x80000000") || e.contains("paraId")),
        "expected paraId overflow error, got: {:?}",
        report.errors
    );
    // Word opens paraIds of 0x80000000 and zero in the body and in comments (i01, i02, h07, 2026-10-10); the schema forbids them, so repair renumbers them.
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "PARA_ID_OUT_OF_RANGE" && !f.word_fatal),
        "{:?}",
        report.findings
    );
}

/// The outer `w:del` on a text box's anchor run does not reach into the text
/// box's own story (en 30ff840c/bb113e88: Word refused the file).
#[test]
fn probe_del_instr_text_outside_its_story_deletion_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:v="urn:schemas-microsoft-com:vml">
  <w:body>
    <w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:pict><v:shape><v:textbox><w:txbxContent>
      <w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText xml:space="preserve"> FILENAME </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:del w:id="2" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:del><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
    </w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:del></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert_eq!(
        report
            .errors
            .iter()
            .filter(|e| e.contains("w:delInstrText") && e.contains("own story"))
            .count(),
        1,
        "expected exactly the orphan delInstrText, got: {:?}",
        report.errors
    );
}

/// A bookmark inside a single-value content control (en 7b649361's title
/// control, en 57c181da's dropdown cell: Word refused both files). A rich-text
/// control may hold one.
#[test]
fn probe_bookmark_in_a_plain_text_content_control_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:sdt><w:sdtPr><w:alias w:val="Title"/><w:text/></w:sdtPr><w:sdtContent>
      <w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/><w:r><w:t>Title</w:t></w:r><w:bookmarkEnd w:id="0"/></w:p>
    </w:sdtContent></w:sdt>
    <w:sdt><w:sdtPr><w:id w:val="8"/></w:sdtPr><w:sdtContent>
      <w:p><w:bookmarkStart w:id="1" w:name="_Ref1"/><w:r><w:t>Scope</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p>
    </w:sdtContent></w:sdt>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    let hits: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("content control"))
        .collect();
    assert_eq!(
        hits.len(),
        2,
        "expected _Toc1's start and end in the text control, got: {:?}",
        report.errors
    );
    assert!(hits.iter().all(|e| e.contains("a text content control")));
}

#[test]
fn minimal_valid_package_passes() {
    let bytes = zip_with_parts(&[("word/document.xml", MINIMAL_DOC)]);
    // May warn about missing content type defaults for nothing else — must pass core checks.
    let report = check_word_valid_package(&bytes);
    assert!(
        report.ok(),
        "minimal package should pass: {:?}",
        report.errors
    );
    // Ensure PartFs can open it too
    PartFs::open(&bytes).expect("open");
}

/// ZIP-LEVEL-01: `to_zip` (deflate level 1) produces a package whose
/// decompressed members are byte-identical to the original, and the
/// re-zipped package passes Ring-1 Word-validity.
#[test]
fn zip_level_01_roundtrip_member_identity() {
    let pkg = PartFs::open(ORIG).expect("open original");
    let original_parts: Vec<(String, Vec<u8>)> = pkg
        .parts()
        .into_iter()
        .map(|name| {
            let data = pkg.part_bytes(&name).expect("part bytes").to_vec();
            (name, data)
        })
        .collect();
    let zip_bytes = pkg.to_zip().expect("to_zip");
    let pkg2 = PartFs::open(&zip_bytes).expect("open re-zipped");
    for (name, original_data) in &original_parts {
        assert_eq!(
            pkg2.part_bytes(name).expect("roundtrip part exists"),
            original_data.as_slice(),
            "part '{name}' differs after to_zip round-trip"
        );
    }
    assert_word_valid_package(&zip_bytes);
}

#[test]
fn probe_unresolved_choice_prefix_in_a_header_fails() {
    // The shape of a B header grafted into A's header part: `wps` is bound
    // only on the shape inside the Choice, so the Choice names it out of scope.
    let header = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
  <w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing>
    <wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"/>
  </w:drawing></mc:Choice><mc:Fallback/></mc:AlternateContent></w:r></w:p>
</w:hdr>"#;
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p/></w:body></w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc), ("word/header1.xml", header)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|error| {
            error.contains("unresolved namespace prefix 'wps'")
                && error.contains("word/header1.xml")
        }),
        "expected unresolved Requires prefix error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_application_requires_in_custom_xml_passes() {
    // `Requires` is an MC prefix list only on `mc:Choice`; a custom XML
    // part's own `Requires` attribute is application data.
    let item = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<plugins xmlns="urn:example:plugins"><dependency Requires="plugin-a plugin-b"/></plugins>"#;
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p/></w:body></w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc), ("customXml/item1.xml", item)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        !report.errors.iter().any(|error| error.contains("plugin-")),
        "application Requires read as an MC prefix list: {:?}",
        report.errors
    );
}

// Comment anchors, as Word reads them: a 61-file Word probe (2026-10-10,
// jubarte-loop/fixplan/n4/cprobe and cprobe2). Each case names its probe.

const S0: &str = r#"<w:commentRangeStart w:id="0"/>"#;
const E0: &str = r#"<w:commentRangeEnd w:id="0"/>"#;
const R0: &str = r#"<w:r><w:commentReference w:id="0"/></w:r>"#;
const KEPT: &str = "<w:r><w:t>Kept.</w:t></w:r>";

/// The one-comment package with `body` before the section properties.
fn comment_report(body: &str) -> common::validity::ValidityReport {
    let document = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    check_word_valid_package(&package_with_comment_graph(
        &document,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    ))
}

fn codes(report: &common::validity::ValidityReport) -> Vec<(&str, bool)> {
    report
        .findings
        .iter()
        .map(|f| (f.code.as_str(), f.word_fatal))
        .collect()
}

/// Word opens these and shows the comment: a reference in a paragraph is
/// all it needs, whatever the range markers do or whatever wraps it.
#[test]
fn probe_comment_shown_from_its_reference_in_a_paragraph_passes() {
    let ins = |x: &str| format!(r#"<w:ins w:id="90" w:author="B">{x}</w:ins>"#);
    let del = |x: &str| format!(r#"<w:del w:id="91" w:author="B">{x}</w:del>"#);
    for (probe, body) in [
        ("c02 reference only", format!("<w:p>{KEPT}{R0}</w:p>")),
        (
            "c05 end and reference",
            format!("<w:p>{KEPT}{E0}{R0}</w:p>"),
        ),
        (
            "c06 markers at body level",
            format!("{S0}<w:p>{KEPT}</w:p>{E0}<w:p>{R0}</w:p>"),
        ),
        (
            "c08 reference in an empty paragraph",
            format!("<w:p>{R0}</w:p>"),
        ),
        (
            "d01 end before start",
            format!("<w:p>{E0}{KEPT}{S0}{R0}</w:p>"),
        ),
        (
            "d02 reference before the range",
            format!("<w:p>{R0}{S0}{KEPT}{E0}</w:p>"),
        ),
        (
            "d05 range over three paragraphs",
            format!("<w:p>{S0}{KEPT}</w:p><w:p>{R0}</w:p><w:p>{KEPT}{E0}</w:p>"),
        ),
        (
            "d10 markers in a table and row",
            format!(
                r#"<w:tbl><w:tblGrid><w:gridCol w:w="3000"/></w:tblGrid>{S0}<w:tr>{E0}<w:tc><w:p>{KEPT}{R0}</w:p></w:tc></w:tr></w:tbl><w:p/>"#
            ),
        ),
        (
            "d12 reference in a deletion",
            format!(
                "<w:p>{S0}{}{E0}{}</w:p>",
                del(r#"<w:r><w:delText>Kept.</w:delText></w:r>"#),
                del(R0).replace("91", "92")
            ),
        ),
        (
            "d13 reference in an insertion",
            format!("<w:p>{S0}{}{E0}</w:p>", ins(&format!("{KEPT}{R0}"))),
        ),
        (
            "d15 reference in a hyperlink",
            format!(r#"<w:p>{S0}<w:hyperlink w:anchor="t">{KEPT}{E0}{R0}</w:hyperlink></w:p>"#),
        ),
        (
            "d16 reference in a content control",
            format!(r#"<w:p>{S0}<w:sdt><w:sdtContent>{KEPT}{E0}{R0}</w:sdtContent></w:sdt></w:p>"#),
        ),
        (
            "d24 reference without its run",
            format!(r#"<w:p>{S0}{KEPT}{E0}<w:commentReference w:id="0"/></w:p>"#),
        ),
        (
            "d35 reference inside its own range",
            format!("<w:p>{S0}{KEPT}{R0}{KEPT}{E0}</w:p>"),
        ),
        (
            "d38 reference in custom XML",
            format!(r#"<w:p>{S0}<w:customXml w:element="x">{KEPT}{E0}{R0}</w:customXml></w:p>"#),
        ),
    ] {
        let report = comment_report(&body);
        assert!(report.findings.is_empty(), "{probe}: {:?}", report.findings);
    }
}

/// No reference in a paragraph: Word opens the file and shows no comment.
/// A lead (content loss), not Word-invalidity.
#[test]
fn probe_comment_without_reference_is_a_lead() {
    for (probe, body) in [
        ("c00 no markers", format!("<w:p>{KEPT}</w:p>")),
        (
            "c03 range, no reference",
            format!("<w:p>{S0}{KEPT}{E0}</w:p>"),
        ),
        ("c04 start only", format!("<w:p>{S0}{KEPT}</w:p>")),
        (
            "c07 markers at body level",
            format!("{S0}{E0}<w:p>{KEPT}</w:p>"),
        ),
    ] {
        let report = comment_report(&body);
        assert_eq!(codes(&report), [("COMMENT_UNANCHORED", false)], "{probe}");
        assert!(report.errors[0].contains("shows no comment"), "{probe}");
    }
}

/// Repeated range markers open (d29, g01), and so do two definitions under
/// one id, Word showing the first (d41): leads. Two references for one
/// comment hang Word (d28, twice).
#[test]
fn probe_repeated_comment_ids() {
    for (probe, body) in [
        (
            "d29 two starts",
            format!("<w:p>{S0}{KEPT}{S0}{KEPT}{E0}{R0}</w:p>"),
        ),
        (
            "g01 two ends",
            format!("<w:p>{S0}{KEPT}{E0}{KEPT}{E0}{R0}</w:p>"),
        ),
    ] {
        let report = comment_report(&body);
        assert_eq!(codes(&report), [("COMMENT_ID_REPEATED", false)], "{probe}");
    }
    let report = comment_report(&format!("<w:p>{S0}{KEPT}{E0}{R0}{R0}</w:p>"));
    assert_eq!(
        codes(&report),
        [("COMMENT_PARTS_INCONSISTENT", true)],
        "d28"
    );

    let comments = COMMENTS.replace(
        "</w:comments>",
        r#"<w:comment w:id="0" w:author="A"><w:p><w:r><w:t>again</w:t></w:r></w:p></w:comment></w:comments>"#,
    );
    let document = COMMENT_DOCUMENT;
    let report = check_word_valid_package(&package_with_comment_graph(
        document,
        &comments,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    ));
    assert!(
        codes(&report).contains(&("COMMENT_ID_REPEATED", false)),
        "d41: {:?}",
        report.findings
    );
}

/// An anchor whose id names no comment: Word refuses a reference (d30), but
/// opens range markers beside the comments part (d31, d32). With no comments
/// part, range markers are refused too (f02).
#[test]
fn probe_orphan_comment_anchors() {
    let five = r#"<w:commentRangeStart w:id="5"/><w:r><w:t>More.</w:t></w:r><w:commentRangeEnd w:id="5"/>"#;
    let report = comment_report(&format!("<w:p>{S0}{KEPT}{E0}{R0}{five}</w:p>"));
    assert_eq!(
        codes(&report),
        [
            ("COMMENT_RANGE_ORPHAN", false),
            ("COMMENT_RANGE_ORPHAN", false)
        ],
        "d31"
    );
    assert!(report.findings.iter().all(|f| f.repairable));

    let report = comment_report(&format!(
        r#"<w:p>{S0}{KEPT}{E0}{R0}<w:r><w:commentReference w:id="5"/></w:r></w:p>"#
    ));
    assert_eq!(codes(&report), [("COMMENT_ANCHOR_ORPHAN", true)], "d30");

    let doc = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:commentRangeStart w:id="42"/><w:r><w:t>Kept.</w:t></w:r><w:commentRangeEnd w:id="42"/></w:p><w:sectPr/></w:body></w:document>"#;
    let report = check_word_valid_package(&zip_with_parts(&[("word/document.xml", doc)]));
    assert_eq!(
        codes(&report),
        [
            ("COMMENT_ANCHOR_ORPHAN", true),
            ("COMMENT_ANCHOR_ORPHAN", true)
        ],
        "f02"
    );
}

/// The comments part is the first comments relationship whose target
/// part exists: a stale relationship to a missing part must not hide a
/// later live one (`notes.xml` here), and Word opens the package either
/// way — this is not a Word-invalid shape. The stale relationship itself
/// is still disclosed as a metadata lead, never word-fatal.
#[test]
fn probe_comments_relationships_tried_in_order() {
    let bytes = zip_with_parts(&[("word/document.xml", COMMENT_DOCUMENT)]);
    let mut pkg = PartFs::open(&bytes).expect("open package");
    // Stale first: a relationship that names a part the package lacks.
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        "notes-old.xml",
    );
    pkg.set_part("word/notes.xml", COMMENTS.as_bytes().to_vec());
    pkg.add_content_type_override(
        "/word/notes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
    );
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        "notes.xml",
    );
    let report = check_word_valid_package(&pkg.to_zip().expect("serialize package"));
    // The comment defined in `notes.xml` anchors cleanly: no orphan or
    // unanchored findings, and nothing Word-fatal — only the stale
    // relationship's metadata and missing-target leads remain.
    assert_eq!(
        codes(&report),
        [
            ("MISSING_REL_TARGET", false),
            ("COMMENT_METADATA_MISMATCH", false)
        ],
        "{:?}",
        report.findings
    );
    assert!(report.findings.iter().all(|f| !f.word_fatal));
}

/// Repair drops the range markers that name no comment and keeps the rest.
#[test]
fn repair_drops_orphan_range_markers() {
    let document = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p>{S0}{KEPT}{E0}{R0}<w:commentRangeStart w:id="5"/><w:commentRangeEnd w:id="5"/></w:p><w:sectPr/></w:body></w:document>"#
    );
    let bytes = package_with_comment_graph(
        &document,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    );
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(repaired.repaired.len(), 2);
    assert!(repaired.remaining.is_empty(), "{:?}", repaired.remaining);
    assert_word_valid_package(&repaired.docx);
}

/// A comment's reference run outside a paragraph, under each wrapper Word
/// refused (2026-10-10): body-level `w:del` (d39) and `w:moveTo` (g02),
/// block `w:sdtContent` (g03) and block `w:customXml` (g04).
#[test]
fn probe_run_under_a_block_wrapper_fails() {
    for (probe, body) in [
        (
            "d39",
            format!(r#"<w:p>{S0}{KEPT}{E0}</w:p><w:del w:id="91" w:author="B">{R0}</w:del>"#),
        ),
        (
            "g02",
            format!(r#"<w:p>{S0}{KEPT}{E0}</w:p><w:moveTo w:id="96" w:author="B">{R0}</w:moveTo>"#),
        ),
        (
            "g03",
            format!(
                "<w:p>{S0}{KEPT}{E0}</w:p><w:sdt><w:sdtContent>{R0}</w:sdtContent></w:sdt><w:p/>"
            ),
        ),
        (
            "g04",
            format!(
                r#"<w:p>{S0}{KEPT}{E0}</w:p><w:customXml w:element="x">{R0}</w:customXml><w:p/>"#
            ),
        ),
    ] {
        let report = comment_report(&body);
        assert_eq!(codes(&report), [("RUN_OUTSIDE_PARAGRAPH", true)], "{probe}");
    }
}

#[test]
fn office_trash_folder_needs_no_content_type() {
    // Office's legacy `[trash]/` folder holds no part: a document of the
    // 0.12.0 sample (52220b20c3) carries `[trash]/0000.dat` with no content
    // type, and Word 16.115 opened it and its redline (2026-10-10).
    let report = check_word_valid_package(&zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("[trash]/0000.dat", "junk"),
    ]));
    assert!(
        !codes(&report)
            .iter()
            .any(|(code, _)| *code == "MISSING_CONTENT_TYPE"),
        "{:?}",
        report.findings
    );
    let report = check_word_valid_package(&zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("word/media/0000.dat", "junk"),
    ]));
    assert!(
        codes(&report).contains(&("MISSING_CONTENT_TYPE", true)),
        "{:?}",
        report.findings
    );
}

/// A run straight under a part's root element, with no story container on
/// the way: Word refused each alone (2026-10-10) under `w:document` past
/// the body (k03), `w:comments` (k02) and `w:footnotes` (k01).
#[test]
fn probe_run_under_a_part_root_fails() {
    const STRAY: &str = "<w:r><w:t>stray</w:t></w:r>";
    let document = COMMENT_DOCUMENT.replace("</w:body>", &format!("</w:body>{STRAY}"));
    let comments = COMMENTS.replace("</w:comments>", &format!("{STRAY}</w:comments>"));
    for (probe, document, comments) in [
        ("k03", document.as_str(), COMMENTS),
        ("k02", COMMENT_DOCUMENT, comments.as_str()),
    ] {
        let report = check_word_valid_package(&package_with_comment_graph(
            document,
            comments,
            COMMENTS_EXTENDED,
            COMMENTS_IDS,
            COMMENTS_EXTENSIBLE,
        ));
        assert_eq!(codes(&report), [("RUN_OUTSIDE_PARAGRAPH", true)], "{probe}");
    }
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:id="1"><w:p>{KEPT}</w:p></w:footnote>{STRAY}</w:footnotes>"#
    );
    let report = check_word_valid_package(&zip_with_parts(&[
        ("word/document.xml", MINIMAL_DOC),
        ("word/footnotes.xml", &footnotes),
    ]));
    assert!(
        codes(&report).contains(&("RUN_OUTSIDE_PARAGRAPH", true)),
        "k01: {:?}",
        report.findings
    );
}

/// A text box saved twice, as DrawingML in `mc:Choice` and VML in
/// `mc:Fallback`, carries the comment's markers in both copies. Word reads
/// one copy and opens the file (k05, 2026-10-10): not a second reference.
#[test]
fn probe_comment_in_both_copies_of_a_dual_saved_text_box_passes() {
    let marked = format!("<w:txbxContent><w:p>{S0}{KEPT}{E0}{R0}</w:p></w:txbxContent>");
    let document = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml" mc:Ignorable="wps"><w:body><w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:inline><wp:extent cx="914400" cy="457200"/><wp:docPr id="1" name="Box"/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:txbx>{marked}</wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape style="width:72pt;height:36pt"><v:textbox>{marked}</v:textbox></v:shape></w:pict></mc:Fallback></mc:AlternateContent></w:r></w:p><w:sectPr/></w:body></w:document>"#
    );
    let report = check_word_valid_package(&package_with_comment_graph(
        &document,
        COMMENTS,
        COMMENTS_EXTENDED,
        COMMENTS_IDS,
        COMMENTS_EXTENSIBLE,
    ));
    assert_eq!(codes(&report), [], "k05");
}

/// The comment parts beside `word/comments.xml` that disagree with it, as
/// Word met them in the 2026-10-10 probe: it opens these and shows the
/// comment, so each is a lead (the test gate still refuses leads).
#[test]
fn probe_comment_metadata_word_opens_is_a_lead() {
    const EX: &str =
        r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml">"#;
    const IDS: &str = r#"<w16cid:commentsIds xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid">"#;
    const CEX: &str = r#"<w16cex:commentsExtensible xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex">"#;
    let ex_end = "</w15:commentsEx>";
    let ids_end = "</w16cid:commentsIds>";
    let cex_end = "</w16cex:commentsExtensible>";
    let overflow = |x: &str| x.replace("11111111", "80000000");
    for (probe, comments, extended, ids, extensible) in [
        (
            "h01 comment paraId in neither side part",
            COMMENTS.replace("11111111", "22222222"),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "h02 commentsExtended entry missing",
            COMMENTS.to_string(),
            format!("{EX}{ex_end}"),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "h03 commentsIds entry missing",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            format!("{IDS}{ids_end}"),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "h04 extra commentsExtended entry",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.replace(ex_end, &format!(r#"<w15:commentEx w15:paraId="10000005"/>{ex_end}"#)),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "h05 commentsExtensible entry missing",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.to_string(),
            format!("{CEX}{cex_end}"),
        ),
        (
            "h07 comment paraId at 0x80000000",
            overflow(COMMENTS),
            overflow(COMMENTS_EXTENDED),
            overflow(COMMENTS_IDS),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "h09 commentsExtended entry repeated",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.replace(ex_end, &format!(r#"<w15:commentEx w15:paraId="11111111"/>{ex_end}"#)),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "j02 commentsIds entry without durableId",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.replace(r#" w16cid:durableId="10000001""#, ""),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "j03 commentsExtensible entry without durableId",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.replace(r#" w16cex:durableId="10000001""#, ""),
        ),
        (
            "j08 comment paragraph without paraId",
            COMMENTS.replace(r#" w14:paraId="11111111""#, ""),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "j10 commentsIds durableId repeated",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.replace(ids_end, &format!(r#"<w16cid:commentId w16cid:paraId="10000004" w16cid:durableId="10000001"/>{ids_end}"#)),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "j16 commentsIds paraId repeated",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.replace(ids_end, &format!(r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000009"/>{ids_end}"#)),
            COMMENTS_EXTENSIBLE.to_string(),
        ),
        (
            "j17 commentsExtensible durableId repeated",
            COMMENTS.to_string(),
            COMMENTS_EXTENDED.to_string(),
            COMMENTS_IDS.to_string(),
            COMMENTS_EXTENSIBLE.replace(cex_end, &format!(r#"<w16cex:commentExtensible w16cex:durableId="10000001"/>{cex_end}"#)),
        ),
    ] {
        let report = check_word_valid_package(&package_with_comment_graph(
            COMMENT_DOCUMENT,
            &comments,
            &extended,
            &ids,
            &extensible,
        ));
        assert!(
            report.findings.iter().any(|f| f.code == "COMMENT_METADATA_MISMATCH"),
            "{probe}: {:?}",
            report.findings
        );
        assert!(
            report.findings.iter().all(|f| !f.word_fatal),
            "{probe}: {:?}",
            report.findings
        );
    }
}

/// What Word refused in the same probe, each confirmed alone: an id that is
/// not hexadecimal (hung), an entry without its paraId, and a side part
/// that is not wired exactly once to the right part ("document loaded
/// empty").
#[test]
fn probe_comment_metadata_word_refuses_is_fatal() {
    const EXTENDED_TYPE: &str =
        "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
    let not_hex = |x: &str| x.replace("11111111", "XYZ");
    let graph = |comments: &str, extended: &str, ids: &str| {
        package_with_comment_graph(
            COMMENT_DOCUMENT,
            comments,
            extended,
            ids,
            COMMENTS_EXTENSIBLE,
        )
    };
    let wired = |change: &dyn Fn(&mut PartFs)| {
        let mut pkg =
            PartFs::open(&graph(COMMENTS, COMMENTS_EXTENDED, COMMENTS_IDS)).expect("open package");
        change(&mut pkg);
        pkg.to_zip().expect("serialize package")
    };
    for (probe, bytes) in [
        (
            "h08 paraId not hexadecimal",
            graph(
                &not_hex(COMMENTS),
                &not_hex(COMMENTS_EXTENDED),
                &not_hex(COMMENTS_IDS),
            ),
        ),
        (
            "h12 commentsExtended entry without paraId",
            graph(
                COMMENTS,
                &COMMENTS_EXTENDED.replace(r#" w15:paraId="11111111""#, ""),
                COMMENTS_IDS,
            ),
        ),
        (
            "j01 commentsIds entry without paraId",
            graph(
                COMMENTS,
                COMMENTS_EXTENDED,
                &COMMENTS_IDS.replace(r#" w16cid:paraId="11111111""#, ""),
            ),
        ),
        (
            "j05 two relationships to commentsExtended",
            wired(&|pkg| {
                pkg.add_document_relationship(
                    "word/document.xml",
                    EXTENDED_TYPE,
                    "commentsExtended.xml",
                );
            }),
        ),
        (
            "j18 commentsExtended without a relationship",
            wired(&|pkg| {
                pkg.remove_relationships_by_type("word/document.xml", EXTENDED_TYPE);
            }),
        ),
    ] {
        let report = check_word_valid_package(&bytes);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == "COMMENT_PARTS_INCONSISTENT" && f.word_fatal),
            "{probe}: {:?}",
            report.findings
        );
    }
}

// Word verdicts: #1134, 2026-10-17. Live text is refused even through w:ins.
fn nested_probe(kinds: &[&str], element: &str) -> Vec<u8> {
    let mut xml = format!("<w:r><w:{element}>x</w:{element}></w:r>");
    if element == "instrText" {
        xml = format!(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>{xml}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#
        );
    }
    for (id, kind) in kinds.iter().enumerate().rev() {
        xml = format!(r#"<w:{kind} w:id="{id}" w:author="Alice">{xml}</w:{kind}>"#);
    }
    common::docx::docx(&format!("<w:p>{xml}</w:p>"))
}

fn refused_nested_text(kinds: &[&str], element: &str, code: &str) {
    let bytes = nested_probe(kinds, element);
    let report = jubarte::validate::validate(&bytes).unwrap();
    let matches: Vec<_> = report.iter().filter(|f| f.code == code).collect();
    assert_eq!(matches.len(), 1, "{report:?}");
    assert!(matches[0].repairable);
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert!(repaired.repaired.iter().any(|f| f.code == code));
    assert!(!repaired.remaining.iter().any(|f| f.code == code));
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    let deleted = if element == "t" {
        "delText"
    } else {
        "delInstrText"
    };
    assert!(
        xml.contains(&format!("<w:{deleted}>x</w:{deleted}>")),
        "{xml}"
    );
    assert_word_valid_package(&repaired.docx);
}

#[test]
fn issue_1134_del_ins_text_is_refused_and_repaired() {
    refused_nested_text(&["del", "ins"], "t", "TEXT_INSIDE_DELETION");
}
#[test]
fn issue_1134_ins_del_ins_text_is_refused_and_repaired() {
    refused_nested_text(&["ins", "del", "ins"], "t", "TEXT_INSIDE_DELETION");
}
#[test]
fn issue_1134_del_ins_field_is_repaired() {
    refused_nested_text(&["del", "ins"], "instrText", "INSTR_TEXT_INSIDE_DELETION");
}
#[test]
fn issue_1134_ins_del_ins_field_is_repaired() {
    refused_nested_text(
        &["ins", "del", "ins"],
        "instrText",
        "INSTR_TEXT_INSIDE_DELETION",
    );
}
#[test]
fn issue_1134_opened_deleted_text_is_healthy() {
    assert_word_valid_package(&nested_probe(&["del", "ins"], "delText"));
}

fn nesting_severity(kinds: &[&str], element: &str, severity: &str) {
    let report = jubarte::validate::validate(&nested_probe(kinds, element)).unwrap();
    let finding = report
        .iter()
        .find(|f| f.code == "NESTED_SAME_REVISION")
        .unwrap();
    assert_eq!(finding.severity, severity, "{finding:?}");
    assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
    assert!(finding.message.contains("Word opens"), "{finding:?}");
    if severity == "warning" {
        assert!(
            finding.message.contains("misreads reject-all"),
            "{finding:?}"
        );
    }
}
#[test]
fn issue_1134_direct_ins_is_info() {
    nesting_severity(&["ins", "ins"], "t", "info");
}
#[test]
fn issue_1134_direct_del_is_info() {
    nesting_severity(&["del", "del"], "delText", "info");
}
#[test]
fn issue_1134_ins_del_ins_is_warning() {
    nesting_severity(&["ins", "del", "ins"], "delText", "warning");
}
#[test]
fn issue_1134_del_ins_del_is_warning() {
    nesting_severity(&["del", "ins", "del"], "delText", "warning");
}

#[test]
fn issue_1134_textbox_is_a_separate_story() {
    let bytes = common::docx::docx(
        r#"<w:p><w:del w:id="1" w:author="Alice"><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>x</w:t></w:r></w:p></w:txbxContent></w:pict></w:r></w:del></w:p>"#,
    );
    let report = jubarte::validate::validate(&bytes).unwrap();
    assert!(
        !report.iter().any(|f| f.code == "TEXT_INSIDE_DELETION"),
        "{report:?}"
    );
}

// #1137: OOXML refuses duplicate revision ids; Word has not been probed.
fn issue_1137_duplicate_revision(element: &str) {
    let content = match element {
        "ins" | "moveFrom" | "moveTo" => "<w:r><w:t>a</w:t></w:r>".to_string(),
        "del" => "<w:r><w:delText>a</w:delText></w:r>".to_string(),
        _ if element.ends_with("Change") && element != "numberingChange" => {
            format!("<w:{}/>", element.strip_suffix("Change").unwrap())
        }
        _ => String::new(),
    };
    let extra = if element == "numberingChange" {
        " w:original=\"1\""
    } else {
        ""
    };
    let mark = format!(r#"<w:{element} w:id="1" w:author="Alice"{extra}>{content}</w:{element}>"#);
    let cell = "<w:tc><w:p/></w:tc>";
    let table = |properties: &str, grid: &str, row: &str| {
        format!(
            "<w:tbl><w:tblPr>{properties}</w:tblPr><w:tblGrid><w:gridCol w:w=\"1000\"/>{grid}</w:tblGrid><w:tr>{row}</w:tr></w:tbl>"
        )
    };
    let marked = match element {
        "rPrChange" => format!("<w:p><w:r><w:rPr>{mark}</w:rPr><w:t>a</w:t></w:r></w:p>"),
        "pPrChange" => format!("<w:p><w:pPr>{mark}</w:pPr><w:r><w:t>a</w:t></w:r></w:p>"),
        "sectPrChange" => format!("<w:p><w:pPr><w:sectPr>{mark}</w:sectPr></w:pPr></w:p>"),
        "numberingChange" => format!("<w:p><w:pPr><w:numPr>{mark}</w:numPr></w:pPr></w:p>"),
        "tblPrChange" => table(&mark, "", cell),
        "tblGridChange" => table("", &mark, cell),
        "tblPrExChange" => table("", "", &format!("<w:tblPrEx>{mark}</w:tblPrEx>{cell}")),
        "trPrChange" => table("", "", &format!("<w:trPr>{mark}</w:trPr>{cell}")),
        "tcPrChange" | "cellIns" | "cellDel" | "cellMerge" => table(
            "",
            "",
            &format!("<w:tc><w:tcPr>{mark}</w:tcPr><w:p/></w:tc>"),
        ),
        "moveFrom" | "moveTo" => format!(
            "<w:p><w:{element}RangeStart w:id=\"10\" w:name=\"probe\"/>{mark}<w:{element}RangeEnd w:id=\"10\"/></w:p>"
        ),
        _ => format!("<w:p>{mark}</w:p>"),
    };
    let second = marked
        .replace("w:id=\"10\"", "w:id=\"11\"")
        .replace("w:name=\"probe\"", "w:name=\"second\"");
    // Pair the moves so duplicate wrapper ids are the only fault (#1214).
    let mut counterparts = String::new();
    if matches!(element, "moveFrom" | "moveTo") {
        let opposite = if element == "moveFrom" {
            "moveTo"
        } else {
            "moveFrom"
        };
        for (name, range, revision) in [("probe", 10, 2), ("second", 11, 3)] {
            counterparts.push_str(&format!(
                r#"<w:p><w:{opposite}RangeStart w:id="{range}" w:name="{name}"/><w:{opposite} w:id="{revision}" w:author="Alice"><w:r><w:t>a</w:t></w:r></w:{opposite}><w:{opposite}RangeEnd w:id="{range}"/></w:p>"#
            ));
        }
    }
    let bytes = common::docx::docx(&format!("{marked}{second}{counterparts}"));
    let package = PartFs::open(&bytes).unwrap();
    let findings = jubarte::validate::ring1(&package);
    let duplicates: Vec<_> = findings
        .iter()
        .filter(|f| f.code == "DUPLICATE_REVISION_ID")
        .collect();
    assert_eq!(duplicates.len(), 1, "{element}: {findings:?}");
    assert_eq!(
        duplicates[0].severity, "error",
        "{element}: {:?}",
        duplicates[0]
    );
    assert!(!duplicates[0].word_fatal, "Word verdict is pending");
    assert!(duplicates[0].repairable);
    assert!(!check_word_valid_package(&bytes).ok());
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert!(
        repaired
            .remaining
            .iter()
            .all(|f| f.code != "DUPLICATE_REVISION_ID")
    );
    assert!(
        check_word_valid_package(&repaired.docx).ok(),
        "{:?}",
        repaired.remaining
    );
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    assert_eq!(xml.matches("w:id=\"1\"").count(), 1, "{xml}");
    let fresh = if element.starts_with("move") { 12 } else { 2 };
    assert_eq!(
        xml.matches(&format!("w:id=\"{fresh}\"")).count(),
        1,
        "{xml}"
    );
    assert_eq!(xml.matches(&format!("<w:{element} ")).count(), 2, "{xml}");
}

macro_rules! issue_1137_revision_probes {
    ($($test:ident: $element:literal),* $(,)?) => {$(
        #[test]
        fn $test() { issue_1137_duplicate_revision($element); }
    )*};
}

issue_1137_revision_probes! {
    issue_1137_duplicate_ins: "ins",
    issue_1137_duplicate_del: "del",
    issue_1137_duplicate_move_from: "moveFrom",
    issue_1137_duplicate_move_to: "moveTo",
    issue_1137_duplicate_run_properties: "rPrChange",
    issue_1137_duplicate_paragraph_properties: "pPrChange",
    issue_1137_duplicate_section_properties: "sectPrChange",
    issue_1137_duplicate_table_properties: "tblPrChange",
    issue_1137_duplicate_table_property_exceptions: "tblPrExChange",
    issue_1137_duplicate_grid: "tblGridChange",
    issue_1137_duplicate_row_properties: "trPrChange",
    issue_1137_duplicate_cell_properties: "tcPrChange",
    issue_1137_duplicate_cell_ins: "cellIns",
    issue_1137_duplicate_cell_del: "cellDel",
    issue_1137_duplicate_cell_merge: "cellMerge",
    issue_1137_duplicate_numbering: "numberingChange",
}

#[test]
fn issue_1137_revision_id_scope_is_one_part() {
    let bytes = zip_with_parts(&[
        (
            "word/document.xml",
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:ins w:id="1" w:author="Alice"><w:r><w:t>a</w:t></w:r></w:ins></w:p><w:sectPr/></w:body></w:document>"#,
        ),
        (
            "word/header1.xml",
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="1" w:author="Alice"><w:r><w:t>b</w:t></w:r></w:ins></w:p></w:hdr>"#,
        ),
    ]);
    assert!(check_word_valid_package(&bytes).ok());
}

// SDK probes accept these cross-kind pairs, including Word's paired style records.
fn issue_1137_accepted_pair(first: &str, second: &str, placement: &str) {
    let mark = |kind: &str| {
        let inner = match kind {
            "del" => "<w:r><w:delText>a</w:delText></w:r>",
            "pPrChange" => "<w:pPr/>",
            "rPrChange" => "<w:rPr/>",
            _ => "<w:r><w:t>a</w:t></w:r>",
        };
        format!("<w:{kind} w:id=\"1\" w:author=\"Alice\">{inner}</w:{kind}>")
    };
    let pair = if first == "pPrChange" {
        format!(
            "<w:pPr>{}</w:pPr><w:rPr>{}</w:rPr>",
            mark(first),
            mark(second)
        )
    } else if second == "rPrChange" {
        format!(
            "{}<w:r><w:rPr>{}</w:rPr><w:t>b</w:t></w:r>",
            mark(first),
            mark(second)
        )
    } else {
        format!("{}{}", mark(first), mark(second))
    };
    let bytes = if placement == "style" {
        common::docx::docx_with(
            "<w:p/>",
            &[common::docx::Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &format!(
                    "<w:styles xmlns:w=\"{}\"><w:style w:type=\"paragraph\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/>{pair}</w:style></w:styles>",
                    common::docx::W_NS
                ),
            }],
        )
    } else {
        let pair = if first == "pPrChange" {
            format!(
                "<w:pPr>{}</w:pPr><w:r><w:rPr>{}</w:rPr><w:t>b</w:t></w:r>",
                mark(first),
                mark(second)
            )
        } else {
            pair
        };
        common::docx::docx(&format!("<w:p>{pair}</w:p>"))
    };
    let findings = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap());
    assert!(
        findings.is_empty(),
        "{first}/{second} in {placement}: {findings:?}"
    );
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert!(repaired.repaired.is_empty(), "{:?}", repaired.repaired);
    assert_eq!(
        repaired.docx, bytes,
        "an accepted pair must stay byte-identical"
    );
}

#[test]
fn issue_1137_cross_kind_ins_del_is_valid() {
    issue_1137_accepted_pair("ins", "del", "body");
}
#[test]
fn issue_1137_cross_kind_moves_are_valid() {
    issue_1137_accepted_pair("moveFrom", "moveTo", "body");
}
#[test]
fn issue_1137_cross_kind_ins_move_is_valid() {
    issue_1137_accepted_pair("ins", "moveFrom", "body");
}
#[test]
fn issue_1137_cross_kind_style_properties_are_valid() {
    issue_1137_accepted_pair("pPrChange", "rPrChange", "style");
}
#[test]
fn issue_1137_cross_kind_body_properties_are_valid() {
    issue_1137_accepted_pair("pPrChange", "rPrChange", "body");
}
#[test]
fn issue_1137_cross_kind_ins_run_properties_are_valid() {
    issue_1137_accepted_pair("ins", "rPrChange", "body");
}

#[test]
fn issue_1137_repair_renumbers_only_the_refused_pair() {
    let bytes = common::docx::docx(
        r#"<w:p><w:ins w:id="1" w:author="Alice"><w:r><w:t>a</w:t></w:r></w:ins><w:del w:id="1" w:author="Bob"><w:r><w:delText>b</w:delText></w:r></w:del><w:ins w:id="1" w:author="Alice"><w:r><w:t>c</w:t></w:r></w:ins></w:p>"#,
    );
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(
        repaired
            .repaired
            .iter()
            .filter(|f| f.code == "DUPLICATE_REVISION_ID")
            .count(),
        1
    );
    let before = common::docx::part_string(&bytes, "word/document.xml").unwrap();
    let expected = before.replace(
        r#"<w:ins w:id="1" w:author="Alice"><w:r><w:t>c"#,
        r#"<w:ins w:id="2" w:author="Alice"><w:r><w:t>c"#,
    );
    let mut dom = jubarte::xmllinq::Dom::new();
    let document = dom.parse_xdocument(&expected);
    assert_eq!(
        common::docx::part_string(&repaired.docx, "word/document.xml").unwrap(),
        dom.serialize_document(document)
    );
    assert_word_valid_package(&repaired.docx);
}

fn issue_1137_context_pair(kind: &str, left: &str, right: &str, duplicate: bool) {
    let chunk = |context: &str| {
        let attributes = r#" w:id="1" w:author="Alice""#;
        if context == "run" {
            let text = if kind == "del" { "delText" } else { "t" };
            return format!(
                "<w:p><w:{kind}{attributes}><w:r><w:{text}>a</w:{text}></w:r></w:{kind}></w:p>"
            );
        }
        let mark = format!("<w:{kind}{attributes}/>");
        if context == "paragraph" {
            format!("<w:p><w:pPr><w:rPr>{mark}</w:rPr></w:pPr><w:r><w:t>a</w:t></w:r></w:p>")
        } else {
            format!(
                "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"1000\"/></w:tblGrid><w:tr><w:trPr>{mark}</w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>"
            )
        }
    };
    let bytes = common::docx::docx(&format!("{}{}", chunk(left), chunk(right)));
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(
        repaired
            .repaired
            .iter()
            .filter(|f| f.code == "DUPLICATE_REVISION_ID")
            .count(),
        usize::from(duplicate),
        "{kind}: {left}/{right}"
    );
    assert_word_valid_package(&repaired.docx);
    if !duplicate {
        assert_eq!(
            repaired.docx, bytes,
            "accepted {kind}: {left}/{right} must stay byte-identical"
        );
    }
}

macro_rules! issue_1137_context_probes {
    ($($test:ident: ($kind:literal, $left:literal, $right:literal, $duplicate:literal)),* $(,)?) => {$(
        #[test]
        fn $test() { issue_1137_context_pair($kind, $left, $right, $duplicate); }
    )*};
}

issue_1137_context_probes! {
    issue_1137_ins_run_and_row_ids_are_separate: ("ins", "run", "row", false),
    issue_1137_ins_run_and_paragraph_mark_ids_are_separate: ("ins", "run", "paragraph", false),
    issue_1137_del_run_and_row_ids_are_separate: ("del", "run", "row", false),
    issue_1137_del_run_and_paragraph_mark_ids_are_separate: ("del", "run", "paragraph", false),
    issue_1137_move_from_run_and_mark_ids_are_separate: ("moveFrom", "run", "paragraph", false),
    issue_1137_move_to_run_and_mark_ids_are_separate: ("moveTo", "run", "paragraph", false),
    issue_1137_ins_row_and_paragraph_mark_ids_conflict: ("ins", "row", "paragraph", true),
    issue_1137_del_row_and_paragraph_mark_ids_conflict: ("del", "row", "paragraph", true),
    issue_1137_move_from_paragraph_mark_ids_conflict: ("moveFrom", "paragraph", "paragraph", true),
    issue_1137_move_to_paragraph_mark_ids_conflict: ("moveTo", "paragraph", "paragraph", true),
}

/// #1214 is a settlement warning: Word opens these ranges without repair.
#[test]
fn split_move_ranges_are_a_ring1_warning_once() {
    let first = r#"<w:moveFrom w:id="2" w:author="Alice"><w:r><w:t>Closing section remains</w:t></w:r></w:moveFrom>"#;
    let last = r#"<w:moveFrom w:id="4" w:author="Alice"><w:r><w:t xml:space="preserve"> last in the original agreement.</w:t></w:r></w:moveFrom>"#;
    for split in [true, false] {
        let middle = if split {
            r#"<w:moveFromRangeEnd w:id="1"/><w:moveFromRangeStart w:id="3" w:name="move1"/>"#
        } else {
            ""
        };
        let end = if split { 3 } else { 1 };
        let body = format!(
            r#"<w:p><w:moveFromRangeStart w:id="1" w:name="move1"/>{first}{middle}{last}<w:moveFromRangeEnd w:id="{end}"/></w:p><w:p><w:moveToRangeStart w:id="5" w:name="move1"/><w:moveTo w:id="6" w:author="Alice"><w:r><w:t>Closing section remains last in the original agreement.</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="5"/></w:p>"#
        );
        let bytes = common::docx::docx(&body);
        let findings: Vec<_> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
            .into_iter()
            .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
            .collect();
        assert_eq!(findings.len(), usize::from(split), "split={split}");
        for finding in &findings {
            assert_eq!(finding.severity, "warning");
            assert!(!finding.word_fatal && !finding.repairable);
            assert!(finding.location.contains("w:moveFromRangeStart[0]"));
        }
        let public: Vec<_> = jubarte::validate::validate(&bytes)
            .unwrap()
            .into_iter()
            .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
            .collect();
        assert_eq!(
            public, findings,
            "triage must not duplicate Ring-1 findings"
        );
        assert_eq!(
            jubarte::debug::findings(&bytes)
                .unwrap()
                .iter()
                .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
                .count(),
            usize::from(split)
        );
    }
}

/// The #1939 base: a package both tools validate clean. The probe splices
/// markers over the first `</w:t></w:r></w:p>`, the audit builder's replace.
const NAMELESS_MOVE_BASE: &[u8] = include_bytes!(
    "corpus/neurotic_docx_bench/corpus/word_based/docx_source/blue_centered_title_demo_style_default_missing.docx"
);

fn splice_first_paragraph_end(markers: &str) -> Vec<u8> {
    let doc = common::docx::part_string(NAMELESS_MOVE_BASE, "word/document.xml")
        .expect("base document.xml");
    let needle = "</w:t></w:r></w:p>";
    assert!(
        doc.contains(needle),
        "the #1939 probe anchors on the first run's end"
    );
    let doc = doc.replacen(needle, &format!("</w:t></w:r>{markers}</w:p>"), 1);
    common::docx::replace_entry(NAMELESS_MOVE_BASE, "word/document.xml", doc.as_bytes())
}

/// #1939: a `moveFromRangeStart` / `moveToRangeStart` whose `w:name` is
/// missing or empty never entered the per-name counts, so the audit's
/// DANGLING-MOVE passed `validate`. `w:name` is schema-required.
/// Word's reaction is unverified, so the finding is a warning lead.
#[test]
fn probe_nameless_move_range_start_is_a_lead() {
    for markers in [
        r#"<w:moveFromRangeStart w:id="501"/><w:moveFromRangeEnd w:id="501"/>"#,
        r#"<w:moveFromRangeStart w:id="501" w:name=""/><w:moveFromRangeEnd w:id="501"/>"#,
        r#"<w:moveToRangeStart w:id="501"/><w:moveToRangeEnd w:id="501"/>"#,
        r#"<w:moveToRangeStart w:id="501" w:name=""/><w:moveToRangeEnd w:id="501"/>"#,
    ] {
        let bytes = splice_first_paragraph_end(markers);
        let report = check_word_valid_package(&bytes);
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.code == "MOVE_RANGE_NAME_MISSING")
            .collect();
        assert_eq!(findings.len(), 1, "{markers}: {:?}", report.findings);
        let finding = findings[0];
        assert_eq!(finding.severity, "warning");
        assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
        assert_eq!(finding.part, "word/document.xml");
        assert_eq!(finding.r#ref, finding.location);
        let side = if markers.contains("moveFrom") {
            "moveFromRangeStart"
        } else {
            "moveToRangeStart"
        };
        assert!(finding.location.contains(side), "{finding:?}");
        assert!(finding.message.contains("w:name"), "{finding:?}");
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.code != "MOVE_RANGE_COUNT_MISMATCH"),
            "a nameless start is not a named count: {markers} {:?}",
            report.findings
        );
        let public: Vec<_> = jubarte::validate::validate(&bytes)
            .unwrap()
            .into_iter()
            .filter(|f| f.code == "MOVE_RANGE_NAME_MISSING")
            .collect();
        assert_eq!(
            public,
            vec![finding.clone()],
            "triage must not duplicate Ring-1 findings"
        );
    }
}

/// The same probe with `w:name="m1"` is the count mismatch the check
/// already reports: one source range and no destination.
#[test]
fn probe_named_dangling_move_still_counts_as_a_mismatch() {
    let bytes = splice_first_paragraph_end(
        r#"<w:moveFromRangeStart w:id="501" w:name="m1"/><w:moveFromRangeEnd w:id="501"/>"#,
    );
    let report = check_word_valid_package(&bytes);
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
        .collect();
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    let finding = findings[0];
    assert_eq!(finding.severity, "warning");
    assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
    assert!(
        finding
            .message
            .contains("m1: 1 source ranges, 0 destination ranges"),
        "{finding:?}"
    );
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.code != "MOVE_RANGE_NAME_MISSING"),
        "{:?}",
        report.findings
    );
}

#[test]
fn issue650_aliased_duplicate_attribute_is_word_fatal() {
    for (part, element, shape) in [
        (
            "word/document.xml",
            "w:p",
            "<w:document xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main' xmlns:a='urn:alias' xmlns:b='urn:alias'><w:body><w:p a:x='Alice' b:x='Bob'><w:r><w:t>Alice, Bob, and Carol Diaz</w:t></w:r></w:p></w:body></w:document>",
        ),
        (
            "word/header1.xml",
            "w:hdr",
            "<w:hdr xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main' a:x='Alice' b:x='Bob' xmlns:a='urn:alias' xmlns:b='urn:alias'/>",
        ),
        (
            "word/footer1.xml",
            "w:ftr",
            "<w:ftr xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main' xmlns:a='urn:alias' xmlns:b='urn:ali&#97;s' a:x='Alice' b:x='Bob'/>",
        ),
    ] {
        let bytes = zip_with_parts(&[(part, shape)]);
        let pkg = PartFs::open(&bytes).unwrap();
        let findings: Vec<_> = jubarte::validate::ring1(&pkg)
            .into_iter()
            .filter(|f| f.code == "DUPLICATE_EXPANDED_ATTRIBUTE")
            .collect();
        assert_eq!(
            findings.len(),
            1,
            "aliased duplicate must be caught in {part}: {findings:?}"
        );
        let finding = &findings[0];
        assert!(finding.word_fatal && !finding.repairable, "{finding:?}");
        assert_eq!(finding.severity, "error");
        assert_eq!(finding.part, part);
        assert!(finding.location.contains(element), "{finding:?}");
        assert!(finding.message.contains(element), "{finding:?}");
    }
}

#[test]
fn issue650_distinct_attribute_namespaces_are_valid() {
    for shape in [
        "<r xmlns:a='urn:a' xmlns:b='urn:b' a:x='Alice' b:x='Bob'/>",
        "<r xmlns='urn:a' xmlns:a='urn:a' x='Alice' a:x='Bob'/>",
        "<r xmlns:a='urn:a' xmlns:b='urn:a'><s xmlns:b='urn:b' a:x='Alice' b:x='Bob'/><s a:x='Carol Diaz'/></r>",
    ] {
        let bytes = zip_with_parts(&[("word/document.xml", shape)]);
        let pkg = PartFs::open(&bytes).unwrap();
        assert!(
            jubarte::validate::ring1(&pkg)
                .iter()
                .all(|f| f.code != "DUPLICATE_EXPANDED_ATTRIBUTE"),
            "{shape}"
        );
    }
}

/// Two ZIP entries naming one OPC part — byte-equal or an ASCII-case
/// alias (ECMA-376 Part 2 §6.2.2.3 M1.12 makes them one part) — is the
/// package Word refused in probes p1, p2, p6, p7 and p9 (#600). The
/// invariant lives at the package open, not in `ring1`: a `PartFs`
/// cannot hold two parts of one identity, so the refusal precedes it —
/// the same place the admission-level invariants of #594/#644/#639
/// live. `jubarte validate` reports it through the typed
/// `DUPLICATE_PART` refusal, and `debug::findings` reports the
/// structural defect for triage.
#[test]
fn probe_duplicate_zip_part_identity_fails() {
    let document = "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body><w:p><w:r><w:t>Carol Diaz</w:t></w:r></w:p></w:body></w:document>";
    for (what, bytes) in [
        (
            "case-aliased main part",
            zip_with_parts(&[
                ("word/document.xml", document),
                ("word/Document.xml", "<w:document/>"),
            ]),
        ),
        (
            "upper-case main part",
            zip_with_parts(&[
                ("word/document.xml", document),
                ("WORD/document.xml", "<w:document/>"),
            ]),
        ),
        (
            "case-aliased content types",
            zip_with_parts(&[
                ("word/document.xml", document),
                ("[content_types].xml", "<Types/>"),
            ]),
        ),
        (
            "case-aliased package rels",
            zip_with_parts(&[("word/document.xml", document), ("_RELS/.RELS", "<R/>")]),
        ),
    ] {
        let err = jubarte::validate::validate(&bytes)
            .expect_err("Word refuses a package with two entries of one part");
        assert!(err.to_string().contains("DUPLICATE_PART"), "{what}: {err}");
        assert!(
            check_word_valid_package(&bytes)
                .errors
                .iter()
                .any(|e| e.contains("DUPLICATE_PART")),
            "{what}: the Ring-1 wrapper reports the refusal"
        );
    }
    // An exact duplicate is the same shape; the writer will not emit it
    // twice under one name, so the fixture renames the entry in place.
    let mut bytes = zip_with_parts(&[
        ("word/document.xm0", "<w:document/>"),
        ("word/document.xml", document),
    ]);
    let (local_at, central_at) = {
        let mut archive = zip::ZipArchive::new(Cursor::new(&*bytes)).unwrap();
        let entry = archive.by_name("word/document.xm0").unwrap();
        (
            usize::try_from(entry.header_start()).unwrap(),
            usize::try_from(entry.central_header_start()).unwrap(),
        )
    };
    // Both headers take the alias, as a hostile writer emits it.
    bytes[local_at + 30..local_at + 47].copy_from_slice(b"word/document.xml");
    bytes[central_at + 46..central_at + 63].copy_from_slice(b"word/document.xml");
    let err = jubarte::validate::validate(&bytes)
        .expect_err("Word refuses two entries with the same name");
    assert!(err.to_string().contains("DUPLICATE_PART"), "{err}");
}

/// A bound content control (#1427): Word fills it from the custom XML
/// store its `w:storeItemID` names, and shows it empty when the main part
/// reaches no such store. Word opens the file, so it is a lead.
fn bound_control_package(with_store: bool) -> Vec<u8> {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:sdt><w:sdtPr><w:dataBinding w:prefixMappings="xmlns:ns0='urn:probe'" w:xpath="/ns0:root[1]/ns0:abstract[1]" w:storeItemID="{11111111-2222-3333-4444-555555555555}"/><w:text/></w:sdtPr>
      <w:sdtContent><w:p><w:r><w:t>Abstract</w:t></w:r></w:p></w:sdtContent></w:sdt>
    <w:sdt><w:sdtPr><w:dataBinding w:xpath="/ns1:coreProperties[1]/ns0:title[1]" w:storeItemID="{6C3C8BC8-F283-45AE-878A-BAB7291924A1}"/><w:text/></w:sdtPr>
      <w:sdtContent><w:p><w:r><w:t>Title</w:t></w:r></w:p></w:sdtContent></w:sdt>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let rels = if with_store {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="../customXml/item1.xml"/>
</Relationships>"#
    } else {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#
    };
    // The store's parts are there either way: only the relationship from
    // the main part makes Word load them.
    zip_with_parts(&[
        ("word/document.xml", doc),
        ("word/_rels/document.xml.rels", rels),
        (
            "customXml/item1.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><root xmlns="urn:probe"><abstract>Abstract</abstract></root>"#,
        ),
        (
            "customXml/_rels/item1.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps" Target="itemProps1.xml"/>
</Relationships>"#,
        ),
        (
            "customXml/itemProps1.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><ds:datastoreItem ds:itemID="{11111111-2222-3333-4444-555555555555}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"><ds:schemaRefs><ds:schemaRef ds:uri="urn:probe"/></ds:schemaRefs></ds:datastoreItem>"#,
        ),
    ])
}

#[test]
fn probe_data_binding_without_store_is_a_lead() {
    let report = check_word_valid_package(&bound_control_package(false));
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "DATA_BINDING_WITHOUT_STORE")
        .collect();
    // The core properties' store lives in docProps: no finding for it.
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(!findings[0].word_fatal);
    assert_eq!(findings[0].severity, "warning");
    assert_eq!(findings[0].part, "word/document.xml");
    assert!(
        findings[0]
            .message
            .contains("{11111111-2222-3333-4444-555555555555}"),
        "{}",
        findings[0].message
    );
}

#[test]
fn a_data_binding_whose_store_the_main_part_reaches_passes_ring1() {
    let report = check_word_valid_package(&bound_control_package(true));
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.code != "DATA_BINDING_WITHOUT_STORE"),
        "{:?}",
        report.findings
    );
}

/// Two `w:customXmlInsRangeStart` may not share an id (OpenXmlValidator:
/// `Sem_UniqueAttributeValue`); a start and its end do. Repair gives the
/// second start and its end one fresh id (#1665).
#[test]
fn custom_xml_range_starts_take_unique_ids() {
    let range = |id: u32| {
        format!(
            r#"<w:customXmlInsRangeStart w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>a</w:t></w:r><w:customXmlInsRangeEnd w:id="{id}"/>"#
        )
    };
    let duplicates = |bytes: &[u8]| {
        jubarte::validate::ring1(&PartFs::open(bytes).unwrap())
            .into_iter()
            .filter(|f| f.code == "DUPLICATE_REVISION_ID")
            .count()
    };
    let unique = common::docx::docx(&format!("<w:p>{}{}</w:p>", range(5), range(6)));
    assert_eq!(duplicates(&unique), 0);
    // The two starts and the two ends each share an id.
    let bytes = common::docx::docx(&format!("<w:p>{}{}</w:p>", range(5), range(5)));
    assert_eq!(duplicates(&bytes), 2);
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(duplicates(&repaired.docx), 0);
    // The end goes with the start that is renumbered: both are repaired.
    let repaired_duplicates = repaired
        .repaired
        .iter()
        .filter(|f| f.code == "DUPLICATE_REVISION_ID")
        .count();
    assert_eq!(repaired_duplicates, 2, "{:?}", repaired.repaired);
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    let ids = |tag: &str| -> Vec<String> {
        xml.match_indices(&format!("<w:{tag} w:id=\""))
            .map(|(at, m)| {
                let id = &xml[at + m.len()..];
                id[..id.find('"').unwrap()].to_string()
            })
            .collect()
    };
    let (starts, ends) = (ids("customXmlInsRangeStart"), ids("customXmlInsRangeEnd"));
    assert_eq!(starts, ends, "{xml}");
    assert_eq!(starts[0], "5", "{xml}");
    assert_ne!(starts[1], "5", "{xml}");
}

fn duplicate_revision_ids(bytes: &[u8]) -> usize {
    jubarte::validate::ring1(&PartFs::open(bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "DUPLICATE_REVISION_ID")
        .count()
}

/// Range ends of one kind take unique ids too: OpenXmlValidator refuses
/// two `w:customXmlInsRangeEnd`, or two `w:moveFromRangeEnd`, that share
/// one (`Sem_UniqueAttributeValue`), and lets two
/// `w:customXmlMoveToRangeEnd` share one. Repair renames no end on its own:
/// it cannot tell which start an end closes (Codex on #1666).
#[test]
fn range_ends_of_one_kind_take_unique_ids() {
    let ends = |start: &str, end: &str| {
        common::docx::docx(&format!(
            r#"<w:p><w:{start} w:id="7" w:name="m" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>a</w:t></w:r><w:{end} w:id="9"/></w:p><w:p><w:{start} w:id="8" w:name="n" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>b</w:t></w:r><w:{end} w:id="9"/></w:p>"#
        ))
    };
    for kind in [
        "customXmlIns",
        "customXmlDel",
        "customXmlMoveFrom",
        "moveFrom",
        "moveTo",
    ] {
        let bytes = ends(&format!("{kind}RangeStart"), &format!("{kind}RangeEnd"));
        assert_eq!(duplicate_revision_ids(&bytes), 1, "{kind}");
        // So the finding is not one repair fixes, and the package comes back
        // as it was (CodeRabbit on #1666).
        let findings = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap());
        assert!(
            findings
                .iter()
                .filter(|f| f.code == "DUPLICATE_REVISION_ID")
                .all(|f| !f.repairable),
            "{kind}: {findings:?}"
        );
        let repaired = jubarte::validate::repair(&bytes).unwrap();
        assert_eq!(repaired.docx, bytes, "{kind}");
        assert!(repaired.repaired.is_empty(), "{kind}");
        let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
        assert_eq!(
            xml.matches(&format!(r#"<w:{kind}RangeEnd w:id="9""#))
                .count(),
            2,
            "{kind}: {xml}"
        );
    }
    let bytes = ends("customXmlMoveToRangeStart", "customXmlMoveToRangeEnd");
    assert_eq!(duplicate_revision_ids(&bytes), 0);
}

/// OpenXmlValidator compares `w:customXmlDelRangeStart` and
/// `w:customXmlDelRangeEnd` ids case-blind: `ABC` and `abc` repeat
/// (Codex on #1666).
#[test]
fn custom_xml_deletion_ids_repeat_whatever_their_case() {
    let range = |id: &str| {
        format!(
            r#"<w:customXmlDelRangeStart w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>a</w:t></w:r><w:customXmlDelRangeEnd w:id="{id}"/>"#
        )
    };
    let bytes = common::docx::docx(&format!("<w:p>{}{}</w:p>", range("ABC"), range("abc")));
    assert_eq!(duplicate_revision_ids(&bytes), 2);
}

/// A custom XML data part holds the application's XML, not
/// WordprocessingML: OpenXmlValidator checks no revision id in it, whatever
/// its name, and repair leaves it alone (Codex on #1666).
#[test]
fn custom_xml_data_holds_no_revision_ids() {
    const CUSTOM_XML: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
    let data = r#"<root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:ins w:id="5"/><w:ins w:id="5"/></root>"#;
    for name in ["customXml/item1.xml", "customXml/item1.dat"] {
        let mut pkg =
            PartFs::open(&common::docx::docx("<w:p><w:r><w:t>x</w:t></w:r></w:p>")).unwrap();
        pkg.set_part(name, data.as_bytes().to_vec());
        pkg.add_content_type_override(&format!("/{name}"), "application/xml");
        pkg.add_document_relationship("word/document.xml", CUSTOM_XML, &format!("../{name}"));
        let bytes = pkg.to_zip().unwrap();
        assert_eq!(duplicate_revision_ids(&bytes), 0, "{name}");
        let repaired = jubarte::validate::repair(&bytes).unwrap();
        let held = PartFs::open(&repaired.docx).unwrap();
        assert_eq!(held.part_bytes(name), Some(data.as_bytes()), "{name}");
    }
}

/// Nor paragraph ids: `PARA_ID_OUT_OF_RANGE` reads no custom XML data
/// part, and the renumbering a document's own paraId sets off leaves the
/// data's bytes alone (codex on #1666).
#[test]
fn custom_xml_data_holds_no_paragraph_ids() {
    const CUSTOM_XML: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
    const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
    let data = format!(r#"<root xmlns:w14="{W14}" w14:paraId="FFFFFFFF"/>"#);
    for name in ["customXml/item1.xml", "customXml/item1.dat"] {
        let body = format!(
            r#"<w:p xmlns:w14="{W14}" w14:paraId="80000001"><w:r><w:t>x</w:t></w:r></w:p>"#
        );
        let mut pkg = PartFs::open(&common::docx::docx(&body)).unwrap();
        pkg.set_part(name, data.as_bytes().to_vec());
        pkg.add_content_type_override(&format!("/{name}"), "application/xml");
        pkg.add_document_relationship("word/document.xml", CUSTOM_XML, &format!("../{name}"));
        let bytes = pkg.to_zip().unwrap();
        let flagged: Vec<String> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
            .into_iter()
            .filter(|f| f.code == "PARA_ID_OUT_OF_RANGE")
            .map(|f| f.part)
            .collect();
        assert_eq!(flagged, ["word/document.xml"], "{name}");
        let repaired = jubarte::validate::repair(&bytes).unwrap();
        let held = PartFs::open(&repaired.docx).unwrap();
        assert_eq!(held.part_bytes(name), Some(data.as_bytes()), "{name}");
    }
}

/// Repair renames the end that closes a renumbered start, not an element
/// of another namespace that shares its local name and id (Codex on
/// #1666).
#[test]
fn repair_renames_the_range_end_in_wordprocessingml() {
    let range = |id: u32, extra: &str| {
        format!(
            r#"<w:customXmlInsRangeStart w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>a</w:t></w:r>{extra}<w:customXmlInsRangeEnd w:id="{id}"/>"#
        )
    };
    let other = r#"<x:customXmlInsRangeEnd xmlns:x="urn:example:other" w:id="5"/>"#;
    let bytes = common::docx::docx(&format!("<w:p>{}{}</w:p>", range(5, ""), range(5, other)));
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(duplicate_revision_ids(&repaired.docx), 0);
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    assert!(
        xml.contains(r#"<x:customXmlInsRangeEnd xmlns:x="urn:example:other" w:id="5""#),
        "{xml}"
    );
}

const CUSTOM_XML_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";

/// Two insertions in one paragraph sharing `w:id` 5.
const TWO_FIVES: &str = r#"<w:p><w:ins w:id="5" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="5" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:r><w:t>b</w:t></w:r></w:ins></w:p>"#;

/// A Word story stays checked whatever `customXml` relationship also
/// reaches it: only a part that nothing but `customXml` relationships
/// reach is custom XML data, and a type that merely ends in `/customXml`
/// is no `customXml` relationship (codex and crush on #1666, round 6).
#[test]
fn a_word_story_stays_checked_whatever_custom_xml_relationship_reaches_it() {
    for rel_type in ["urn:application/customXml", CUSTOM_XML_REL] {
        let mut pkg = PartFs::open(&common::docx::docx(TWO_FIVES)).unwrap();
        pkg.add_document_relationship("word/document.xml", rel_type, "document.xml");
        assert_eq!(
            duplicate_revision_ids(&pkg.to_zip().unwrap()),
            1,
            "main document, {rel_type}"
        );
    }
    const HEADER_REL: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
    let header = format!(
        r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{TWO_FIVES}</w:hdr>"#
    );
    let mut pkg = PartFs::open(&common::docx::docx("<w:p><w:r><w:t>x</w:t></w:r></w:p>")).unwrap();
    pkg.set_part("word/header1.xml", header.into_bytes());
    pkg.add_content_type_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    pkg.add_document_relationship("word/document.xml", HEADER_REL, "header1.xml");
    pkg.add_document_relationship("word/document.xml", CUSTOM_XML_REL, "header1.xml");
    assert_eq!(duplicate_revision_ids(&pkg.to_zip().unwrap()), 1, "header");
}

/// Custom XML data the package's own relationships reach is data too
/// (glm-5.3 on #1666, round 6).
#[test]
fn custom_xml_data_the_package_reaches_holds_no_revision_ids() {
    let data = r#"<root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:ins w:id="5"/><w:ins w:id="5"/></root>"#;
    let mut pkg = PartFs::open(&common::docx::docx("<w:p><w:r><w:t>x</w:t></w:r></w:p>")).unwrap();
    pkg.set_part("customXml/item1.xml", data.as_bytes().to_vec());
    pkg.add_content_type_override("/customXml/item1.xml", "application/xml");
    pkg.add_package_relationship(CUSTOM_XML_REL, "customXml/item1.xml");
    assert_eq!(duplicate_revision_ids(&pkg.to_zip().unwrap()), 0);
}

/// Repair pairs a renumbered start with its end by the key the duplicate
/// check reads: `customXmlDelRange` ids compare case-blind, so the end
/// `ABC` that follows the start `abc` is its end (MiniMax on #1666,
/// round 6).
#[test]
fn repair_pairs_a_case_blind_range_end_with_its_start() {
    let range = |start: &str, end: &str, text: &str| {
        format!(
            r#"<w:customXmlDelRangeStart w:id="{start}" w:author="Alice" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>{text}</w:t></w:r><w:customXmlDelRangeEnd w:id="{end}"/>"#
        )
    };
    let bytes = common::docx::docx(&format!(
        "<w:p>{}{}</w:p>",
        range("ABC", "abc", "a"),
        range("abc", "ABC", "b")
    ));
    assert_eq!(duplicate_revision_ids(&bytes), 2);
    let repaired = jubarte::validate::repair(&bytes).unwrap();
    assert_eq!(duplicate_revision_ids(&repaired.docx), 0);
    let xml = common::docx::part_string(&repaired.docx, "word/document.xml").unwrap();
    let ids = |tag: &str| -> Vec<String> {
        xml.match_indices(&format!("<w:{tag} w:id=\""))
            .map(|(at, m)| {
                let id = &xml[at + m.len()..];
                id[..id.find('"').unwrap()].to_string()
            })
            .collect()
    };
    let (starts, ends) = (ids("customXmlDelRangeStart"), ids("customXmlDelRangeEnd"));
    assert_eq!(
        (starts[0].as_str(), ends[0].as_str()),
        ("ABC", "abc"),
        "{xml}"
    );
    assert_eq!(starts[1], ends[1], "{xml}");
    assert!(!starts[1].eq_ignore_ascii_case("abc"), "{xml}");
}

/// A one-row table whose two cells hold `cells`, the row's properties
/// `tr_pr`, in a document.
fn table_package(tr_pr: &str, cells: [&str; 2]) -> Vec<u8> {
    let cell = |content: &str| {
        format!(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{content}</w:tc>"#)
    };
    common::docx::docx(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr>{tr_pr}{}{}</w:tr></w:tbl><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#,
        cell(cells[0]),
        cell(cells[1])
    ))
}

/// A paragraph whose mark carries `revision` (`ins` or `del`, or none).
fn marked(text: &str, revision: Option<&str>) -> String {
    let mark = revision.map_or(String::new(), |kind| {
        format!(
            r#"<w:pPr><w:rPr><w:{kind} w:id="{}" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></w:rPr></w:pPr>"#,
            text.bytes().map(u32::from).sum::<u32>()
        )
    });
    format!("<w:p>{mark}<w:r><w:t>{text}</w:t></w:r></w:p>")
}

fn cell_final_marks(bytes: &[u8]) -> Vec<jubarte::validate::Finding> {
    jubarte::validate::ring1(&PartFs::open(bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "CELL_FINAL_MARK_REVISION")
        .collect()
}

/// Word opens a table cell whose final paragraph mark is inserted or
/// deleted while its row is not, but Accept All or Reject All never
/// returns (#1669: `zapk`, `z2c`, `z3c`, `zap`, Word 16, 2026-10-23).
/// Word's own Compare never writes it: the revision goes on the mark
/// before. An error, not Word-fatal (the file opens), not repairable.
#[test]
fn probe_a_revision_on_a_cells_final_mark_fails() {
    for kind in ["ins", "del"] {
        let bytes = table_package(
            "",
            [
                &format!("{}{}", marked("Old.", None), marked("New.", Some(kind))),
                &marked("B.", None),
            ],
        );
        let found = cell_final_marks(&bytes);
        assert_eq!(found.len(), 1, "{kind}: {found:?}");
        let finding = &found[0];
        assert_eq!(finding.part, "word/document.xml");
        assert_eq!(
            finding.location,
            "w:body[0]/w:tbl[0]/w:tr[0]/w:tc[0]/w:p[1]"
        );
        assert_eq!(finding.severity, "error", "{finding:?}");
        assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
        assert!(
            finding.message.contains(&format!("w:{kind}")),
            "{finding:?}"
        );
        assert!(!check_word_valid_package(&bytes).ok());
    }
    // A cell's only paragraph, and one in a block content control.
    let only = table_package("", [&marked("New.", Some("ins")), &marked("B.", None)]);
    assert_eq!(cell_final_marks(&only).len(), 1);
    let wrapped = table_package(
        "",
        [
            &format!(
                r#"{}<w:sdt><w:sdtPr><w:id w:val="1"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>"#,
                marked("Old.", None),
                marked("New.", Some("del"))
            ),
            &marked("B.", None),
        ],
    );
    assert_eq!(cell_final_marks(&wrapped).len(), 1);
}

/// Only the stories Accept or Reject All walks count: the same cell in
/// the glossary's building blocks is no story, and the comparer copies it
/// as it was (`a_part_that_is_no_story_is_copied_as_it_was`); in a header
/// it is one (codex on #1680).
#[test]
fn a_cells_final_mark_counts_only_in_a_story() {
    const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let table = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc>{}{}</w:tc></w:tr></w:tbl><w:p/>"#,
        marked("Old.", None),
        marked("New.", Some("ins"))
    );
    let document = format!(
        r#"<w:document xmlns:w="{W}" xmlns:r="{R}"><w:body><w:p><w:r><w:t>Body.</w:t></w:r></w:p><w:sectPr><w:headerReference w:type="default" r:id="rId2"/></w:sectPr></w:body></w:document>"#
    );
    let rels = format!(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{R}/glossaryDocument" Target="glossary/document.xml"/><Relationship Id="rId2" Type="{R}/header" Target="header1.xml"/></Relationships>"#
    );
    let glossary = format!(
        r#"<w:glossaryDocument xmlns:w="{W}"><w:docParts><w:docPart><w:docPartPr><w:name w:val="Block"/></w:docPartPr><w:docPartBody>{table}</w:docPartBody></w:docPart></w:docParts></w:glossaryDocument>"#
    );
    let header = format!(r#"<w:hdr xmlns:w="{W}">{table}</w:hdr>"#);
    let bytes = zip_with_parts(&[
        ("word/document.xml", &document),
        ("word/_rels/document.xml.rels", &rels),
        ("word/glossary/document.xml", &glossary),
        ("word/header1.xml", &header),
    ]);
    let parts: Vec<String> = cell_final_marks(&bytes)
        .into_iter()
        .map(|f| f.part)
        .collect();
    assert_eq!(parts, ["word/header1.xml"]);
}

/// Word's own shapes pass: the revision on the mark before the cell's
/// last (an append, as Word's Compare writes it), and every mark of an
/// inserted or deleted row revised with the row (`w:trPr/w:ins`,
/// `w:trPr/w:del`; Word 16 Compare, 2026-10-23, #1669). The body's
/// final mark is not a cell's: Word resolves it (#1669, `zbap`, `zbak`).
#[test]
fn word_shapes_of_a_cells_final_mark_pass() {
    let append = table_package(
        "",
        [
            &format!("{}{}", marked("Old.", Some("ins")), marked("New.", None)),
            &marked("B.", None),
        ],
    );
    assert_eq!(cell_final_marks(&append), []);
    for kind in ["ins", "del"] {
        let row = table_package(
            &format!(
                r#"<w:trPr><w:{kind} w:id="5" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></w:trPr>"#
            ),
            [&marked("C.", Some(kind)), &marked("D.", Some(kind))],
        );
        assert_eq!(cell_final_marks(&row), [], "{kind}");
        assert_word_valid_package(&row);
    }
    let body = common::docx::docx(&format!(
        "{}{}",
        marked("Alpha.", None),
        marked("Beta.", Some("ins"))
    ));
    assert_eq!(cell_final_marks(&body), []);
    // A nested table's cells in a deleted or inserted outer row: the outer
    // row's revision covers them, whether or not the nested rows repeat it
    // (Open-Xml-PowerTools marks only the outer row). Word 16 resolves
    // both ways (2026-10-23, `nl`: Accept All and Reject All return the
    // two sources).
    for kind in ["ins", "del"] {
        let nested = format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}"#,
            marked("Inner.", Some(kind)),
            marked("Outer.", Some(kind))
        );
        let outer = table_package(
            &format!(
                r#"<w:trPr><w:{kind} w:id="5" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></w:trPr>"#
            ),
            [&nested, &marked("D.", Some(kind))],
        );
        assert_eq!(cell_final_marks(&outer), [], "nested {kind}");
        assert_word_valid_package(&outer);
    }
}

/// A part that writes WordprocessingML in the default namespace (`<tc>`,
/// no prefix) is checked like any other (codex on #1680, round 9: the
/// check's byte prefilter looked for `:tc>` only).
#[test]
fn probe_a_revised_final_mark_in_an_unprefixed_part_fails() {
    let w = common::docx::W_NS;
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<document xmlns="{w}" xmlns:w="{w}"><body><tbl><tblPr><tblW w:w="2000" w:type="dxa"/></tblPr><tblGrid><gridCol w:w="2000"/></tblGrid><tr><tc><tcPr><tcW w:w="2000" w:type="dxa"/></tcPr><p><r><t>Old.</t></r></p><p><pPr><rPr><ins w:id="1" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></rPr></pPr><r><t>New.</t></r></p></tc></tr></tbl><p><r><t>After.</t></r></p><sectPr><pgSz w:w="12240" w:h="15840"/></sectPr></body></document>"#
    );
    let bytes = common::docx::replace_entry(
        &common::docx::docx("<w:p/>"),
        "word/document.xml",
        document.as_bytes(),
    );
    let found = cell_final_marks(&bytes);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0].location,
        "w:body[0]/w:tbl[0]/w:tr[0]/w:tc[0]/w:p[1]"
    );
}

/// A cell that a row-level content control wraps (`w:tr/w:sdt/…/w:tc`)
/// takes its row's revision like any other (codex on #1680, round 9: the
/// check read the cell's parent as its row, a false positive).
#[test]
fn a_cell_in_a_row_level_content_control_takes_its_rows_revision() {
    for kind in ["ins", "del"] {
        let cell = |text: &str| {
            format!(
                r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{}</w:tc>"#,
                marked(text, Some(kind))
            )
        };
        let bytes = common::docx::docx(&format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:{kind} w:id="5" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></w:trPr>{}<w:sdt><w:sdtPr><w:id w:val="7"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt></w:tr></w:tbl><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#,
            cell("C."),
            cell("D.")
        ));
        assert_eq!(cell_final_marks(&bytes), [], "{kind}");
        // Without the row's revision the wrapped cell is flagged.
        let bare = String::from_utf8(
            common::docx::part_string(&bytes, "word/document.xml")
                .unwrap()
                .into_bytes(),
        )
        .unwrap()
        .replace(
            &format!(r#"<w:trPr><w:{kind} w:id="5" w:author="Ana" w:date="2026-10-23T00:00:00Z"/></w:trPr>"#),
            "",
        );
        let bare = common::docx::replace_entry(&bytes, "word/document.xml", bare.as_bytes());
        assert_eq!(cell_final_marks(&bare).len(), 2, "{kind}");
    }
}

/// A cell's tags may carry whitespace before their `>` (`<w:tc >`,
/// `</w:tc\n>`): the check still reads the part (codex on #1680: the
/// byte prefilter wanted a literal `tc>`).
#[test]
fn probe_a_revised_final_mark_in_cells_with_spaced_tags_fails() {
    let w = common::docx::W_NS;
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{w}"><w:body><w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc ><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Old.</w:t></w:r></w:p>{}</w:tc
></w:tr></w:tbl><w:p><w:r><w:t>After.</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#,
        marked("New.", Some("ins"))
    );
    let bytes = common::docx::replace_entry(
        &common::docx::docx("<w:p/>"),
        "word/document.xml",
        document.as_bytes(),
    );
    let found = cell_final_marks(&bytes);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0].location,
        "w:body[0]/w:tbl[0]/w:tr[0]/w:tc[0]/w:p[1]"
    );
}

/// A custom XML data part is application data, not a Word story: a cell
/// shape in it is no cell of the document (codex on #1680).
#[test]
fn a_cell_shape_in_a_custom_xml_data_part_is_not_checked() {
    let w = common::docx::W_NS;
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{w}"><w:body><w:p><w:r><w:t>Body.</w:t></w:r></w:p></w:body></w:document>"#
    );
    let item = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:tbl xmlns:w="{w}"><w:tr><w:tc><w:p><w:r><w:t>Old.</w:t></w:r></w:p>{}</w:tc></w:tr></w:tbl>"#,
        marked("New.", Some("ins"))
    );
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="../customXml/item1.xml"/>
</Relationships>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", &doc),
        ("word/_rels/document.xml.rels", rels),
        ("customXml/item1.xml", &item),
    ]);
    assert_eq!(cell_final_marks(&bytes), []);
    // The same shape in the main part is flagged.
    let main = doc.replace(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &format!(
            "{}<w:p/>",
            item.split_once("?>\n")
                .unwrap()
                .1
                .replace(&format!(r#" xmlns:w="{w}""#), "")
        ),
    );
    let bytes = zip_with_parts(&[
        ("word/document.xml", &main),
        ("word/_rels/document.xml.rels", rels),
        ("customXml/item1.xml", &item),
    ]);
    assert_eq!(cell_final_marks(&bytes).len(), 1);
}

/// The header and footer references of [`deleted_section_package`]
/// (`rIdX0` is the header, `rIdX1` the footer).
const HEADER_FOOTER_REFS: &str = r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#;

/// A redline whose first section break is deleted, the header's and the
/// footer's runs all deleted. `field` is the footer's field code; each
/// `*_deleted` deletes that story's closing mark; `break_refs` opens the
/// deleted break's `w:sectPr` and `final_refs` the body's own.
fn deleted_section_package(
    field: &str,
    header_deleted: bool,
    footer_deleted: bool,
    break_refs: &str,
    final_refs: &str,
) -> Vec<u8> {
    use common::docx::{Part, W_NS};
    let by = r#"w:author="Ana" w:date="2026-10-25T00:00:00Z""#;
    let mark = |deleted: bool, id: u32| {
        if deleted {
            format!(r#"<w:rPr><w:del w:id="{id}" {by}/></w:rPr>"#)
        } else {
            String::new()
        }
    };
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:pPr><w:pStyle w:val="Header"/>{}</w:pPr><w:del w:id="10" {by}><w:r><w:delText>Confidential</w:delText></w:r></w:del></w:p></w:hdr>"#,
        mark(header_deleted, 11)
    );
    let footer = format!(
        r#"<w:ftr xmlns:w="{W_NS}"><w:p><w:pPr><w:jc w:val="right"/>{}</w:pPr><w:del w:id="20" {by}><w:r><w:delText xml:space="preserve">Page </w:delText></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText xml:space="preserve"> {field} </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del></w:p></w:ftr>"#,
        mark(footer_deleted, 21)
    );
    let body = format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {by}/></w:rPr><w:sectPr>{break_refs}</w:sectPr></w:pPr><w:del w:id="2" {by}><w:r><w:delText>Old section.</w:delText></w:r></w:del></w:p><w:p><w:r><w:t>Kept.</w:t></w:r></w:p>"#
    );
    let rel = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
    let ct = "application/vnd.openxmlformats-officedocument.wordprocessingml.";
    common::docx::docx_with_sect(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: &format!("{ct}header+xml"),
                rel_type: &format!("{rel}header"),
                xml: &header,
            },
            Part {
                name: "word/footer1.xml",
                content_type: &format!("{ct}footer+xml"),
                rel_type: &format!("{rel}footer"),
                xml: &footer,
            },
        ],
        final_refs,
    )
}

fn deleted_section_page_fields(bytes: &[u8]) -> Vec<jubarte::validate::Finding> {
    jubarte::validate::ring1(&PartFs::open(bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "DELETED_SECTION_PAGE_FIELD")
        .collect()
}

/// #1833: Word's own redline of the file_N family. A deleted break
/// references a footer whose closing paragraph holds PAGE or NUMPAGES.
/// Word 16.115 crashed opening it whatever the stories' marks: both live,
/// either one deleted (probes n0, n23, n24), both deleted (s176, s472,
/// round 18), and with a live section referencing the footer as well (n17,
/// g1, g3), all 2026-10-25. It opened 8441c4ae21 and 82348eef4b: a lead,
/// not Word-fatal, not repairable.
#[test]
fn probe_a_page_footer_a_deleted_break_references_is_a_lead() {
    let footer_ref = r#"<w:footerReference w:type="default" r:id="rIdX1"/>"#;
    for (field, header_deleted, footer_deleted, final_refs) in [
        ("PAGE", false, false, ""),
        ("NUMPAGES", false, false, ""),
        ("PAGE", true, false, ""),
        ("PAGE", false, true, ""),
        ("PAGE", true, true, ""),
        ("PAGE", true, true, footer_ref),
    ] {
        let bytes = deleted_section_package(
            field,
            header_deleted,
            footer_deleted,
            HEADER_FOOTER_REFS,
            final_refs,
        );
        let found = deleted_section_page_fields(&bytes);
        let case = format!(
            "{field} header_deleted={header_deleted} footer_deleted={footer_deleted} final_refs={final_refs}"
        );
        assert_eq!(found.len(), 1, "{case}: {found:?}");
        let finding = &found[0];
        assert_eq!(finding.part, "word/footer1.xml", "{case}");
        assert_eq!(finding.location, "w:p[0]", "{case}");
        assert_eq!(finding.severity, "warning", "{case}: {finding:?}");
        assert!(
            !finding.word_fatal && !finding.repairable,
            "{case}: {finding:?}"
        );
    }
}

/// #1833: what Word opens passes. The references moved from the deleted
/// break to the live final section (g2, g4 and all ten flagged redlines,
/// round 18), or a field that is no page number (a30c48b63b's DOCPROPERTY).
#[test]
fn a_page_footer_no_deleted_break_references_passes() {
    for (field, break_refs, final_refs, case) in [
        (
            "PAGE",
            "",
            HEADER_FOOTER_REFS,
            "references on the live section",
        ),
        ("DOCPROPERTY Title", HEADER_FOOTER_REFS, "", "no page field"),
    ] {
        let bytes = deleted_section_package(field, true, true, break_refs, final_refs);
        assert_eq!(deleted_section_page_fields(&bytes), [], "{case}");
        assert_word_valid_package(&bytes);
    }
    // The field in the last paragraph of a content control that ends the
    // story, every mark live: Word opened both such footers of the sample
    // (25e6d4b508, 7c98e8f2b6).
    let mut pkg = PartFs::open(&deleted_section_package(
        "PAGE",
        false,
        false,
        HEADER_FOOTER_REFS,
        "",
    ))
    .unwrap();
    let footer = pkg.part_string("word/footer1.xml").unwrap();
    let (start, end) = (
        footer.find("<w:p>").unwrap(),
        footer.rfind("</w:p>").unwrap() + 6,
    );
    let wrapped = format!(
        "{}<w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt>{}",
        &footer[..start],
        &footer[start..end],
        &footer[end..]
    );
    pkg.set_part("word/footer1.xml", wrapped.into_bytes());
    let bytes = pkg.to_zip().unwrap();
    assert_eq!(deleted_section_page_fields(&bytes), [], "content control");
    assert_word_valid_package(&bytes);
}

/// A body that deletes a table and moves a paragraph's words past it, its
/// blocks named by letter:
/// - `P`: mark deleted, deleted text, inserted text;
/// - `p`: `P` without its inserted text, which `S` then starts with;
/// - `D`: mark deleted, deleted text only;
/// - `T`: a table every row of which is deleted;
/// - `t`: a table with a deleted row and a live one;
/// - `L`: a live paragraph; `S`: a live one, the last of a case;
/// - `K`: mark deleted, a live word, inserted text;
/// - `X`: `P` holding a deleted section break;
/// - `M`: `P` with a live mark.
fn ins_before_table_body(blocks: &str) -> String {
    let by = r#"w:author="Ana" w:date="2026-10-25T00:00:00Z""#;
    // Every revision its own id, so Ring-1 sees no duplicate.
    let next = std::cell::Cell::new(1);
    let id = || {
        next.set(next.get() + 1);
        next.get()
    };
    let deleted_mark = || format!(r#"<w:rPr><w:del w:id="{}" {by}/></w:rPr>"#, id());
    let deleted = |text: &str| {
        format!(
            r#"<w:del w:id="{}" {by}><w:r><w:delText>{text}</w:delText></w:r></w:del>"#,
            id()
        )
    };
    let inserted = || {
        format!(
            r#"<w:ins w:id="{}" {by}><w:r><w:t>New terms.</w:t></w:r></w:ins>"#,
            id()
        )
    };
    let row = |deleted_row: bool| {
        let (row_mark, mark, text) = if deleted_row {
            (
                format!(r#"<w:trPr><w:del w:id="{}" {by}/></w:trPr>"#, id()),
                deleted_mark(),
                deleted("Fee"),
            )
        } else {
            (
                String::new(),
                String::new(),
                "<w:r><w:t>Fee</w:t></w:r>".to_string(),
            )
        };
        format!(
            r#"<w:tr>{row_mark}<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:pPr>{mark}</w:pPr>{text}</w:p></w:tc></w:tr>"#
        )
    };
    let table = |rows: String| {
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>{rows}</w:tbl>"#
        )
    };
    let mut moved = false;
    let mut body = String::new();
    for block in blocks.chars() {
        body.push_str(&match block {
            'P' => format!(
                "<w:p><w:pPr>{}</w:pPr>{}{}</w:p>",
                deleted_mark(),
                deleted("Old terms."),
                inserted()
            ),
            'p' => {
                moved = true;
                format!(
                    "<w:p><w:pPr>{}</w:pPr>{}</w:p>",
                    deleted_mark(),
                    deleted("Old terms.")
                )
            }
            'D' => format!(
                "<w:p><w:pPr>{}</w:pPr>{}</w:p>",
                deleted_mark(),
                deleted("Gone.")
            ),
            'T' => table(row(true)),
            't' => table(row(true) + &row(false)),
            'L' => "<w:p><w:r><w:t>Live.</w:t></w:r></w:p>".to_string(),
            'S' if moved => format!("<w:p>{}<w:r><w:t>Kept.</w:t></w:r></w:p>", inserted()),
            'S' => "<w:p><w:r><w:t>Kept.</w:t></w:r></w:p>".to_string(),
            'K' => format!(
                "<w:p><w:pPr>{}</w:pPr><w:r><w:t>Stays.</w:t></w:r>{}</w:p>",
                deleted_mark(),
                inserted()
            ),
            'X' => format!(
                r#"<w:p><w:pPr>{}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:pPr>{}{}</w:p>"#,
                deleted_mark(),
                deleted("Old terms."),
                inserted()
            ),
            'M' => format!("<w:p>{}{}</w:p>", deleted("Old terms."), inserted()),
            other => panic!("no block {other}"),
        });
    }
    body
}

fn inserted_before_deleted_tables(bytes: &[u8]) -> Vec<jubarte::validate::Finding> {
    jubarte::validate::ring1(&PartFs::open(bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "INSERTED_TEXT_BEFORE_DELETED_TABLE")
        .collect()
}

/// #1851: Word's own Compare writes a deleted paragraph mark whose
/// paragraph keeps inserted words before a deleted table: Accept All joins
/// those words to the paragraph after the table. Word 16.115 crashed
/// opening 6 of the 16 release-sample redlines (0.14.0) whose paragraph sat
/// right before the table, its table cache corrupt at the deleted row's end
/// (fix23 probe30). It opened 10: a lead, not Word-fatal, not repairable.
#[test]
fn probe_inserted_text_before_a_deleted_table_is_a_lead() {
    for (blocks, location) in [
        ("PTS", "w:body[0]/w:p[0]"),
        ("LPTTS", "w:body[0]/w:p[1]"),
        ("PTDTS", "w:body[0]/w:p[0]"),
        ("PTSPTS", "w:body[0]/w:p[0]"),
    ] {
        let bytes = common::docx::docx(&ins_before_table_body(blocks));
        let found = inserted_before_deleted_tables(&bytes);
        let expected = if blocks == "PTSPTS" { 2 } else { 1 };
        assert_eq!(found.len(), expected, "{blocks}: {found:?}");
        let finding = &found[0];
        assert_eq!(finding.part, "word/document.xml", "{blocks}");
        assert_eq!(finding.location, location, "{blocks}");
        assert_eq!(finding.severity, "warning", "{blocks}: {finding:?}");
        assert!(
            !finding.word_fatal && !finding.repairable,
            "{blocks}: {finding:?}"
        );
    }
}

/// #1851: the words already in the paragraph Accept All keeps pass (Word
/// opened w06, w10, w14 and w17 so, fix23 probe33), and so does every
/// paragraph whose words no table separates from that paragraph or that
/// keeps words of its own: a live row, a live block, a live mark, a
/// section break, no table, no paragraph after the table. A deleted
/// paragraph between the words and the table passes too: that shape
/// crashed Word no more often than any redline.
#[test]
fn inserted_text_no_deleted_table_separates_passes() {
    for (blocks, case) in [
        ("pTS", "the words moved past the table"),
        (
            "pDTDS",
            "the words moved past the table and the deleted paragraphs",
        ),
        ("PtS", "a live row"),
        ("PLTS", "a live paragraph between"),
        ("MTS", "a live mark"),
        ("KTS", "a live word besides the inserted ones"),
        ("XTS", "a deleted section break"),
        ("PDS", "no table"),
        (
            "PDTS",
            "a deleted paragraph between the words and the table",
        ),
        ("PT", "no paragraph after the table"),
        ("PTKS", "a paragraph keeping words after the table"),
    ] {
        let bytes = common::docx::docx(&ins_before_table_body(blocks));
        assert_eq!(
            inserted_before_deleted_tables(&bytes),
            [],
            "{blocks}: {case}"
        );
        assert_word_valid_package(&bytes);
    }
}

/// #1800: only self-referential or unresolved shape types warn. Word also
/// writes repeated ids, including paired AlternateContent fallback shapes.
#[test]
fn vml_shape_type_warnings_are_part_scoped() {
    let wrap = |body: &str| {
        format!(
            r#"<w:document xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml"><w:body>{body}</w:body></w:document>"#,
            jubarte::namespaces::W::URI
        )
    };
    let bad = wrap(
        r##"<v:shapetype id="same"/><v:shape id="same" type="#same"/><v:shape id="other" type="#missing"/>"##,
    );
    let bytes = zip_with_parts(&[("word/document.xml", &bad)]);
    let findings: Vec<_> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "VML_SHAPE_TYPE_INVALID")
        .collect();
    assert_eq!(findings.len(), 2, "{findings:?}");
    for finding in findings {
        assert_eq!(finding.severity, "warning");
        assert!(!finding.word_fatal && !finding.repairable);
        assert_eq!(finding.part, "word/document.xml");
        assert!(!finding.location.is_empty());
    }
    let good = wrap(
        r##"<v:shapetype id="t"/><v:shape id="s" type="#t"/><v:shape id="s" type="#t"/><v:rect id="s"/>"##,
    );
    let missing = wrap(r##"<v:shape id="s" type="#t"/>"##);
    let bytes = zip_with_parts(&[("word/document.xml", &good), ("word/header1.xml", &missing)]);
    let findings: Vec<_> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code.contains("VML"))
        .collect();
    assert_eq!(findings.len(), 1, "repeated ids are allowed: {findings:?}");
    assert_eq!(findings[0].part, "word/header1.xml");
}

#[test]
fn vml_fallback_type_does_not_warn() {
    let xml = r##"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><w:body><v:shape id="s"/><mc:AlternateContent><mc:Choice Requires="v"><v:shape id="s"/></mc:Choice><mc:Fallback><v:shape id="s" type="#s"/><v:shape id="s" type="#missing"/></mc:Fallback></mc:AlternateContent></w:body></w:document>"##;
    let bytes = zip_with_parts(&[("word/document.xml", xml)]);
    let findings: Vec<_> = jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code.contains("VML"))
        .collect();
    assert!(findings.is_empty(), "fallback must not warn: {findings:?}");
}

#[test]
fn issue1711_ruby_directly_under_paragraph_is_schema_invalid_not_word_fatal() {
    let ruby = "<w:ruby><w:rubyPr><w:rubyAlign w:val=\"center\"/><w:hps w:val=\"10\"/><w:hpsRaise w:val=\"20\"/><w:hpsBaseText w:val=\"20\"/><w:lid w:val=\"en-US\"/></w:rubyPr><w:rt><w:r><w:t>ka</w:t></w:r></w:rt><w:rubyBase><w:r><w:t>Ka.</w:t></w:r></w:rubyBase></w:ruby>";
    for wrapped in [false, true] {
        let content = if wrapped {
            format!("<w:r>{ruby}</w:r>")
        } else {
            ruby.to_owned()
        };
        let bytes = common::docx::docx(&format!("<w:p>{content}</w:p>"));
        let pkg = PartFs::open(&bytes).unwrap();
        let findings: Vec<_> = jubarte::validate::ring1(&pkg)
            .into_iter()
            .filter(|f| f.code == "RUBY_OUTSIDE_RUN")
            .collect();
        assert_eq!(
            findings.len(),
            usize::from(!wrapped),
            "direct paragraph ruby must be diagnosed; a run-contained ruby must pass"
        );
        if let Some(finding) = findings.first() {
            assert_eq!(finding.severity, "error");
            assert_eq!(finding.part, "word/document.xml");
            assert!(finding.location.ends_with("w:p[0]/w:ruby[0]"));
            assert!(!finding.word_fatal && !finding.repairable);
        }
    }
}
