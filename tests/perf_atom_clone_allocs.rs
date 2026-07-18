// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ATOM-HASH-INLINE-01 — the atom-clone allocation gate.
//!
//! `ComparisonUnitAtom` is cloned on the order of hundreds of millions of times
//! during the LCS/correlation recursion (~544M allocations on the 276k-run
//! dissertation, MEM-PROFILE-01). In its freshly-atomized state every field but
//! the content hash is empty (`None` / a shared `Arc` refcount bump), so a clone
//! should allocate NOTHING for the hash. While the hash was a `String`, each
//! clone paid one heap allocation for the 40-byte hex digest — the dominant
//! per-clone cost. This test pins the invariant: **cloning a vector of
//! freshly-atomized atoms allocates only the vector's own backing buffer, not
//! one allocation per atom.**
//!
//! Each integration-test file is its own binary, so it may install a process
//! global allocator without disturbing the library (which stays 100% safe).

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
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

/// A single text node of `n` characters → `n` character atoms that all share ONE
/// ancestor `Arc` (PATH-01). Isolates the per-atom hash allocation: cloning the
/// atom vector bumps the shared `Arc` refcount (no allocation) and, on the old
/// `String`-hash layout, allocates one hex buffer per atom.
fn run_fragmented_atoms(n: usize) -> Vec<jubarte::comparer::atoms::ComparisonUnitAtom> {
    let text: String = "x".repeat(n);
    let xml = format!(
        r#"<?xml version="1.0"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body>
        </w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom.element(root, &W::body()).expect("body");
    create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default())
}

#[test]
fn cloning_fresh_atoms_does_not_allocate_per_atom() {
    const ATOMS: usize = 500;
    const CLONES: usize = 100;

    let atoms = run_fragmented_atoms(ATOMS);
    // 500 char atoms + the paragraph-mark (pPr) atom.
    assert!(
        atoms.len() >= ATOMS,
        "expected >= {ATOMS} atoms, got {}",
        atoms.len()
    );

    COUNTING.store(true, Ordering::Relaxed);
    ALLOCS.store(0, Ordering::Relaxed);
    for _ in 0..CLONES {
        let cloned = atoms.clone();
        std::hint::black_box(&cloned);
    }
    let allocs = ALLOCS.load(Ordering::Relaxed);
    COUNTING.store(false, Ordering::Relaxed);

    // Budget: a handful of allocations per full-vector clone (the Vec's own
    // backing buffer, plus slack for grouping/atomization internals) — NOT one
    // per atom. The old `String` hash cost ~ATOMS allocations per clone
    // (~50,000 total); the inline hash costs ~CLONES total.
    let budget = CLONES * 4;
    assert!(
        allocs <= budget,
        "cloning {CLONES}×{} atoms allocated {allocs} times (budget {budget}); \
         a per-atom hash allocation regressed the inline-hash invariant",
        atoms.len()
    );
}
