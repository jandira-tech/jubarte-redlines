// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn a_panicking_conversion_restores_the_options() {
    // A Python caller gets PanicException and converts again on the
    // same thread: the failed call's options must not stay in force.
    let options = PdfOptions {
        revisions: RevisionStyle::Word,
        comments: CommentPlacement::End,
        changed_only: true,
        ..PdfOptions::default()
    };
    let caught = std::panic::catch_unwind(|| {
        with_options(options, || panic!("layout failed"));
    });
    assert!(caught.is_err());
    assert!(matches!(
        REVISIONS.with(std::cell::Cell::get),
        RevisionStyle::Conventional
    ));
    assert_eq!(
        PAGE_PLAN.with(std::cell::Cell::get),
        (CommentPlacement::Margin, false)
    );
}
