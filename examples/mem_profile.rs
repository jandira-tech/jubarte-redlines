// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! MEM-PROFILE-01 (TODO item 1b) — peak-heap attribution for the alignment
//! stage on run-fragmented inputs.
//!
//! A counting `#[global_allocator]` wraps the system allocator and records the
//! live-bytes high-water mark. We measure three things against the SAME pair:
//!
//! 1. baseline: just the two file buffers resident (no compare)
//! 2. a-vs-b: the real diff (every alignment allocation incurred)
//! 3. a-vs-a: identical pair (the short-circuit path)
//!
//! The gap between (2)/(3) and (1) is the compare-time peak. The gap between
//! (2) and (3) is the "triggered by ANY diff" cost the TODO calls out: an
//! identical pair short-circuits the atom correlation, so its peak collapses to
//! near the baseline, while a one-atom-different pair pays essentially the same
//! peak as a fully-rewritten one — the cost tracks atom (run) count, not edits.
//!
//! Run (release, system allocator, no mimalloc):
//! ```bash
//! cargo run --release --example mem_profile --no-default-features -- \
//!   <original.docx> <modified.docx>
//! ```
//! Defaults to the dissertation pair under neurotic_docx_bench when no args.

// The library denies unsafe crate-wide (Cargo `[lints]`); a counting global
// allocator is inherently unsafe. Scope the allow to this diagnostic example.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);

#[inline]
fn bump(delta: usize) {
    // Relaxed is fine: we only need a consistent high-water read at quiescent
    // points (between the timed regions), not a linearizable ordering.
    let now = LIVE.fetch_add(delta, Ordering::Relaxed) + delta;
    ALLOCS.fetch_add(1, Ordering::Relaxed);
    let mut peak = PEAK.load(Ordering::Relaxed);
    while now > peak {
        match PEAK.compare_exchange_weak(peak, now, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            bump(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                bump(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        p
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn reset_peak() {
    PEAK.store(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
    ALLOCS.store(0, Ordering::Relaxed);
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn run(label: &str, original: &[u8], modified: &[u8]) {
    reset_peak();
    let live_before = LIVE.load(Ordering::Relaxed);
    let t = Instant::now();
    let result = jubarte::document_comparer::compare_documents(original, modified, "mem-profile");
    let elapsed = t.elapsed();
    let peak = PEAK.load(Ordering::Relaxed);
    let allocs = ALLOCS.load(Ordering::Relaxed);
    match result {
        Ok(out) => {
            let compare_peak = peak.saturating_sub(live_before);
            println!(
                "{label:>10}: peak_live {:>10.1} MiB | compare_peak {:>10.1} MiB | allocs {:>12} | out {:>8.2} MiB | {:>8.2} s",
                mib(peak),
                mib(compare_peak),
                allocs,
                mib(out.len()),
                elapsed.as_secs_f64(),
            );
        }
        Err(e) => {
            println!(
                "{label:>10}: ERROR {e:?} | peak_live {:.1} MiB | allocs {allocs}",
                mib(peak)
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (a_path, b_path) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        // Canonical tree is ~/T/jubarte-redlines (a symlink → temp/T); resolve
        // HOME at runtime instead of hard-coding an absolute machine path.
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/arthrod".to_string());
        let base =
            format!("{home}/T/neurotic_docx_bench/docx-viewer/public/word_based/dissertacao");
        (
            format!("{base}/dissertacao-a.docx"),
            format!("{base}/dissertacao-b.docx"),
        )
    };

    let a = std::fs::read(&a_path).unwrap_or_else(|e| panic!("read {a_path}: {e}"));
    let b = std::fs::read(&b_path).unwrap_or_else(|e| panic!("read {b_path}: {e}"));

    println!("original: {a_path} ({:.2} MiB)", mib(a.len()));
    println!("modified: {b_path} ({:.2} MiB)", mib(b.len()));
    println!(
        "  baseline: {:.1} MiB resident (both file buffers)",
        mib(LIVE.load(Ordering::Relaxed))
    );
    println!();

    // Real diff: pays the full alignment allocation.
    run("a-vs-b", &a, &b);
    // Identical pair: short-circuits the atom correlation.
    run("a-vs-a", &a, &a);
}
