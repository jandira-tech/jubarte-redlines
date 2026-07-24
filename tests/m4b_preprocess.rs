// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.B — preprocess + block-hashing tests (starts with P0 accept/reject).

use jubarte::namespaces::W;
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::{Dom, NodeId};

fn body_from(dom: &mut Dom, inner: &str) -> NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

/// P0 — reject removes inserted content; accept keeps it.
#[test]
fn p0_insert_accept_vs_reject() {
    // accept: <w:ins> unwrapped → "X" survives
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:ins><w:r><w:t>X</w:t></w:r></w:ins></w:p>",
    );
    let accepted = accept_revisions_document(&mut dom, body);
    assert_eq!(dom.value(accepted), "X", "accept keeps inserted text");

    // reject: inserted content removed → ""
    let mut dom2 = Dom::new();
    let body2 = body_from(
        &mut dom2,
        "<w:p><w:ins><w:r><w:t>X</w:t></w:r></w:ins></w:p>",
    );
    let rejected = reject_revisions_document(&mut dom2, body2);
    assert_eq!(dom2.value(rejected), "", "reject removes inserted text");
}

/// P0 — reject restores deleted content; accept removes it.
#[test]
fn p0_delete_accept_vs_reject() {
    // accept: <w:del> dropped → "" (deletion takes effect)
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:del><w:r><w:delText>Y</w:delText></w:r></w:del></w:p>",
    );
    let accepted = accept_revisions_document(&mut dom, body);
    assert_eq!(dom.value(accepted), "", "accept removes deleted text");

    // reject: deletion undone → "Y" restored as w:t
    let mut dom2 = Dom::new();
    let body2 = body_from(
        &mut dom2,
        "<w:p><w:del><w:r><w:delText>Y</w:delText></w:r></w:del></w:p>",
    );
    let rejected = reject_revisions_document(&mut dom2, body2);
    assert_eq!(dom2.value(rejected), "Y", "reject restores deleted text");
    // and it should be a plain w:t now, not w:delText
    let has_deltext = dom2
        .descendants(rejected, Some(&W::name("delText")))
        .is_empty();
    assert!(has_deltext, "delText converted back to w:t");
}

/// P0 — a clean run is unchanged by both.
#[test]
fn p0_clean_run_unchanged() {
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>Z</w:t></w:r></w:p>");
    let a = accept_revisions_document(&mut dom, body);
    assert_eq!(dom.value(a), "Z");
    let mut dom2 = Dom::new();
    let body2 = body_from(&mut dom2, "<w:p><w:r><w:t>Z</w:t></w:r></w:p>");
    let r = reject_revisions_document(&mut dom2, body2);
    assert_eq!(dom2.value(r), "Z");
}

use jubarte::comparer::preprocess::{
    block_hash_string, clone_for_structure_hash, remove_existing_powertools_markup,
    test_for_invalid_content,
};
use jubarte::namespaces::PT;

/// M4.B.1 — remove-pt keeps only pt:Unid; invalid-content guard.
#[test]
fn m4_b1_remove_pt_and_invalid() {
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>x</w:t></w:r></w:p>");
    let p = dom.elements(body, Some(&W::p()))[0];
    dom.set_attribute_value(p, &PT::unid(), Some("KEEP"));
    dom.set_attribute_value(p, &PT::sha1_hash(), Some("DROP"));
    dom.set_attribute_value(p, &PT::status(), Some("DROP2"));
    remove_existing_powertools_markup(&mut dom, body);
    assert_eq!(dom.attribute(p, &PT::unid()), Some("KEEP"));
    assert_eq!(dom.attribute(p, &PT::sha1_hash()), None);
    assert_eq!(dom.attribute(p, &PT::status()), None);

    // invalid guard: only altChunk/subDoc/contentPart
    let mut d2 = Dom::new();
    let b2 = body_from(&mut d2, "<w:p><w:subDoc/></w:p>");
    assert!(
        test_for_invalid_content(&d2, b2)
            .unwrap_err()
            .contains("subDoc")
    );
    let mut d3 = Dom::new();
    let b3 = body_from(&mut d3, "<w:p><w:r><w:t>ok</w:t></w:r></w:p>");
    assert!(test_for_invalid_content(&d3, b3).is_ok());
}

/// M4.B.2 — hash-string strips exactly one wml default-xmlns.
#[test]
fn m4_b2_hash_string() {
    let mut dom = Dom::new();
    // a bare <w:p> serializes with the default xmlns on the root
    let p = dom.new_element(W::p());
    let s = block_hash_string(&dom, p);
    assert!(
        !s.contains("xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\""),
        "the single wml default-xmlns must be stripped, got: {s}"
    );
    assert!(s.contains("<w:p"), "element prefix retained: {s}");
}

/// M4.B.3 — structure clone drops text, keeps structure.
#[test]
fn m4_b3_structure_clone() {
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>aaa</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let tbl = dom.elements(body, Some(&W::name("tbl")))[0];
    let sc = clone_for_structure_hash(&mut dom, tbl).unwrap();
    assert_eq!(dom.value(sc), "", "all text dropped");
    // structure preserved: tbl>tr>tc>p>r>t still present as elements
    assert!(
        !dom.descendants(sc, Some(&W::t())).is_empty(),
        "w:t element node kept (just empty)"
    );

    // same structure, different text → identical structure-hash strings
    let mut d2 = Dom::new();
    let body2 = body_from(
        &mut d2,
        "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>zzzzz</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let tbl2 = d2.elements(body2, Some(&W::name("tbl")))[0];
    let sc2 = clone_for_structure_hash(&mut d2, tbl2).unwrap();
    assert_eq!(block_hash_string(&dom, sc), block_hash_string(&d2, sc2));
}

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{clone_block_level_content_for_hashing, null_rel_resolver};

fn clone_hash(dom: &mut Dom, node: NodeId, settings: &WmlComparerSettings) -> NodeId {
    clone_block_level_content_for_hashing(dom, node, true, settings, &null_rel_resolver)
}

/// M4.B.4a — drops, footnote-ref empty, run-merge, text transform.
#[test]
fn m4_b4a_text_run_para() {
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let mut dom = Dom::new();
    // pPr/rPr/bookmarks dropped; adjacent single-w:t runs merged
    let body = body_from(
        &mut dom,
        "<w:p><w:pPr><w:jc/></w:pPr><w:bookmarkStart/><w:r><w:rPr><w:b/></w:rPr><w:t>Hel</w:t></w:r><w:r><w:t>lo</w:t></w:r></w:p>",
    );
    let p = dom.elements(body, Some(&W::p()))[0];
    let c = clone_hash(&mut dom, p, &s);
    // pPr dropped
    assert!(dom.element(c, &W::p_pr()).is_none(), "pPr dropped");
    // the two single-w:t runs merged into ONE run with text "Hello"
    let runs = dom.elements(c, Some(&W::r()));
    assert_eq!(runs.len(), 1, "adjacent single-w:t runs merged");
    assert_eq!(dom.value(runs[0]), "Hello");

    // footnoteReference → bare empty element (no w:id)
    let mut d2 = Dom::new();
    let b2 = body_from(
        &mut d2,
        "<w:p><w:r><w:footnoteReference w:id=\"7\"/></w:r></w:p>",
    );
    let p2 = d2.elements(b2, Some(&W::p()))[0];
    let c2 = clone_hash(&mut d2, p2, &s);
    let fr = d2.descendants(c2, Some(&W::name("footnoteReference")));
    assert_eq!(fr.len(), 1);
    assert_eq!(
        d2.attribute(fr[0], &W::id()),
        None,
        "w:id dropped from footnoteReference"
    );
}

/// M4.B.4a — conflate maps space→NBSP in the merged run text.
#[test]
fn m4_b4a_conflate_space_to_nbsp() {
    let s = WmlComparerSettings::default(); // conflate on
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>a b</w:t></w:r></w:p>");
    let p = dom.elements(body, Some(&W::p()))[0];
    let c = clone_hash(&mut dom, p, &s);
    let r = dom.elements(c, Some(&W::r()))[0];
    assert_eq!(dom.value(r), "a\u{00A0}b", "space conflated to NBSP");
}

/// M4.B.4b — table cases: tcPr keeps only gridSpan; gridSpan val is no-namespace.
#[test]
fn m4_b4b_table_cases() {
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:tbl><w:tr><w:tc><w:tcPr><w:tcW/><w:gridSpan w:val=\"2\"/></w:tcPr><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let tbl = dom.elements(body, Some(&W::name("tbl")))[0];
    let c = clone_hash(&mut dom, tbl, &s);
    let tcpr = dom.descendants(c, Some(&W::name("tcPr")));
    assert_eq!(tcpr.len(), 1);
    // tcPr keeps only gridSpan (w:tcW dropped)
    assert!(
        dom.element(tcpr[0], &W::name("tcW")).is_none(),
        "tcW dropped from tcPr"
    );
    let gs = dom.element(tcpr[0], &W::name("gridSpan")).unwrap();
    // val is the no-namespace attribute
    assert_eq!(
        dom.attribute(gs, &jubarte::xmllinq::XName::get("val", "")),
        Some("2")
    );
}

/// M4.B.4c — relationship attribute replaced by resolver value.
#[test]
fn m4_b4c_rel_id_branch() {
    use jubarte::namespaces::{A, R};
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    // a:blip is in s_ElementsWithRelationshipIds; r:embed is a rel attr
    let body = body_from(
        &mut dom,
        "<w:p><w:r><w:drawing><a:blip xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" r:embed=\"rId5\"/></w:drawing></w:r></w:p>",
    );
    let p = dom.elements(body, Some(&W::p()))[0];
    let resolver = |rid: &str| Some(format!("HASH({rid})"));
    let c = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &resolver);
    let blip = dom.descendants(c, Some(&A::name("blip")));
    assert_eq!(blip.len(), 1);
    assert_eq!(
        dom.attribute(blip[0], &R::name("embed")),
        Some("HASH(rId5)")
    );
}

/// M4.B.4d — VML shape drops style/id/type; default drops pt:* + trim attrs.
#[test]
fn m4_b4d_vml_and_default() {
    use jubarte::namespaces::VML;
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:r><w:pict><v:shape xmlns:v=\"urn:schemas-microsoft-com:vml\" id=\"S1\" type=\"T1\" style=\"width:1pt\" alt=\"keep\"/></w:pict></w:r></w:p>",
    );
    let p = dom.elements(body, Some(&W::p()))[0];
    let c = clone_hash(&mut dom, p, &s);
    let shape = dom.descendants(c, Some(&VML::name("shape")));
    assert_eq!(shape.len(), 1);
    assert_eq!(
        dom.attribute(shape[0], &jubarte::xmllinq::XName::get("id", "")),
        None
    );
    assert_eq!(
        dom.attribute(shape[0], &jubarte::xmllinq::XName::get("type", "")),
        None
    );
    assert_eq!(
        dom.attribute(shape[0], &jubarte::xmllinq::XName::get("style", "")),
        None
    );
    assert_eq!(
        dom.attribute(shape[0], &jubarte::xmllinq::XName::get("alt", "")),
        Some("keep")
    );
}

use jubarte::comparer::preprocess::{
    add_sha1_hash_to_block_level_content, block_sha1, hash_block_level_content,
};

/// M4.B.5 — AddSha1HashToBlockLevelContent: stamps; formatting-only ⇒ same hash;
/// only tbl/tr get StructureSHA1Hash.
#[test]
fn m4_b5_add_sha1_hash() {
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>same</w:t></w:r></w:p><w:p><w:r><w:t>same</w:t></w:r></w:p><w:p><w:r><w:t>different</w:t></w:r></w:p>",
    );
    add_sha1_hash_to_block_level_content(&mut dom, body, &s, &null_rel_resolver);
    let ps = dom.elements(body, Some(&W::p()));
    let h0 = dom.attribute(ps[0], &PT::sha1_hash()).unwrap().to_string();
    let h1 = dom.attribute(ps[1], &PT::sha1_hash()).unwrap().to_string();
    let h2 = dom.attribute(ps[2], &PT::sha1_hash()).unwrap().to_string();
    assert_eq!(h0, h1, "formatting-only difference ⇒ identical SHA1Hash");
    assert_ne!(h0, h2, "different text ⇒ different SHA1Hash");
    assert!(
        dom.attribute(ps[0], &PT::structure_sha1_hash()).is_none(),
        "p has no structure hash"
    );

    // table gets StructureSHA1Hash on tbl + tr
    let mut d2 = Dom::new();
    let b2 = body_from(
        &mut d2,
        "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    add_sha1_hash_to_block_level_content(&mut d2, b2, &s, &null_rel_resolver);
    let tbl = d2.elements(b2, Some(&W::name("tbl")))[0];
    let tr = d2.descendants(tbl, Some(&W::name("tr")))[0];
    assert!(d2.attribute(tbl, &PT::sha1_hash()).is_some());
    assert!(
        d2.attribute(tbl, &PT::structure_sha1_hash()).is_some(),
        "tbl has structure hash"
    );
    assert!(
        d2.attribute(tr, &PT::structure_sha1_hash()).is_some(),
        "tr has structure hash"
    );
}

/// M4.B.6 — HashBlockLevelContent two-pass correlate.
#[test]
fn m4_b6_hash_block_level_content() {
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    // source: two paragraphs with Unids U1, U2
    let source = body_from(
        &mut dom,
        "<w:p><w:r><w:t>orig1</w:t></w:r></w:p><w:p><w:r><w:t>orig2</w:t></w:r></w:p>",
    );
    let sps = dom.elements(source, Some(&W::p()));
    dom.set_attribute_value(sps[0], &PT::unid(), Some("U1"));
    dom.set_attribute_value(sps[1], &PT::unid(), Some("U2"));
    // after-projection: one para with U1 (matching), one with U3 (no source match)
    let after = body_from(
        &mut dom,
        "<w:p><w:r><w:t>final1</w:t></w:r></w:p><w:p><w:r><w:t>x</w:t></w:r></w:p>",
    );
    let aps = dom.elements(after, Some(&W::p()));
    dom.set_attribute_value(aps[0], &PT::unid(), Some("U1"));
    dom.set_attribute_value(aps[1], &PT::unid(), Some("U3"));

    // expected hash for the after-U1 block
    let clone = jubarte::comparer::preprocess::clone_block_level_content_for_hashing(
        &mut dom,
        aps[0],
        true,
        &s,
        &null_rel_resolver,
    );
    let expected = block_sha1(&dom, clone);

    hash_block_level_content(&mut dom, source, after, &s, &null_rel_resolver).unwrap();

    assert_eq!(
        dom.attribute(sps[0], &PT::correlated_sha1_hash()),
        Some(expected.as_str()),
        "source U1 gets the after-U1 hash"
    );
    assert!(
        dom.attribute(sps[1], &PT::correlated_sha1_hash()).is_none(),
        "source U2 has no match in after ⇒ no CorrelatedSHA1Hash"
    );
}

/// M4.B.7 — volatile `w14:paraId` / `w14:textId` must not influence the block
/// hash: two paragraphs with identical content but different paraIds produce
/// the same SHA1. Regression guard for the inpi del=468 → 0 fix (PR #14 gem;
/// the strip itself landed on main as `is_volatile_para_attr`).
#[test]
fn m4_b7_volatile_id_attrs_stripped() {
    use jubarte::namespaces::W14;
    let s = WmlComparerSettings::default();

    let hash_of = |xml: &str| -> String {
        let mut dom = Dom::new();
        let body = body_from(&mut dom, xml);
        let p = dom.elements(body, Some(&W::p()))[0];
        let cloned =
            clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
        block_sha1(&dom, cloned)
    };

    // same content, differing w14:paraId / w14:textId — must hash identically
    let h_a = hash_of(
        r#"<w:p w14:paraId="11111111" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:r w14:textId="AAAAAAAA"><w:t>same</w:t></w:r></w:p>"#,
    );
    let h_b = hash_of(
        r#"<w:p w14:paraId="22222222" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:r w14:textId="BBBBBBBB"><w:t>same</w:t></w:r></w:p>"#,
    );
    assert_eq!(
        h_a, h_b,
        "volatile w14:paraId/textId must not affect block hash"
    );

    // a paragraph WITHOUT the volatile attrs must hash the same as one WITH them
    let h_c = hash_of(r#"<w:p><w:r><w:t>same</w:t></w:r></w:p>"#);
    assert_eq!(
        h_a, h_c,
        "presence/absence of volatile ids must not affect hash"
    );

    // sanity: the clone path actually stripped the attrs (so the helper is wired)
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        r#"<w:p w14:paraId="11111111" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:r w14:textId="AAAAAAAA"><w:t>x</w:t></w:r></w:p>"#,
    );
    let p = dom.elements(body, Some(&W::p()))[0];
    let cloned = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let r = dom.elements(cloned, Some(&W::r()))[0];
    assert!(
        dom.attribute(cloned, &W14::name("paraId")).is_none(),
        "w14:paraId stripped from clone"
    );
    assert!(
        dom.attribute(r, &W14::name("textId")).is_none(),
        "w14:textId stripped from clone"
    );
}
