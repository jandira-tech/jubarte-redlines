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
fn probe_duplicate_revision_id_fails() {
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
    assert!(
        report.errors.iter().any(|e| e.contains("duplicate w:id")),
        "expected duplicate w:id error, got: {:?}",
        report.errors
    );
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
