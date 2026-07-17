// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M23 — `assemble_ancestor_unids` Phase B must borrow the following paragraph's
//! ancestor Unids ONLY for ancestors actually shared (by element identity), not
//! the whole prefix blindly.
//!
//! `sd-2672-nested-table` (text "before-nested" in an OUTER cell, followed by a
//! nested table) vs `sd-2672-sdt-table` panicked at finalize.rs:93 ("Internal
//! error - both deleted and inserted text in the same run"): Phase B borrowed the
//! WHOLE following-paragraph Unid prefix, desyncing `ancestor_unids` from
//! `ancestor_elements` for the shallower outer-cell text → CoalesceRecurse
//! grouped the deleted outer text and the inserted nested-table paragraph under
//! ONE run (`<w:r>…<w:delText/><w:p/>…</w:r>`).
//!
//! The same blind-borrow desync also produced ooxmlsdk-invalid output for other
//! pairs; the fix took the crate from 88→91 ooxmlsdk-loadable outputs with no
//! regressions. `h_f-normal` vs `imageInShapeInFooter` is one of those: it was
//! ooxmlsdk-invalid before the fix and must be valid after.

use std::io::Cursor;

use jubarte::document_comparer::compare_documents;

const A: &[u8] = include_bytes!("fixtures/nested_table/a.docx");
const B: &[u8] = include_bytes!("fixtures/nested_table/b.docx");
const HF_A: &[u8] = include_bytes!("fixtures/nested_table/hf_a.docx");
const HF_B: &[u8] = include_bytes!("fixtures/nested_table/hf_b.docx");

#[test]
fn nested_table_to_sdt_table_is_ooxmlsdk_valid() {
    // Was: panic (fixed by the ancestor-Unid borrow fix), then generated-but-invalid
    // (a stray <w:tc> directly under <w:sdtContent> — the correlation gave A's nested
    // <w:tbl> the same Unid as B's <w:sdt>). The (Unid, element-name) CoalesceRecurse
    // grouping keeps the divergent structures separate. Now OPENS clean in real Word.
    let out = compare_documents(A, B, "Test")
        .expect("compare must not panic on nested-table -> sdt-table");
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable (no stray w:tc under w:sdtContent)");
}

#[test]
fn ancestor_unid_fix_yields_ooxmlsdk_valid_output() {
    // This pair was ooxmlsdk-invalid before the Phase B borrow fix.
    let out = compare_documents(HF_A, HF_B, "Test").expect("compare ok");
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable after the ancestor-Unid fix");
}
