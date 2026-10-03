// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word marks a changed paragraph word by word, or replaces it whole, by one
//! rule (Word 16, 1178 single-paragraph probes, 2026-10-03): the kept span
//! — the characters of the words its alignment keeps with the blanks inside
//! a kept run and on either side of it — over the characters of the longer
//! side reaches 0.15 or the paragraph is replaced. The rule is constant
//! from 6 to 4800 words and for lone words and runs of 2 to 16, and Word
//! applies it again to each window between the anchors it keeps, so a
//! rewritten stretch with a stray shared word or two is replaced as one
//! block inside a word-level paragraph.
//!
//! The engine used to void each common run shorter than 2 % of the window,
//! which replaced every paragraph whose kept runs are short (Word keeps
//! them from 12 % of the characters) and kept a long run in a paragraph
//! Word replaces.

mod common;

use common::docx::{docx, para};
use jubarte::changes::list_changes;
use jubarte::document_comparer::compare_documents;

/// A six-letter word unique to `n` within a vocabulary `v` (0 or 1).
fn word(v: u32, n: usize) -> String {
    let mut x = n as u32;
    let mut s = String::new();
    for _ in 0..5 {
        s.push((b'a' + (x % 26) as u8) as char);
        x /= 26;
    }
    s.push(if v == 0 { 'q' } else { 'z' });
    s
}

/// `n` words; the revision keeps `runs` runs of `run` words, evenly spaced,
/// and rewrites the rest from a disjoint vocabulary. Returns the two texts
/// and the kept words.
fn pair(n: usize, runs: usize, run: usize) -> (String, String, Vec<String>) {
    let a: Vec<String> = (0..n).map(|i| word(0, i)).collect();
    let mut kept = Vec::new();
    let mut b = Vec::with_capacity(n);
    let gap = n / runs.max(1);
    for (i, w) in a.iter().enumerate() {
        if runs > 0 && i % gap < run && i / gap < runs {
            kept.push(w.clone());
            b.push(w.clone());
        } else {
            b.push(word(1, i));
        }
    }
    (a.join(" "), b.join(" "), kept)
}

/// `"{kind} {target} {text}"` per change, in document order.
fn changes(a: &str, b: &str) -> Vec<String> {
    let side = |body: &str| docx(&format!("{}{}{}", para("Clause"), para(body), para("End")));
    let ours = compare_documents(&side(a), &side(b), "Comparison").expect("compare");
    list_changes(&ours)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {} {}", c.kind, c.target, c.text))
        .collect()
}

fn mentions(changes: &[String], word: &str) -> usize {
    changes.iter().filter(|c| c.contains(word)).count()
}

#[test]
fn a_paragraph_keeping_a_sixth_of_its_words_in_short_runs_is_marked_word_by_word() {
    // 16 runs of 4 among 400 words: 16 % of the words, 13.7 % of the
    // characters. Word keeps every run (wave7 `len400_k16_r04`); the 2 %
    // run gate voided each 4-word run (4/801 of the window).
    let (a, b, kept) = pair(400, 16, 4);
    let changes = changes(&a, &b);
    assert!(
        changes.iter().all(|c| c.contains(" text ")),
        "only text changes, no paragraph marks: {changes:#?}"
    );
    for w in &kept {
        assert_eq!(
            mentions(&changes, w),
            0,
            "kept word {w} must stay plain: {changes:#?}"
        );
    }
    assert!(
        changes.len() >= 2 * 16,
        "a deletion and an insertion per gap: {changes:#?}"
    );
}

#[test]
fn a_paragraph_keeping_a_tenth_of_its_words_is_replaced_whole() {
    // 10 runs of 4 among 400 words: 10 % of the words, 8.6 % of the
    // characters. Word replaces the paragraph (wave7 `len400_k10_r04`).
    let (a, b, kept) = pair(400, 10, 4);
    let changes = changes(&a, &b);
    for w in &kept {
        assert!(
            changes
                .iter()
                .any(|c| c.starts_with("Deletion") && c.contains(w)),
            "kept word {w} must be deleted with the paragraph: {changes:#?}"
        );
        assert!(
            changes
                .iter()
                .any(|c| c.starts_with("Insertion") && c.contains(w)),
            "kept word {w} must be inserted with the paragraph: {changes:#?}"
        );
    }
    let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
    assert_eq!(
        text.len(),
        2,
        "one deletion and one insertion: {changes:#?}"
    );
}

#[test]
fn a_long_run_in_a_paragraph_keeping_a_tenth_does_not_save_it() {
    // One run of 40 among 400 words passes the old 2 % run gate (40/801)
    // but keeps 8.6 % of the characters: replaced (wave1 `w400_k10`).
    let (a, b, kept) = pair(400, 1, 40);
    let changes = changes(&a, &b);
    let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
    assert_eq!(
        text.len(),
        2,
        "one deletion and one insertion: {changes:#?}"
    );
    assert!(
        text.iter()
            .all(|c| c.contains(&kept[0]) && c.contains(&kept[39])),
        "the run goes with the paragraph: {changes:#?}"
    );
}

#[test]
fn a_rewritten_stretch_after_a_kept_opening_is_replaced_as_one_block() {
    // 100 kept words, then 300 rewritten ones that share two stray words
    // with the original, both words that also occur in the opening. The
    // paragraph keeps a quarter of its characters, so it is word-level;
    // the strays are not unique to the paragraph, so they are no anchors
    // of it, and the stretch after the opening is a gap judged on its own:
    // it keeps under 1 %, so it is replaced as one block, strays included
    // (`m47_stopword_lone_anchor`: a lone shared "and" or "text" does not
    // shred the sentence around it). Inside the block the insertion
    // precedes the deletion, as Word writes them.
    let mut a: Vec<String> = (0..400).map(|i| word(0, i)).collect();
    a[180] = a[20].clone();
    a[260] = a[40].clone();
    let mut b = a.clone();
    for (i, w) in b.iter_mut().enumerate().skip(100) {
        if i != 180 && i != 260 {
            *w = word(1, i);
        }
    }
    let changes = changes(&a.join(" "), &b.join(" "));
    let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
    assert_eq!(
        text.len(),
        2,
        "one deletion and one insertion: {changes:#?}"
    );
    for stray in [&a[180], &a[260]] {
        assert!(
            text.iter().all(|c| c.contains(stray)),
            "stray word {stray} goes with the block: {changes:#?}"
        );
    }
    assert!(text[0].starts_with("Insertion"), "{changes:#?}");
    assert!(text[1].starts_with("Deletion"), "{changes:#?}");
    for w in a[..100].iter().filter(|w| *w != &a[20] && *w != &a[40]) {
        assert_eq!(
            mentions(&changes, w),
            0,
            "opening word {w} stays plain: {changes:#?}"
        );
    }
}

#[test]
fn a_word_unique_to_both_sides_anchors_inside_a_rewritten_stretch() {
    // The same stretch, but the two strays occur nowhere else: Word's
    // first pass links every word unique to both sides (its alignment
    // holds every such link on the 2026-10-03 probes), so they stay plain
    // and the stretch splits around them.
    let a: Vec<String> = (0..400).map(|i| word(0, i)).collect();
    let mut b = a.clone();
    for (i, w) in b.iter_mut().enumerate().skip(100) {
        if i != 180 && i != 260 {
            *w = word(1, i);
        }
    }
    let changes = changes(&a.join(" "), &b.join(" "));
    let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
    assert_eq!(
        text.len(),
        6,
        "three gaps, a deletion and an insertion each: {changes:#?}"
    );
    for stray in [&a[180], &a[260]] {
        assert_eq!(
            mentions(&changes, stray),
            0,
            "stray word {stray} stays plain: {changes:#?}"
        );
    }
}

#[test]
fn an_insertion_precedes_the_deletion_it_replaces() {
    // Word writes the inserted text before the deleted text it replaces,
    // whole paragraphs (wave7 `len400_k10_r04`: the inserted paragraph,
    // then the deleted one) and blocks inside one alike.
    let (a, b, _) = pair(400, 10, 4);
    let changes = changes(&a, &b);
    let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
    assert!(text[0].starts_with("Insertion"), "{changes:#?}");
    assert!(text[1].starts_with("Deletion"), "{changes:#?}");
}

#[test]
fn a_unit_without_text_changed_beside_kept_words_is_marked() {
    // A tab that becomes a break has no text on either side; the anchor
    // extension must not pair the two by their emptiness (the units hash
    // by element, so the comparer deletes the tab and inserts the break).
    let (a, b, _) = pair(240, 16, 4);
    let body = |text: &str, mid: &str| {
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r><w:r>{mid}</w:r><w:r><w:t xml:space="preserve"> {text}</w:t></w:r></w:p>"#
        )
    };
    let side = |text: &str, mid: &str| {
        docx(&format!(
            "{}{}{}",
            para("Clause"),
            body(text, mid),
            para("End")
        ))
    };
    let ours = compare_documents(&side(&a, "<w:tab/>"), &side(&b, "<w:br/>"), "Comparison")
        .expect("compare");
    let xml = common::docx::part_string(&ours, "word/document.xml").expect("document.xml");
    // The revision element enclosing `needle`: the last `w:ins`/`w:del`
    // opened and not yet closed before it.
    let enclosing = |needle: &str| -> &'static str {
        let at = xml
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} missing: {xml}"));
        let before = &xml[..at];
        let open = ["<w:ins ", "<w:del "]
            .iter()
            .filter_map(|tag| before.rfind(tag).map(|i| (i, *tag)))
            .max_by_key(|(i, _)| *i);
        let close = ["</w:ins>", "</w:del>"]
            .iter()
            .filter_map(|tag| before.rfind(tag))
            .max();
        match open {
            Some((i, tag)) if close.is_none_or(|c| c < i) => {
                if tag == "<w:ins " {
                    "ins"
                } else {
                    "del"
                }
            }
            _ => "plain",
        }
    };
    assert_eq!(enclosing("<w:tab"), "del", "{xml}");
    assert_eq!(enclosing("<w:br"), "ins", "{xml}");
}

#[test]
fn a_short_paragraph_keeping_a_lone_word_is_marked_word_by_word() {
    // The bench's redline of document_100_ultimate_demo × double_spacing_bold_demo
    // keeps " document " and "." of this 12-word paragraph: with the
    // paragraph mark, 12 of the longer side's 80 characters, 0.15 exactly
    // (short1 and edge1 put the band at 0.149–0.151 for 12-word
    // paragraphs), and Word marks it word by word.
    let changes = changes(
        "This final document showcases the complete range of styling options available.",
        "Bold double-spaced text for easy document editing and review.",
    );
    assert_eq!(
        mentions(&changes, "document"),
        0,
        "the kept word stays plain: {changes:#?}"
    );
    assert!(
        changes
            .iter()
            .any(|c| c.starts_with("Deletion") && c.contains("showcases")),
        "{changes:#?}"
    );
    assert!(
        changes
            .iter()
            .any(|c| c.starts_with("Insertion") && c.contains("editing")),
        "{changes:#?}"
    );
}

/// `n` words, the revision keeping the first and the last when asked and
/// rewriting the rest. Returns the two texts.
fn ends(n: usize, first: bool, last: bool) -> (String, String) {
    let a: Vec<String> = (0..n).map(|i| word(0, i)).collect();
    let b: Vec<String> = a
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if (first && i == 0) || (last && i + 1 == n) {
                w.clone()
            } else {
                word(1, i)
            }
        })
        .collect();
    (a.join(" "), b.join(" "))
}

#[test]
fn a_lone_first_word_does_not_anchor_a_rewritten_paragraph() {
    // Word replaces a paragraph that keeps only its first word, at 12, 20,
    // 40, 80, 160 and 300 words alike (edge1 `first-both`, 12 of 12
    // replaced): the opening word is no anchor, the ratio rules.
    for n in [12, 40, 300] {
        let (a, b) = ends(n, true, false);
        let changes = changes(&a, &b);
        let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
        assert_eq!(
            text.len(),
            2,
            "{n} words: one deletion and one insertion: {changes:#?}"
        );
        assert!(
            text.iter().all(|c| c.contains(&word(0, 0))),
            "{n} words: the first word goes with the paragraph: {changes:#?}"
        );
    }
}

#[test]
fn a_lone_last_word_does_not_anchor_a_rewritten_paragraph() {
    // The same for the last word (edge1 `last-both`, 12 of 12 replaced).
    for n in [12, 40, 300] {
        let (a, b) = ends(n, false, true);
        let changes = changes(&a, &b);
        let text: Vec<&String> = changes.iter().filter(|c| c.contains(" text ")).collect();
        assert_eq!(
            text.len(),
            2,
            "{n} words: one deletion and one insertion: {changes:#?}"
        );
        assert!(
            text.iter().all(|c| c.contains(&word(0, n - 1))),
            "{n} words: the last word goes with the paragraph: {changes:#?}"
        );
    }
}

#[test]
fn both_end_words_kept_tip_a_short_paragraph_to_word_level() {
    // Two six-letter words and their blanks are 14 of a 12-word
    // paragraph's 83 characters — 15 of 84 with the mark, 0.179 — and Word
    // keeps them (edge1 `ends-both` at 12 words, both seeds); at 20 words
    // the same two words are 15 of 140, 0.107, and Word replaces the
    // paragraph (both seeds).
    let (a, b) = ends(12, true, true);
    let short = changes(&a, &b);
    for w in [word(0, 0), word(0, 11)] {
        assert_eq!(
            mentions(&short, &w),
            0,
            "kept word {w} stays plain: {short:#?}"
        );
    }
    let (a, b) = ends(20, true, true);
    let longer = changes(&a, &b);
    let text: Vec<&String> = longer.iter().filter(|c| c.contains(" text ")).collect();
    assert_eq!(text.len(), 2, "replaced whole: {longer:#?}");
    assert!(
        text.iter()
            .all(|c| c.contains(&word(0, 0)) && c.contains(&word(0, 19))),
        "{longer:#?}"
    );
}

#[test]
fn a_replaced_hyperlink_paragraph_inserts_before_it_deletes() {
    // Word's redline of super_editor hyperlink_multiple_runs × diff_before10
    // (corpus a098bd5ad9): the hyperlink paragraph "Click here now" is
    // replaced by "One two", and Word writes the insertion first, the
    // deleted field after it. The verdict branch emitted the deleted
    // hyperlink first; the writer's insertion-first reorder only saw bare
    // runs (2000-pair control, 2026-10-03: −15.3 on the PDF score).
    let link = "<w:p><w:hyperlink w:anchor=\"top\"><w:r><w:t xml:space=\"preserve\">Click </w:t></w:r>\
                <w:r><w:t>here</w:t></w:r><w:r><w:t xml:space=\"preserve\"> now</w:t></w:r></w:hyperlink></w:p>";
    let side = |body: &str| docx(&format!("{}{}{}", para("Clause"), body, para("End")));
    let ours =
        compare_documents(&side(link), &side(&para("One two")), "Comparison").expect("compare");
    let changes: Vec<String> = list_changes(&ours)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {}", c.kind, c.text))
        .collect();
    let ins = changes
        .iter()
        .position(|c| c.starts_with("Insertion") && c.contains("One two"));
    let del = changes
        .iter()
        .position(|c| c.starts_with("Deletion") && c.contains("Click"));
    assert!(
        matches!((ins, del), (Some(i), Some(d)) if i < d),
        "the insertion precedes the deleted hyperlink; changes={changes:?}"
    );
}

#[test]
fn a_replaced_external_hyperlink_paragraph_inserts_before_it_deletes() {
    // The corpus pair itself (a098bd5ad9) links by r:id: the writer keeps
    // the `w:hyperlink` wrapper around the deleted runs, and the swap must
    // read that wrapper as the deletion it holds.
    let link = "<w:p><w:hyperlink r:id=\"rIdH\"><w:r><w:t xml:space=\"preserve\">Click </w:t></w:r>\
                <w:r><w:t>here</w:t></w:r><w:r><w:t xml:space=\"preserve\"> now</w:t></w:r></w:hyperlink></w:p>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdH\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" \
          Target=\"https://example.com/\" TargetMode=\"External\"/></Relationships>";
    let side = |body: &str| docx(&format!("{}{}{}", para("Clause"), body, para("End")));
    let base =
        common::docx::replace_entry(&side(link), "word/_rels/document.xml.rels", rels.as_bytes());
    let ours = compare_documents(&base, &side(&para("One two")), "Comparison").expect("compare");
    let changes: Vec<String> = list_changes(&ours)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {}", c.kind, c.text))
        .collect();
    let ins = changes
        .iter()
        .position(|c| c.starts_with("Insertion") && c.contains("One two"));
    let del = changes
        .iter()
        .position(|c| c.starts_with("Deletion") && c.contains("Click"));
    assert!(
        matches!((ins, del), (Some(i), Some(d)) if i < d),
        "the insertion precedes the deleted hyperlink; changes={changes:?}"
    );
    let xml = common::docx::part_string(&ours, "word/document.xml").expect("document");
    assert!(
        xml.contains("<w:hyperlink")
            && xml.find("<w:ins").unwrap() < xml.find("<w:hyperlink").unwrap(),
        "the hyperlink wrapper survives and follows the insertion"
    );
}
