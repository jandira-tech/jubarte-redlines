//! M15 — coalesce adjacent same-status revision wrappers.
//!
//! Word never emits two adjacent `w:ins` (or two adjacent `w:del`) wrappers: it
//! wraps consecutive same-status runs in a SINGLE wrapper. We emit one wrapper
//! per run, inflating the `w:ins`/`w:del` element count ~2x vs Word (observed:
//! up to 24 adjacent `w:ins` and 19 adjacent `w:del` in a single document).
//!
//! `coalesce_adjacent_runs`/`coalesce_key` merge `w:r` and (gated) `w:del` by
//! concatenating text, but have NO `w:ins` branch, so adjacent insertions are
//! never merged. The fix is a wrapper-merge: move the runs of consecutive
//! same-author/date wrappers into the first, keeping each run intact (so per-run
//! formatting is preserved) — text-preserving, so the golden text-parity holds.

use jubarte::comparer::finalize::coalesce_adjacent_revisions;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const WNS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

#[test]
fn merges_adjacent_same_status_revision_wrappers() {
    // Two adjacent w:ins with DIFFERENT run formatting, then two adjacent w:del.
    let xml = format!(
        concat!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p>",
            "<w:ins w:id=\"1\" w:author=\"A\" w:date=\"D\"><w:r><w:rPr><w:b/></w:rPr><w:t>X</w:t></w:r></w:ins>",
            "<w:ins w:id=\"2\" w:author=\"A\" w:date=\"D\"><w:r><w:rPr><w:i/></w:rPr><w:t>Y</w:t></w:r></w:ins>",
            "<w:del w:id=\"3\" w:author=\"A\" w:date=\"D\"><w:r><w:delText>P</w:delText></w:r></w:del>",
            "<w:del w:id=\"4\" w:author=\"A\" w:date=\"D\"><w:r><w:delText>Q</w:delText></w:r></w:del>",
            "</w:p></w:body></w:document>"
        ),
        w = WNS
    );
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();

    coalesce_adjacent_revisions(&mut d, root);

    let inss = d.descendants(root, Some(&W::ins()));
    let dels = d.descendants(root, Some(&W::del()));
    assert_eq!(
        inss.len(),
        1,
        "two adjacent w:ins must merge into ONE wrapper"
    );
    assert_eq!(
        dels.len(),
        1,
        "two adjacent w:del must merge into ONE wrapper"
    );
    // both runs kept inside the single wrapper (formatting NOT concatenated away)
    assert_eq!(
        d.elements(inss[0], Some(&W::r())).len(),
        2,
        "both insertion runs preserved as separate runs"
    );
    assert_eq!(
        d.elements(dels[0], Some(&W::r())).len(),
        2,
        "both deletion runs preserved as separate runs"
    );
}

#[test]
fn does_not_merge_across_status_or_author() {
    // ins next to del must NOT merge; ins with a different author must NOT merge.
    let xml = format!(
        concat!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p>",
            "<w:ins w:id=\"1\" w:author=\"A\" w:date=\"D\"><w:r><w:t>X</w:t></w:r></w:ins>",
            "<w:del w:id=\"2\" w:author=\"A\" w:date=\"D\"><w:r><w:delText>P</w:delText></w:r></w:del>",
            "<w:ins w:id=\"3\" w:author=\"B\" w:date=\"D\"><w:r><w:t>Y</w:t></w:r></w:ins>",
            "<w:ins w:id=\"4\" w:author=\"A\" w:date=\"D\"><w:r><w:t>Z</w:t></w:r></w:ins>",
            "</w:p></w:body></w:document>"
        ),
        w = WNS
    );
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();

    coalesce_adjacent_revisions(&mut d, root);

    // author B's ins stays separate from author A's ins (3 ins total: X, Y(B), Z(A)).
    assert_eq!(
        d.descendants(root, Some(&W::ins())).len(),
        3,
        "different authors must not merge"
    );
    assert_eq!(d.descendants(root, Some(&W::del())).len(), 1);
}
