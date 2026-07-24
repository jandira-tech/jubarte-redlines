// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C2 — comments union-carryover contract (synthetic).
//!
//! Mechanism (not fixture-specific):
//! 1. Comments present on A∪B survive as a non-empty comments part when either
//!    side has comments, with anchors in the body.
//! 2. Orphan anchors (refs without a `w:comment` definition) are not emitted
//!    (Ring-1 validity).
//! 3. When only A has comments, those comments are still carried (union).

mod common;

use std::collections::HashSet;
use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::{MC, W};
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use common::validity::assert_word_valid_package;

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn pkg_with_comment(body_text: &str, comment_id: &str, comment_body: &str) -> Vec<u8> {
    pkg_with_comment_author(body_text, comment_id, comment_body, "A")
}

fn pkg_with_comment_author(
    body_text: &str,
    comment_id: &str,
    comment_body: &str,
    author: &str,
) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p>
      <w:commentRangeStart w:id="{comment_id}"/>
      <w:r><w:t xml:space="preserve">{body_text}</w:t></w:r>
      <w:commentRangeEnd w:id="{comment_id}"/>
      <w:r><w:commentReference w:id="{comment_id}"/></w:r>
    </w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="{comment_id}" w:author="{author}" w:date="2020-01-01T00:00:00Z" w:initials="A">
    <w:p><w:r><w:t>{comment_body}</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/></Relationships>"#;

    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in [
            ("[Content_Types].xml", ct.as_slice()),
            ("_rels/.rels", root_rels.as_slice()),
            ("word/_rels/document.xml.rels", doc_rels.as_slice()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.start_file("word/comments.xml", opt).unwrap();
        z.write_all(comments.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn plain_pkg(text: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(ct).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(root_rels).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn comment_ids(pkg: &PartFs) -> HashSet<String> {
    let Some(xml) = pkg.part_string("word/comments.xml") else {
        return HashSet::new();
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    dom.elements(root, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("id")).map(str::to_string))
        .collect()
}

fn anchor_ids(pkg: &PartFs) -> HashSet<String> {
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let mut ids = HashSet::new();
    for name in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
        for e in dom.descendants(root, Some(&W::name(name))) {
            if let Some(id) = dom.attribute(e, &W::name("id")) {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

fn open_valid_output(out: &[u8]) -> PartFs {
    assert_word_valid_package(out);
    PartFs::open(out).expect("open")
}

/// A has a comment on shared text; B revises surrounding text. Comment body
/// must survive and every anchor id must resolve to a comment definition.
#[test]
fn a_side_comment_survives_with_matched_anchors() {
    let a = pkg_with_comment("Hello shared world", "0", "note-on-hello");
    let b = plain_pkg("Hello shared WORLD revised");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);
    let anchors = anchor_ids(&pkg);
    assert!(
        !defs.is_empty(),
        "A∪B union must carry at least A's comment definition"
    );
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors: anchors={anchors:?} defs={defs:?}"
    );
}

/// Comments carryover is a package-validity invariant, not a Word-visual
/// formatting pass: it must run under the PowerTools-faithful preset too.
/// (`6351117` moved `carry_comments` out of the `merge_replaced_paragraphs`
/// umbrella; this is the red that change shipped without — pre-fix, the
/// faithful preset silently dropped the union and the anchor triplet.)
#[test]
fn comments_carry_under_powertools_faithful_preset() {
    let a = pkg_with_comment("Hello shared world", "0", "note-on-hello");
    let b = plain_pkg("Hello shared WORLD revised");
    let settings = WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        ..WmlComparerSettings::powertools_faithful()
    };
    let out = compare_documents_with_settings(&a, &b, &settings).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);
    let anchors = anchor_ids(&pkg);
    assert!(
        !defs.is_empty(),
        "comment definitions must be carried under the faithful preset too"
    );
    assert!(
        !anchors.is_empty(),
        "the comment anchor triplet must be re-injected under the faithful preset"
    );
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors under the faithful preset: anchors={anchors:?} defs={defs:?}"
    );
}

/// Only B has comments: parts + anchors carried (superset path).
#[test]
fn b_only_comments_carried() {
    let a = plain_pkg("Base text alpha");
    let b = pkg_with_comment("Base text alpha plus", "3", "b-side-note");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);
    let anchors = anchor_ids(&pkg);
    assert!(!defs.is_empty(), "B-only comments must be carried");
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors: anchors={anchors:?} defs={defs:?}"
    );
}

/// Equal comment bodies on distinct non-empty ranges are distinct comments.
/// Body text alone is not a logical identity and must never silently discard
/// one author's annotation.
#[test]
fn same_body_comments_on_distinct_ranges_are_preserved() {
    // Build B with two comments sharing body text "same note" on different
    // spans. Both anchors and both definitions must survive.
    fn pkg_two_dup_comments() -> Vec<u8> {
        use std::io::{Cursor, Write};
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;
        let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p>
      <w:commentRangeStart w:id="0"/>
      <w:r><w:t xml:space="preserve">first span</w:t></w:r>
      <w:commentRangeEnd w:id="0"/>
      <w:r><w:commentReference w:id="0"/></w:r>
    </w:p>
    <w:p>
      <w:commentRangeStart w:id="1"/>
      <w:r><w:t xml:space="preserve">second span</w:t></w:r>
      <w:commentRangeEnd w:id="1"/>
      <w:r><w:commentReference w:id="1"/></w:r>
    </w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#;
        let comments = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="0" w:author="A" w:date="2020-01-01T00:00:00Z" w:initials="A">
    <w:p><w:r><w:t>same note</w:t></w:r></w:p>
  </w:comment>
  <w:comment w:id="1" w:author="A" w:date="2020-01-01T00:00:00Z" w:initials="A">
    <w:p><w:r><w:t>same note</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#;
        let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/></Types>"#;
        let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
        let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/></Relationships>"#;
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = ZipWriter::new(&mut buf);
            let opt = SimpleFileOptions::default();
            for (name, data) in [
                ("[Content_Types].xml", ct.as_slice()),
                ("_rels/.rels", root_rels.as_slice()),
                ("word/_rels/document.xml.rels", doc_rels.as_slice()),
            ] {
                z.start_file(name, opt).unwrap();
                z.write_all(data).unwrap();
            }
            z.start_file("word/document.xml", opt).unwrap();
            z.write_all(doc.as_bytes()).unwrap();
            z.start_file("word/comments.xml", opt).unwrap();
            z.write_all(comments.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }
    let a = plain_pkg("first span second span base");
    let b = pkg_two_dup_comments();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);
    assert_eq!(
        defs.len(),
        2,
        "distinct anchored comments must not collapse by body text, got {defs:?}"
    );
    let anchors = anchor_ids(&pkg);
    assert_eq!(
        anchors, defs,
        "both distinct ranges must keep their comment ids"
    );
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors: anchors={anchors:?} defs={defs:?}"
    );
}

/// Equal bodies in A and B do not make B a comment superset when the comments
/// annotate different source ranges. Both sides belong in the union.
#[test]
fn cross_document_same_body_comments_on_different_ranges_are_unioned() {
    let a = pkg_with_comment("alpha review target", "0", "same note");
    let b = pkg_with_comment("beta review target", "7", "same note");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);
    let anchors = anchor_ids(&pkg);

    assert_eq!(
        defs.len(),
        2,
        "body-only superset detection must not discard A's distinct comment: {defs:?}"
    );
    assert_eq!(anchors, defs, "both sides must retain resolved anchors");
}

/// Definition metadata is part of identity: two reviewers can legitimately
/// leave the same prose on the same range.
#[test]
fn same_body_and_range_from_different_authors_are_preserved() {
    let a = pkg_with_comment_author("shared target", "0", "looks good", "Alice");
    let b = pkg_with_comment_author("shared target", "7", "looks good", "Bob");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let defs = comment_ids(&pkg);

    assert_eq!(
        defs.len(),
        2,
        "same-body comments from different authors are distinct: {defs:?}"
    );
    assert_eq!(anchor_ids(&pkg), defs, "both comments must remain anchored");
}

#[derive(Clone, Copy)]
enum CommentGraphFixture {
    CollisionA,
    CollisionB,
    OrphanedParent,
}

fn pkg_with_comment_identity_graph(fixture: CommentGraphFixture) -> Vec<u8> {
    let (
        anchors,
        comments,
        comments_extended,
        comments_ids,
        comments_extensible,
        comments_namespace_attrs,
    ) = match fixture {
        CommentGraphFixture::CollisionA => (
            r#"<w:commentRangeStart w:id="0"/>
      <w:commentRangeStart w:id="1"/>
      <w:r><w:t>shared comment target</w:t></w:r>
      <w:commentRangeEnd w:id="1"/>
      <w:r><w:commentReference w:id="1"/></w:r>
      <w:commentRangeEnd w:id="0"/>
      <w:r><w:commentReference w:id="0"/></w:r>"#,
            r#"<w:comment w:id="0" w:author="A" mc:PreserveAttributes="w16:commentMarker">
    <mc:AlternateContent>
      <mc:Choice Requires="w16"><w:p/></mc:Choice>
      <mc:Fallback><w:p/></mc:Fallback>
    </mc:AlternateContent>
    <w:p w14:paraId="11111111" w16:commentMarker="1"><w:r><w:t>A parent</w:t></w:r></w:p>
  </w:comment>
  <w:comment w:id="1" w:author="A">
    <w:p w14:paraId="22222222"><w:r><w:t>A reply</w:t></w:r></w:p>
  </w:comment>"#,
            r#"<w15:commentEx w15:paraId="11111111" w15:done="0"/>
  <w15:commentEx w15:paraId="22222222" w15:paraIdParent="11111111" w15:done="0"/>"#,
            r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000001"/>
  <w16cid:commentId w16cid:paraId="22222222" w16cid:durableId="10000002"/>"#,
            r#"<w16cex:commentExtensible w16cex:durableId="10000001" w16cex:dateUtc="2020-01-01T00:00:00Z"/>
  <w16cex:commentExtensible w16cex:durableId="10000002" w16cex:dateUtc="2020-01-02T00:00:00Z"/>"#,
            r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:w16="http://schemas.microsoft.com/office/word/2016/wordml"
            mc:Ignorable="w14 w16""#,
        ),
        CommentGraphFixture::CollisionB => (
            r#"<w:commentRangeStart w:id="0"/>
      <w:r><w:t>shared comment target</w:t></w:r>
      <w:commentRangeEnd w:id="0"/>
      <w:r><w:commentReference w:id="0"/></w:r>"#,
            r#"<w:comment w:id="0" w:author="B">
    <w:p w14:paraId="11111111"><w:r><w:t>B parent</w:t></w:r></w:p>
  </w:comment>"#,
            r#"<w15:commentEx w15:paraId="11111111" w15:done="0"/>"#,
            r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000001"/>"#,
            r#"<w16cex:commentExtensible w16cex:durableId="10000001" w16cex:dateUtc="2021-01-01T00:00:00Z"/>"#,
            r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:w16="urn:conflicting-comment-prefix"
            mc:Ignorable="w14 w16""#,
        ),
        CommentGraphFixture::OrphanedParent => (
            r#"<w:r><w:t>shared comment target</w:t></w:r>
      <w:commentRangeStart w:id="1"/>
      <w:r><w:t> with reply</w:t></w:r>
      <w:commentRangeEnd w:id="1"/>
      <w:r><w:commentReference w:id="1"/></w:r>"#,
            r#"<w:comment w:id="0" w:author="B">
    <w:p w14:paraId="11111111"><w:r><w:t>orphaned parent</w:t></w:r></w:p>
  </w:comment>
  <w:comment w:id="1" w:author="B">
    <w:p w14:paraId="22222222"><w:r><w:t>anchored reply</w:t></w:r></w:p>
  </w:comment>"#,
            r#"<w15:commentEx w15:paraId="11111111" w15:done="0"/>
  <w15:commentEx w15:paraId="22222222" w15:paraIdParent="11111111" w15:done="0"/>"#,
            r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="20000001"/>
  <w16cid:commentId w16cid:paraId="22222222" w16cid:durableId="20000002"/>"#,
            r#"<w16cex:commentExtensible w16cex:durableId="20000001" w16cex:dateUtc="2021-01-01T00:00:00Z"/>
  <w16cex:commentExtensible w16cex:durableId="20000002" w16cex:dateUtc="2021-01-02T00:00:00Z"/>"#,
            r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            mc:Ignorable="w14""#,
        ),
    };

    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>{anchors}</w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"
            {comments_namespace_attrs}>
  {comments}
</w:comments>"#
    );
    let comments_extended = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml">
  {comments_extended}
</w15:commentsEx>"#
    );
    let comments_ids = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w16cid:commentsIds xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid">
  {comments_ids}
</w16cid:commentsIds>"#
    );
    let comments_extensible = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w16cex:commentsExtensible xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex">
  {comments_extensible}
</w16cex:commentsExtensible>"#
    );
    let content_types = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/><Override PartName="/word/commentsExtended.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml"/><Override PartName="/word/commentsIds.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml"/><Override PartName="/word/commentsExtensible.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let document_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/><Relationship Id="rId2" Type="http://schemas.microsoft.com/office/2011/relationships/commentsExtended" Target="commentsExtended.xml"/><Relationship Id="rId3" Type="http://schemas.microsoft.com/office/2016/09/relationships/commentsIds" Target="commentsIds.xml"/><Relationship Id="rId4" Type="http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible" Target="commentsExtensible.xml"/></Relationships>"#;

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default();
        for (name, data) in [
            ("[Content_Types].xml", content_types.as_slice()),
            ("_rels/.rels", root_rels.as_slice()),
            ("word/_rels/document.xml.rels", document_rels.as_slice()),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        for (name, data) in [
            ("word/document.xml", document.as_bytes()),
            ("word/comments.xml", comments.as_bytes()),
            ("word/commentsExtended.xml", comments_extended.as_bytes()),
            ("word/commentsIds.xml", comments_ids.as_bytes()),
            (
                "word/commentsExtensible.xml",
                comments_extensible.as_bytes(),
            ),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf.into_inner()
}

fn local_attribute_values(pkg: &PartFs, part: &str, local_name: &str) -> HashSet<String> {
    let xml = pkg.part_string(part).expect("part");
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).expect("root");
    dom.descendants(root, None)
        .into_iter()
        .filter_map(|element| {
            dom.attributes(element)
                .into_iter()
                .find(|(name, _)| name.local_name() == local_name)
                .map(|(_, value)| value)
        })
        .collect()
}

fn root_namespace_binding(pkg: &PartFs, part: &str, prefix: &str) -> Option<String> {
    let xml = pkg.part_string(part).expect("part");
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).expect("root");
    dom.attributes(root)
        .into_iter()
        .find(|(name, _)| dom.is_namespace_declaration(name) && name.local_name() == prefix)
        .map(|(_, value)| value)
}

#[test]
fn cross_document_para_id_collisions_are_reallocated_across_the_comment_graph() {
    let a = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionA);
    let b = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionB);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);

    let comment_para_ids = local_attribute_values(&pkg, "word/comments.xml", "paraId");
    assert_eq!(
        comment_para_ids.len(),
        3,
        "every surviving comment paragraph needs a document-unique paraId"
    );
    assert!(
        comment_para_ids.contains("11111111"),
        "B's established paraId should remain stable"
    );

    let extended_para_ids = local_attribute_values(&pkg, "word/commentsExtended.xml", "paraId");
    let ids_para_ids = local_attribute_values(&pkg, "word/commentsIds.xml", "paraId");
    assert_eq!(extended_para_ids, comment_para_ids);
    assert_eq!(ids_para_ids, comment_para_ids);

    let parent_para_ids = local_attribute_values(&pkg, "word/commentsExtended.xml", "paraIdParent");
    assert!(
        parent_para_ids.is_subset(&comment_para_ids),
        "renumbered parent references must resolve: parents={parent_para_ids:?}, paraIds={comment_para_ids:?}"
    );

    let durable_ids = local_attribute_values(&pkg, "word/commentsIds.xml", "durableId");
    assert_eq!(
        durable_ids.len(),
        3,
        "every surviving comment identity needs a document-unique durableId"
    );
    assert_eq!(
        local_attribute_values(&pkg, "word/commentsExtensible.xml", "durableId"),
        durable_ids,
        "commentsExtensible must carry exactly the durable IDs from commentsIds"
    );
}

#[test]
fn cloned_comments_retain_root_namespace_context_for_mce_qnames() {
    let a = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionA);
    let b = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionB);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);
    let comments_xml = pkg.part_string("word/comments.xml").expect("comments");

    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&comments_xml);
    let root = dom.root(document).expect("root");
    let requires = dom
        .descendants(root, None)
        .into_iter()
        .find_map(|element| {
            dom.attributes(element)
                .into_iter()
                .find(|(name, _)| name.local_name() == "Requires")
                .map(|(_, value)| value)
        })
        .expect("cloned AlternateContent choice");
    assert_ne!(
        requires, "w16",
        "the source prefix conflicts with B and must be rebound"
    );
    assert_eq!(
        root_namespace_binding(&pkg, "word/comments.xml", &requires).as_deref(),
        Some("http://schemas.microsoft.com/office/word/2016/wordml"),
        "a QName token in mc:Choice/@Requires must resolve in root scope"
    );
    let ignorable: HashSet<&str> = dom
        .attribute(root, &MC::name("Ignorable"))
        .unwrap_or("")
        .split_whitespace()
        .collect();
    assert!(
        ignorable.contains(requires.as_str()),
        "the cloned w16 attribute namespace must remain mc:Ignorable: {comments_xml}"
    );
    let preserve_attributes = dom
        .descendants(root, Some(&W::name("comment")))
        .into_iter()
        .find_map(|comment| dom.attribute(comment, &MC::name("PreserveAttributes")))
        .expect("cloned PreserveAttributes");
    assert_eq!(
        preserve_attributes,
        format!("{requires}:commentMarker"),
        "QName-valued MCE attributes must be rewritten with the rebound prefix"
    );
}

#[test]
fn orphan_cleanup_removes_parent_edges_to_dropped_comment_paragraphs() {
    let a = plain_pkg("shared comment target with reply");
    let b = pkg_with_comment_identity_graph(CommentGraphFixture::OrphanedParent);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = open_valid_output(&out);

    assert_eq!(comment_ids(&pkg), HashSet::from(["1".to_string()]));
    let comment_para_ids = local_attribute_values(&pkg, "word/comments.xml", "paraId");
    assert_eq!(comment_para_ids, HashSet::from(["22222222".to_string()]));
    assert_eq!(
        local_attribute_values(&pkg, "word/commentsExtended.xml", "paraId"),
        comment_para_ids
    );
    assert_eq!(
        local_attribute_values(&pkg, "word/commentsIds.xml", "paraId"),
        comment_para_ids
    );
    assert!(
        local_attribute_values(&pkg, "word/commentsExtended.xml", "paraIdParent").is_empty(),
        "a surviving comment must not retain an edge to a dropped parent paragraph"
    );
    assert_eq!(
        local_attribute_values(&pkg, "word/commentsExtensible.xml", "durableId"),
        HashSet::from(["20000002".to_string()]),
        "orphan cleanup must remove extensible metadata keyed by the dead durableId"
    );
}
