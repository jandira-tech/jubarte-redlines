// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Criterion benchmarks: exact wall-time of redline creation
//! (`document_comparer::compare_documents`) on real fixture pairs.
//!
//! Run with `cargo bench` — reports mean/median with confidence intervals and
//! tracks regressions across runs under `target/criterion/`. Pairs whose
//! fixtures are absent (e.g. from a published crate, where `tests/` is not
//! packaged) are skipped with a note rather than failing.

use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use jubarte::document_comparer::compare_documents;

/// (bench id, original, modified) — spanning the size/shape spectrum:
/// dense-edit canonical pair (small bytes, edits in nearly every paragraph —
/// the engine's worst-case shape), short-into-long big doc, table-heavy
/// batch pair, and a comment-heavy fresh pair.
const PAIRS: &[(&str, &str, &str)] = &[
    (
        "canonical_dense_edits",
        "tests/fixtures/redline/original.docx",
        "tests/fixtures/redline/modified.docx",
    ),
    (
        "short_into_long",
        "tests/corpus/broken_ones_two/sources/file_130.docx",
        "tests/corpus/broken_ones_two/sources/file_131.docx",
    ),
    (
        "tables_bookmark_vmerge",
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx",
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx",
    ),
    (
        "comment_heavy",
        "tests/corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition.docx",
        "tests/corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition_removal.docx",
    ),
];

fn bench_compare_documents(c: &mut Criterion) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // P0-LAB-01: split short statistical cases from slower trials so Criterion
    // sampling stays practical. Multi-minute wall claims use tools/perf/* ABBA.
    let short: &[&str] = &["canonical_dense_edits"];
    let slow: &[&str] = &["short_into_long", "tables_bookmark_vmerge", "comment_heavy"];

    let mut fast = c.benchmark_group("compare_documents_fast");
    fast.sample_size(20)
        .measurement_time(Duration::from_secs(8));
    for (id, a_rel, b_rel) in PAIRS {
        if !short.contains(id) {
            continue;
        }
        let (a_path, b_path) = (root.join(a_rel), root.join(b_rel));
        if !a_path.is_file() || !b_path.is_file() {
            eprintln!("skip {id}: fixtures not present ({a_rel})");
            continue;
        }
        // Fixture I/O is outside the measured closure (setup-once).
        let a = std::fs::read(&a_path).expect("read original");
        let b = std::fs::read(&b_path).expect("read modified");
        fast.bench_function(*id, |bencher| {
            bencher.iter(|| {
                let out =
                    compare_documents(black_box(&a), black_box(&b), "Bench").expect("compare");
                black_box(out)
            });
        });
    }
    fast.finish();

    let mut slow_g = c.benchmark_group("compare_documents_slow");
    slow_g
        .sample_size(10)
        .measurement_time(Duration::from_secs(15));
    for (id, a_rel, b_rel) in PAIRS {
        if !slow.contains(id) {
            continue;
        }
        let (a_path, b_path) = (root.join(a_rel), root.join(b_rel));
        if !a_path.is_file() || !b_path.is_file() {
            eprintln!("skip {id}: fixtures not present ({a_rel})");
            continue;
        }
        let a = std::fs::read(&a_path).expect("read original");
        let b = std::fs::read(&b_path).expect("read modified");
        slow_g.bench_function(*id, |bencher| {
            bencher.iter(|| {
                let out =
                    compare_documents(black_box(&a), black_box(&b), "Bench").expect("compare");
                black_box(out)
            });
        });
    }
    slow_g.finish();
}

criterion_group!(benches, bench_compare_documents);
criterion_main!(benches);
