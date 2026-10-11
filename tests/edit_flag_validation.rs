// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pure validation of operations shared by the native, Python and npm CLIs.
use jubarte::edit::flags::{FlagOp, Verb, editing_mode_conflict, plan_from_flags};
use jubarte::edit::{EditPlan, ExistingRevisions};

fn build(verb: Verb, op: FlagOp) -> Result<EditPlan, String> {
    // Explicit revision policy avoids document inspection: these tests only
    // exercise the operation and editing-mode validators.
    plan_from_flags(
        verb,
        &[op],
        "Reviewer",
        None,
        Some(ExistingRevisions::Refuse),
        &[],
    )
    .map(|built| built.plan)
}

#[test]
fn add_rejects_delete_and_resolve_for_both_paragraphs_and_comments() {
    for at in ["p0", "c0", "c42"] {
        for resolve in [false, true] {
            let op = FlagOp {
                at: at.into(),
                delete: !resolve,
                resolve,
                ..Default::default()
            };
            let error = build(Verb::Add, op.clone()).unwrap_err();
            assert!(
                error.contains("add takes no --delete or --resolve"),
                "{error}"
            );
            if at.starts_with('c') {
                assert!(
                    build(Verb::Edit, op).is_ok(),
                    "edit still supports the comment operation"
                );
            }
        }
    }
}

#[test]
fn add_comments_reject_styles_for_whole_paragraphs_anchors_and_replies() {
    for (at, anchor, comment) in [
        ("p0", None, true),
        ("p0", Some("Fees"), false),
        ("c0", None, false),
    ] {
        let mut op = FlagOp {
            at: at.into(),
            anchor: anchor.map(str::to_string),
            comment,
            content: Some("Please explain.".into()),
            ..Default::default()
        };
        assert!(build(Verb::Add, op.clone()).is_ok());
        for style in ["bold", "Heading2"] {
            op.styles = vec![style.into()];
            let error = build(Verb::Add, op.clone()).unwrap_err();
            assert!(error.contains("a comment takes no --style"), "{error}");
        }
    }
}

#[test]
fn add_paragraphs_reject_unsupported_styles_mixed_with_supported_styles() {
    for style in [
        "strike",
        "caps",
        "font=Calibri",
        "size=0.5",
        "size=1638",
        "color=FF0000",
    ] {
        let op = FlagOp {
            at: "p0".into(),
            content: Some("Fees".into()),
            styles: vec!["bold".into(), style.into()],
            ..Default::default()
        };
        let error = build(Verb::Add, op).unwrap_err();
        assert!(
            error.contains("a new paragraph takes --style"),
            "{style}: {error}"
        );
    }
}

#[test]
fn editing_mode_conflicts_only_with_keep() {
    for policy in [
        ExistingRevisions::Keep,
        ExistingRevisions::Accept,
        ExistingRevisions::Reject,
        ExistingRevisions::Refuse,
    ] {
        let mut plan = build(
            Verb::Add,
            FlagOp {
                at: "p0".into(),
                content: Some("Fees".into()),
                ..Default::default()
            },
        )
        .unwrap();
        plan.existing_revisions = policy;
        let conflict = editing_mode_conflict(&plan);
        if policy == ExistingRevisions::Keep {
            assert!(
                conflict
                    .unwrap()
                    .contains("--existing-revisions accept or reject")
            );
        } else {
            assert_eq!(conflict, None, "{policy:?}");
        }
    }
}
