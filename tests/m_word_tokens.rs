// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The words Word Compare works in: a run of letters, digits and apostrophes
//! is one word (`R1C1`, `abc123`, `Q3FY26`, `Ä1`, `don't`); every other
//! punctuation mark or sign stands alone (`_`, `-`, `$`, `§`, `—`, `“`), `.`
//! and `,` between digits too (`1.5`, `1,000`); a fullwidth character is a
//! word of its own (`Ab１`).
//!
//! Word 16 probes, 2026-10-01 (tests/fixtures/word_probes/tokens): each pair
//! changes one token per sentence or table cell; `*_word_redline.docx` is
//! Word's own Compare of the pair. The redline split `R1C1 → 1` into
//! `[-R1C-]1`, where Word deletes `R1C1` and inserts `1`.

use jubarte::changes::list_changes;
use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::comparer::atoms::ComparisonUnit;
use jubarte::comparer::units::get_comparison_unit_list;
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const PROBES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/word_probes/tokens"
);

/// The text changes of a redline, in document order: `insertion "1"`.
fn text_changes(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .expect("list changes")
        .into_iter()
        .filter(|c| c.target == "text")
        .map(|c| format!("{:?} {:?}", c.kind, c.text))
        .collect()
}

fn probe(name: &str) {
    let read =
        |suffix: &str| std::fs::read(format!("{PROBES}/{name}_{suffix}.docx")).expect("probe");
    let ours = compare_documents(&read("a"), &read("b"), "Comparison").expect("compare");
    assert_eq!(
        text_changes(&ours),
        text_changes(&read("word_redline")),
        "{name}"
    );
}

#[test]
fn letters_and_digits_change_whole_in_sentences() {
    probe("prose");
}

#[test]
fn letters_and_digits_change_whole_in_table_cells() {
    probe("cell");
}

#[test]
fn underscore_hyphen_period_and_fullwidth_split_words() {
    probe("separators");
}

#[test]
fn ascii_punctuation_splits_words_but_the_apostrophe() {
    probe("punctuation");
}

#[test]
fn unicode_punctuation_and_signs_split_words() {
    probe("unicode_punctuation");
}

/// The words of a one-paragraph body, spaces and the paragraph mark left out.
fn words(text: &str, settings: &WmlComparerSettings) -> Vec<String> {
    let mut dom = Dom::new();
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body></w:document>",
        W::URI
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let atoms = create_comparison_unit_atom_list(&mut dom, body, settings);
    let units = get_comparison_unit_list(&dom, &atoms, settings);
    let ComparisonUnit::Group(paragraph) = &units[0] else {
        panic!("a paragraph group");
    };
    paragraph
        .contents
        .iter()
        .filter_map(|unit| match unit {
            ComparisonUnit::Word(word) => Some(
                word.contents
                    .iter()
                    .map(|atom| dom.value(atom.content_element))
                    .collect::<String>(),
            ),
            ComparisonUnit::Group(_) => None,
        })
        .filter(|word| !word.trim().is_empty())
        .collect()
}

#[test]
fn word_mode_words_follow_word() {
    let word = WmlComparerSettings::default();
    assert!(word.merge_replaced_paragraphs, "the default is Word mode");
    for (text, expected) in [
        ("R1C1", vec!["R1C1"]),
        ("Q3FY26", vec!["Q3FY26"]),
        ("Ä1", vec!["Ä1"]),
        ("file_137.docx", vec!["file", "_", "137", ".", "docx"]),
        ("1.5", vec!["1", ".", "5"]),
        ("Left-aligned", vec!["Left", "-", "aligned"]),
        ("2a-3b", vec!["2a", "-", "3b"]),
        ("3.2(a)", vec!["3", ".", "2", "(", "a", ")"]),
        ("2026-10-01", vec!["2026", "-", "10", "-", "01"]),
        ("Ab１", vec!["Ab", "１"]),
        ("1,000", vec!["1", ",", "000"]),
        ("don't", vec!["don't"]),
        ("it’s", vec!["it’s"]),
        ("US$5", vec!["US", "$", "5"]),
        ("a§b", vec!["a", "§", "b"]),
        ("a—b", vec!["a", "—", "b"]),
        ("a‘b", vec!["a", "‘", "b"]),
        ("नमस्ते", vec!["नमस्ते"]),
    ] {
        assert_eq!(words(text, &word), expected, "{text}");
    }
}

#[test]
fn powertools_mode_keeps_its_words() {
    // The Open-Xml-PowerTools preset tokenizes as PowerTools does.
    let faithful = WmlComparerSettings::powertools_faithful();
    assert_eq!(words("1.5", &faithful), vec!["1.5"]);
    assert_eq!(words("snake_a", &faithful), vec!["snake_a"]);
    assert_eq!(
        words("Left-aligned", &faithful),
        vec!["Left", "-", "aligned"]
    );
}
