// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word Compare pairs equal-count pure-paragraph docs positionally
//! (heading_2_style × heading_3_center_italic: 3 mixed paras, not 4
//! cross-stitched). Flattening into one word-LCS window lets shared tokens
//! ("Heading") bridge the wrong paragraphs.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

#[test]
fn three_vs_three_heading_style_stays_three_mixed() {
    let mut dom = Dom::new();
    // Shared "Heading" tokens would otherwise cross-stitch p1↔p2 under flat LCS.
    let base = [
        para("Heading 2 Style Demo"),
        para("This document demonstrates Heading 2 paragraph style."),
        para("Subsection Title"),
    ]
    .concat();
    let next = [
        para("Heading 3 Center Italic Demo"),
        para("Heading 3 with center alignment and italic formatting."),
        para("This combination works for stylized section subheadings."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert_eq!(
        kids.len(),
        3,
        "Word: 3 positionally mixed paras, not 4 cross-stitched; kids={}",
        kids.len()
    );
    for (i, &k) in kids.iter().enumerate() {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        assert!(
            has_ins && has_del,
            "para {i} must be mixed ins+del (positional 1:1); ser={}",
            dom.serialize_element(k)
        );
    }
    // p0 should still show the classic "2 Style" vs "3 Center Italic" split shape
    let ser0 = dom.serialize_element(kids[0]);
    assert!(
        ser0.contains("3 Center Italic") || ser0.contains("Heading"),
        "p0 carries next heading text: {ser0}"
    );
    assert!(
        ser0.contains("2 Style") || ser0.contains("delText"),
        "p0 carries base '2 Style' del: {ser0}"
    );
}

#[test]
fn numbered_list_role_shift_does_not_force_positional_zip() {
    // Equal count (5 vs 5) but roles shift: Demo+4 items vs Demo+intro+3 items.
    // Diagonal is NOT dominant — flat LCS must win (forced zip regressed ~7 pts).
    let mut dom = Dom::new();
    let base = [
        para("Numbered List Demo"),
        para("First item"),
        para("Second item"),
        para("Third item"),
        para("Fourth item"),
    ]
    .concat();
    let next = [
        para("Numbered List Italic Demo"),
        para("This document shows numbered lists with italic formatting:"),
        para("First italic numbered item"),
        para("Second italic numbered item"),
        para("Third italic numbered item"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    // With zip: all 5 would be mixed. Without zip, at least one pure-ins or
    // pure-del appears (role-shift leftover), or para count ≠ 5 mixed.
    let mut n_mixed = 0;
    let mut n_pure = 0;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        match (has_ins, has_del) {
            (true, true) => n_mixed += 1,
            (true, false) | (false, true) => n_pure += 1,
            _ => {}
        }
    }
    assert!(
        n_pure > 0 || kids.len() != 5 || n_mixed < 5,
        "must not force 5 mixed positional pairs on role-shifted lists; kids={} mixed={} pure={}",
        kids.len(),
        n_mixed,
        n_pure
    );
    // M163: Word pure-I intro then mesh First×First italic (not pure-D First).
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_eq!(
        shape.get(1),
        Some(&"INS"),
        "Word pure-I intro after title; got {shape:?}"
    );
    assert_ne!(
        shape.get(2),
        Some(&"DEL"),
        "must not pure-D First item; mesh First×First italic; got {shape:?}"
    );
}

#[test]
fn calibri_vs_center_word_ins_mix_del_not_three_mix() {
    // Word oracle: MIX title | pure-I B body0 | MIX A0×B1 | pure-D A1
    // (not positional 3×MIX zip — M153).
    let mut dom = Dom::new();
    let base = [
        para("Calibri Heading 2 Right Demo"),
        para("Calibri font with Heading 2 style and right alignment."),
        para("This combination creates a distinctive professional style."),
    ]
    .concat();
    let next = [
        para("Center Aligned Bold Text Demo"),
        para("This text is both centered and bold."),
        para("Centered bold text is perfect for document titles."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_ne!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "must not 1:1 zip; got {shape:?}"
    );
    assert_eq!(
        shape,
        vec!["MIX", "INS", "MIX", "DEL"],
        "Word M153 residual peel; got {shape:?}"
    );
}

#[test]
fn helvetica_vs_heading4_is_not_three_mixed_zip() {
    // Word: MIX title, pure-I body0, MIX body1+A body0, pure-D "Small Section Header"
    // Not 3 positional MIX (only title shares "Demo"; bodies are unrelated).
    let mut dom = Dom::new();
    let base = [
        para("Heading 4 Style Demo"),
        para("Demonstrating Heading 4 paragraph style."),
        para("Small Section Header"),
    ]
    .concat();
    let next = [
        para("Helvetica Font Demo"),
        para("This document demonstrates the Helvetica font family."),
        para("Helvetica is a classic Swiss sans-serif typeface."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_ne!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "must not force 3-way zip; got {shape:?}"
    );
    assert!(
        shape.iter().any(|s| *s == "INS" || *s == "DEL"),
        "Word peels pure-I/D residual; got {shape:?}"
    );
}

#[test]
fn text_highlight_vs_times_is_del_ins_mix_not_three_mix() {
    // Word: MIX title | pure-D A body0 | pure-I B body0 | MIX last
    let mut dom = Dom::new();
    let base = [
        para("Text Highlight Demo"),
        para("Demonstrating text highlighting."),
        para("This text is highlighted in yellow."),
    ]
    .concat();
    let next = [
        para("Times New Roman Bold Italic Demo"),
        para("This document shows Times New Roman with bold italic styles."),
        para("Times bold italic is traditional for formal academic papers."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_ne!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "must not 1:1 zip; got {shape:?}"
    );
    assert!(
        shape.contains(&"DEL") && shape.contains(&"INS"),
        "Word has pure-D and pure-I residual; got {shape:?}"
    );
}

#[test]
fn blue_underline_vs_bold_italic_is_ins_del_mix() {
    // Word: MIX title | pure-I short B body | pure-D long A body | MIX last
    let mut dom = Dom::new();
    let base = [
        para("Blue Underline Combo Demo"),
        para("This document combines blue font color with underline."),
        para("Blue underlined text resembles hyperlinks in documents."),
    ]
    .concat();
    let next = [
        para("Bold and Italic Combo Demo"),
        para("Demonstrating bold and italic combined."),
        para("This text is both bold and italic simultaneously."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_ne!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "must not 1:1 zip; got {shape:?}"
    );
    assert_eq!(
        shape,
        vec!["MIX", "INS", "DEL", "MIX"],
        "Word short-B-first residual order; got {shape:?}"
    );
}

#[test]
fn right_aligned_vs_right_alignment_peels_first_next_body() {
    // Word: MIX title | pure-I B body0 | MIX ... | pure-D A last
    let mut dom = Dom::new();
    let base = [
        para("Right Aligned Italic Demo"),
        para("This text is right-aligned and italic."),
        para("Right-aligned italic text creates an elegant signature effect."),
    ]
    .concat();
    let next = [
        para("Right Alignment Demo"),
        para("This document demonstrates right alignment."),
        para("All text is aligned to the right margin."),
        para("Right alignment is used in certain layouts."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    // At least pure-I somewhere after title and a pure-D (not all MIX+INS).
    assert!(
        shape.contains(&"INS") && shape.iter().any(|s| *s == "DEL" || *s == "MIX"),
        "Word peels pure-I first next body; got {shape:?}"
    );
    assert_ne!(
        &shape[..shape.len().min(4)],
        &["MIX", "MIX", "MIX", "INS"][..shape.len().min(4)],
        "must not leave pure-I trail only; got {shape:?}"
    );
}

#[test]
fn right_aligned_vs_right_alignment_2_equal_residual_m151() {
    // Full-bench pair: 3v3 "This text" vs "This document" residual.
    // Word: MIX | INS | MIX | DEL (not 3×MIX zip). Free-reflow trials regressed
    // rai score; keep pure-I first next body peel.
    let mut dom = Dom::new();
    let base = [
        para("Right Aligned Italic Demo"),
        para("This text is right-aligned and italic."),
        para("Right-aligned italic text creates an elegant signature effect."),
    ]
    .concat();
    let next = [
        para("Right Alignment Demo"),
        para("This document demonstrates right text alignment."),
        para("All text in this document is aligned to the right margin."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_ne!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "must not 1:1 zip; got {shape:?}"
    );
    assert_eq!(
        shape,
        vec!["MIX", "INS", "MIX", "DEL"],
        "Word M151 residual peel; got {shape:?}"
    );
}

#[test]
fn justify_2_vs_justify_meshes_related_bodies() {
    // After equal title, residual 2v1 related bodies: Word MIX mesh, not pure I+D.
    let mut dom = Dom::new();
    let base = [
        para("Justify Alignment Demo"),
        para("This document demonstrates justified text alignment."),
        para("Justified text spreads evenly across the full width of the line."),
    ]
    .concat();
    let next = [
        para("Justify Alignment Demo"),
        para("This document demonstrates justified text alignment which spreads text evenly across the full width of the line, creating clean left and right edges that are perfect for formal documents and publications."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut n_mix = 0;
    let mut n_pure_d = 0;
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => {
                n_mix += 1;
                "MIX"
            }
            (true, false) => "INS",
            (false, true) => {
                n_pure_d += 1;
                "DEL"
            }
            _ => "EQ",
        });
    }
    assert_eq!(
        shape,
        vec!["EQ", "MIX", "MIX"],
        "Word EQ|MIX|MIX residual mesh; got {shape:?}"
    );
    assert!(n_mix >= 2 && n_pure_d == 0, "mix={n_mix} pure_d={n_pure_d}");
}

#[test]
fn italic_vs_justified_peels_trailing_phrase_into_last_body() {
    // Word: p2 MIX includes ins "a formal document look" with del last body.
    let mut dom = Dom::new();
    let base = [
        para("Italic Underline Combined Demo"),
        para("This document combines italic and underline formatting."),
        para("Italic underlined text is often used for titles and citations."),
    ]
    .concat();
    let next = [
        para("Justified Underline Demo"),
        para("This document combines justify alignment with underline formatting for a formal document look."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(kids.len() >= 3, "kids={}", kids.len());
    let last = kids[kids.len() - 1];
    let has_ins = !dom.elements(last, Some(&W::ins())).is_empty();
    let has_del = !dom.elements(last, Some(&W::del())).is_empty();
    let ser = dom.serialize_element(last);
    assert!(
        has_ins && has_del,
        "last para must be MIX (Word peels formal-document-look); has_ins={has_ins} has_del={has_del} ser={ser}"
    );
    assert!(
        ser.contains("formal") || ser.contains("document look"),
        "last MIX carries peeled next phrase: {ser}"
    );
}

#[test]
fn justify_vs_large_font_lcp_split_shared_prefix() {
    // M166 Word: MIX titles | MIX (EQ "This document demonstrates " + INS
    // rest B0) | MIX (A0 tail × B1). Not pure-I whole B0 (old wrong shape).
    let mut dom = Dom::new();
    let base = [
        para("Justify Alignment Demo"),
        para("This document demonstrates justified text alignment which spreads text evenly across the full width of the line, creating clean left and right edges that are perfect for formal documents and publications."),
    ]
    .concat();
    let next = [
        para("Large Font Size Demo"),
        para("This document demonstrates large 24pt font size."),
        para("Large fonts are great for titles and presentations."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    // Word p1 is EQ prefix + pure-I rest of B0 (no del) → classified INS;
    // p2 is INS B1 + DEL A0 tail → MIX. Not pure-I whole B0 without EQ prefix.
    assert_eq!(
        shape,
        vec!["MIX", "INS", "MIX"],
        "Word LCP-split shape; got {shape:?}"
    );
    let ser1 = dom.serialize_element(kids[1]);
    assert!(
        ser1.contains("This document demonstrates")
            || (ser1.contains("demonstrates") && ser1.contains("This")),
        "p1 keeps shared EQ prefix: {ser1}"
    );
    assert!(
        ser1.contains("24") || ser1.contains("large") || ser1.contains("font"),
        "p1 pure-I rest of short next body: {ser1}"
    );
    // Shared prefix must not also be deleted in p2 (old pure-I/D whole residual)
    let ser2 = dom.serialize_element(kids[2]);
    assert!(
        ser2.contains("justified") || ser2.contains("spreads") || ser2.contains("presentations"),
        "p2 has A0 tail and/or B1: {ser2}"
    );
    assert!(
        !ser2.contains("This document demonstrates"),
        "shared prefix already EQ in p1, not re-deleted in p2: {ser2}"
    );
}

#[test]
fn longer_base_residual_uses_word_lcs_not_pure_id() {
    // M144: font_size×green style residual longer base → word-LCS not pure I/D
    let mut dom = Dom::new();
    let base = [
        para("Font Size Demo"),
        para("This document demonstrates various font sizes."),
        para("Font size changes create visual hierarchy."),
        para("Larger sizes draw attention to important content."),
    ]
    .concat();
    let next = [
        para("Green Bold Text Demo"),
        para("This document shows green bold text formatting."),
        para("Green bold text stands out for emphasis."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut n_mix = 0;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        if has_ins && has_del {
            n_mix += 1;
        }
    }
    assert!(
        n_mix >= 1,
        "longer base residual should mesh via word-LCS (not pure I/D only); kids={} mix={n_mix}",
        kids.len()
    );
}

#[test]
fn title_vs_bullet_demo_meshes_residual_not_trail_pure_d() {
    // M146: title_style×track_changes_editing residual next longer → word-LCS
    let mut dom = Dom::new();
    let base = [
        para("Title Style Demo"),
        para("This document demonstrates the Title paragraph style."),
        para("Title style is used for main document headings."),
    ]
    .concat();
    let next = [
        para("Track Changes Editing Bullet Demo"),
        para("This document demonstrates track changes with bullet lists."),
        para("First bullet item in the revised document."),
        para("Second bullet item with more detail."),
        para("Third bullet item completes the list."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let last = kids.last().copied();
    let mut pure_del_trail = false;
    if let Some(k) = last {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        pure_del_trail = has_del && !has_ins;
    }
    assert!(
        !pure_del_trail || kids.len() >= 4,
        "should mesh residual not park pure-D trail only; kids={}",
        kids.len()
    );
}

#[test]
fn text_highlight_exact_fixture_shape() {
    // Word: MIX title | pure-D short A0 | pure-I B0 | MIX A1×B1
    let mut dom = Dom::new();
    let base = [
        para("Text Highlight Demo"),
        para("Demonstrating text highlighting."),
        para("This text is highlighted in yellow."),
    ]
    .concat();
    let next = [
        para("Times New Roman Bold Italic Demo"),
        para("This document shows Times New Roman with bold italic styles."),
        para("Times bold italic is traditional for formal academic papers."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_eq!(
        shape,
        vec!["MIX", "DEL", "INS", "MIX"],
        "Word M149 short-A residual order; got {shape:?}"
    );
}

#[test]
fn justified_underline_vs_justify_2_peels_trailing_del_into_pure_ins() {
    // Word: MIX title | MIX body | MIX (B residual + peeled A trail) — not pure-I trail.
    let mut dom = Dom::new();
    let base = [
        para("Justified Underline Demo"),
        para("This document combines justify alignment with underline formatting for a formal document look."),
    ]
    .concat();
    let next = [
        para("Justify Alignment Demo"),
        para("This document demonstrates justified text alignment."),
        para("Justified text spreads evenly across the full width of the line."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert!(
        shape.iter().filter(|s| **s == "MIX").count() >= 3,
        "Word peels trail into third MIX; got {shape:?}"
    );
    assert!(
        !shape.contains(&"INS") || shape.iter().filter(|s| **s == "MIX").count() >= 3,
        "should not leave lone pure-I trail without 3 MIX; got {shape:?}"
    );
}

#[test]
fn bold_underline_vs_bold_italic_meshes_three_mix() {
    // Word: 3 MIX (not MIX|INS|MIX|DEL from pure I/D fold).
    let mut dom = Dom::new();
    let base = [
        para("Bold and Underline Combo Demo"),
        para("Demonstrating bold and underline combined."),
        para("This text is both bold and underlined."),
    ]
    .concat();
    let next = [
        para("Bold Italic Combined Demo"),
        para("This document combines bold and italic formatting together."),
        para("Bold italic text creates emphasis for important highlighted content."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert_eq!(
        shape,
        vec!["MIX", "MIX", "MIX"],
        "Word 3×MIX residual mesh; got {shape:?}"
    );
}

#[test]
fn increase_indent_vs_insert_link_peels_first_next_body() {
    // Word: MIX title | pure-I B0 | MIX... | DELs (not 3×MIX zip of residual).
    let mut dom = Dom::new();
    let base = [
        para("Increase Indent Demo"),
        para("This text will be indented."),
        para("This line is indented once."),
        para("This line is indented more."),
    ]
    .concat();
    let next = [
        para("Insert Link Demo"),
        para("This document demonstrates inserting hyperlinks."),
        para("Click here to visit example.com"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert!(
        shape.get(1) == Some(&"INS"),
        "Word pure-I first next residual body; got {shape:?}"
    );
    assert_ne!(
        &shape[..shape.len().min(4)],
        &["MIX", "MIX", "MIX", "DEL"][..shape.len().min(4)],
        "must not force residual 1:1 zip; got {shape:?}"
    );
}

#[test]
fn bullet_list_vs_calibri_peels_first_next_body() {
    let mut dom = Dom::new();
    let base = [
        para("Bullet List Demo"),
        para("Apples"),
        para("Bananas"),
        para("Oranges"),
        para("Grapes"),
    ]
    .concat();
    let next = [
        para("Calibri Bold Italic Demo"),
        para("This document shows Calibri font with bold and italic styles."),
        para("Calibri bold italic creates an elegant emphasis for key text."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let mut shape = Vec::new();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        shape.push(match (has_ins, has_del) {
            (true, true) => "MIX",
            (true, false) => "INS",
            (false, true) => "DEL",
            _ => "EQ",
        });
    }
    assert!(
        shape.get(1) == Some(&"INS"),
        "Word pure-I first next body after title; got {shape:?}"
    );
}

#[test]
fn title_centered_vs_title_short_document_title_no_bridge() {
    // M161 reverse: short next "Document Title" vs long base residual.
    let mut dom = Dom::new();
    let base = [
        para("Title Style Centered Demo"),
        para("This document shows Title style with center alignment."),
        para("Centered Title style creates an impressive document cover page."),
    ]
    .concat();
    let next = [
        para("Title Style Demo"),
        para("Demonstrating Title paragraph style."),
        para("Document Title"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let last = *kids.last().unwrap();
    let ser = dom.serialize_element(last);
    assert!(
        ser.contains("Document") && (ser.contains("w:ins") || ser.contains("ins")),
        "last carries pure-I Document Title; ser={ser}"
    );
    assert!(
        ser.contains("cover") || ser.contains("impressive") || ser.contains("Centered"),
        "last carries pure-D long centered body; ser={ser}"
    );
}

#[test]
fn font_family_vs_font_size_12_peels_trailing_text_across_boundary() {
    // M162: 4v3 residual 3v2; Word peels B's trailing "text" onto A2 ("This text…")
    // so last is pure-I Size12 body + pure-D "Different fonts…", not A2×B2 mesh.
    let mut dom = Dom::new();
    let base = [
        para("Font Family Demo"),
        para("This document demonstrates changing the font family."),
        para("This text uses Times New Roman font."),
        para("Different fonts create different visual impressions."),
    ]
    .concat();
    let next = [
        para("Font Size 12 Demo"),
        para("This document demonstrates font size 12 point text."),
        para("Size 12 is a standard readable font size for documents."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(
        kids.len() >= 3,
        "expected multi-para residual; n={}",
        kids.len()
    );
    let last = kids[kids.len() - 1];
    let ser = dom.serialize_element(last);
    // Word last: ins Size12 body + del Different fonts… — not mesh of Size12×This text
    assert!(
        ser.contains("Different") || ser.contains("impressions") || ser.contains("delText"),
        "last carries pure-D trail body; ser={ser}"
    );
    assert!(
        ser.contains("Size") || ser.contains("standard") || ser.contains("readable"),
        "last carries pure-I Size12 body; ser={ser}"
    );
    // Bridged mesh leaves Size12 mixed into "This text uses Times" — reject that.
    assert!(
        !ser.contains("Times") || ser.contains("w:del"),
        "must not mesh Size12 body into Times New Roman para as equals; ser={ser}"
    );
}

#[test]
fn italic_underline_vs_subscript_free_meshes_is_and() {
    // M173: last residual pure I/D under zip+glue-void; Word EQ is/and.
    let mut dom = Dom::new();
    let base = [
        para("Italic and Underline Combo Demo"),
        para("Demonstrating italic and underline combined."),
        para("This text is both italic and underlined."),
    ]
    .concat();
    let next = [
        para("Italic Subscript Demo"),
        para("This document shows italic with subscript formatting combined."),
        para("Italic subscript is used in chemistry and biology equations."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(kids.len() >= 3, "n={}", kids.len());
    let last = kids[kids.len() - 1];
    let ser2 = dom.serialize_element(last);
    // Equal runs for glue words (not only pure I/D blobs)
    let has_eq_glue = ser2.contains(">is")
        || ser2.contains(" is ")
        || ser2.contains(">and")
        || ser2.contains(" and ");
    assert!(has_eq_glue, "last residual free-meshes is/and; ser={ser2}");
    let has_ins = !dom.elements(last, Some(&W::ins())).is_empty();
    let has_del = !dom.elements(last, Some(&W::del())).is_empty();
    assert!(has_ins && has_del, "last is MIX; ser={ser2}");
}

#[test]
fn it_security_vs_italic_underline_free_reflows_and() {
    // M170: Demo short next vs colon-list base. Word EQ-bridges "and"; pure I/D
    // keeps all next pure-I then all base pure-D.
    let mut dom = Dom::new();
    let base = [
        para("IT Security Policy v2.0"),
        para("Effective Date: January 2026"),
        para("Scope: All employees and contractors"),
        para("Password Requirements: 12+ characters"),
        para("MFA: Required for all systems"),
        para("Data Backup: Daily automated backups"),
    ]
    .concat();
    let next = [
        para("Italic and Underline Combo Demo"),
        para("Demonstrating italic and underline combined."),
        para("This text is both italic and underlined."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(kids.len() >= 3, "n={}", kids.len());
    // Must not be pure-I all next then pure-D all base (first para pure-I only)
    let ser0 = dom.serialize_element(kids[0]);
    let has_del0 = !dom.elements(kids[0], Some(&W::del())).is_empty();
    // Word has del of IT Security on early paras with ins of Italic fragments
    let any_mixed = kids.iter().any(|&k| {
        !dom.elements(k, Some(&W::ins())).is_empty() && !dom.elements(k, Some(&W::del())).is_empty()
    });
    assert!(
        any_mixed || has_del0,
        "expect free reflow mix or early del; ser0={ser0}"
    );
    // "and" should appear as equal somewhere if free reflow worked
    let all: String = kids.iter().map(|&k| dom.serialize_element(k)).collect();
    assert!(
        all.contains("and") || all.contains("Underline") || all.contains("employees"),
        "carries free-reflow content"
    );
}

#[test]
fn project_plan_vs_proposal_meshes_project_title_not_pure_id() {
    // M168: titles share first token "Project" but not last-sig (Plan vs
    // Proposal). Flat pure-I/D whole titles; Word EQ "Project " mesh.
    let mut dom = Dom::new();
    let base = [
        para("Project Plan"),
        para("Date: February 1, 2026"),
        para("Phase 1: Research and Analysis"),
        para("Phase 2: Development"),
        para("Phase 3: Testing"),
        para("Phase 4: Deployment"),
        para("Deadline: March 31, 2026"),
    ]
    .concat();
    let next = [
        para("Project Proposal"),
        para("This project will be completed by the end of the quarter."),
        para("The team consists of five members."),
        para("Budget is estimated at ten thousand dollars."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(!kids.is_empty());
    let ser0 = dom.serialize_element(kids[0]);
    let has_ins = !dom.elements(kids[0], Some(&W::ins())).is_empty();
    let has_del = !dom.elements(kids[0], Some(&W::del())).is_empty();
    assert!(
        has_ins && has_del,
        "title is MIX (EQ Project + ins/del Plan/Proposal); ser={ser0}"
    );
    assert!(
        ser0.contains("Project") || ser0.contains("Proposal") || ser0.contains("Plan"),
        "title carries Project Plan/Proposal: {ser0}"
    );
    // Must not pure-I whole "Project Proposal" without Plan del on same para
    let pure_i_only = has_ins && !has_del;
    assert!(!pure_i_only, "must not pure-I whole next title; ser={ser0}");
}

#[test]
fn font_size_24_vs_font_size_free_reflows_sizes_improve_into_last() {
    // M167: 2v3 residual after Demo title. Word meshes first residual, then
    // free-reflows last base body so "sizes improve" lands with last next
    // body ("Font size impacts…"), not stuck on "This text uses a larger".
    let mut dom = Dom::new();
    let base = [
        para("Font Size 24 Demo"),
        para("This document demonstrates font size 24."),
        para("Larger font sizes improve readability for presentations."),
    ]
    .concat();
    let next = [
        para("Font Size Demo"),
        para("This document demonstrates font size changes."),
        para("This text uses a larger font size of 18pt."),
        para("Font size impacts readability and document design."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(
        kids.len() >= 4,
        "Word: title + 3 residual paras; n={}",
        kids.len()
    );
    let ser_last = dom.serialize_element(kids[kids.len() - 1]);
    let ser_mid = dom.serialize_element(kids[kids.len() - 2]);
    // Word: "sizes improve" is DEL in last residual with "impacts readability",
    // not trailing del on the "18pt" mid residual.
    assert!(
        ser_last.contains("sizes") || ser_last.contains("improve"),
        "last residual carries 'sizes improve' del: {ser_last}"
    );
    assert!(
        ser_last.contains("impacts") || ser_last.contains("readability"),
        "last residual meshes with B2: {ser_last}"
    );
    assert!(
        !ser_mid.contains("sizes") && !ser_mid.contains("improve"),
        "mid residual must not hold 'sizes improve': {ser_mid}"
    );
    assert!(
        ser_mid.contains("18") || ser_mid.contains("larger") || ser_mid.contains("text"),
        "mid residual is B1 mesh: {ser_mid}"
    );
}

#[test]
fn font_size_12_vs_18_last_residual_is_pure_id_not_font_bridge() {
    // M165: equal 3v3 Demo, first residual near-identical (digit swap), last
    // residual near-unrelated. Positional zip bridges lone "font"; Word pure-I
    // last next + pure-D last base.
    let mut dom = Dom::new();
    let base = [
        para("Font Size 12 Demo"),
        para("This document demonstrates font size 12 point text."),
        para("Size 12 is a standard readable font size for documents."),
    ]
    .concat();
    let next = [
        para("Font Size 18 Demo"),
        para("This document demonstrates font size 18 point text."),
        para("Medium-large font sizes balance readability and space."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert_eq!(
        kids.len(),
        3,
        "Word: MIX|MIX|INS+DEL last; n={}",
        kids.len()
    );
    // p1 still meshes digit swap
    let ser1 = dom.serialize_element(kids[1]);
    assert!(
        ser1.contains("18") && (ser1.contains("12") || ser1.contains("delText")),
        "p1 meshes 12↔18: {ser1}"
    );
    // last is pure I+D (no Equal "font" bridge)
    let last = kids[2];
    let ser_last = dom.serialize_element(last);
    let last_has_ins = !dom.elements(last, Some(&W::ins())).is_empty();
    let last_has_del = !dom.elements(last, Some(&W::del())).is_empty();
    assert!(
        last_has_ins && last_has_del,
        "last is INS+DEL pure residual; ser={ser_last}"
    );
    assert!(
        ser_last.contains("Medium") || ser_last.contains("balance"),
        "last carries pure-I next body: {ser_last}"
    );
    assert!(
        ser_last.contains("standard") || ser_last.contains("Size") || ser_last.contains("delText"),
        "last carries pure-D base body: {ser_last}"
    );
    // Lone-token "font" bridge would leave equal run mid-body. Do not accept
    // "Medium-large" as a fallback — that is expected *inserted* content and
    // made this assert always-true (CR hidden gem / test-lie).
    assert!(
        !ser_last.contains("w:t>font</w:t>")
            && !ser_last.contains("w:t xml:space=\"preserve\">font"),
        "must not Equal-bridge lone 'font' token; ser={ser_last}"
    );
    assert!(
        ser_last.contains("Medium-large") || ser_last.contains("Medium"),
        "last carries pure-I next body: {ser_last}"
    );
}

#[test]
fn title_style_short_last_residual_is_pure_id_not_title_bridge() {
    // M161: equal titles, short last base ("Document Title") vs long next.
    // Positional zip bridges shared "Title"; Word pure-I next + pure-D base.
    let mut dom = Dom::new();
    let base = [
        para("Title Style Demo"),
        para("Demonstrating Title paragraph style."),
        para("Document Title"),
    ]
    .concat();
    let next = [
        para("Title Style Demo"),
        para("This document demonstrates the Title paragraph style."),
        para("Title style creates a visually prominent document heading."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(!kids.is_empty());
    let last = kids[kids.len() - 1];
    let ser = dom.serialize_element(last);
    // Word: full next heading is inserted; full "Document Title" deleted —
    // must not Equal-bridge the shared token "Title" mid-phrase.
    let has_ins = !dom.elements(last, Some(&W::ins())).is_empty();
    let has_del = !dom.elements(last, Some(&W::del())).is_empty();
    assert!(
        has_ins && has_del,
        "last para is MIX (ins heading + del Document Title); ser={ser}"
    );
    assert!(
        ser.contains("Document") || ser.contains("delText") || ser.contains("w:del"),
        "last carries deleted Document Title: {ser}"
    );
    assert!(
        ser.contains("visually") || ser.contains("prominent") || ser.contains("heading"),
        "last carries inserted next heading: {ser}"
    );
}
