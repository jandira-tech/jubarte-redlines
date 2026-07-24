// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ALLOC-ATTRIBUTE-01 — attribute the compare-time ALLOCATION COUNT (not peak
//! bytes) by call site and size class.
//!
//! MEM-PROFILE-01 measured ~544M total allocations on the dissertation and
//! MEM-ATTRIBUTE-01 attributed the PEAK BYTES (a 3 GiB DOM-arena block plus
//! per-node overhead). Those are orthogonal axes: peak is atom-size-invariant,
//! but the 544M alloc COUNT drives the ~30 s wall-clock and is the metric the
//! optimization goal actually named. This harness names WHERE the 544M come
//! from: a counting `#[global_allocator]` buckets every allocation by size class
//! and samples a backtrace every `SAMPLE_EVERY` allocations, so the dominant
//! allocation SITES fall out (reentrancy-guarded so the profiler's own
//! backtrace allocations are neither counted nor recursed into).
//!
//! Run (release, system allocator):
//! ```bash
//! cargo run --release --example alloc_attribute --no-default-features
//! ```

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::backtrace::Backtrace;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const NBUCKETS: usize = 40;
/// Capture one backtrace per this many allocations. 544M / 50k ≈ 11k samples —
/// enough resolution to rank sites, cheap enough to symbolize at the end.
const SAMPLE_EVERY: usize = 50_000;
const MAX_SAMPLES: usize = 40_000;

thread_local! {
    /// Set while the profiler captures a backtrace (which itself allocates); those
    /// allocations must be neither counted nor recursed into.
    static IN_HOOK: Cell<bool> = const { Cell::new(false) };
}

static ALLOC_CNT_BY_BUCKET: [AtomicUsize; NBUCKETS] = [const { AtomicUsize::new(0) }; NBUCKETS];
static TOTAL_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static COUNTING: AtomicUsize = AtomicUsize::new(0);
/// Sampled `(size_bucket, backtrace)`; symbolized post-run to avoid paying
/// symbolication cost inside the timed region.
static SAMPLES: Mutex<Vec<(usize, Backtrace)>> = Mutex::new(Vec::new());

struct Counting;

#[inline]
fn bucket(size: usize) -> usize {
    if size <= 1 {
        0
    } else {
        (usize::BITS - 1 - size.leading_zeros()) as usize
    }
    .min(NBUCKETS - 1)
}

fn sample_bt(b: usize) {
    let already = IN_HOOK.with(|f| f.replace(true));
    if already {
        return;
    }
    let bt = Backtrace::force_capture();
    if let Ok(mut v) = SAMPLES.lock()
        && v.len() < MAX_SAMPLES
    {
        v.push((b, bt));
    }
    IN_HOOK.with(|f| f.set(false));
}

#[inline]
fn on_alloc(size: usize) {
    if COUNTING.load(Ordering::Relaxed) == 0 {
        return;
    }
    if IN_HOOK.with(|f| f.get()) {
        return; // profiler's own allocation — do not count
    }
    ALLOC_CNT_BY_BUCKET[bucket(size)].fetch_add(1, Ordering::Relaxed);
    let n = TOTAL_ALLOCS.fetch_add(1, Ordering::Relaxed) + 1;
    if n.is_multiple_of(SAMPLE_EVERY) {
        sample_bt(bucket(size));
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            on_alloc(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // A realloc is a fresh allocation event (it can move + re-copy); count it.
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            on_alloc(new_size);
        }
        p
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Reduce a backtrace to its first few jubarte frames — the allocation SITE.
fn site_key(bt: &Backtrace) -> String {
    let s = bt.to_string();
    let frames: Vec<String> = s
        .lines()
        .filter(|l| l.contains("jubarte::"))
        .map(|l| {
            // Keep the symbol, drop the "NN: " index and any trailing address.
            let t = l.trim();
            let t = t.split_once(": ").map(|(_, r)| r).unwrap_or(t);
            t.split_whitespace().next().unwrap_or(t).to_string()
        })
        // Skip the allocator/collection glue that every site shares.
        .filter(|f| !f.contains("mem_profile") && !f.contains("alloc_attribute"))
        .take(4)
        .collect();
    if frames.is_empty() {
        "<no jubarte frames>".to_string()
    } else {
        frames.join(" <- ")
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/arthrod".to_string());
    let base = format!("{home}/T/neurotic_docx_bench/docx-viewer/public/word_based/dissertacao");
    let a_path = format!("{base}/dissertacao-a.docx");
    let b_path = format!("{base}/dissertacao-b.docx");
    let a = std::fs::read(&a_path).unwrap_or_else(|e| panic!("read {a_path}: {e}"));
    let b = std::fs::read(&b_path).unwrap_or_else(|e| panic!("read {b_path}: {e}"));
    println!("original: {a_path}");
    println!("modified: {b_path}\n");

    COUNTING.store(1, Ordering::Relaxed);
    let out = jubarte::document_comparer::compare_documents(&a, &b, "alloc-attribute")
        .expect("compare ok");
    COUNTING.store(0, Ordering::Relaxed);

    let total = TOTAL_ALLOCS.load(Ordering::Relaxed);
    println!(
        "total allocations: {total} | out {:.2} MiB\n",
        out.len() as f64 / 1048576.0
    );

    // Size-class histogram of allocation COUNT.
    println!("{:>8}  {:>14}  {:>7}   bar", "sizecls", "allocs", "share");
    let mut rows: Vec<(usize, usize)> = (0..NBUCKETS)
        .map(|i| (i, ALLOC_CNT_BY_BUCKET[i].load(Ordering::Relaxed)))
        .filter(|&(_, c)| c > 0)
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1));
    for (i, cnt) in &rows {
        let lo = 1usize << i;
        let hi = lo.saturating_mul(2) - 1;
        let share = *cnt as f64 / total as f64;
        let bar = "#".repeat((share * 40.0).round() as usize);
        println!("{lo:>6}-{hi:<6} {cnt:>14}  {:>6.1}%   {bar}", share * 100.0);
    }

    // Aggregate sampled sites: each sample stands for ~SAMPLE_EVERY allocations.
    let samples = SAMPLES
        .lock()
        .map(|v| {
            v.iter()
                .map(|(b, bt)| (*b, site_key(bt)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut by_site: std::collections::HashMap<String, (usize, [usize; NBUCKETS])> =
        std::collections::HashMap::new();
    for (b, key) in &samples {
        let e = by_site.entry(key.clone()).or_insert((0, [0; NBUCKETS]));
        e.0 += 1;
        e.1[*b] += 1;
    }
    let mut sites: Vec<(String, usize, [usize; NBUCKETS])> =
        by_site.into_iter().map(|(k, (c, h))| (k, c, h)).collect();
    sites.sort_by_key(|r| std::cmp::Reverse(r.1));
    println!(
        "\n=== dominant allocation SITES (sampled 1/{SAMPLE_EVERY}; {} samples) ===",
        samples.len()
    );
    for (key, cnt, hist) in sites.iter().take(18) {
        let est = cnt * SAMPLE_EVERY;
        let share = est as f64 / total as f64 * 100.0;
        // Dominant size bucket for this site.
        let (bi, _) = hist
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .unwrap_or((0, &0));
        let lo = 1usize << bi;
        println!(
            "\n  ~{est:>11} allocs ({share:>4.1}%)  [mostly {lo}-{}B]",
            (lo * 2 - 1)
        );
        println!("    {key}");
    }
}
