// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every pair of small table cells, compared in both modes: the redline is
//! Word-valid (no revision on a cell's final mark, #1669; no cell ending in
//! a table), accepts to the revised document and rejects to the original,
//! block for block (#1684). A cell is one or two blocks from paragraphs
//! `One.` and `Two.`, an empty paragraph, a nested table holding `Inner.` or
//! `Other.`, and a content control around `Sdt.`; it ends in a paragraph or
//! the content control.
//!
//! Every redline must be Word-valid. The pairs in [`KNOWN`] still miss B
//! or A in Word mode, as their issues say; one that starts to pass must
//! leave the list. Where only a content control differs, the outcome is
//! listed in `tests/data/cell_final_marks_pairs_controls.txt` (its header
//! says why): any outcome that appears or goes must change that list.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::check_word_valid_package;
use jubarte::comparer::{CompareMode, WmlComparerSettings};
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};

/// The block a cell code's character stands for.
fn block(code: char) -> String {
    let nested = |text: &str| {
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
            para(text)
        )
    };
    match code {
        '1' => para("One."),
        '2' => para("Two."),
        'e' => "<w:p/>".to_string(),
        'n' => nested("Inner."),
        'm' => nested("Other."),
        's' => format!(
            "<w:sdt><w:sdtPr/><w:sdtContent>{}</w:sdtContent></w:sdt>",
            para("Sdt.")
        ),
        _ => unreachable!("no block {code}"),
    }
}

/// Every cell code: one block, or two, ending in a paragraph or the
/// content control.
fn cells() -> Vec<String> {
    let ends = ['1', '2', 'e', 's'];
    let all = ['1', '2', 'e', 'n', 'm', 's'];
    let mut out: Vec<String> = ends.iter().map(|c| c.to_string()).collect();
    for first in all {
        for last in ends {
            out.push(format!("{first}{last}"));
        }
    }
    out
}

/// A table between two paragraphs: its first cell holds "Full.", its
/// second the blocks of `cell`.
fn document(cell: &str) -> Vec<u8> {
    let tc = |inner: &str| {
        format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
    };
    let blocks: String = cell.chars().map(block).collect();
    docx(&format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        tc(&para("Full.")),
        tc(&blocks),
        para("After.")
    ))
}

/// The body's blocks as `[tbl [tr [tc [p text]]]]`: tables, rows, cells,
/// paragraphs with their visible text, and content controls (`[sdt …]`),
/// so a block control that ends up inside a paragraph shows (crush on
/// #1680, round 10).
fn shape(bytes: &[u8]) -> String {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let token = regex::Regex::new(
        r"<w:(tbl|tr|tc|p|sdt)\b[^>]*?(/?)>|</w:(tbl|tr|tc|p|sdt)>|<w:t(?: [^>]*)?>([^<]*)</w:t>",
    )
    .unwrap();
    let mut out = String::new();
    for c in token.captures_iter(&xml) {
        if let Some(open) = c.get(1) {
            out.push('[');
            out.push_str(open.as_str());
            if !c[2].is_empty() {
                out.push(']');
            }
        } else if c.get(3).is_some() {
            out.push(']');
        } else {
            out.push_str(&c[4]);
        }
    }
    out
}

/// `shape` without its content controls.
fn without_controls(shape: &str) -> String {
    let mut out = String::new();
    let mut open = Vec::new();
    let mut rest = shape;
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("[sdt") {
            open.push(true);
            rest = after;
            continue;
        }
        match c {
            '[' => {
                open.push(false);
                out.push(c);
            }
            ']' => {
                if open.pop() == Some(false) {
                    out.push(c);
                }
            }
            _ => out.push(c),
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// How a result whose blocks match differs from the wanted one in its
/// content controls: one inside a paragraph (`inline-husk`), fewer
/// (`control-dropped`), or more (`control-kept`).
fn control_outcome(got: &str, wanted: &str) -> &'static str {
    let inline = regex::Regex::new(r"\[p[^\[\]]*\[sdt").unwrap();
    if inline.is_match(got) && !inline.is_match(wanted) {
        "inline-husk"
    } else if got.matches("[sdt").count() < wanted.matches("[sdt").count() {
        "control-dropped"
    } else {
        "control-kept"
    }
}

/// The content-control outcomes, `from to mode which outcome` a line.
const CONTROLS: &str = include_str!("data/cell_final_marks_pairs_controls.txt");

/// The Word-mode pairs whose Accept All (or Reject All) still misses B (or
/// A), each with its issue: `(from, to, which)`.
const KNOWN: &[(&str, &str, &str)] = &[
    // #1686: the comparer drops an empty paragraph the cell gains or loses.
    ("1", "ee", "Accept"),
    ("11", "2e", "Accept"),
    ("2", "ee", "Accept"),
    ("22", "1e", "Accept"),
    ("ee", "1", "Reject"),
    ("ee", "2", "Reject"),
    // #1687: one side's cell ends inside a content control, and the final
    // marks do not pair across it (Word-valid since #1684).
    ("11", "2s", "Accept"),
    ("11", "es", "Accept"),
    ("11", "ms", "Accept"),
    ("11", "ns", "Accept"),
    ("11", "s", "Accept"),
    ("11", "ss", "Accept"),
    ("22", "1s", "Accept"),
    ("22", "es", "Accept"),
    ("22", "ms", "Accept"),
    ("22", "ns", "Accept"),
    ("22", "s", "Accept"),
    ("22", "ss", "Accept"),
    ("e", "1s", "Accept"),
    ("e", "2s", "Accept"),
    ("e", "ms", "Accept"),
    ("e", "ns", "Accept"),
    ("e", "ss", "Accept"),
    ("ee", "1s", "Accept"),
    ("ee", "2s", "Accept"),
    ("ee", "ms", "Accept"),
    ("ee", "ns", "Accept"),
    ("ee", "s", "Accept"),
    ("ee", "ss", "Accept"),
    ("m1", "ns", "Accept"),
    ("m2", "ns", "Accept"),
    ("me", "1s", "Accept"),
    ("me", "2s", "Accept"),
    ("me", "es", "Accept"),
    ("me", "s", "Accept"),
    ("me", "ss", "Accept"),
    ("ms", "n1", "Accept"),
    ("ms", "n2", "Accept"),
    ("n1", "ms", "Accept"),
    ("n2", "ms", "Accept"),
    ("ne", "1s", "Accept"),
    ("ne", "2s", "Accept"),
    ("ne", "es", "Accept"),
    ("ne", "s", "Accept"),
    ("ne", "ss", "Accept"),
    ("ns", "m1", "Accept"),
    ("ns", "m2", "Accept"),
    ("ss", "1", "Accept"),
    ("ss", "11", "Accept"),
    ("ss", "12", "Accept"),
    ("ss", "2", "Accept"),
    ("ss", "21", "Accept"),
    ("ss", "22", "Accept"),
    ("ss", "e1", "Accept"),
    ("ss", "e2", "Accept"),
    ("ss", "m1", "Accept"),
    ("ss", "m2", "Accept"),
    ("ss", "n1", "Accept"),
    ("ss", "n2", "Accept"),
];

/// Compare `from` with every other cell in both modes; every failure, at
/// once.
fn check_from(from: &str) {
    let a = document(from);
    let mut failures = Vec::new();
    let mut controls = Vec::new();
    for to in cells().iter().filter(|to| to.as_str() != from) {
        let b = document(to);
        for mode in [CompareMode::Word, CompareMode::PowerTools] {
            let settings = WmlComparerSettings::new(mode);
            let what = format!("{from} -> {to} ({mode:?})");
            let out = match compare_documents_with_settings(&a, &b, &settings) {
                Ok(out) => out,
                Err(e) => {
                    failures.push(format!("{what}: compare failed: {e}"));
                    continue;
                }
            };
            let report = check_word_valid_package(&out);
            if !report.ok() {
                failures.push(format!("{what}: {:?}", report.errors));
            }
            let known = |which: &str| {
                mode == CompareMode::Word && KNOWN.contains(&(from, to.as_str(), which))
            };
            for (which, got, wanted) in [
                ("Accept", shape(&accept_revisions(&out).unwrap()), shape(&b)),
                ("Reject", shape(&reject_revisions(&out).unwrap()), shape(&a)),
            ] {
                let blocks = without_controls(&got) == without_controls(&wanted);
                match (blocks, known(which)) {
                    (false, false) => {
                        failures.push(format!("{what}: {which} All gives {got}"));
                    }
                    (true, true) => {
                        failures.push(format!(
                            "{what}: {which} All now passes; drop it from KNOWN"
                        ));
                    }
                    _ => {}
                }
                if blocks && got != wanted {
                    controls.push(format!(
                        "{from} {to} {mode:?} {which} {}",
                        control_outcome(&got, &wanted)
                    ));
                }
            }
        }
    }
    let listed: Vec<&str> = CONTROLS
        .lines()
        .filter(|line| line.split(' ').next() == Some(from))
        .collect();
    for line in &controls {
        if !listed.contains(&line.as_str()) {
            failures.push(format!("a content-control outcome to list: {line}"));
        }
    }
    for line in listed {
        if !controls.iter().any(|c| c == line) {
            failures.push(format!("a content-control outcome gone; drop it: {line}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_cell_codes_cover_every_shape_once() {
    let cells = cells();
    assert_eq!(cells.len(), 28);
    let mut unique = cells.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), cells.len());
    assert_eq!(
        shape(&document("n1")),
        "[pBefore.][tbl[tr[tc[pFull.]][tc[tbl[tr[tc[pInner.]]]][pOne.]]]][pAfter.]"
    );
    assert_eq!(
        shape(&document("es")),
        "[pBefore.][tbl[tr[tc[pFull.]][tc[p][sdt[pSdt.]]]]][pAfter.]"
    );
    assert_eq!(
        without_controls(&shape(&document("es"))),
        "[pBefore.][tbl[tr[tc[pFull.]][tc[p][pSdt.]]]][pAfter.]"
    );
    let wanted = "[tc[p][sdt[pSdt.]]]";
    assert_eq!(control_outcome("[tc[p[sdt]]]", wanted), "inline-husk");
    assert_eq!(control_outcome("[tc[p][pSdt.]]", wanted), "control-dropped");
    assert_eq!(
        control_outcome("[tc[sdt[p]][sdt[pSdt.]]]", wanted),
        "control-kept"
    );
}

macro_rules! from {
    ($($name:ident: $cell:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_from($cell);
            }
        )*
    };
}

from! {
    from_1: "1", from_2: "2", from_e: "e", from_s: "s",
    from_11: "11", from_12: "12", from_1e: "1e", from_1s: "1s",
    from_21: "21", from_22: "22", from_2e: "2e", from_2s: "2s",
    from_e1: "e1", from_e2: "e2", from_ee: "ee", from_es: "es",
    from_n1: "n1", from_n2: "n2", from_ne: "ne", from_ns: "ns",
    from_m1: "m1", from_m2: "m2", from_me: "me", from_ms: "ms",
    from_s1: "s1", from_s2: "s2", from_se: "se", from_ss: "ss",
}
