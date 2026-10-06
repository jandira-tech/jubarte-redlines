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
fn probe_dangling_comment_parent_fails() {
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
fn probe_dangling_comment_durable_id_fails() {
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
            error.contains("comment definition id '1'") && error.contains("commentReference")
        }),
        "expected unanchored comment definition error, got: {:?}",
        report.errors
    );
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
fn probe_duplicate_comment_para_id_fails() {
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
}

#[test]
fn probe_comment_parent_cycle_fails() {
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

#[test]
fn probe_paraid_overflow_fails() {
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
