// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! MEM-ATTRIBUTE-01 — attribute the compare-time peak footprint by allocation
//! SIZE CLASS, snapshotting the live-bytes histogram at the exact peak moment.
//!
//! MEM-PROFILE-01 established the 10.7 GiB peak on the dissertation. Two causal
//! experiments then proved the peak is INDEPENDENT of the atom struct: inlining
//! the atom hash (−24 B/atom) and ballooning the atom (+256 B/atom) both left the
//! peak byte-identical. So the peak is NOT `ComparisonUnitAtom` storage — it is
//! held by something atom-size-invariant. This harness finds what *size class*
//! that something is: a peak dominated by many small (~10–40 B) blocks points at
//! per-atom String/keys; a peak dominated by few large blocks points at big Vecs
//! (reference vectors / DP tables / DOM arena chunks).
//!
//! Run (release, system allocator):
//! ```bash
//! cargo run --release --example mem_attribute --no-default-features
//! ```

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const NBUCKETS: usize = 40; // 2^0 .. 2^39 bytes
/// Any single allocation at least this big gets its backtrace captured — these
/// are the multi-GB monoliths that dominate the peak; there are only a handful,
/// so the capture cost is negligible.
const BIG_THRESHOLD: usize = 256 * 1024 * 1024;

thread_local! {
    /// Reentrancy guard: capturing a backtrace allocates, and we must not recurse
    /// back into the hook (those allocations go to System untracked).
    static IN_HOOK: Cell<bool> = const { Cell::new(false) };
}

/// (size, backtrace) for each big allocation, symbolized post-run.
static BIG: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());

fn record_big(size: usize) {
    let already = IN_HOOK.with(|f| f.replace(true));
    if already {
        return; // inside the hook already — do not recurse
    }
    let bt = std::backtrace::Backtrace::force_capture().to_string();
    if let Ok(mut v) = BIG.lock() {
        v.push((size, bt));
    }
    IN_HOOK.with(|f| f.set(false));
}

struct Attributing;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
// Live bytes currently attributed to each power-of-two size bucket.
static LIVE_BY_BUCKET: [AtomicUsize; NBUCKETS] = {
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; NBUCKETS]
};
// Snapshot of LIVE_BY_BUCKET taken when a new peak (ratcheted +1%) is observed.
static PEAK_BY_BUCKET: [AtomicUsize; NBUCKETS] = {
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; NBUCKETS]
};
// Live block COUNT per bucket (to distinguish many-small from few-large).
static LIVE_CNT_BY_BUCKET: [AtomicUsize; NBUCKETS] = {
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; NBUCKETS]
};
static PEAK_CNT_BY_BUCKET: [AtomicUsize; NBUCKETS] = {
    const Z: AtomicUsize = AtomicUsize::new(0);
    [Z; NBUCKETS]
};
static SNAP_THRESHOLD: AtomicUsize = AtomicUsize::new(0);
static COUNTING: AtomicUsize = AtomicUsize::new(0); // 0 = off, 1 = on

#[inline]
fn bucket(size: usize) -> usize {
    // floor(log2(size)), clamped; size 0 -> bucket 0.
    if size <= 1 {
        0
    } else {
        (usize::BITS - 1 - (size.leading_zeros())) as usize
    }
    .min(NBUCKETS - 1)
}

#[inline]
fn on_alloc(size: usize) {
    if COUNTING.load(Ordering::Relaxed) == 0 {
        return;
    }
    if size >= BIG_THRESHOLD {
        record_big(size);
    }
    let b = bucket(size);
    LIVE_BY_BUCKET[b].fetch_add(size, Ordering::Relaxed);
    LIVE_CNT_BY_BUCKET[b].fetch_add(1, Ordering::Relaxed);
    let now = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    // Ratchet: only re-snapshot when we exceed the last snapshot by >1%, to bound
    // snapshot cost while still capturing the true peak distribution.
    let thr = SNAP_THRESHOLD.load(Ordering::Relaxed);
    if now > thr {
        SNAP_THRESHOLD.store(now + now / 100, Ordering::Relaxed);
        if now > PEAK.load(Ordering::Relaxed) {
            PEAK.store(now, Ordering::Relaxed);
        }
        for i in 0..NBUCKETS {
            PEAK_BY_BUCKET[i].store(LIVE_BY_BUCKET[i].load(Ordering::Relaxed), Ordering::Relaxed);
            PEAK_CNT_BY_BUCKET[i]
                .store(LIVE_CNT_BY_BUCKET[i].load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }
}

#[inline]
fn on_dealloc(size: usize) {
    if COUNTING.load(Ordering::Relaxed) == 0 {
        return;
    }
    let b = bucket(size);
    LIVE_BY_BUCKET[b].fetch_sub(size, Ordering::Relaxed);
    LIVE_CNT_BY_BUCKET[b].fetch_sub(1, Ordering::Relaxed);
    LIVE.fetch_sub(size, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for Attributing {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            on_alloc(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        on_dealloc(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            on_dealloc(layout.size());
            on_alloc(new_size);
        }
        p
    }
}

#[global_allocator]
static GLOBAL: Attributing = Attributing;

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/arthrod".to_string());
    let base = format!("{home}/T/neurotic_docx_bench/docx-viewer/public/word_based/dissertacao");
    let a_path = format!("{base}/dissertacao-a.docx");
    let b_path = format!("{base}/dissertacao-b.docx");
    let a = std::fs::read(&a_path).unwrap_or_else(|e| panic!("read {a_path}: {e}"));
    let b = std::fs::read(&b_path).unwrap_or_else(|e| panic!("read {b_path}: {e}"));
    println!("original: {a_path} ({:.2} MiB)", mib(a.len()));
    println!("modified: {b_path} ({:.2} MiB)", mib(b.len()));

    COUNTING.store(1, Ordering::Relaxed);
    let out = jubarte::document_comparer::compare_documents(&a, &b, "mem-attribute")
        .expect("compare ok");
    COUNTING.store(0, Ordering::Relaxed);

    let peak = PEAK.load(Ordering::Relaxed);
    println!("\ncompare_peak {:.1} MiB | out {:.2} MiB\n", mib(peak), mib(out.len()));
    println!(
        "{:>8}  {:>14}  {:>12}  {:>7}   {}",
        "sizecls", "live@peak(MiB)", "blocks", "share", "bar"
    );
    // Collect and sort buckets by live bytes at peak, descending.
    let mut rows: Vec<(usize, usize, usize)> = (0..NBUCKETS)
        .map(|i| {
            (
                i,
                PEAK_BY_BUCKET[i].load(Ordering::Relaxed),
                PEAK_CNT_BY_BUCKET[i].load(Ordering::Relaxed),
            )
        })
        .filter(|&(_, bytes, _)| bytes > 0)
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1));
    let total: usize = rows.iter().map(|r| r.1).sum();
    for (i, bytes, cnt) in rows {
        let lo = 1usize << i;
        let hi = lo.saturating_mul(2) - 1;
        let share = bytes as f64 / total as f64;
        let bar = "#".repeat((share * 40.0).round() as usize);
        println!(
            "{lo:>6}-{hi:<6} {:>13.1}  {cnt:>12}  {:>6.1}%   {bar}",
            mib(bytes),
            share * 100.0,
        );
    }
    println!("\nsum of tracked buckets at peak: {:.1} MiB", mib(total));

    // Name the multi-GB monoliths: the biggest single allocations and where they
    // came from. Dedup identical backtraces, keeping count + max size.
    let mut big = BIG.lock().map(|v| v.clone()).unwrap_or_default();
    big.sort_by(|a, b| b.0.cmp(&a.0));
    println!("\n=== biggest single allocations (>= 256 MiB), top sites ===");
    let mut shown = 0;
    let mut seen_frames: Vec<String> = Vec::new();
    for (size, bt) in &big {
        // Key each site by its first few user (jubarte) frames.
        let key: String = bt
            .lines()
            .filter(|l| l.contains("jubarte"))
            .take(4)
            .collect::<Vec<_>>()
            .join(" | ");
        if seen_frames.contains(&key) {
            continue;
        }
        seen_frames.push(key);
        println!("\n--- {:.1} MiB single block ---", mib(*size));
        // Print the jubarte frames (skip std/alloc noise).
        for line in bt.lines() {
            if line.contains("jubarte")
                || line.contains("comparer")
                || line.contains("xmllinq")
            {
                println!("  {}", line.trim());
            }
        }
        shown += 1;
        if shown >= 6 {
            break;
        }
    }
    println!("\ntotal big-alloc events captured: {}", big.len());
}
