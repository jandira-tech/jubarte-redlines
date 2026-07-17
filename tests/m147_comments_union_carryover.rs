//! C2 — comments union-carryover contract (synthetic).
//!
//! Mechanism (not fixture-specific):
//! 1. Comments present on A∪B survive as a non-empty comments part when either
//!    side has comments, with anchors in the body.
//! 2. Orphan anchors (refs without a `w:comment` definition) are not emitted
//!    (Ring-1 validity).
//! 3. When only A has comments, those comments are still carried (union).

use std::collections::HashSet;
use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

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
  <w:comment w:id="{comment_id}" w:author="A" w:date="2020-01-01T00:00:00Z" w:initials="A">
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

/// A has a comment on shared text; B revises surrounding text. Comment body
/// must survive and every anchor id must resolve to a comment definition.
#[test]
fn a_side_comment_survives_with_matched_anchors() {
    let a = pkg_with_comment("Hello shared world", "0", "note-on-hello");
    let b = plain_pkg("Hello shared WORLD revised");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
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

/// Only B has comments: parts + anchors carried (superset path).
#[test]
fn b_only_comments_carried() {
    let a = plain_pkg("Base text alpha");
    let b = pkg_with_comment("Base text alpha plus", "3", "b-side-note");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
    let defs = comment_ids(&pkg);
    let anchors = anchor_ids(&pkg);
    assert!(!defs.is_empty(), "B-only comments must be carried");
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors: anchors={anchors:?} defs={defs:?}"
    );
}

/// Duplicate body texts under distinct ids collapse to one def (Word keeps
/// one of each body on document_100×lots_of_comments).
#[test]
fn duplicate_comment_bodies_deduped() {
    // Build B with two comments sharing body text "same note" on different
    // spans — Word redlines keep a single def per unique body.
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
    let pkg = PartFs::open(&out).expect("open");
    let defs = comment_ids(&pkg);
    assert_eq!(
        defs.len(),
        1,
        "duplicate body texts must collapse to one comment def, got {defs:?}"
    );
    let anchors = anchor_ids(&pkg);
    assert!(
        anchors.iter().all(|id| defs.contains(id)),
        "no orphan anchors after dedupe: anchors={anchors:?} defs={defs:?}"
    );
}

#[derive(Clone, Copy)]
enum CommentGraphFixture {
    CollisionA,
    CollisionB,
    OrphanedParent,
}

fn pkg_with_comment_identity_graph(fixture: CommentGraphFixture) -> Vec<u8> {
    let (anchors, comments, comments_extended, comments_ids) = match fixture {
        CommentGraphFixture::CollisionA => (
            r#"<w:commentRangeStart w:id="0"/>
      <w:commentRangeStart w:id="1"/>
      <w:r><w:t>shared comment target</w:t></w:r>
      <w:commentRangeEnd w:id="1"/>
      <w:r><w:commentReference w:id="1"/></w:r>
      <w:commentRangeEnd w:id="0"/>
      <w:r><w:commentReference w:id="0"/></w:r>"#,
            r#"<w:comment w:id="0" w:author="A">
    <w:p w14:paraId="11111111"><w:r><w:t>A parent</w:t></w:r></w:p>
  </w:comment>
  <w:comment w:id="1" w:author="A">
    <w:p w14:paraId="22222222"><w:r><w:t>A reply</w:t></w:r></w:p>
  </w:comment>"#,
            r#"<w15:commentEx w15:paraId="11111111" w15:done="0"/>
  <w15:commentEx w15:paraId="22222222" w15:paraIdParent="11111111" w15:done="0"/>"#,
            r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000001"/>
  <w16cid:commentId w16cid:paraId="22222222" w16cid:durableId="10000002"/>"#,
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
            r#"<w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="20000001"/>"#,
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
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
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
    let content_types = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/><Override PartName="/word/commentsExtended.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml"/><Override PartName="/word/commentsIds.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let document_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/><Relationship Id="rId2" Type="http://schemas.microsoft.com/office/2011/relationships/commentsExtended" Target="commentsExtended.xml"/><Relationship Id="rId3" Type="http://schemas.microsoft.com/office/2016/09/relationships/commentsIds" Target="commentsIds.xml"/></Relationships>"#;

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

#[test]
fn cross_document_para_id_collisions_are_reallocated_across_the_comment_graph() {
    let a = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionA);
    let b = pkg_with_comment_identity_graph(CommentGraphFixture::CollisionB);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");

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
}

#[test]
fn orphan_cleanup_removes_parent_edges_to_dropped_comment_paragraphs() {
    let a = plain_pkg("shared comment target with reply");
    let b = pkg_with_comment_identity_graph(CommentGraphFixture::OrphanedParent);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");

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
}
