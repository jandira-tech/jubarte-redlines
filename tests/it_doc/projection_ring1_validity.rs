// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Word-validity of compare outputs and their projections, asserted at the
//! layer that owns `ring1`.
//!
//! The comparer's unit tests used to check `crate::validate::ring1` inline,
//! before the layering rules stopped the redlines layer from reaching the
//! doc layer; these tests keep that property on the public API.

use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::opc::PartFs;
use jubarte::validate::ring1;

/// (original, modified) pairs spanning dense prose edits, a letter, tables
/// and a comment-heavy document.
const PROSE: (&[u8], &[u8]) = (
    include_bytes!("../fixtures/redline/original.docx"),
    include_bytes!("../fixtures/redline/modified.docx"),
);
const LETTER: (&[u8], &[u8]) = (
    include_bytes!("../../examples/adoption/08-accept-reject/original.docx"),
    include_bytes!("../../examples/adoption/08-accept-reject/revised.docx"),
);
const TABLES: (&[u8], &[u8]) = (
    include_bytes!(
        "../corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx"
    ),
    include_bytes!(
        "../corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx"
    ),
);
const COMMENTS: (&[u8], &[u8]) = (
    include_bytes!(
        "../corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition.docx"
    ),
    include_bytes!(
        "../corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition_removal.docx"
    ),
);

#[test]
fn compare_outputs_and_projections_stay_ring1_valid() {
    for (label, (original, modified)) in [
        ("prose", PROSE),
        ("letter", LETTER),
        ("tables", TABLES),
        ("comments", COMMENTS),
    ] {
        let compared = compare_documents(original, modified, "T2.2 Bot")
            .unwrap_or_else(|e| panic!("{label}: {e}"));
        let accepted = accept_revisions(&compared).expect("accept");
        let rejected = reject_revisions(&compared).expect("reject");
        for (what, docx) in [
            ("compared", &compared),
            ("accepted", &accepted),
            ("rejected", &rejected),
        ] {
            let pkg = PartFs::open(docx).unwrap_or_else(|e| panic!("{label}/{what}: {e}"));
            let findings = ring1(&pkg);
            assert!(findings.is_empty(), "{label}/{what} findings: {findings:?}");
        }
    }
}
