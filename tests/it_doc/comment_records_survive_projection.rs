// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment records survive a compare and its projections faithfully.
//!
//! The comparer's `organic_source_snapshot` used to assert this through
//! `crate::comments::list_comments`; the redlines layer may no longer reach
//! the doc layer, so the record-level property lives here, on the public
//! API. Comments whose anchor the revised document removed are settled by
//! the compare even when rejecting (PowerTools-faithful settlement), so
//! the invariants asserted are: projections never invent or alter comment
//! records, every survivor matches its source record field for field, and
//! repeating a projection is a fixed point.

use jubarte::comments::{CommentRecord, list_comments};
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const ORIGINAL: &[u8] = include_bytes!(
    "../corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition.docx"
);
const REVISED: &[u8] = include_bytes!(
    "../corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition_removal.docx"
);

/// The fields a projection must neither invent nor alter. Identity fields
/// (`id`, `ref`, `paragraph`) are excluded: projections renumber them, the
/// same normalization the comparer's snapshot applied.
fn identity(record: &CommentRecord) -> (String, Option<String>, Option<String>, String, bool) {
    (
        record.author.clone(),
        record.initials.clone(),
        record.date.clone(),
        record.text.clone(),
        record.done,
    )
}

#[test]
fn projections_keep_the_source_comment_records() {
    let compared =
        compare_documents(ORIGINAL, REVISED, "T2.2 Bot").expect("comment-heavy pair compares");
    let rejected = reject_revisions(&compared).expect("reject");
    let accepted = accept_revisions(&compared).expect("accept");

    let original_records = list_comments(ORIGINAL).expect("original comments");
    let revised_records = list_comments(REVISED).expect("revised comments");
    let rejected_records = list_comments(&rejected).expect("rejected comments");
    let accepted_records = list_comments(&accepted).expect("accepted comments");

    for (label, projected, source) in [
        ("rejected", &rejected_records, &original_records),
        ("accepted", &accepted_records, &revised_records),
    ] {
        let source_identities: Vec<_> = source.iter().map(identity).collect();
        assert!(
            !projected.is_empty(),
            "{label}: comments survive the projection"
        );
        for record in projected {
            let found = source_identities.contains(&identity(record));
            assert!(
                found,
                "{label}: {record:?} has no source record; projections \
                 never invent or alter comment records"
            );
        }
    }

    let repeated = accept_revisions(&accepted).expect("repeat accept");
    assert_eq!(
        rejected_records.iter().map(identity).collect::<Vec<_>>(),
        list_comments(&reject_revisions(&rejected).expect("repeat reject"))
            .expect("re-rejected comments")
            .iter()
            .map(identity)
            .collect::<Vec<_>>(),
        "re-rejecting a rejected document is a no-op for comments"
    );
    assert_eq!(
        accepted_records.iter().map(identity).collect::<Vec<_>>(),
        list_comments(&repeated)
            .expect("re-accepted comments")
            .iter()
            .map(identity)
            .collect::<Vec<_>>(),
        "re-accepting an accepted document is a no-op for comments"
    );
}
