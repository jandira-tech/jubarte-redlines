// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Validity repairs on produced output: hyperlinks out of revisions, inherited invalidity, deleted-text kinds.

use super::*;

/// `w:hyperlink` is not in CT_RunTrackChange's content model: a `w:ins`/`w:del`
/// holds runs, not a hyperlink wrapper. Deleting a whole header or footer wrapped
/// the source's hyperlink inside the `w:del` and produced XML Word refuses to open
/// — `Sch_InvalidElementContentExpectingComplex` at `/w:ftr/w:p/w:del`, which was
/// two of the five Ring 3 open failures (eigenpal…table_2 and
/// sample_document_word_repair…small_font_size).
///
/// Word's shape is the inverse: the hyperlink stays where it is and the revision
/// moves inside it. So `<del>a b <hyperlink>c</hyperlink> d</del>` becomes
/// `<del>a b</del><hyperlink><del>c</del></hyperlink><del>d</del>` — document
/// order preserved, the revision's own author/date carried onto each piece, and no
/// content gained or lost. Each piece gets a fresh `w:id` so the split never mints
/// duplicates.
///
/// `w:fldSimple` gets the same treatment for the same reason: an inserted footer
/// wrapped B's bare `PAGE`/`NUMPAGES` fields inside the `w:ins` (9c337ad7 in the
/// 2026-09-26 English redlines).
///
/// Runs to fixpoint because a hoisted hyperlink can itself contain a nested
/// revision that wraps another hyperlink.
///
/// Allocates its own `w:id`s from one past the highest already in `root`, so it is
/// correct standalone — on the main body the later `fix_up_revision_ids` renumbers
/// everything anyway, but the package-level validity sweep runs this over parts
/// (headers, footers) that never reach that pass.
/// Move `w:hyperlink` out of `w:ins` / `w:del` wrappers.
///
/// Validity, not parity: OOXML forbids a `w:hyperlink` inside
/// `w:ins` / `w:del`. The deleted-whole-header / footer pass
/// swallowed a source hyperlink into `w:del` and Word refused to
/// open the result. Unconditional and last (catches whichever pass
/// above produced the bad shape) and runs before the renumber below
/// fixes up duplicate `w:id`s the split leaves behind.
pub fn hoist_hyperlinks_out_of_revisions(dom: &mut Dom, root: NodeId) {
    let rev_names = [W::ins(), W::del()];
    let wrappers = [W::hyperlink(), W::fld_simple()];
    let mut next_id = dom
        .descendants_and_self(root, None)
        .into_iter()
        .filter_map(|n| {
            dom.attribute(n, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0)
        + 1;
    // Bounded: each pass strictly reduces the number of hyperlinks that have a
    // revision ancestor, so this cannot spin on a pathological document.
    for _ in 0..16 {
        let targets: Vec<NodeId> = rev_names
            .iter()
            .flat_map(|n| dom.descendants(root, Some(n)))
            .filter(|&rev| {
                dom.elements(rev, None)
                    .into_iter()
                    .any(|c| dom.name(c).is_some_and(|n| wrappers.contains(&n)))
            })
            .collect();
        if targets.is_empty() {
            return;
        }
        for rev in targets {
            split_revision_around_hyperlinks(dom, rev, &mut next_id);
        }
    }
}

/// One step of [`hoist_hyperlinks_out_of_revisions`]: replace `rev` in its parent
/// with the alternating sequence of same-shaped revisions and hyperlinks.
pub(super) fn split_revision_around_hyperlinks(dom: &mut Dom, rev: NodeId, next_id: &mut u32) {
    let Some(rev_name) = dom.name(rev) else {
        return;
    };
    let wrappers = [W::hyperlink(), W::fld_simple()];
    let attrs = dom.attributes(rev);
    let children = dom.nodes(rev);

    // Clone `rev`'s element identity (name + attributes) for one of the pieces.
    let w_id = W::id();
    let like_rev = |dom: &mut Dom, next_id: &mut u32| -> NodeId {
        let e = dom.new_element(rev_name.clone());
        for (name, value) in &attrs {
            dom.set_attribute_value(e, name, Some(value));
        }
        dom.set_attribute_value(e, &w_id, Some(&next_id.to_string()));
        *next_id += 1;
        e
    };

    let mut pending: Option<NodeId> = None; // open revision collecting plain children
    for child in children {
        dom.remove(child);
        if dom.name_is(child, &W::fld_simple()) && dom.elements(child, None).is_empty() {
            // An empty field has no result to carry the revision: hoisted
            // bare it stays live, and Word 16.115 hangs on it in a deleted
            // paragraph (b15, 0.12.0 sample). Word's redline spells it as a
            // complex field inside the revision.
            let runs = complex_field_runs(dom, child, rev_name == W::del());
            let target = match pending {
                Some(p) => p,
                None => {
                    let p = like_rev(dom, next_id);
                    dom.add_before_self(rev, p);
                    pending = Some(p);
                    p
                }
            };
            for run in runs {
                dom.add(target, run);
            }
            continue;
        }
        if dom.name(child).is_some_and(|n| wrappers.contains(&n)) {
            pending = None;
            // The hyperlink keeps its place; the revision moves inside it, wrapping
            // whatever the hyperlink held.
            let inner_children = dom.nodes(child);
            if !inner_children.is_empty() {
                let inner = like_rev(dom, next_id);
                for ic in inner_children {
                    dom.remove(ic);
                    dom.add(inner, ic);
                }
                dom.add(child, inner);
            }
            dom.add_before_self(rev, child);
            continue;
        }
        let target = match pending {
            Some(p) => p,
            None => {
                let p = like_rev(dom, next_id);
                dom.add_before_self(rev, p);
                pending = Some(p);
                p
            }
        };
        dom.add(target, child);
    }
    dom.remove(rev);
}

/// An empty `w:fldSimple` as the runs of its complex form: `begin` (with the
/// field's `w:dirty` / `w:fldLock`), the code (`w:delInstrText` when
/// `deleted`), `separate` and `end`.
pub(super) fn complex_field_runs(dom: &mut Dom, field: NodeId, deleted: bool) -> Vec<NodeId> {
    let char_run = |dom: &mut Dom, kind: &str| {
        let run = dom.new_element(W::r());
        let fld_char = dom.new_element(W::fld_char());
        dom.set_attribute_value(fld_char, &W::name("fldCharType"), Some(kind));
        dom.add(run, fld_char);
        (run, fld_char)
    };
    let (begin, begin_char) = char_run(dom, "begin");
    for flag in ["dirty", "fldLock"] {
        if let Some(value) = dom.attribute(field, &W::name(flag)).map(str::to_string) {
            dom.set_attribute_value(begin_char, &W::name(flag), Some(&value));
        }
    }
    let code_run = dom.new_element(W::r());
    let code = dom.new_element(W::name(if deleted { "delInstrText" } else { "instrText" }));
    let instr = dom
        .attribute(field, &W::name("instr"))
        .unwrap_or("")
        .to_string();
    if instr.trim() != instr {
        dom.set_attribute_value(code, &XNamespace::xml().name("space"), Some("preserve"));
    }
    dom.add_text(code, &instr);
    dom.add(code_run, code);
    let (separate, _) = char_run(dom, "separate");
    let (end, _) = char_run(dom, "end");
    vec![begin, code_run, separate, end]
}

/// Repair invalidity we did not create but would otherwise ship.
///
/// We copy styles.xml, numbering.xml and the header/footer parts through from the
/// source, so a source that is already invalid makes *our* output the file Word
/// refuses to open. Three of the five Ring 3 open failures were exactly this — the
/// defect is in the input, but the user is handed our redline. Each rule below is
/// deletion-or-default only: it removes markup that cannot legally be where it is,
/// or supplies the schema's own neutral value. None of them can change a document
/// that was already valid.
///
/// 1. **Unqualified attributes on `w:` elements.** wml.xsd is
///    `attributeFormDefault="qualified"`, so every WordprocessingML attribute is
///    `w:`-prefixed and an unqualified one is invalid by construction. Seen as
///    `paragraphProperties="[object Object]"` on `w:style` — a JS stringification
///    artifact from whatever produced the source. Namespace declarations and
///    attributes in any other namespace (`xml:space`, `mc:Ignorable`, `r:id`) are
///    untouched.
/// 2. **`w:shd` with no `w:val`.** CT_Shd makes `w:val` required. A source writing
///    `<w:shd w:fill="1F4E79"/>` means "this fill, no pattern", which is spelled
///    `w:val="clear"` — so that is what we supply, keeping the author's fill.
/// 3. **`w:highlight` under `w:lvl/w:rPr`.** The numbering symbol's run properties
///    do not accept it. Sources carry `<w:highlight w:val="none"/>` there, which
///    asserts nothing, so dropping it costs no formatting.
///
/// Repair source-inherited validity issues that the redline carries
/// verbatim from the inputs.
///
/// Fixes OOXML shapes that Word silently tolerates (or repairs on
/// open) but strict validators (ooxmlsdk, OOXML SDK) reject with
/// `UnexpectedTag` / `Sch_InvalidElementContent*`. Each fix targets
/// one shape; the pass mutates `root` in place.
pub fn repair_inherited_invalidity(dom: &mut Dom, root: NodeId) {
    for el in dom.descendants_and_self(root, None) {
        let Some(name) = dom.name(el) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        for (attr, _) in dom.attributes(el) {
            if attr.namespace_name().is_empty()
                && !dom.is_namespace_declaration(&attr)
                && attr.local_name() != "xmlns"
            {
                dom.set_attribute_value(el, &attr, None);
            }
        }
    }
    // `ST_HexColor` has no empty value: an empty fill says nothing, and the
    // schema refuses it (#1712).
    let fill = W::fill();
    for shd in dom.descendants_and_self(root, Some(&W::shd())) {
        if dom.attribute(shd, &W::val()).is_none() {
            dom.set_attribute_value(shd, &W::val(), Some("clear"));
        }
        if dom
            .attribute(shd, &fill)
            .is_some_and(|f| f.trim().is_empty())
        {
            dom.set_attribute_value(shd, &fill, None);
        }
    }
    // `CT_TblGridCol` declares `w:w` alone: the width is in twips already
    // (#1712).
    let kind = W::type_();
    for col in dom.descendants_and_self(root, Some(&W::grid_col())) {
        dom.set_attribute_value(col, &kind, None);
    }
    // A theme colour with no `val` (the schema requires one): `auto` changes
    // nothing on screen, because the theme colour overrides `val`.
    for color in dom.descendants_and_self(root, Some(&W::name("color"))) {
        if dom.attribute(color, &W::val()).is_none() {
            dom.set_attribute_value(color, &W::val(), Some("auto"));
        }
    }
    let lvl = W::name("lvl");
    let r_pr = W::r_pr();
    for hl in dom.descendants(root, Some(&W::name("highlight"))) {
        let in_numbering_symbol = dom
            .parent(hl)
            .filter(|&p| dom.name(p).as_ref() == Some(&r_pr))
            .and_then(|p| dom.parent(p))
            .is_some_and(|g| dom.name(g).as_ref() == Some(&lvl));
        if in_numbering_symbol {
            dom.remove(hl);
        }
    }
}

/// Normalize deleted text throughout a produced comparison, including runs
/// that did not go through `convert_run_text_to_del_text`.
///
/// Word's redlines emit `delText` / `delInstrText`. This comparison cleanup
/// was introduced after five corpus documents Word refused carried
/// `instrText` under `del`, commonly in deleted PAGE/NUMPAGES fields. That
/// observation does not make every live text kind under `del` schema-invalid:
/// Transitional EG_RunInnerContent permits both live and deleted text kinds.
/// Identical-input exits therefore preserve the source spelling.
///
/// `w:moveFrom` is deliberately excluded: Word Compare keeps plain `w:t`
/// inside it (see `convert_run_text_to_del_text`'s callers).
pub fn enforce_deleted_text_kinds(dom: &mut Dom, root: NodeId) {
    fn walk(dom: &mut Dom, node: NodeId) {
        for c in dom.nodes(node) {
            if !dom.is_element(c) {
                continue;
            }
            match dom.name(c).as_ref() {
                // A nested revision owns its own text kind — an insertion inside a
                // deletion still holds live `w:t`.
                Some(n) if n == &W::ins() || n == &W::move_from() || n == &W::move_to() => {
                    continue;
                }
                Some(n) if n == &W::t() => dom.set_name(c, W::del_text()),
                Some(n) if n == &W::instr_text() => dom.set_name(c, W::del_instr_text()),
                _ => walk(dom, c),
            }
        }
    }
    for del in dom.descendants(root, Some(&W::del())) {
        walk(dom, del);
    }
}
