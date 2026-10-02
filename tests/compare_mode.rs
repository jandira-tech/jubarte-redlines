// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `CompareMode` names the two supported comparer configurations, and the
//! `with_*` builders change only the scalars that are safe to tune.

use jubarte::admission::InputLimits;
use jubarte::comparer::{CompareMode, WmlComparerSettings};

/// `WmlComparerSettings` has no `PartialEq` (it carries `f64`s); its
/// `Debug` rendering covers every field.
fn same(a: &WmlComparerSettings, b: &WmlComparerSettings) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

#[test]
fn each_mode_is_its_preset() {
    assert!(same(
        &WmlComparerSettings::new(CompareMode::Word),
        &WmlComparerSettings::default()
    ));
    assert!(same(
        &WmlComparerSettings::new(CompareMode::PowerTools),
        &WmlComparerSettings::powertools_faithful()
    ));
    assert_eq!(CompareMode::default(), CompareMode::Word);
}

#[test]
fn builders_change_only_their_own_field() {
    let limits = InputLimits {
        max_entries: 50,
        ..InputLimits::compare()
    };
    let built = WmlComparerSettings::new(CompareMode::PowerTools)
        .with_author("Reviewer")
        .with_date("2026-01-02T03:04:05Z")
        .with_detail_threshold(0.3)
        .with_input_limits(limits);
    let expected = WmlComparerSettings {
        author_for_revisions: "Reviewer".to_string(),
        date_time_for_revisions: "2026-01-02T03:04:05Z".to_string(),
        detail_threshold: 0.3,
        input_limits: limits,
        ..WmlComparerSettings::powertools_faithful()
    };
    assert!(same(&built, &expected), "{built:?}");
}

#[test]
fn a_built_word_mode_redline_matches_the_struct_literal_one() {
    const ORIGINAL: &[u8] = include_bytes!("fixtures/redline/original.docx");
    const MODIFIED: &[u8] = include_bytes!("fixtures/redline/modified.docx");
    let built = WmlComparerSettings::new(CompareMode::Word).with_author("A");
    let literal = WmlComparerSettings {
        author_for_revisions: "A".to_string(),
        ..WmlComparerSettings::default()
    };
    let compare = |s: &WmlComparerSettings| {
        jubarte::document_comparer::compare_documents_with_settings(ORIGINAL, MODIFIED, s)
            .expect("compare")
    };
    assert_eq!(compare(&built), compare(&literal));
}
