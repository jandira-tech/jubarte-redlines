// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M473 — sibling of the M471 rotation: the deletion mark lands one slot
//! late. When a whole base paragraph is deleted inside a replace region, our
//! produce emits its content with a LIVE bare mark plus a separate
//! contentless MARK-DEL shell:
//!
//!   ours:   Pi   --  [del content…]        (bare <w:pPr/>, live mark)
//!           Pi+1 MD  []                    (contentless MARK-DEL shell)
//!
//! The content paragraph must carry MARK-DEL — its live mark makes
//! accept-all strand an empty paragraph document B never had. The rewrite
//! RESTAMPS the shell's marked pPr onto the content paragraph and leaves
//! the shell alone: whether the shell should ALSO be dropped depends on
//! whether A had one deleted paragraph (ooxml_size_rstyle × strike: Word
//! merges) or two (file_36 × file_37: Word keeps [MD content][MD empty]) —
//! produce-side provenance a finalize pass cannot see, and merging when
//! wrong shifts every page below (−30 pixels on file_36) while an extra
//! deleted-empty line costs almost nothing.
//!
//! Oracle: super_editor__ooxml_size_rstyle_linked_combos_demo ×
//! super_editor__ooxml_strike_rstyle_linked_combos_dem (two hits in one
//! document: "Sample size text" and "Styled 18pt text").

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn body_paras(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    loop {
        let i = match (rest.find("<w:p "), rest.find("<w:p>")) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        out.push(after[..j].to_string());
        rest = &after[j + 6..];
    }
    out
}

fn mark_del(p: &str) -> bool {
    // MARK-DEL: a <w:del> inside the pPr's rPr.
    let Some(ppr_end) = p.find("</w:pPr>") else {
        return false;
    };
    p[..ppr_end].contains("<w:rPr>") && {
        let rpr = &p[p.find("<w:rPr>").unwrap()..ppr_end];
        rpr.contains("<w:del ")
    }
}

fn del_only_live_mark(p: &str) -> bool {
    // bare pPr (no properties, no mark revision), all content deleted.
    let bare_ppr = p.contains("<w:pPr />") || p.contains("<w:pPr/>");
    bare_ppr
        && p.contains("<w:delText")
        && !p.contains("<w:ins ")
        && !{
            // any live (non-del) text
            let mut live = false;
            let mut rest = p;
            while let Some(i) = rest.find("<w:t") {
                // count only w:t outside w:del blocks — cheap check: the
                // fixture paragraphs have no EQ runs, so any <w:t> (not
                // <w:delText>) is live text.
                if rest[i..].starts_with("<w:t ") || rest[i..].starts_with("<w:t>") {
                    live = true;
                    break;
                }
                rest = &rest[i + 4..];
            }
            live
        }
}

fn contentless(p: &str) -> bool {
    !p.contains("<w:r>") && !p.contains("<w:r ")
}

#[test]
fn del_mark_sits_on_its_own_deleted_content() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_size_rstyle_linked_combos_demo_017c9552.docx");
    let b = src.join("super_editor__ooxml_strike_rstyle_linked_combos_dem_b8167cd3.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    let paras = body_paras(&xml);

    // Invariant: the late-mark split shape must not exist anywhere.
    for w in paras.windows(2) {
        assert!(
            !(del_only_live_mark(&w[0]) && mark_del(&w[1]) && contentless(&w[1])),
            "deletion mark stranded one slot late after: {}",
            &w[0][..w[0].len().min(300)]
        );
    }

    // Specific: both fully-deleted base paragraphs carry their own MARK-DEL.
    for needle in ["Sample size text", "Styled 18pt text"] {
        let p = paras
            .iter()
            .find(|p| p.contains(needle))
            .unwrap_or_else(|| panic!("{needle} paragraph missing"));
        assert!(
            mark_del(p),
            "{needle} paragraph must carry MARK-DEL on its own mark"
        );
    }
}
