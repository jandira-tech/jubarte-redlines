//! Pair 01 (word_tolerated_duplicate_ppr vs word_tolerated_misplaced_link)
//! Word emits pure insert-all-next then delete-all-base. Ours LCS-matches the
//! single letter "a" into a MIX paragraph and interleaves base deletions mid-flow.
//! Evidence: batch_to_fix rank-1 Word redline vs our redline body order.

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

/// Minimal recreation of pair 01: short base with lone "a"/"x"/"b" list items,
/// long unrelated next whose first sentence contains the word "a".
#[test]
fn pair01_junk_letter_a_must_not_mix_unrelated_docs() {
    let mut dom = Dom::new();
    // base: numbered-ish short paras like word_tolerated_duplicate_ppr
    let base = [
        para("a"),
        para("x"),
        para(""),
        para("x"),
        para(""),
        para("b"),
    ]
    .concat();
    // next: long bold-tester style content containing the word "a"
    let next = [
        para("OOXML w:b (bold) tester: ST_OnOff variants, rStyle-only cases, rStyle + inline overrides, and a linked style pair (paragraph+character) to verify behavior."),
        para("A) ST_OnOff values for w:b on a run:"),
        para("  - w:b w:val=(w:val omitted): Sample text"),
        para("  - w:b w:val=true: Sample text"),
        para("  - w:b w:val=1: Sample text"),
        para("  - w:b w:val=on: Sample text"),
        para("  - w:b w:val=false: Sample text"),
        para("  - w:b w:val=0: Sample text"),
        para("  - w:b w:val=off: Sample text"),
        para(""),
        para("B) rStyle-only cases (no inline w:b):"),
        para("  - rStyle=SD_BoldChar (should be bold): Sample via SD_BoldChar"),
        para("  - rStyle=Strong (built-in bold): Sample via Strong"),
        para("  - rStyle=SD_PlainChar (not bold): Sample via SD_PlainChar"),
        para(""),
        para("C) combinations:"),
        para("  - rStyle=SD_BoldChar + w:b=0 => expect NOT bold"),
        para("  - rStyle=Strong + w:b=true => expect bold"),
        para(""),
        para("Table examples (linked style + inline overrides):"),
        // tiny table so H4 flattens mixed para+table like the real pair
        String::from(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>
           <w:tblGrid><w:gridCol w:w="4680"/><w:gridCol w:w="4680"/></w:tblGrid>
           <w:tr><w:tc><w:p><w:r><w:t>Should be bold</w:t></w:r></w:p></w:tc>
                 <w:tc><w:p><w:r><w:t>Should NOT be bold</w:t></w:r></w:p></w:tc></w:tr>
           </w:tbl>"#,
        ),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom.elements(body, None);

    // Classify each top-level block
    let mut classes = Vec::new();
    for &k in &kids {
        let name = dom
            .name(k)
            .map(|n| n.local_name().to_string())
            .unwrap_or_default();
        if name == "sectPr" {
            continue;
        }
        let has_ins = !dom.descendants(k, Some(&W::ins())).is_empty();
        let has_del = !dom.descendants(k, Some(&W::del())).is_empty();
        let has_eq_t = dom.descendants(k, Some(&W::t())).iter().any(|&t| {
            !dom.ancestors_and_self(t, None)
                .into_iter()
                .any(|a| matches!(dom.name(a), Some(n) if n == W::ins() || n == W::del()))
        });
        let class = match (has_ins, has_del, has_eq_t) {
            (true, true, _) | (_, _, true) if has_ins || has_del => "MIX",
            (true, false, false) => "INS",
            (false, true, false) => "DEL",
            _ => "OTHER",
        };
        // also capture a short text preview
        let mut text = String::new();
        for t in dom.descendants(k, Some(&W::t())) {
            text.push_str(&dom.value(t));
        }
        for t in dom.descendants(k, Some(&W::name("delText"))) {
            text.push_str(&format!("[D:{}]", dom.value(t)));
        }
        classes.push((class, text.chars().take(40).collect::<String>()));
    }

    for (i, (c, t)) in classes.iter().enumerate() {
        eprintln!("[{i:02}] {c} {t:?}");
    }

    // Word shape: all INS (next) first, then all DEL (base). No MIX.
    let mix: Vec<_> = classes
        .iter()
        .enumerate()
        .filter(|(_, (c, _))| *c == "MIX")
        .collect();
    assert!(
        mix.is_empty(),
        "Word keeps unrelated docs as pure INS then pure DEL; MIX junk-anchor on letter 'a': {mix:?}"
    );
    let first_del = classes.iter().position(|(c, _)| *c == "DEL");
    let last_ins = classes.iter().rposition(|(c, _)| *c == "INS");
    if let (Some(fd), Some(li)) = (first_del, last_ins) {
        assert!(
            li < fd,
            "INS block must fully precede DEL block (Word order); last_ins={li} first_del={fd} classes={classes:?}"
        );
    }
    // All base letters deleted as pure DEL, not equal
    let del_text: String = classes
        .iter()
        .filter(|(c, _)| *c == "DEL")
        .map(|(_, t)| t.as_str())
        .collect();
    assert!(
        del_text.contains("[D:a]"),
        "base 'a' must be pure del: {del_text}"
    );
    assert!(
        del_text.contains("[D:b]"),
        "base 'b' must be pure del: {del_text}"
    );
}

/// Pair 02 shape: base 3×3 plain table vs next vMerge table. Word mixes cell
/// content (AAA + del R1C1 in the same cell). Pre-fix: whole-table del rows
/// then ins rows because structure hashes differed under the merged-cell
/// fallback.
#[test]
fn pair02_vmerge_structure_mismatch_still_mixes_cells() {
    let mut dom = Dom::new();
    let base_tbl = r#"<w:tbl>
      <w:tblPr><w:tblW w:w="6000" w:type="dxa"/></w:tblPr>
      <w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid>
      <w:tr><w:tc><w:p><w:r><w:t>R1C1</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R1C2</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R1C3</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>R2C1</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R2C2</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R2C3</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>R3C1</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R3C2</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>R3C3</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl>"#;
    // next: fewer columns, vMerge — structure hash differs
    let next_tbl = r#"<w:tbl>
      <w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>
      <w:tblGrid><w:gridCol w:w="4680"/><w:gridCol w:w="4680"/></w:tblGrid>
      <w:tr><w:tc><w:p><w:r><w:t>AAA</w:t></w:r></w:p></w:tc>
            <w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr><w:p><w:r><w:t>BBB</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>CCC</w:t></w:r></w:p></w:tc>
            <w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p><w:r><w:t></w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>DDD</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>EEE</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>FFF</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>GGG</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl>"#;
    let (r1, b1) = doc_body(&mut dom, base_tbl);
    let (r2, b2) = doc_body(&mut dom, next_tbl);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    // First table must carry BOTH next's AAA (ins) and base's R1C1 (del) —
    // not pure-del-all-base-rows then pure-ins-all-next-rows.
    assert!(
        x.contains("AAA") && x.contains("R1C1") && x.contains("<w:ins") && x.contains("delText"),
        "vMerge structure mismatch must still cell-mix AAA with deleted R1C1: {}",
        &x[..x.len().min(2500)]
    );
    // Sanity: they should appear in the same table (one tbl element), not as
    // fully separate del-table-then-ins-table blocks with no shared cells.
    let first_tbl_end = x.find("</w:tbl>").expect("a table");
    let first_tbl = &x[..first_tbl_end];
    assert!(
        first_tbl.contains("AAA") && first_tbl.contains("R1C1"),
        "AAA and R1C1 must share the first table (positional row merge): {}",
        &first_tbl[..first_tbl.len().min(2000)]
    );
}

/// Pair 06 shape: long unrelated base vs short (3-para) next. Word emits
/// ins-all-next first; the both-sides->3 contentful-group gate used to skip
/// the word-mode short-circuit and leave deleted-first order.
#[test]
fn pair06_short_vs_long_unrelated_inserts_first() {
    let mut dom = Dom::new();
    // long base: 6 distinct paras
    let base = (0..6)
        .map(|i| para(&format!("base unique paragraph number {i} lorem alpha{i}")))
        .collect::<String>();
    // short next: 3 paras (below the old both-sides >3 threshold)
    let next = [
        para("Open Sans Bold Underline Demo"),
        para("This document shows Open Sans font with bold and underline."),
        para("Open Sans bold underline creates a distinctive modern heading style."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first = dom.serialize_element(dom.elements(body, Some(&W::p()))[0]);
    assert!(
        first.contains("Open Sans") && first.contains("<w:ins") && !first.contains("delText"),
        "short next must lead as pure INS (Word order), got: {first}"
    );
    let last = dom.serialize_element(*dom.elements(body, Some(&W::p())).last().unwrap());
    assert!(
        last.contains("base unique") && last.contains("delText"),
        "long base must trail as pure DEL, got: {last}"
    );
}

/// Eigenpal-like: long unrelated base with empties vs short next that starts
/// with title+empty. Word emits [I title][I empty][D base…]; empty pPr EQ
/// anchors used to produce EQ empties mid-gap and reorder.
#[test]
fn eigenpal_empty_para_not_equal_across_unrelated() {
    let mut dom = Dom::new();
    let mut base = String::new();
    base.push_str(&para("eigenpal/docx-editor"));
    base.push_str(&para("Project Charter"));
    base.push_str(&para(""));
    for i in 0..12 {
        base.push_str(&para(&format!("base body paragraph {i} unique alpha{i}")));
        if i % 3 == 0 {
            base.push_str(&para(""));
        }
    }
    let next = [
        para("Employee Directory"),
        para(""),
        para("Name Department Role"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<_> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&e| dom.name(e) == Some(W::p()))
        .collect();
    // First must be pure INS of next title
    let first = dom.serialize_element(paras[0]);
    assert!(
        first.contains("Employee Directory") && first.contains("<w:ins"),
        "next title leads as ins: {first}"
    );
    // No EQ empty: every empty para should carry ins or del marker
    for (i, &p) in paras.iter().enumerate() {
        let x = dom.serialize_element(p);
        let pure_empty_eq = !x.contains("<w:ins")
            && !x.contains("<w:del")
            && !x.contains("delText")
            && !x.contains("Employee")
            && !x.contains("eigenpal")
            && !x.contains("Project")
            && !x.contains("base body")
            && !x.contains("Name Department");
        if pure_empty_eq && x.contains("<w:p") {
            // allow sect-only? shouldn't happen
            panic!("EQ empty paragraph at index {i} (Word would mark ins/del): {x}");
        }
    }
}

/// Empty deleted paragraphs carry only pPr/rPr del (no body w:del). They must
/// still count as fully-deleted so ins-before-del reordering can fire
/// (support_tickets_table_table_bookmark_end: Word III then DDD then table).
#[test]
fn empty_mark_only_del_participates_in_ins_before_del() {
    let mut dom = Dom::new();
    // base: two short paras + empty + marker para
    let base = [
        para("Support tickets intro"),
        para("Ticket body line"),
        para(""),
    ]
    .concat();
    // next: longer intro (3 paras) unrelated
    let next = [
        para("Table Widths"),
        para("This document includes tables."),
        para("Test 1 Fixed Width"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<_> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&e| dom.name(e) == Some(W::p()))
        .collect();
    let first = dom.serialize_element(paras[0]);
    assert!(
        first.contains("Table Widths") && first.contains("<w:ins"),
        "inserted next content must lead: {first}"
    );
    // find first delText-bearing para index
    let first_del = paras.iter().position(|&p| {
        let x = dom.serialize_element(p);
        x.contains("delText") || (x.contains("<w:del") && x.contains("Support"))
    });
    let last_ins = paras.iter().rposition(|&p| {
        let x = dom.serialize_element(p);
        x.contains("<w:ins") && !x.contains("delText")
    });
    if let (Some(fd), Some(li)) = (first_del, last_ins) {
        assert!(
            li < fd,
            "all pure-ins must precede pure-del cluster (last_ins={li} first_del={fd})"
        );
    }
}
