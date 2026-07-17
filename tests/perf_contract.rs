// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! P0-LAB-01 contract tests — durable lab pieces from LCS_PERF_PLAN.md.
//!
//! Drives the **shipped** surfaces:
//!   - `jubarte::perf` counters/snapshot/JSON
//!   - `tools/perf/summarize.py` (median/MAD/verdict + seeded regression exit)
//!   - `tools/perf/quality_compare.py` (paired ledger gate + seeded exit)
//!   - profile-off path leaves compare output intact (no-op instrumentation)

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

use jubarte::document_comparer::compare_documents;
use jubarte::perf::{self, Snapshot, Stage};

// The perf surface intentionally uses process-global atomics. Tests that call
// reset/snapshot must therefore not overlap under Rust's parallel test runner.
static PERF_STATE: Mutex<()> = Mutex::new(());

fn lock_perf_state() -> MutexGuard<'static, ()> {
    PERF_STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn tool(name: &str) -> PathBuf {
    crate_root().join("tools/perf").join(name)
}

#[test]
fn perf_counters_reset_and_snapshot_json_shape() {
    let _guard = lock_perf_state();
    perf::reset();
    let z = perf::snapshot();
    // Default builds: always zero. Feature-on builds: zero after reset.
    assert_eq!(z.lcs_calls, 0);
    let j = z.to_json();
    assert!(j.contains("\"lcs_calls\":0"), "json={j}");
    assert!(j.contains("stage_ns"), "json={j}");
}

#[test]
fn perf_time_stage_and_inc_are_callable() {
    let _guard = lock_perf_state();
    perf::reset();
    let got = perf::time_stage(Stage::Lcs, || 7u8);
    assert_eq!(got, 7);
    perf::inc_lcs_calls();
    perf::add_lcs_window_area(2, 5);
    perf::inc_corr_run_scans();
    let s = perf::snapshot();
    if perf::ENABLED {
        assert_eq!(s.lcs_calls, 1);
        assert_eq!(s.lcs_window_area, 10);
        assert_eq!(s.corr_run_scans, 1);
        // stage timer may be 0 on ultra-fast Instant resolution; just ensure in range.
        let _ = s.stage_ns[Stage::Lcs as usize];
    } else {
        assert_eq!(s, Snapshot::default());
    }
    perf::reset();
}

#[test]
fn compare_documents_unaffected_by_default_perf_hooks() {
    let _guard = lock_perf_state();
    // Output-equivalence smoke: the default (feature-off) path must still
    // produce a valid redline for the in-repo dense-edit fixture pair.
    let root = crate_root();
    let a = root.join("tests/fixtures/redline/original.docx");
    let b = root.join("tests/fixtures/redline/modified.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("skip: redline fixtures absent");
        return;
    }
    perf::reset();
    let bytes_a = std::fs::read(&a).expect("read a");
    let bytes_b = std::fs::read(&b).expect("read b");
    let out1 = compare_documents(&bytes_a, &bytes_b, "Lab").expect("compare 1");
    let out2 = compare_documents(&bytes_a, &bytes_b, "Lab").expect("compare 2");
    // ZIP members are not byte-stable, but both must be non-empty PK packages.
    assert!(out1.starts_with(b"PK"), "out1 not a zip");
    assert!(out2.starts_with(b"PK"), "out2 not a zip");
    assert!(out1.len() > 100 && out2.len() > 100);
    // Default feature-off: counters stay zero through real compares.
    if !perf::ENABLED {
        assert_eq!(perf::snapshot(), Snapshot::default());
    }
}

#[test]
fn summarize_py_seeded_regression_exits_nonzero() {
    let py = tool("summarize.py");
    assert!(py.is_file(), "missing {}", py.display());
    let status = Command::new("python3")
        .arg(&py)
        .arg("--seeded-regression-demo")
        .status()
        .expect("spawn summarize.py");
    // Seeded demo intentionally returns 1 when the regression gate fires.
    assert_eq!(
        status.code(),
        Some(1),
        "summarize seeded demo must exit 1 (gate fired); got {:?}",
        status.code()
    );
}

#[test]
fn summarize_py_parses_abba_tsv_and_reports() {
    let py = tool("summarize.py");
    let dir = tempfile::tempdir().expect("tmpdir");
    let tsv = dir.path().join("summary.tsv");
    // Balanced A/B — expect noise / not regress.
    let mut body = String::from("round\tfixture\ttag\treal\tuser\tsys\tmaxrss\n");
    for r in 1..=2 {
        for (tag, real) in [("A", 10.0), ("B", 10.05), ("B", 9.95), ("A", 10.1)] {
            body.push_str(&format!("{r}\tpdense_15k\t{tag}\t{real}\t9.0\t0.1\t1000\n"));
        }
    }
    std::fs::write(&tsv, &body).expect("write tsv");
    let out = Command::new("python3")
        .arg(&py)
        .arg(&tsv)
        .arg("--allow-regress")
        .output()
        .expect("run summarize");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"overall\""),
        "missing overall in {stdout}"
    );
    assert!(stdout.contains("pdense_15k"), "missing fixture in {stdout}");
}

#[test]
fn quality_compare_seeded_regression_exits_nonzero() {
    let py = tool("quality_compare.py");
    assert!(py.is_file(), "missing {}", py.display());
    let status = Command::new("python3")
        .arg(&py)
        .arg("--seeded-regression-demo")
        .status()
        .expect("spawn quality_compare.py");
    assert_eq!(
        status.code(),
        Some(1),
        "quality_compare seeded demo must exit 1; got {:?}",
        status.code()
    );
}

#[test]
fn quality_compare_accepts_equal_ledgers() {
    let py = tool("quality_compare.py");
    let dir = tempfile::tempdir().expect("tmpdir");
    let base = dir.path().join("base.tsv");
    let cand = dir.path().join("cand.tsv");
    let body = "pair_a\t90.0\npair_b\t80.5\npair_c\t70.0\n";
    std::fs::write(&base, body).unwrap();
    std::fs::write(&cand, body).unwrap();
    let out = Command::new("python3")
        .arg(&py)
        .arg(&base)
        .arg(&cand)
        .output()
        .expect("run quality_compare");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"ok\": true") || stdout.contains("\"ok\":true"));
}

#[test]
fn run_abba_matrix_script_exists_and_is_executable_contract() {
    let sh = tool("run_abba_matrix.sh");
    assert!(sh.is_file());
    let meta = std::fs::metadata(&sh).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "run_abba_matrix.sh must be executable"
        );
    }
    let trials = tool("run_trials.sh");
    assert!(trials.is_file(), "run_trials.sh missing");
}
