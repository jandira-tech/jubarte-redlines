// Scratch cost measurement for the PR #1689 review. NOT COMMITTED.
// Compiles on the base commit too (no SCRATCH_COUNTER here).

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

mod common;

use common::docx::docx;
use jubarte::edit::{EditPlan, PrepareOptions, prepare};
use std::time::Instant;

const DEFAULT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: false,
    role: jubarte::edit::PrimaryRole::Clean,
    syntax: jubarte::edit::PlanSyntax::V1,
    skip_failed: false,
};

fn plan_keep(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{operations}]}}"#
    ))
    .unwrap()
}

/// `n` paragraphs, each with a marked run of its own id.
fn source(n: usize) -> Vec<u8> {
    let mut body = String::new();
    for i in 0..n {
        body.push_str(&format!(
            r#"<w:p><w:r><w:rPr><w:ins w:id="{}" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>marked{i} run</w:t></w:r></w:p>"#,
            i + 1
        ));
    }
    docx(&body)
}

/// One operation per paragraph, each splitting its marked run: every
/// operation is refused on its own.
fn ops(n: usize) -> String {
    (0..n)
        .map(|i| {
            format!(
                r#"{{"kind":"replace","paragraph":"p{i}","find":"marked{i}","replacement":"M"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// `--skip-failed` over a plan whose every operation is refused by the
/// candidate checks.
#[test]
fn t1_skip_failed_many_failures() {
    for n in [1usize, 4, 16, 64] {
        let source = source(n);
        let p = plan_keep(&ops(n));
        let options = PrepareOptions {
            skip_failed: true,
            ..DEFAULT
        };
        let t = Instant::now();
        let prepared = prepare(&source, &p, &options);
        let elapsed = t.elapsed();
        println!(
            "T1 n={n} elapsed={elapsed:?} result={:?}",
            prepared
                .as_ref()
                .map(|p| p.skipped.len())
                .map_err(|e| e.code.clone())
        );
    }
}

/// The same plan without `--skip-failed`: one refusal.
#[test]
fn t2_refusal_without_skip() {
    for n in [1usize, 4, 16, 64] {
        let source = source(n);
        let p = plan_keep(&ops(n));
        let t = Instant::now();
        let refused = prepare(&source, &p, &DEFAULT);
        let elapsed = t.elapsed();
        println!(
            "T2 n={n} elapsed={elapsed:?} err={:?}",
            refused.as_ref().err().map(|e| e.code.clone())
        );
    }
}
