// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ALLOC-LEAN-01 — the serialize namespace-prefix allocation gate.
//!
//! `serialize_element` resolves a namespace prefix (e.g. `"w"`) for every element
//! name and every attribute name. The old code obtained each prefix from
//! `Scope::assign`, which returned an owned `String`; on run-fragmented documents
//! this heap-allocated a 1-byte prefix tens of millions of times — ALLOC-ATTRIBUTE-01
//! measured `Scope::assign`/`emit` prefix allocations at ~20% of the 547M total
//! allocations on the dissertation. `Scope::ensure_prefix` registers the binding
//! without allocating in the common already-bound case, and `write_attributes`
//! borrows the resolved prefix as `&str` instead of cloning it.
//!
//! This pins the invariant that the number of allocations serializing an element
//! whose attributes all share ONE namespace does NOT scale with the attribute
//! count: the shared prefix is bound once, then only looked up. The pre-ALLOC-LEAN-01
//! path allocated ~2 prefix `String`s per attribute (a discarded `assign` in emit's
//! first pass + a `resolve_prefix` clone in `write_attributes`), i.e. >= 2*ATTRS
//! allocations, blowing the budget below.
//!
//! Each integration-test file is its own binary, so it may install a process global
//! allocator without disturbing the library (which stays 100% safe).

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

struct Counting;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static COUNTING: AtomicBool = AtomicBool::new(false);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// A single `w:p` carrying `n_attrs` distinct attributes ALL in the `w` namespace
/// (so they share one prefix), inside a root that declares `xmlns:w`.
fn element_with_shared_ns_attrs(n_attrs: usize) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut attrs = String::new();
    for i in 0..n_attrs {
        // Distinct local names, one shared namespace → one shared prefix.
        let _ = write!(attrs, " w:a{i}=\"{i}\"");
    }
    let xml = format!(
        r#"<?xml version="1.0"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:p{attrs}/>
        </w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let p = dom.element(root, &W::name("p")).expect("w:p");
    (dom, p)
}

#[test]
fn serializing_shared_namespace_attrs_does_not_allocate_per_attr_prefix() {
    const ATTRS: usize = 800;
    let (dom, p) = element_with_shared_ns_attrs(ATTRS);

    COUNTING.store(true, Ordering::Relaxed);
    ALLOCS.store(0, Ordering::Relaxed);
    let s = dom.serialize_element(p);
    let allocs = ALLOCS.load(Ordering::Relaxed);
    COUNTING.store(false, Ordering::Relaxed);
    std::hint::black_box(&s);

    // Sanity: every attribute made it into the output (so we really serialized them).
    assert!(
        s.matches("w:a").count() >= ATTRS,
        "expected >= {ATTRS} attrs in output, got {}",
        s.matches("w:a").count()
    );

    // The old path allocated ~2 prefix `String`s per attribute (≥ 2*ATTRS = 1600).
    // ensure_prefix + borrowed `&str` allocate ZERO per attribute; the residual
    // allocations are the attribute `Vec`'s growth, the reusable tag prefix, and the
    // output buffer's doubling — all independent of ATTRS's prefix churn (a few dozen
    // total). A budget of ATTRS/2 sits far below the old cost and far above the new.
    let budget = ATTRS / 2;
    assert!(
        allocs <= budget,
        "serializing {ATTRS} shared-namespace attrs allocated {allocs} times \
         (budget {budget}); a per-attribute prefix String allocation regressed \
         ALLOC-LEAN-01 (Scope::ensure_prefix / borrowed write_attributes prefix)"
    );
}
