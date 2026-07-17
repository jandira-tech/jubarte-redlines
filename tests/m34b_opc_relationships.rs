// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M34b — `PartFs` relationship helpers (B.4 external-hyperlink fix).
//!
//! `reconcile_dangling_relationships` (comparer/parts.rs) now calls
//! `add_document_relationship_external` instead of `add_document_relationship`
//! for `row.external` rels, so absolute-URI targets keep
//! `TargetMode="External"` (illegal for the default Internal mode). These
//! tests exercise the two new `PartFs` methods directly (unit-level, no
//! compare pipeline involved) AND pin down the PRIOR/unchanged behavior of
//! `add_document_relationship` (still Internal-mode / `target_mode: None`),
//! so a future refactor can't silently flip the default.

use jubarte::opc::PartFs;

fn open_fixture() -> PartFs {
    let bytes = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    PartFs::open(&bytes).unwrap()
}

/// PRIOR BEHAVIOR (unchanged): `add_document_relationship` still creates a
/// plain Internal-mode relationship — `target_mode` stays `None`. This is
/// the path every non-external caller (the vast majority) still takes.
#[test]
fn add_document_relationship_prior_behavior_is_internal() {
    let mut pkg = open_fixture();
    let rid = pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
        "media/image1.png",
    );

    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    let r = rels.items.iter().find(|r| r.id == rid).unwrap();
    assert_eq!(r.target, "media/image1.png");
    assert!(
        r.target_mode.is_none(),
        "internal relationships carry no TargetMode attribute: {:?}",
        r.target_mode
    );
}

/// CHANGED CODE: `add_document_relationship_external` must mint a
/// relationship AND stamp it `TargetMode="External"` in one call.
#[test]
fn add_document_relationship_external_sets_target_mode() {
    let mut pkg = open_fixture();
    let rid = pkg.add_document_relationship_external(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
        "https://example.com/external-target",
    );

    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    let r = rels.items.iter().find(|r| r.id == rid).unwrap();
    assert_eq!(r.target, "https://example.com/external-target");
    assert_eq!(
        r.target_mode.as_deref(),
        Some("External"),
        "external relationship must carry TargetMode=External"
    );
}

/// CHANGED CODE: `set_rel_target_mode_external` flips an EXISTING
/// (previously Internal) relationship to External in place, without
/// disturbing its id/type/target.
#[test]
fn set_rel_target_mode_external_flips_existing_relationship() {
    let mut pkg = open_fixture();
    let rid = pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
        "https://example.com/was-internal",
    );
    // sanity: starts out Internal (prior behavior), like any other rel
    {
        let rels = pkg.read_rels_for("word/document.xml").unwrap();
        let r = rels.items.iter().find(|r| r.id == rid).unwrap();
        assert!(r.target_mode.is_none());
    }

    pkg.set_rel_target_mode_external("word/document.xml", &rid);

    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    let r = rels.items.iter().find(|r| r.id == rid).unwrap();
    assert_eq!(r.id, rid, "id unchanged");
    assert_eq!(
        r.target, "https://example.com/was-internal",
        "target unchanged"
    );
    assert_eq!(r.target_mode.as_deref(), Some("External"));
}

/// `set_rel_target_mode_external` on an unknown rel id is a no-op (mirrors
/// the `find` guard in the implementation) — it must not panic or create a
/// phantom relationship.
#[test]
fn set_rel_target_mode_external_unknown_id_is_noop() {
    let mut pkg = open_fixture();
    let before = pkg
        .read_rels_for("word/document.xml")
        .map(|r| r.items.len())
        .unwrap_or(0);

    pkg.set_rel_target_mode_external("word/document.xml", "rIdDoesNotExist");

    let after = pkg.read_rels_for("word/document.xml").unwrap();
    assert_eq!(after.items.len(), before, "no relationship added");
    assert!(
        after.items.iter().all(|r| r.id != "rIdDoesNotExist"),
        "no phantom relationship created"
    );
}

/// CHANGED CODE, round-trip: `TargetMode="External"` must survive a
/// serialize (`to_zip`) + re-open cycle, since that's the path the compare
/// pipeline actually exercises (write the output package, hand it back).
#[test]
fn external_target_mode_survives_zip_round_trip() {
    let mut pkg = open_fixture();
    let rid = pkg.add_document_relationship_external(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
        "https://example.com/round-trip",
    );
    let bytes = pkg.to_zip().unwrap();

    let reopened = PartFs::open(&bytes).unwrap();
    let rels = reopened.read_rels_for("word/document.xml").unwrap();
    let r = rels.items.iter().find(|r| r.id == rid).unwrap();
    assert_eq!(r.target, "https://example.com/round-trip");
    assert_eq!(r.target_mode.as_deref(), Some("External"));
}

/// PRIOR BEHAVIOR round-trip: a plain Internal relationship added via
/// `add_document_relationship` must round-trip with NO TargetMode attribute
/// at all (not `Some("Internal")`, just absent) — this is the default OPC
/// packaging behavior every existing (non-external) caller relies on.
#[test]
fn internal_target_mode_absent_after_zip_round_trip() {
    let mut pkg = open_fixture();
    let rid = pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
        "media/image2.png",
    );
    let bytes = pkg.to_zip().unwrap();

    let reopened = PartFs::open(&bytes).unwrap();
    let rels = reopened.read_rels_for("word/document.xml").unwrap();
    let r = rels.items.iter().find(|r| r.id == rid).unwrap();
    assert!(r.target_mode.is_none());
}
