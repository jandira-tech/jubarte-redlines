// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! V2-13 (issue #845): the checker core for `--expect` evaluations.
//!
//! The names follow the edit plan §5 / proposal §10:
//! - `expectations_bind_source_and_check_reopened_output` — the main test
//!   name named in the plan.
//! - The example-document outcomes from `examples/README.md` are checked
//!   against `fixtures/edit-v2/example-source.docx` with the
//!   `fixtures/edit-v2/expectations.json` typed as `ExpectationsV1`.
//! - Refusal tests cover missing/unmapped/split/merged/ambiguous targets.
//! - Parser tests cover closed shape and the closed field set.

mod common;

use common::docx::{Part, W_NS, docx, docx_with_sect};
use jubarte::expect::{
    CheckKind, CheckOutcome, ExpectationsV1, Outcome, RefusalReason, evaluate, parse_expectations,
};
use jubarte::inspect::{paragraphs, source_sha256};

const EXAMPLE_SOURCE: &[u8] = include_bytes!("fixtures/edit-v2/example-source.docx");
const EXAMPLE_EXPECTATIONS: &str = include_str!("fixtures/edit-v2/expectations.json");

// Helper: a tiny docx with one paragraph "Hello world." and a second
// "Second paragraph.". pN is the settled short id the inspector prints.
fn two_paragraph_docx() -> Vec<u8> {
    let body = r#"<w:p><w:r><w:t>Hello world.</w:t></w:r></w:p>
<w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>
<w:sectPr/>"#;
    docx(body)
}

fn identity_correspondence(source_bytes: &[u8]) -> jubarte::expect::SourceToOutputCorrespondence {
    jubarte::expect::identity_correspondence(source_bytes).expect("identity correspondence")
}

#[test]
fn expectations_bind_source_and_check_reopened_output() {
    // Issue #845 / edit plan §5: the named test. Binds the
    // expectations' hash to the inspected file's bytes, then evaluates
    // the three predicates on the final projection of those same bytes
    // (the identity correspondence: the inspected file IS the reopened
    // output, the binding rule of #943 for the standalone case).
    //
    // The shipped expectations describe a desired output (a 45-day
    // payment period; "the charge is payable" present; "the fee is
    // payable" absent). The example source carries the original
    // 30-day period and the original "fee is payable" wording; the
    // README documents this. So the typed checks fail on the source,
    // and the natural-language requirement stays unassessed. The
    // acceptance is the BINDING: same source, evaluated against itself,
    // every check reports a real outcome.
    let parsed = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse expectations");
    let report = evaluate(
        &parsed,
        EXAMPLE_SOURCE,
        EXAMPLE_SOURCE,
        &identity_correspondence(EXAMPLE_SOURCE),
    );
    let payment = report
        .checks
        .iter()
        .find(|c| c.key == "payment-period")
        .expect("payment-period check ran");
    // Source says "30 days", expectations want "45 days": the typed
    // check fails; the binding held.
    assert_eq!(payment.outcome, Outcome::Failed);
    assert_eq!(
        payment.evidence.as_deref(),
        Some("Payment is due within 30 days.")
    );
    let fee = report
        .checks
        .iter()
        .filter(|c| c.key == "fee-wording")
        .collect::<Vec<_>>();
    assert_eq!(fee.len(), 2, "fee-wording carries two typed checks");
    // Source still says "the fee is payable": text_contains("the charge
    // is payable") fails, text_absent("the fee is payable") fails.
    assert!(
        fee.iter()
            .any(|c| matches!(c.kind, CheckKind::TextContains) && c.outcome == Outcome::Failed)
    );
    assert!(
        fee.iter()
            .any(|c| matches!(c.kind, CheckKind::TextAbsent) && c.outcome == Outcome::Failed)
    );
    // Receipt: path, digest, schema version, run counts, no completeness claim.
    assert_eq!(report.receipt.schema_version, 1);
    assert!(!report.receipt.independent);
    assert!(!report.receipt.complete_task);
    assert_eq!(
        report.receipt.source_sha256,
        "54ef59e60b1986dd06131bfab3d6647d2f70b095ef3c09d698b4e221855c383e"
    );
    // Exactly the three typed checks ran (review row 13: the exact
    // count, not a floor a phantom check could slip under).
    assert_eq!(report.receipt.checks_run, 3);
    assert_eq!(report.receipt.checks_refused, 0);
    assert_eq!(report.receipt.checks_unsupported, 0);
    // The commercial-review requirement is unassessed (its checks list
    // is empty): listed apart from the checks, counted once.
    assert_eq!(
        report.unassessed_requirements,
        ["commercial-review".to_string()]
    );
    assert_eq!(report.receipt.requirements_unassessed, 1);
    assert_eq!(
        report.checks.len(),
        3,
        "no pseudo-check for the unassessed requirement"
    );
}

#[test]
fn output_index_outside_the_projection_refuses_instead_of_panicking() {
    // Review row 1 / edit plan:216: a stale correspondence or a shorter
    // reopened output must refuse with its own stable code, never panic.
    // The identity correspondence of a two-paragraph source maps p1 to
    // output paragraph 1; a valid one-paragraph output leaves that index
    // outside the projection.
    let source = two_paragraph_docx();
    let one_paragraph_output = docx("<w:p><w:r><w:t>Only.</w:t></w:r></w:p><w:sectPr/>");
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "Second paragraph."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&source)
    ))
    .expect("parse");
    let report = evaluate(
        &spec,
        &source,
        &one_paragraph_output,
        &identity_correspondence(&source),
    );
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(
        report.checks[0].reason,
        Some(RefusalReason::StaleCorrespondence)
    );
}

#[test]
fn from_entries_refuses_duplicate_short_ids() {
    // Review row 1: `from_entries` accepted duplicate `short_id`s and
    // `lookup` silently returned the first, hiding the construction bug
    // until it surfaced as the wrong paragraph or an out-of-bounds
    // index. A correspondence names each source paragraph once, so a
    // duplicate refuses at construction.
    let duplicate = jubarte::expect::SourceToOutputCorrespondence::from_entries(vec![
        jubarte::expect::CorrespondenceEntry::one_to_one("p0", 0),
        jubarte::expect::CorrespondenceEntry::one_to_one("p0", 1),
    ]);
    assert!(
        duplicate.is_err(),
        "duplicate short ids must refuse: {duplicate:?}"
    );
    let err = duplicate.unwrap_err();
    assert!(
        err.contains("p0") && err.contains("duplicate"),
        "the refusal names the duplicated id: {err}"
    );
    // Distinct ids still build, and lookup stays correct across many
    // entries (review row 10: indexed lookup, order preserved).
    let many: Vec<_> = (0..50)
        .map(|n| jubarte::expect::CorrespondenceEntry::one_to_one(format!("p{n}"), n))
        .collect();
    let correspondence = jubarte::expect::SourceToOutputCorrespondence::from_entries(many)
        .expect("distinct short ids");
    assert_eq!(correspondence.entries().len(), 50);
    for n in [0usize, 7, 49] {
        assert_eq!(
            correspondence
                .lookup(&format!("p{n}"))
                .map(|e| e.output.clone()),
            Some(vec![n]),
            "lookup finds p{n}"
        );
    }
    assert!(correspondence.lookup("p50").is_none());
}

#[test]
fn invalid_output_bytes_refuse_each_check_with_its_real_kind_and_target() {
    // Review row 2: the InvalidOutput branch fabricated
    // `kind: text_equals, target: ""` for every check. A refused check
    // keeps the kind and target it was declared with, and the receipt
    // counts it the same way the evaluated path would.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "paragraph"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(
        &spec,
        &bytes,
        b"not a docx",
        &identity_correspondence(&bytes),
    );
    let check = &report.checks[0];
    assert_eq!(check.kind, CheckKind::TextContains);
    assert_eq!(check.target, "p1");
    assert_eq!(check.outcome, Outcome::Refused);
    assert_eq!(check.reason, Some(RefusalReason::InvalidOutput));
    // Same meaning as the evaluated path (review row 3): nothing ran,
    // the refusal is counted as refused, and no target is listed.
    assert_eq!(report.receipt.checks_run, 0);
    assert_eq!(report.receipt.checks_refused, 1);
    assert!(report.receipt.targets.is_empty());
}

#[test]
fn refused_checks_do_not_count_as_run_or_populate_targets() {
    // Review row 3: `checks_run` counted every outcome that was not
    // `Unassessed`, so a run where every check refused (here: a stale
    // source digest) reported `checks_run = N`, and `targets` was
    // filled before the refusals. Only evaluated checks run; only
    // evaluated checks list their target.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{wrong_hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }},
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "Second"
        }}
      ]
    }}
  ]
}}"#,
        wrong_hash = "0".repeat(64)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks.len(), 2);
    assert!(
        report
            .checks
            .iter()
            .all(|c| c.outcome == Outcome::Refused && c.reason == Some(RefusalReason::StaleSource))
    );
    assert_eq!(report.receipt.checks_run, 0, "refused checks did not run");
    assert!(
        report.receipt.targets.is_empty(),
        "no target was evaluated: {:?}",
        report.receipt.targets
    );
}

const HEADER_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

/// A one-paragraph body plus a default header with two paragraphs, the
/// ids `jubarte read` prints: `p0`, `header1`, `header1.p1`.
fn docx_with_header() -> Vec<u8> {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>Header first.</w:t></w:r></w:p><w:p><w:r><w:t>Header second.</w:t></w:r></w:p></w:hdr>"#
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CONTENT_TYPE,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn story_paragraph_targets_resolve_through_the_correspondence() {
    // Review row 4: `identity_correspondence` built entries from body
    // paragraphs only, so the story ids the inspector prints (`header1`,
    // `header1.p1`) were structurally unreachable and the refusal lied
    // that the target "is not in the correspondence". The identity
    // correspondence covers story paragraphs too, and the checker
    // projects them in the same order.
    let bytes = docx_with_header();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "header-first",
      "description": "The header's first paragraph is intact.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "header1", "long_id": "header1.w:p.j:short.header1" }},
          "text": "Header first."
        }},
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "header1.p1", "long_id": "header1.w:p.j:short.header1.p1" }},
          "text": "second"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let correspondence = identity_correspondence(&bytes);
    assert!(
        correspondence.lookup("header1").is_some(),
        "the identity correspondence carries the header's first paragraph"
    );
    let report = evaluate(&spec, &bytes, &bytes, &correspondence);
    assert_eq!(report.checks.len(), 2);
    for check in &report.checks {
        assert_eq!(check.outcome, Outcome::Passed, "{check:?}");
    }
    assert_eq!(report.checks[0].evidence.as_deref(), Some("Header first."));
    assert_eq!(report.checks[1].evidence.as_deref(), Some("Header second."));
}

#[test]
fn disagreeing_id_pair_refuses_ambiguous() {
    // Review row 6 / schema $defs.target: "Both IDs must identify the
    // SAME entity". The pair is judged through the one ids codec: a
    // long id whose `j:short` value names another paragraph, or whose
    // element names another entity type, refuses `Ambiguous` instead of
    // evaluating as if the pair were consistent.
    let bytes = two_paragraph_docx();
    for (short_id, long_id) in [
        ("p0", "document.w:p.j:short.p9"),
        ("p0", "document.w:ins.w:id.12"),
        ("header1.p1", "header1.w:p.j:short.header1.p2"),
    ] {
        let spec = parse_expectations(&format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "{short_id}", "long_id": "{long_id}" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
            hash = source_sha256(&bytes)
        ))
        .expect("parse");
        let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
        assert_eq!(
            report.checks[0].outcome,
            Outcome::Refused,
            "{short_id} + {long_id}"
        );
        assert_eq!(
            report.checks[0].reason,
            Some(RefusalReason::Ambiguous),
            "{short_id} + {long_id} name different entities"
        );
    }
}

#[test]
fn a_native_key_pair_must_name_the_very_paragraph_in_the_source() {
    // #1431 review: a `w14:paraId` long id was proven against the part
    // and element only, so `check --expect` and guarded apply accepted
    // any nonempty value under any attribute name while the short-id
    // entry carried the evaluation. The pair is judged against the
    // source's own `w14:paraId` inventory under the shared native-key
    // policy: the value must be the exact key the pair resolver would
    // print for the paragraph the short id names (valid range, unique
    // in its story, raw spelling), and the attribute must be a
    // registered long-id key. p0 and p1 carry unique paraIds; p2 and
    // p3 repeat one, so generation spells `j:short` for them.
    let w14 = "http://schemas.microsoft.com/office/word/2010/wordml";
    let bytes = docx(&format!(
        r#"<w:p xmlns:w14="{w14}" w14:paraId="1111ABCD"><w:r><w:t>Hello world.</w:t></w:r></w:p>
<w:p xmlns:w14="{w14}" w14:paraId="2222ABCD"><w:r><w:t>Second.</w:t></w:r></w:p>
<w:p xmlns:w14="{w14}" w14:paraId="3333ABCD"><w:r><w:t>Third.</w:t></w:r></w:p>
<w:p xmlns:w14="{w14}" w14:paraId="3333ABCD"><w:r><w:t>Fourth.</w:t></w:r></w:p>"#
    ));
    for (short_id, long_id, expected) in [
        ("p0", "document.w:p.w14:paraId.1111ABCD", Outcome::Passed),
        // j:short stays acceptable next to a native key.
        ("p0", "document.w:p.j:short.p0", Outcome::Passed),
        ("p2", "document.w:p.j:short.p2", Outcome::Passed),
        // The value names another paragraph's key.
        ("p0", "document.w:p.w14:paraId.2222ABCD", Outcome::Refused),
        // No paragraph carries this key.
        ("p0", "document.w:p.w14:paraId.DEADBEEF", Outcome::Refused),
        // Outside Word's key range: never a native long id.
        ("p0", "document.w:p.w14:paraId.FFFFFFFF", Outcome::Refused),
        // The key must be spelled as generation prints it.
        ("p0", "document.w:p.w14:paraId.1111abcd", Outcome::Refused),
        // A repeated paraId is no key: p2's pair is j:short.
        ("p2", "document.w:p.w14:paraId.3333ABCD", Outcome::Refused),
        // Not a registered long-id key attribute.
        ("p0", "document.w:p.w14:textId.1111ABCD", Outcome::Refused),
        ("p0", "document.w:p.w:rsidR.1111ABCD", Outcome::Refused),
    ] {
        let spec = parse_expectations(&format!(
            r#"{{
    "schema_version": 1,
    "source": {{ "sha256": "{hash}" }},
    "text_view": "final",
    "requirements": [
      {{
        "key": "k",
        "description": "d",
        "checks": [
          {{
            "kind": "text_absent",
            "target": {{ "short_id": "{short_id}", "long_id": "{long_id}" }},
            "text": "never present"
          }}
        ]
      }}
    ]
  }}"#,
            hash = source_sha256(&bytes)
        ))
        .expect("parse");
        let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
        assert_eq!(report.checks[0].outcome, expected, "{short_id} + {long_id}");
        if expected == Outcome::Refused {
            assert_eq!(
                report.checks[0].reason,
                Some(RefusalReason::Ambiguous),
                "{short_id} + {long_id}"
            );
        }
    }
}

#[test]
fn an_entry_marked_identity_not_ok_refuses_ambiguous() {
    // The correspondence itself can veto a pair (V2-08's resolver will
    // set `identity_ok` when it finds the halves disagreeing while
    // building the mapping).
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let correspondence = jubarte::expect::SourceToOutputCorrespondence::from_entries(vec![
        jubarte::expect::CorrespondenceEntry {
            short_id: "p0".to_string(),
            output: vec![0],
            identity_ok: false,
        },
    ])
    .expect("distinct short ids");
    let report = evaluate(&spec, &bytes, &bytes, &correspondence);
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(report.checks[0].reason, Some(RefusalReason::Ambiguous));
}

#[test]
fn receipt_without_meta_does_not_fake_a_digest() {
    // Review row 11: the default `evaluate()` entry point filled the
    // digest-shaped `expectations_sha256` field with "" when the caller
    // supplied no `ReceiptMeta`, so the receipt looked like it recorded
    // file identity while recording none. A missing digest must not
    // masquerade as one.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(
        report.receipt.expectations_sha256, None,
        "an absent digest is None, never an empty string that reads like one"
    );
    // The fixed final projection is recorded (review row 11).
    assert_eq!(report.receipt.text_view, jubarte::expect::TextView::Final);
}

#[test]
fn unassessed_requirements_are_listed_apart_from_checks() {
    // Review row 12: a natural-language requirement was emitted as a
    // pseudo-check with a fabricated `kind: text_equals` and an empty
    // target, so a consumer counting checks by kind overcounted and
    // could not tell a pseudo-entry from a real one. Unassessed
    // requirements are listed apart, with no fabricated kind.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "typed",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }},
    {{
      "key": "natural",
      "description": "The doc should look polished.",
      "checks": []
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks.len(), 1);
    assert_eq!(report.checks[0].key, "typed");
    // The unassessed requirement is listed apart, with no fabricated
    // kind, and the receipt counts it once (review rows 3 and 12).
    assert_eq!(report.unassessed_requirements, ["natural".to_string()]);
    assert_eq!(report.receipt.requirements_unassessed, 1);
    assert_eq!(report.receipt.checks_run, 1);
    assert_eq!(report.receipt.checks_refused, 0);
    assert_eq!(report.receipt.checks_unsupported, 0);
}

#[test]
fn readme_edited_outcome_passes_the_example_expectations() {
    // Review row 7 / issue #845 acceptance: the README's outcomes for
    // the EDITED plan -- the three typed predicates passing on the
    // edited five-paragraph output, `commercial-review` unassessed --
    // were never evaluated (every passing test used source == output).
    // Here the reopened bytes differ from the source, and the
    // correspondence is built explicitly for the shift the inserted
    // paragraph causes: the insert after p0 moves p1->2, p2->3, p3->4
    // in the output's projection.
    let plan =
        jubarte::edit::v2::PlanV2::from_json(include_str!("fixtures/edit-v2/template.edited.json"))
            .expect("parse the design's edited plan");
    let out = jubarte::edit::v2_apply::apply_plan(EXAMPLE_SOURCE, &plan)
        .expect("apply the design's edited plan");
    // The README's five-paragraph final logical text (examples/README
    // lines 24-28), computed from the reopened primary bytes.
    let edited_texts: Vec<String> = paragraphs(&out.redline)
        .expect("reopen the redline")
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(
        edited_texts,
        [
            "Payment is due within 45 days.",
            "Invoices must state the payment due date.",
            "The fee is due; the charge is payable.",
            "One  space.",
            "Notice is due within 30\u{00A0}days.",
        ],
        "the applied edit produces the README's edited output"
    );

    let spec = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse expectations");
    let correspondence = jubarte::expect::SourceToOutputCorrespondence::from_entries(vec![
        jubarte::expect::CorrespondenceEntry::one_to_one("p0", 0),
        jubarte::expect::CorrespondenceEntry::one_to_one("p1", 2),
        jubarte::expect::CorrespondenceEntry::one_to_one("p2", 3),
        jubarte::expect::CorrespondenceEntry::one_to_one("p3", 4),
    ])
    .expect("distinct short ids");
    let report = evaluate(&spec, EXAMPLE_SOURCE, &out.redline, &correspondence);

    let payment = report
        .checks
        .iter()
        .find(|c| c.key == "payment-period")
        .expect("payment-period check ran");
    assert_eq!(payment.outcome, Outcome::Passed);
    assert_eq!(
        payment.evidence.as_deref(),
        Some("Payment is due within 45 days.")
    );
    let fee = report
        .checks
        .iter()
        .filter(|c| c.key == "fee-wording")
        .collect::<Vec<_>>();
    assert_eq!(fee.len(), 2);
    assert!(
        fee.iter()
            .any(|c| matches!(c.kind, CheckKind::TextContains) && c.outcome == Outcome::Passed),
        "text_contains passes on the edited output"
    );
    assert!(
        fee.iter()
            .any(|c| matches!(c.kind, CheckKind::TextAbsent) && c.outcome == Outcome::Passed),
        "text_absent passes on the edited output"
    );
    // Exactly three checks ran against bytes that differ from the
    // source; the commercial requirement stays unassessed.
    assert_eq!(report.receipt.checks_run, 3);
    assert_eq!(report.receipt.checks_refused, 0);
    assert_eq!(
        report.unassessed_requirements,
        ["commercial-review".to_string()]
    );
    assert_eq!(report.receipt.requirements_unassessed, 1);
    assert_eq!(
        report.receipt.targets,
        vec!["p0".to_string(), "p1".to_string()]
    );
}

#[test]
fn identity_correspondence_uses_paragraphs_of_the_inspected_file() {
    // The standalone case (#943): the inspected file IS the reopened
    // output. The correspondence must list every paragraph the inspector
    // reports, in order, with the same short ids.
    let bytes = two_paragraph_docx();
    let source = identity_correspondence(&bytes);
    let inspected = paragraphs(&bytes).unwrap();
    assert_eq!(source.entries().len(), inspected.len());
    for (entry, paragraph) in source.entries().iter().zip(inspected.iter()) {
        assert_eq!(entry.short_id, paragraph.id);
    }
}

#[test]
fn text_equals_passes_when_final_text_matches() {
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "first-eq",
      "description": "p0 must equal Hello world.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    let check = &report.checks[0];
    assert_eq!(check.outcome, Outcome::Passed);
    assert_eq!(check.evidence.as_deref(), Some("Hello world."));
}

#[test]
fn text_equals_empty_string_is_allowed() {
    // Schema rule: text_equals carries no minLength on `text`; empty is
    // valid. A paragraph with no text runs equals "".
    let body = r#"<w:p/><w:p><w:r><w:t>kept</w:t></w:r></w:p><w:sectPr/>"#;
    let bytes = docx(body);
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "empty-allowed",
      "description": "p0 may be empty.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": ""
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Passed);
}

#[test]
fn text_equals_fails_on_mismatch() {
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "first-eq",
      "description": "p0 must equal a different text.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Different text."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Failed);
    assert_eq!(report.checks[0].evidence.as_deref(), Some("Hello world."));
}

#[test]
fn text_contains_passes_when_text_occurs() {
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "p1-contains",
      "description": "p1 must contain 'paragraph'.",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "paragraph"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Passed);
}

#[test]
fn text_absent_passes_when_text_is_not_there() {
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "p0-no-greeting",
      "description": "p0 must not contain 'Goodbye'.",
      "checks": [
        {{
          "kind": "text_absent",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Goodbye"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Passed);
}

#[test]
fn text_absent_refuses_when_target_vanished() {
    // A vanished paragraph is not a passing absence (proposal §10,
    // issue #937). The identity correspondence has no entry for the
    // missing target, so the check refuses.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "p5-absent",
      "description": "p5 (which does not exist) must not contain anything.",
      "checks": [
        {{
          "kind": "text_absent",
          "target": {{ "short_id": "p5", "long_id": "document.w:p.j:short.p5" }},
          "text": "anything"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn text_contains_refuses_when_target_vanished() {
    // Same refusal: a missing mapped paragraph is not a passing
    // containment check either.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "p5-contains",
      "description": "p5 (missing) must contain text.",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p5", "long_id": "document.w:p.j:short.p5" }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn text_equals_refuses_when_target_vanished() {
    // A missing mapped paragraph is not a passing equality either.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "p5-equals",
      "description": "p5 (missing) must equal text.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p5", "long_id": "document.w:p.j:short.p5" }},
          "text": "anything"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn merged_target_refuses() {
    // Build a custom correspondence that maps p0 to BOTH p0 and p1 in the
    // output. The initial profile requires exact one-to-one correspondence
    // (issue #937 / proposal §10): a target mapped to multiple paragraphs
    // refuses.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "merged",
      "description": "p0 of source mapped ambiguously.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let correspondence = jubarte::expect::SourceToOutputCorrespondence::from_entries(vec![
        jubarte::expect::CorrespondenceEntry::merged("p0", &[0, 1]),
    ])
    .expect("distinct short ids");
    let report = evaluate(&spec, &bytes, &bytes, &correspondence);
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn split_target_refuses() {
    // A source target mapped to an empty set of output paragraphs is a
    // split (or vanished): refuses, not passes.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "split",
      "description": "p0 of source split into nothing.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let correspondence = jubarte::expect::SourceToOutputCorrespondence::from_entries(vec![
        jubarte::expect::CorrespondenceEntry::split("p0"),
    ])
    .expect("distinct short ids");
    let report = evaluate(&spec, &bytes, &bytes, &correspondence);
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn source_hash_mismatch_refuses() {
    // The expectations' source.sha256 must name the bytes the
    // correspondence is built from (#937, #1047).
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{wrong_hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "x",
      "description": "p0 must equal Hello world.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        wrong_hash = "0".repeat(64)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
}

#[test]
fn natural_language_requirement_is_unassessed() {
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "natural",
      "description": "The doc should look polished.",
      "checks": []
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    // Unassessed: listed apart from the checks, with no fabricated
    // kind, and nothing ran.
    assert!(
        report.checks.is_empty(),
        "no pseudo-check: {:?}",
        report.checks
    );
    assert_eq!(report.unassessed_requirements, ["natural".to_string()]);
    assert_eq!(report.receipt.requirements_unassessed, 1);
    assert_eq!(report.receipt.checks_run, 0);
    assert!(report.receipt.targets.is_empty());
}

#[test]
fn parser_rejects_unknown_top_level_field() {
    // Closed shape: deny_unknown_fields. An extra field at the root is
    // refused by the parser.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [],
  "extra": "not allowed"
}}"#,
        hash = source_sha256(&bytes)
    ));
    assert!(
        spec.is_err(),
        "expected unknown field to be refused: {spec:?}"
    );
}

#[test]
fn parser_rejects_unknown_field_in_a_check() {
    // Closed shape: a check with an extra field is refused.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world.",
          "extra": "no"
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ));
    assert!(
        spec.is_err(),
        "expected unknown check field to be refused: {spec:?}"
    );
}

#[test]
fn parser_rejects_non_lowercase_hex_sha256() {
    // The schema's hash pattern is `^[0-9a-f]{64}$`: uppercase hex is
    // refused.
    let bytes = two_paragraph_docx();
    let bad_hash = source_sha256(&bytes).to_uppercase();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{bad_hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": []
    }}
  ]
}}"#
    ));
    assert!(
        spec.is_err(),
        "expected uppercase hex to be refused: {spec:?}"
    );
}

#[test]
fn parser_rejects_hash_with_trailing_newline() {
    // The schema's hash pattern is anchored: a trailing newline is
    // refused. (#968, schema precision.)
    let spec = parse_expectations(
        "{\n  \"schema_version\": 1,\n  \"source\": { \"sha256\": \"0000000000000000000000000000000000000000000000000000000000000000\\n\" },\n  \"text_view\": \"final\",\n  \"requirements\": [\n    { \"key\": \"k\", \"description\": \"d\", \"checks\": [] }\n  ]\n}\n",
    );
    assert!(spec.is_err());
}

#[test]
fn parser_rejects_hash_shorter_than_64() {
    // Exact 64 characters, no more, no less.
    let spec = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "00000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [
    { "key": "k", "description": "d", "checks": [] }
  ]
}"#,
    );
    assert!(spec.is_err());
}

#[test]
fn parser_rejects_empty_requirements() {
    // The schema requires minItems: 1 on requirements.
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": []
}"#,
    );
    assert!(result.is_err());
}

#[test]
fn parser_rejects_text_contains_empty_text() {
    // text_contains and text_absent require minLength: 1 on `text`.
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [
    {
      "key": "k",
      "description": "d",
      "checks": [
        {
          "kind": "text_contains",
          "target": { "short_id": "p0", "long_id": "document.w:p.j:short.p0" },
          "text": ""
        }
      ]
    }
  ]
}"#,
    );
    assert!(result.is_err());
}

#[test]
fn parser_rejects_unknown_check_kind() {
    // The closed `oneOf` allows only the three typed check kinds.
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [
    {
      "key": "k",
      "description": "d",
      "checks": [
        {
          "kind": "text_starts_with",
          "target": { "short_id": "p0", "long_id": "document.w:p.j:short.p0" },
          "text": "x"
        }
      ]
    }
  ]
}"#,
    );
    assert!(result.is_err());
}

#[test]
fn parser_rejects_text_view_other_than_final() {
    // The contract is `text_view: "final"` only at this stage (proposal
    // §10).
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "tracked",
  "requirements": [
    {
      "key": "k",
      "description": "d",
      "checks": []
    }
  ]
}"#,
    );
    assert!(result.is_err());
}

#[test]
fn parser_rejects_key_with_whitespace() {
    // Keys are `[A-Za-z][A-Za-z0-9_.-]*` with no whitespace.
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [
    {
      "key": "two words",
      "description": "d",
      "checks": []
    }
  ]
}"#,
    );
    assert!(result.is_err());
}

#[test]
fn parser_rejects_duplicate_keys() {
    // The schema's $comment calls for a "compiler check" on unique keys.
    let result = parse_expectations(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [
    {
      "key": "k",
      "description": "first",
      "checks": []
    },
    {
      "key": "k",
      "description": "second",
      "checks": []
    }
  ]
}"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("duplicate requirement key"));
}

#[test]
fn examples_file_parses_with_closed_shape() {
    // The example file the design ships parses without error. Parser and
    // schema file agree on this example (V2-13 acceptance).
    let parsed = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse example");
    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.text_view, jubarte::expect::TextView::Final);
    assert_eq!(parsed.requirements.len(), 3);
}

#[test]
fn receipt_records_path_digest_and_schema_version() {
    // #1017 / proposal §10: a receipt records path, digest, schema
    // version, checks run, targets, declared provenance. No claim of
    // independence or completeness.
    let bytes = two_paragraph_docx();
    let expectations_json = format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    );
    let spec = parse_expectations(&expectations_json).expect("parse");
    let report = jubarte::expect::evaluate_with_receipt(
        &spec,
        &bytes,
        &bytes,
        &identity_correspondence(&bytes),
        jubarte::expect::ReceiptMeta {
            path: Some("/tmp/expectations.json".to_string()),
            digest: Some(jubarte::expect::content_sha256(
                expectations_json.as_bytes(),
            )),
            declared_provenance: None,
        },
    );
    let receipt = &report.receipt;
    assert_eq!(receipt.schema_version, 1);
    assert_eq!(
        receipt.expectations_sha256.as_deref(),
        Some(jubarte::expect::content_sha256(expectations_json.as_bytes()).as_str())
    );
    assert_eq!(
        receipt.expectations_path.as_deref(),
        Some("/tmp/expectations.json")
    );
    assert_eq!(receipt.source_sha256, source_sha256(&bytes));
    assert_eq!(receipt.text_view, jubarte::expect::TextView::Final);
    assert_eq!(receipt.checks_run, 1);
    assert_eq!(receipt.checks_refused, 0);
    assert_eq!(receipt.checks_unsupported, 0);
    assert_eq!(receipt.requirements_unassessed, 0);
    assert_eq!(receipt.targets, vec!["p0".to_string()]);
    assert!(!receipt.independent);
    assert!(!receipt.complete_task);
}

#[test]
fn declared_provenance_is_carried_through() {
    // #950: a separately supplied expectations file is not a claim of
    // independence. The caller can name a provenance string; the receipt
    // carries it verbatim without vouching for it.
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate_with_provenance(
        &spec,
        &bytes,
        &bytes,
        &identity_correspondence(&bytes),
        "lawyer-supplied".to_string(),
    );
    assert_eq!(
        report.receipt.declared_provenance.as_deref(),
        Some("lawyer-supplied")
    );
    assert!(!report.receipt.independent);
}

fn evaluate_with_provenance(
    spec: &ExpectationsV1,
    source: &[u8],
    output: &[u8],
    correspondence: &jubarte::expect::SourceToOutputCorrespondence,
    provenance: String,
) -> jubarte::expect::ExpectationReport {
    jubarte::expect::evaluate_with_receipt(
        spec,
        source,
        output,
        correspondence,
        jubarte::expect::ReceiptMeta {
            declared_provenance: Some(provenance),
            ..Default::default()
        },
    )
}

#[test]
fn unsupported_target_kind_refuses() {
    // A check whose target names a table, not a paragraph, is unsupported
    // by the initial profile. Review row 5: the target-kind check must
    // fire before the correspondence lookup, so a non-paragraph target
    // refuses `Unsupported` even when no correspondence entry carries it
    // (before, the lookup missed first and the refusal said
    // `TargetMissing`, leaving the Unsupported branch dead).
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "t0", "long_id": "document.w:t.j:short.t0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(
        report.checks[0].reason,
        Some(RefusalReason::Unsupported),
        "a non-paragraph target refuses as unsupported, not as missing"
    );
}

#[test]
fn a_passing_typed_check_does_not_mark_its_requirement_satisfied() {
    // #1048 / proposal §10: a typed check passing is one check, not
    // the whole requirement. The report exposes per-check outcomes and
    // a separate unassessed listing, and nothing else: a requirement
    // with a passing check leaves the unassessed listing (it was
    // assessed, by exactly that check), a requirement with no checks
    // enters it, and no field collapses the checks into a satisfied
    // requirement (review row 13).
    let bytes = two_paragraph_docx();
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "ambitious",
      "description": "Should also be commercially appropriate.",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }},
    {{
      "key": "natural",
      "description": "The doc should look polished.",
      "checks": []
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    let check = &report.checks[0];
    assert_eq!(check.outcome, Outcome::Passed);
    assert_eq!(check.key, "ambitious");
    let _: &CheckOutcome = check;
    // Exactly the one typed check ran, and only the check-less
    // requirement is unassessed: the passing check assessed its
    // requirement's typed predicate without claiming the requirement
    // satisfied.
    assert_eq!(report.receipt.checks_run, 1);
    assert_eq!(report.unassessed_requirements, ["natural".to_string()]);
    assert_eq!(report.receipt.requirements_unassessed, 1);
}

// ---------------------------------------------------------------------------
// Parity between `parse_expectations` and the bundled JSON Schema.
//
// `parse_expectations` enforces every constraint the schema states in plain
// Rust so the WASM and Python lockfiles do not pull in `jsonschema` at
// runtime. The bundled schema (see `bundled_schema_is_loaded_from_disk_in_tests`
// for where it comes from) is the contract; the Rust parser must accept
// what the schema accepts and reject what the schema rejects. The corpus
// test below runs every shared positive/negative case through BOTH the
// Rust parser and the schema validator and asserts they agree. Two
// document classes are parser-only refusals, pinned by their own test:
// `schema_version` written as a float (`1.0`), and a JSON object with a
// duplicated field.
// ---------------------------------------------------------------------------

const BUNDLED_SCHEMA_PATH: &str = "schemas/expectations-v1.schema.json";

fn schema_validator() -> jsonschema::Validator {
    let schema_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), BUNDLED_SCHEMA_PATH);
    let schema_text = std::fs::read_to_string(&schema_path).unwrap_or_else(|e| {
        panic!("bundled expectations JSON Schema at {schema_path} is unreadable: {e}")
    });
    let schema_value: serde_json::Value =
        serde_json::from_str(&schema_text).expect("bundled schema is valid JSON");
    jsonschema::validator_for(&schema_value).expect("bundled schema builds a validator")
}

/// Run `document` through both `parse_expectations` and the bundled JSON
/// Schema validator, and assert they agree on accept/reject. `expect_accepted`
/// is what the schema says; the assertion fails if either side disagrees.
fn assert_parity(document: &str, expect_accepted: bool, label: &str) {
    let validator = schema_validator();
    let value: serde_json::Value = serde_json::from_str(document)
        .unwrap_or_else(|e| panic!("[{label}] document is not valid JSON: {e}"));
    let schema_says = validator.is_valid(&value);
    let parser_says = parse_expectations(document).is_ok();
    assert_eq!(
        schema_says, expect_accepted,
        "[{label}] bundled schema disagreed with expected accept={expect_accepted}"
    );
    assert_eq!(
        parser_says, expect_accepted,
        "[{label}] parse_expectations disagreed with expected accept={expect_accepted}"
    );
    assert_eq!(
        schema_says, parser_says,
        "[{label}] parse_expectations and the bundled schema disagreed: \
         parser_ok={parser_says}, schema_ok={schema_says}"
    );
}

#[test]
fn parse_expectations_and_bundled_schema_agree_on_the_corpus() {
    // POSITIVE cases: documents the parser must accept, and the bundled
    // schema must accept. The design example is the canonical one; the
    // others are the typed-check shapes the parser must accept too.

    // 1. The design example as shipped in `examples/expectations.json`.
    assert_parity(EXAMPLE_EXPECTATIONS, true, "design example");

    // 2. text_equals with non-empty text (basic shape).
    let bytes = two_paragraph_docx();
    let hash = source_sha256(&bytes);
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }}
  ]
}}"#
        ),
        true,
        "text_equals non-empty text",
    );

    // 3. text_equals with EMPTY text (schema allows empty on text_equals).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": ""
        }}
      ]
    }}
  ]
}}"#
        ),
        true,
        "text_equals empty text",
    );

    // 4. text_contains with non-empty text.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "paragraph"
        }}
      ]
    }}
  ]
}}"#
        ),
        true,
        "text_contains non-empty text",
    );

    // 5. text_absent with non-empty text.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_absent",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Goodbye"
        }}
      ]
    }}
  ]
}}"#
        ),
        true,
        "text_absent non-empty text",
    );

    // 6. Empty checks array (unassessed requirement). Valid per the
    //    schema: `checks` is required but its items are not.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": "d", "checks": [] }}
  ]
}}"#
        ),
        true,
        "empty checks list",
    );

    // 7. Multiple requirements with multiple distinct check kinds.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "first",
      "description": "first",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "Hello world."
        }}
      ]
    }},
    {{
      "key": "second",
      "description": "second",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "paragraph"
        }},
        {{
          "kind": "text_absent",
          "target": {{ "short_id": "p1", "long_id": "document.w:p.j:short.p1" }},
          "text": "missing"
        }}
      ]
    }}
  ]
}}"#
        ),
        true,
        "multiple requirements and mixed check kinds",
    );

    // NEGATIVE cases: documents the parser must reject, and the bundled
    // schema must reject. The disagreement would mean the parser is
    // missing a constraint or the schema drifted from the Rust type.

    // 8. Unknown top-level field (closed shape).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}],
  "extra": "not allowed"
}}"#
        ),
        false,
        "unknown top-level field",
    );

    // 9. Unknown field inside a check (closed shape).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "x",
          "extra": "no"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "unknown field inside a check",
    );

    // 10. Unknown field inside the source object (closed shape).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}", "path": "/tmp/x.docx" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "unknown field inside source",
    );

    // 11. schema_version != 1.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 2,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "schema_version != 1",
    );

    // 12. UPPERCASE hex sha256.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{}" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#,
            hash.to_uppercase()
        ),
        false,
        "uppercase hex sha256",
    );

    // 13. Trailing newline on the hash (anchored pattern).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}\n" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "hash with trailing newline",
    );

    // 14. sha256 shorter than 64 chars.
    assert_parity(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "00000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [ { "key": "k", "description": "d", "checks": [] } ]
}"#,
        false,
        "sha256 shorter than 64",
    );

    // 15. sha256 longer than 64 chars.
    assert_parity(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "000000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [ { "key": "k", "description": "d", "checks": [] } ]
}"#,
        false,
        "sha256 longer than 64",
    );

    // 16. sha256 containing a non-hex char.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{}" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#,
            "z".repeat(64)
        ),
        false,
        "sha256 with non-hex character",
    );

    // 17. Empty requirements array (minItems: 1).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": []
}}"#
        ),
        false,
        "empty requirements array",
    );

    // 18. text_contains with EMPTY text (the schema's per-kind minLength).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_contains",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": ""
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "text_contains with empty text",
    );

    // 19. text_absent with EMPTY text.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_absent",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": ""
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "text_absent with empty text",
    );

    // 20. Unknown check kind (closed oneOf).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_starts_with",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "unknown check kind",
    );

    // 21. text_view other than "final".
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "tracked",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "text_view other than final",
    );

    // 22. Requirement key containing whitespace (regex `not` clause).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "two words", "description": "d", "checks": [] }}
  ]
}}"#
        ),
        false,
        "requirement key with whitespace",
    );

    // 23. Requirement key starting with a digit (regex `^[A-Za-z]...`).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "1leading", "description": "d", "checks": [] }}
  ]
}}"#
        ),
        false,
        "requirement key starting with digit",
    );

    // 24. Duplicate requirement keys (the schema's `$comment` "compiler
    //     check": `uniqueItems` does not express it for object keys, so
    //     the parser enforces it directly. The bundled schema itself
    //     does NOT reject this case, so we don't add it to the parity
    //     set; the parser-enforced rule is covered by the dedicated
    //     `parser_rejects_duplicate_keys` test above.)

    // 25. Empty description (minLength: 1).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": "", "checks": [] }}
  ]
}}"#
        ),
        false,
        "empty description",
    );

    // 26. Empty target.short_id.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "", "long_id": "document.w:p.j:short.p0" }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "empty target.short_id",
    );

    // 27. Empty target.long_id.
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "" }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "empty target.long_id",
    );

    // 28. Missing `description` on a requirement (required field).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "checks": [] }}
  ]
}}"#
        ),
        false,
        "missing description",
    );

    // 29. Missing `checks` on a requirement (required field).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": "d" }}
  ]
}}"#
        ),
        false,
        "missing checks",
    );

    // 30. Missing top-level `source` (required field).
    assert_parity(
        r#"{
  "schema_version": 1,
  "text_view": "final",
  "requirements": [ { "key": "k", "description": "d", "checks": [] } ]
}"#,
        false,
        "missing source",
    );

    // 31-35. Wrong JSON types and null values, so the `type` keyword is
    //        the deciding keyword for each field (review row 9).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": 5, "checks": [] }}
  ]
}}"#
        ),
        false,
        "description with a number type",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": "d", "checks": "x" }}
  ]
}}"#
        ),
        false,
        "checks with a string type",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": 12
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "check text with a number type",
    );

    assert_parity(
        r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": {}
}"#,
        false,
        "requirements with an object type",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{ "key": "k", "description": null, "checks": [] }}
  ]
}}"#
        ),
        false,
        "null description",
    );

    // 36. Unknown field inside the target object (closed shape at the
    //     `target` level, exercising `additionalProperties` there).
    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0", "extra": 1 }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "unknown field inside target",
    );

    // 37-42. Missing `required` fields not covered by cases 28-30
    //        (review row 9).
    assert_parity(
        &format!(
            r#"{{
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "missing schema_version",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "requirements": [{{ "key": "k", "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "missing text_view",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [{{ "description": "d", "checks": [] }}]
}}"#
        ),
        false,
        "missing requirement key",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }},
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "missing check kind",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "target": {{ "short_id": "p0", "long_id": "document.w:p.j:short.p0" }}
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "missing check text",
    );

    assert_parity(
        &format!(
            r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "k",
      "description": "d",
      "checks": [
        {{
          "kind": "text_equals",
          "text": "x"
        }}
      ]
    }}
  ]
}}"#
        ),
        false,
        "missing check target",
    );
}

#[test]
fn parser_is_stricter_than_the_bundled_schema_on_two_pinned_divergences() {
    // Review row 8: two document classes are refused by the parser and
    // accepted by the bundled schema. The direction is the safe one
    // (the parser is never laxer), and this test pins both divergences
    // explicitly instead of letting the corpus test's name overstate
    // "every document".
    let validator = schema_validator();

    // (a) `"schema_version": 1.0` -- JSON Schema `const` compares
    // numbers by mathematical equality (draft 2020-12), so the schema
    // accepts 1.0 for `const: 1`; serde cannot deserialize a float
    // into the `u32` field.
    let float_version = r#"{
  "schema_version": 1.0,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [ { "key": "k", "description": "d", "checks": [] } ]
}"#;
    let value: serde_json::Value = serde_json::from_str(float_version).expect("valid JSON");
    assert!(
        validator.is_valid(&value),
        "the schema's numeric const accepts 1.0"
    );
    assert!(
        parse_expectations(float_version).is_err(),
        "the parser refuses a float schema_version"
    );

    // (b) A JSON object with a duplicated field: the schema side reads
    // `serde_json::Value` (last value wins), the derived Deserialize
    // errors on the duplicate field.
    let duplicate_key = r#"{
  "schema_version": 1,
  "source": { "sha256": "0000000000000000000000000000000000000000000000000000000000000000" },
  "text_view": "final",
  "requirements": [ { "key": "k", "key": "k", "description": "d", "checks": [] } ]
}"#;
    let value: serde_json::Value = serde_json::from_str(duplicate_key).expect("valid JSON");
    assert!(
        validator.is_valid(&value),
        "last-value-wins Value parsing validates the duplicate field"
    );
    assert!(
        parse_expectations(duplicate_key).is_err(),
        "the parser refuses a duplicated JSON field"
    );
}

#[test]
fn bundled_schema_is_loaded_from_disk_in_tests() {
    // The bundled schema file ships as the production parsing contract,
    // following the `$id`/`$comment` convention of
    // schemas/edit-plan-v2.schema.json, and is derived from the design
    // artifact docs/design/cli-v2/jubarte-expectations-v1.schema.json;
    // only the `$id`, title and `$comment` metadata differ, every
    // constraint is the design's. It is preserved so this parity test, and any external
    // validator, can check that `parse_expectations` and the schema
    // agree. The runtime crate does not.
    let schema_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), BUNDLED_SCHEMA_PATH);
    let schema_text = std::fs::read_to_string(&schema_path)
        .expect("bundled schema file exists at schemas/expectations-v1.schema.json");
    assert!(
        schema_text.contains("\"$id\": \"urn:jubarte:expectations-v1:paragraph-text\""),
        "bundled schema follows the shipped $id convention"
    );
    assert!(
        schema_text.contains("docs/design/cli-v2/jubarte-expectations-v1.schema.json"),
        "bundled schema cites the design artifact it derives from"
    );
}

// ---------------------------------------------------------------------------
// V2-13 wiring (issue #845): the correspondence the apply path builds.
// ---------------------------------------------------------------------------

use jubarte::edit::{PrepareOptions, prepare_v2, v2::PlanV2};

fn prepare_plan(source: &[u8], plan_json: &str) -> jubarte::edit::PreparedEdit {
    let plan = PlanV2::from_json(plan_json).expect("plan parses");
    prepare_v2(source, &plan, &PrepareOptions::default()).expect("prepare")
}

fn correspondence_of(
    prepared: &jubarte::edit::PreparedEdit,
) -> &jubarte::expect::SourceToOutputCorrespondence {
    prepared
        .correspondence
        .as_ref()
        .expect("v2 carries a correspondence")
}

#[test]
fn apply_correspondence_maps_source_targets_into_the_chosen_primary() {
    // The design example: template.edited.json rewrites p0 and p1 and
    // inserts a paragraph after p0, tracked. The chosen primary is the
    // tracked redline; its final projection holds five paragraphs, so the
    // source's p0 stays at output 0 and p1 shifts to 2 — exactly what the
    // apply path knows from the transaction, not from re-matching text.
    let prepared = prepare_plan(
        EXAMPLE_SOURCE,
        include_str!("fixtures/edit-v2/template.edited.json"),
    );
    let correspondence = correspondence_of(&prepared);
    for (id, output) in [("p0", 0), ("p1", 2), ("p2", 3), ("p3", 4)] {
        let entry = correspondence.lookup(id).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(
            entry.output,
            [output],
            "{id} must map one-to-one to output paragraph {output}"
        );
    }
    let parsed = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse expectations");
    let report = evaluate(&parsed, EXAMPLE_SOURCE, &prepared.primary, correspondence);
    assert!(!report.has_blocking_failure(), "{:?}", report.checks);
    assert_eq!(report.receipt.checks_run, 3);
    assert_eq!(report.receipt.requirements_unassessed, 1);
    let payment = report
        .checks
        .iter()
        .find(|c| c.key == "payment-period")
        .expect("payment check ran");
    // The chosen-primary final text: 45 days, judged on the reopened
    // tracked redline's final projection.
    assert_eq!(payment.outcome, Outcome::Passed);
    assert_eq!(
        payment.evidence.as_deref(),
        Some("Payment is due within 45 days.")
    );
}

#[test]
fn unchanged_primary_evaluates_identity_and_reports_the_missing_change() {
    // The generated template is an all-unchanged scaffold: the primary is
    // the exact source bytes, so the identity correspondence over the
    // source is the output walk. The expectations honestly FAIL on it (the
    // 45-day period was never made); an unchanged result is never a pass.
    let prepared = prepare_plan(
        EXAMPLE_SOURCE,
        include_str!("fixtures/edit-v2/template.generated.json"),
    );
    assert_eq!(prepared.primary, EXAMPLE_SOURCE);
    let correspondence = correspondence_of(&prepared);
    let identity = identity_correspondence(EXAMPLE_SOURCE);
    assert_eq!(
        correspondence.entries(),
        identity.entries(),
        "an unchanged primary's correspondence is the identity over the source"
    );
    let parsed = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse expectations");
    let report = evaluate(&parsed, EXAMPLE_SOURCE, &prepared.primary, correspondence);
    assert!(report.has_blocking_failure());
    assert_eq!(report.receipt.checks_run, 3);
    let payment = report
        .checks
        .iter()
        .find(|c| c.key == "payment-period")
        .expect("payment check ran");
    assert_eq!(payment.outcome, Outcome::Failed);
    assert_eq!(
        payment.evidence.as_deref(),
        Some("Payment is due within 30 days.")
    );
}

#[test]
fn directly_deleted_paragraph_target_refuses_split() {
    // Track off: delete_paragraph removes the node from the primary, so
    // the source paragraph maps to nothing and every check on it refuses
    // Split — a vanished paragraph never passes text_absent.
    let body = concat!(
        r#"<w:p><w:r><w:t>Alpha.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Beta.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Gamma.</w:t></w:r></w:p>"#,
    );
    let source = docx(body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "off", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "delete_paragraph", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}}}
  ]
}}"#
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    for (id, output) in [("p0", vec![0]), ("p1", Vec::new()), ("p2", vec![1])] {
        assert_eq!(
            correspondence.lookup(id).expect(id).output,
            output,
            "{id} maps through the deletion"
        );
    }
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "beta-gone",
      "description": "Beta removed.",
      "checks": [
        {{"kind": "text_absent", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "text": "Beta"}}
      ]
    }}
  ]
}}"#
    ))
    .expect("parse");
    let report = evaluate(&spec, &source, &prepared.primary, correspondence);
    assert!(report.has_blocking_failure());
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(report.checks[0].reason, Some(RefusalReason::Split));
}

#[test]
fn tracked_deleted_paragraph_target_refuses_under_the_final_view() {
    // Track on: delete_paragraph keeps the `<w:p>` in the redline with
    // every run deleted and a deleted mark, so the final view holds no
    // paragraph of its own. The correspondence must refuse the target —
    // a vanished paragraph never passes text_absent (#1084 review F2,
    // proposal §10) — instead of mapping it one-to-one to the empty
    // projection slot.
    let body = concat!(
        r#"<w:p><w:r><w:t>Alpha.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Beta.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Gamma.</w:t></w:r></w:p>"#,
    );
    let source = docx(body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "on", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "delete_paragraph", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}}}
  ]
}}"#
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    for (id, output) in [("p0", vec![0]), ("p1", Vec::new()), ("p2", vec![2])] {
        assert_eq!(
            correspondence.lookup(id).expect(id).output,
            output,
            "{id} maps through the tracked deletion's final view"
        );
    }
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "beta-gone",
      "description": "Beta removed.",
      "checks": [
        {{"kind": "text_absent", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "text": "Beta"}}
      ]
    }}
  ]
}}"#
    ))
    .expect("parse");
    let report = evaluate(&spec, &source, &prepared.primary, correspondence);
    assert!(report.has_blocking_failure());
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(report.checks[0].reason, Some(RefusalReason::Split));
}

#[test]
fn tracked_insertion_beside_an_untouched_target_stays_one_to_one() {
    // The mirror guard: an inserted paragraph's mark (`w:pPr/w:rPr/w:ins`)
    // exists in the final view, so the untouched neighbours around it keep
    // their exact one-to-one mapping and their checks evaluate (#1084
    // review F2).
    let body = concat!(
        r#"<w:p><w:r><w:t>Alpha.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Beta.</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Gamma.</w:t></w:r></w:p>"#,
    );
    let source = docx(body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "on", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "insert_paragraph", "target": {{"short_id": "p0", "long_id": "document.w:p.j:short.p0"}}, "position": "after", "text": "Inserted."}}
  ]
}}"#
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    for (id, output) in [("p0", 0), ("p1", 2), ("p2", 3)] {
        assert_eq!(
            correspondence.lookup(id).expect(id).output,
            [output],
            "{id} stays one-to-one beside the tracked insertion"
        );
    }
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "beta-kept",
      "description": "Beta untouched.",
      "checks": [
        {{"kind": "text_equals", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "text": "Beta."}}
      ]
    }}
  ]
}}"#
    ))
    .expect("parse");
    let report = evaluate(&spec, &source, &prepared.primary, correspondence);
    assert!(!report.has_blocking_failure(), "{:?}", report.checks);
    assert_eq!(report.checks[0].outcome, Outcome::Passed);
    assert_eq!(report.checks[0].evidence.as_deref(), Some("Beta."));
}

#[test]
fn kept_deleted_mark_with_surviving_text_merges_into_the_next_and_refuses() {
    // A source-side deleted paragraph mark kept under `keep`, with content
    // that survives the final view, merges into the next paragraph there:
    // both source paragraphs share one final paragraph and refuse merged,
    // even beside an unrelated tracked body edit (#1084 review F2).
    const REV: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
    let body = format!(
        concat!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>Beta</w:t></w:r></w:p>"#,
        ),
        REV = REV,
    );
    let source = docx(&body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "on", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "rewrite", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "before_sha256": "{beta}", "text": "Beta revised."}}
  ]
}}"#,
        beta = jubarte::expect::content_sha256("Beta".as_bytes()),
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    for id in ["p0", "p1"] {
        let entry = correspondence.lookup(id).expect(id);
        assert_eq!(
            entry.output.len(),
            2,
            "{id} refuses as merged: {:?}",
            entry.output
        );
    }
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "beta-wording",
      "description": "Beta reworded.",
      "checks": [
        {{"kind": "text_equals", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "text": "Beta revised."}}
      ]
    }}
  ]
}}"#
    ))
    .expect("parse");
    let report = evaluate(&spec, &source, &prepared.primary, correspondence);
    assert!(report.has_blocking_failure());
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(report.checks[0].reason, Some(RefusalReason::Merged));
}

#[test]
fn kept_deleted_mark_merging_into_an_insertion_still_refuses() {
    // The merge chain lands on the first follower without a deleted mark,
    // which an inserted paragraph can be: the marked source paragraph
    // still refuses merged (its final paragraph is not its own), while
    // the paragraph after the insertion keeps its one-to-one mapping
    // (#1084 review F2).
    const REV: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
    let body = format!(
        concat!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>Beta</w:t></w:r></w:p>"#,
        ),
        REV = REV,
    );
    let source = docx(&body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "on", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "insert_paragraph", "target": {{"short_id": "p0", "long_id": "document.w:p.j:short.p0"}}, "position": "after", "text": "Inserted."}}
  ]
}}"#
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    let marked = correspondence.lookup("p0").expect("p0");
    assert_ne!(
        marked.output.len(),
        1,
        "the merge-marked paragraph refuses: {:?}",
        marked.output
    );
    assert_eq!(
        correspondence.lookup("p1").expect("p1").output,
        [2],
        "p1 stays one-to-one past the insertion"
    );
}

#[test]
fn rebound_output_copy_targets_output_ids_and_round_trips() {
    // The apply-side receipt copy: the edited primary's own digest, p1
    // rebound to the output's p2, and the copy is a schema-valid
    // expectations document that standalone check semantics accept on the
    // published bytes (identity correspondence).
    let prepared = prepare_plan(
        EXAMPLE_SOURCE,
        include_str!("fixtures/edit-v2/template.edited.json"),
    );
    let parsed = parse_expectations(EXAMPLE_EXPECTATIONS).expect("parse expectations");
    let rebound =
        jubarte::expect::rebind_to_output(&parsed, correspondence_of(&prepared), &prepared.primary)
            .expect("rebind");
    assert_eq!(rebound.source.sha256, source_sha256(&prepared.primary));
    let targets: Vec<(String, String)> = rebound
        .requirements
        .iter()
        .flat_map(|r| {
            r.checks
                .iter()
                .map(|c| (c.target.short_id.clone(), c.target.long_id.clone()))
        })
        .collect();
    assert_eq!(
        targets,
        [
            ("p0".to_string(), "document.w:p.j:short.p0".to_string()),
            ("p2".to_string(), "document.w:p.j:short.p2".to_string()),
            ("p2".to_string(), "document.w:p.j:short.p2".to_string()),
        ]
    );
    // Round trip: the copy parses and evaluates clean on the primary.
    let json = serde_json::to_string(&rebound).expect("serializes");
    let reparsed = parse_expectations(&json).expect("rebound copy is schema-valid");
    let report = evaluate(
        &reparsed,
        &prepared.primary,
        &prepared.primary,
        &identity_correspondence(&prepared.primary),
    );
    assert!(!report.has_blocking_failure(), "{:?}", report.checks);
    assert_eq!(report.receipt.checks_run, 3);
    // The unassessed requirement travels as unassessed.
    assert_eq!(report.receipt.requirements_unassessed, 1);
}

#[test]
fn settlement_merged_target_refuses_merged() {
    // A deleted paragraph mark, accepted by the plan's policy, merges p0
    // into p1: two source paragraphs share one settled paragraph, so both
    // refuse as merged — the initial profile requires exact one-to-one
    // correspondence (proposal §10).
    const REV: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
    let body = format!(
        concat!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>Beta</w:t></w:r></w:p>"#,
        ),
        REV = REV,
    );
    let source = docx(&body);
    let hash = source_sha256(&source);
    let plan = format!(
        r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "on", "existing_revisions": "accept", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": []
}}"#
    );
    let prepared = prepare_plan(&source, &plan);
    let correspondence = correspondence_of(&prepared);
    for id in ["p0", "p1"] {
        let entry = correspondence.lookup(id).expect(id);
        assert_eq!(
            entry.output.len(),
            2,
            "{id} refuses as merged: {:?}",
            entry.output
        );
    }
    let spec = parse_expectations(&format!(
        r#"{{
  "schema_version": 1,
  "source": {{ "sha256": "{hash}" }},
  "text_view": "final",
  "requirements": [
    {{
      "key": "alpha",
      "description": "Alpha text.",
      "checks": [
        {{"kind": "text_contains", "target": {{"short_id": "p0", "long_id": "document.w:p.j:short.p0"}}, "text": "Alpha"}}
      ]
    }}
  ]
}}"#
    ))
    .expect("parse");
    let report = evaluate(&spec, &source, &prepared.primary, correspondence);
    assert!(report.has_blocking_failure());
    assert_eq!(report.checks[0].outcome, Outcome::Refused);
    assert_eq!(report.checks[0].reason, Some(RefusalReason::Merged));
}
