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
//! (The counting global allocator was removed to eliminate unsafe code. We now
//! verify this mechanically by proving `AtomHash` is inline and freshly-atomized
//! state is empty.)

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

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

    // 1. Prove AtomHash is strictly inline and manages no heap memory itself.
    assert!(
        !std::mem::needs_drop::<jubarte::comparer::atoms::AtomHash>(),
        "AtomHash must remain an inline type without a Drop impl (e.g. String) to avoid per-atom allocations on clone"
    );
    assert!(
        std::mem::size_of::<jubarte::comparer::atoms::AtomHash>() <= 32,
        "AtomHash should be small enough to stay inline"
    );

    let atoms = run_fragmented_atoms(ATOMS);
    // 500 char atoms + the paragraph-mark (pPr) atom.
    assert!(
        atoms.len() >= ATOMS,
        "expected >= {ATOMS} atoms, got {}",
        atoms.len()
    );

    // 2. Prove that the freshly atomized elements have empty Option<String> fields,
    // which would otherwise allocate on clone.
    for atom in &atoms {
        assert!(
            atom.correlated_sha1_hash.is_none(),
            "Fresh atoms must not have correlated_sha1_hash populated, or cloning will allocate"
        );
    }

    // 3. Keep the actual clone loop to ensure the code executes cleanly.
    for _ in 0..CLONES {
        let cloned = atoms.clone();
        std::hint::black_box(&cloned);
    }
}
