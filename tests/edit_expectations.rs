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

use common::docx::docx;
use jubarte::expect::{
    CheckKind, CheckOutcome, ExpectationsV1, Outcome, evaluate, parse_expectations,
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
    assert!(report.receipt.checks_run >= 3);
    // The commercial-review requirement is unassessed (its checks list is empty).
    let unassessed = report
        .checks
        .iter()
        .filter(|c| c.key == "commercial-review")
        .collect::<Vec<_>>();
    assert_eq!(unassessed.len(), 1);
    assert_eq!(unassessed[0].outcome, Outcome::Unassessed);
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
    ]);
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
    ]);
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
    let check = &report.checks[0];
    assert_eq!(check.outcome, Outcome::Unassessed);
    // A passing typed check on the same requirement is not a satisfied
    // requirement, but here there are no typed checks at all; the
    // requirement IS unassessed.
    assert_eq!(check.key, "natural");
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
        receipt.expectations_sha256,
        jubarte::expect::content_sha256(expectations_json.as_bytes())
    );
    assert_eq!(
        receipt.expectations_path.as_deref(),
        Some("/tmp/expectations.json")
    );
    assert_eq!(receipt.source_sha256, source_sha256(&bytes));
    assert_eq!(receipt.checks_run, 1);
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
    // by the initial profile. The parser accepts the shape (the
    // closed-object contract); the evaluator refuses because the target
    // does not resolve to a paragraph in the correspondence.
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
}

#[test]
fn a_passing_typed_check_does_not_mark_its_requirement_satisfied() {
    // #1048 / proposal §10: a typed check passing is one check, not
    // the whole requirement. The requirement's satisfaction is a
    // separate judgement, the caller can compute, over its typed checks
    // and the requirement's natural-language description. The report
    // carries per-check outcomes; it does not collapse them.
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
    }}
  ]
}}"#,
        hash = source_sha256(&bytes)
    ))
    .expect("parse");
    let report = evaluate(&spec, &bytes, &bytes, &identity_correspondence(&bytes));
    let check = &report.checks[0];
    assert_eq!(check.outcome, Outcome::Passed);
    // No `requirement_satisfied` is implied; the caller judges.
    assert_eq!(check.key, "ambitious");
    // Documents the field types the test file relies on.
    let _: &CheckOutcome = check;
}

// ---------------------------------------------------------------------------
// Parity between `parse_expectations` and the bundled JSON Schema.
//
// `parse_expectations` enforces every constraint the schema states in plain
// Rust so the WASM and Python lockfiles do not pull in `jsonschema` at
// runtime. The bundled schema (mirrored from `docs/design/cli-v2/`) is the
// contract; the Rust parser must accept what the schema accepts and reject
// what the schema rejects. This test exercises every positive/negative case
// used elsewhere in this file (plus the design example) through BOTH the
// Rust parser and the schema validator, and asserts they agree.
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
fn parse_expectations_and_bundled_schema_agree_on_every_document() {
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
}

#[test]
fn bundled_schema_is_loaded_from_disk_in_tests() {
    // The bundled schema file is preserved so this parity test, and any
    // external validator, can check that `parse_expectations` and the
    // schema agree. The runtime crate does not.
    let schema_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), BUNDLED_SCHEMA_PATH);
    let schema_text = std::fs::read_to_string(&schema_path)
        .expect("bundled schema file exists at schemas/expectations-v1.schema.json");
    assert!(
        schema_text.contains("\"$id\": \"urn:jubarte:proposal:expectations-v1\""),
        "bundled schema carries the design's id"
    );
}
