// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! REJECT-LOSSLESS-01 — reject must restore/remove tracked content that is
//! nested inside a *transparent run container* (`w:fldSimple`, `mc:Choice` /
//! `mc:Fallback`, …), not only content whose direct parent is `w:p` /
//! `w:hyperlink`.
//!
//! Evidence (D-2 accept/reject scoreboard, jubarte-native engine lens):
//! `reject-all main story diverges from the base document` on
//! `file_172_173` (a deleted ` NUMWORDS ` field result), `file_28_29`
//! (`mc:AlternateContent`), and 3 more. The redline wraps the run in
//! `<w:fldSimple><w:del><w:r><w:delText>…` ; reject's `reverse_revisions`
//! only flipped `w:del`→`w:ins` under `w:p`/`w:hyperlink`, so the field-result
//! `w:del` stayed a `w:del` and the trailing accept DROPPED it — silent data
//! loss. Reject must be lossless: every remaining content del↔ins is flipped.

use jubarte::namespaces::W;
use jubarte::revision_processor::reject_revisions_document;
use jubarte::xmllinq::Dom;

/// Reject the given body inner XML; return the concatenated `w:t` text of the
/// original (pre-revision) projection.
fn reject_text(inner: &str) -> String {
    let xml = format!(
        "<w:document xmlns:w=\"{}\" \
         xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\">\
         <w:body>{inner}</w:body></w:document>",
        W::URI
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let out = reject_revisions_document(&mut dom, root);
    dom.descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect()
}

/// A DELETED field result (`w:del` inside `w:fldSimple`) is RESTORED on reject —
/// its text reappears, because reject un-does the deletion.
#[test]
fn reject_restores_deleted_field_result_inside_fldsimple() {
    let got = reject_text(
        "<w:p>\
           <w:r><w:t xml:space=\"preserve\">Num words: </w:t></w:r>\
           <w:fldSimple w:instr=\" NUMWORDS \">\
             <w:del w:author=\"R\" w:id=\"1\" w:date=\"1970-01-01T00:00:00Z\">\
               <w:r><w:delText>12</w:delText></w:r>\
             </w:del>\
           </w:fldSimple>\
         </w:p>",
    );
    assert_eq!(
        got, "Num words: 12",
        "reject must RESTORE the deleted ` NUMWORDS ` field result `12` \
         (a w:del nested in w:fldSimple), not drop it"
    );
}

/// An INSERTED run nested in `w:fldSimple` is REMOVED on reject — reject un-does
/// the insertion, so the text must not survive.
#[test]
fn reject_removes_inserted_run_inside_fldsimple() {
    let got = reject_text(
        "<w:p>\
           <w:r><w:t xml:space=\"preserve\">Pages: </w:t></w:r>\
           <w:fldSimple w:instr=\" NUMPAGES \">\
             <w:ins w:author=\"R\" w:id=\"2\" w:date=\"1970-01-01T00:00:00Z\">\
               <w:r><w:t>99</w:t></w:r>\
             </w:ins>\
           </w:fldSimple>\
         </w:p>",
    );
    assert_eq!(
        got, "Pages: ",
        "reject must REMOVE the inserted field result `99` \
         (a w:ins nested in w:fldSimple)"
    );
}

/// `mc:AlternateContent` (markup-compatibility) is another transparent run
/// container: a deleted run inside `mc:Choice` must be restored on reject.
#[test]
fn reject_restores_deleted_run_inside_mc_choice() {
    let got = reject_text(
        "<w:p>\
           <w:r><w:t xml:space=\"preserve\">This document demonstrates </w:t></w:r>\
           <mc:AlternateContent>\
             <mc:Choice Requires=\"w14\">\
               <w:del w:author=\"R\" w:id=\"3\" w:date=\"1970-01-01T00:00:00Z\">\
                 <w:r><w:delText>valid uses</w:delText></w:r>\
               </w:del>\
             </mc:Choice>\
           </mc:AlternateContent>\
         </w:p>",
    );
    assert_eq!(
        got, "This document demonstrates valid uses",
        "reject must RESTORE a w:del nested in mc:AlternateContent/mc:Choice"
    );
}
