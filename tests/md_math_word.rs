// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Office Math round trip against Word's own files (`tests/fixtures/math`,
//! `SOURCES.md` names each). Each fixture is read to LaTeX, written back to
//! a docx, and every equation's shape (its elements and text, without
//! properties, fonts or how Word split its runs) must match the equation
//! Word wrote. A fixture whose shape we cannot keep is listed with why, and
//! the list stays honest: a listed fixture that comes back right fails.

mod common;

use std::fmt::Write as _;
use std::path::Path;

use jubarte::markdown::{
    DocxOptions, MarkdownOptions, ReadOptions, docx_to_markdown, markdown_to_docx, read,
};

/// Fixtures whose equations do not come back, and why.
const KNOWN: &[(&str, &str)] = &[];

/// Each equation's shape: element names with their children in brackets,
/// text merged across runs (`+…+` inserted, `-…-` deleted), properties
/// and other `w:` elements left out. Whitespace is the reader's: runs of it
/// collapse to one space, and none is kept at an argument's edges, which
/// LaTeX ignores.
fn shapes(document: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut shape = String::new();
    let mut depth = 0usize;
    let mut hidden = 0usize;
    let mut text_open = false;
    let mut change: Option<char> = None;
    let mut rest = document;
    while let Some(open) = rest.find('<') {
        let text = &rest[..open];
        if depth > 0 && hidden == 0 && text_open {
            match change {
                Some(mark) => {
                    let _ = write!(shape, "{mark}{}{mark}", unescape(text));
                }
                None => shape.push_str(&unescape(text)),
            }
        }
        let Some(close) = rest[open..].find('>').map(|i| open + i) else {
            break;
        };
        let tag = &rest[open + 1..close];
        rest = &rest[close + 1..];
        let closing = tag.starts_with('/');
        let empty = tag.ends_with('/');
        let name = tag
            .trim_start_matches('/')
            .split([' ', '/', '>'])
            .next()
            .unwrap_or("");
        if name == "m:oMath" {
            if closing {
                depth = 0;
                out.push(std::mem::take(&mut shape));
            } else if empty {
                out.push(String::new());
            } else {
                depth = 1;
            }
            continue;
        }
        if depth == 0 {
            continue;
        }
        if name.ends_with("Pr") || name.starts_with("w:") && name != "w:ins" && name != "w:del" {
            if !empty {
                if closing {
                    hidden = hidden.saturating_sub(1);
                } else {
                    hidden += 1;
                }
            }
            continue;
        }
        if name == "w:ins" || name == "w:del" {
            if hidden == 0 && !empty {
                change = (!closing).then_some(if name == "w:ins" { '+' } else { '-' });
            }
            continue;
        }
        if hidden > 0 {
            continue;
        }
        if name == "m:t" {
            text_open = !closing && !empty;
            continue;
        }
        // Runs merge: Word splits them where it likes.
        if name == "m:r" {
            continue;
        }
        let short = name.trim_start_matches("m:");
        if closing {
            shape.push(')');
        } else if empty {
            let _ = write!(shape, "{short}()");
        } else {
            let _ = write!(shape, "{short}(");
        }
    }
    out.iter()
        .map(|s| {
            s.replace('\u{2061}', "")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .replace("( ", "(")
                .replace(" )", ")")
        })
        .collect()
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("read")
    .markdown
}

/// The conversion view (`jubarte convert X.docx -o X.md`): no front
/// matter or id lines, and its table cells escape their own backslashes.
fn conversion(docx: &[u8]) -> String {
    docx_to_markdown(docx, &MarkdownOptions::default())
        .expect("convert")
        .markdown
}

#[test]
fn word_equations_round_trip_through_latex() {
    round_trip_fixtures(view);
}

#[test]
fn word_equations_round_trip_through_the_conversion_view() {
    round_trip_fixtures(conversion);
}

/// Each fixture read with `read_view`, written back, and its equations'
/// shapes compared with Word's.
fn round_trip_fixtures(read_view: fn(&[u8]) -> String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/math");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .expect("fixtures")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "docx"))
        .collect();
    names.sort();
    assert_eq!(names.len(), 21, "{names:?}");
    let mut failures = Vec::new();
    for path in names {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read(&path).expect("fixture");
        let word = shapes(&common::docx::part_string(&source, "word/document.xml").unwrap());
        let markdown = read_view(&source);
        let written = markdown_to_docx(&markdown, &DocxOptions::default())
            .expect("write")
            .docx;
        common::validity::assert_word_valid_package(&written);
        let ours = shapes(&common::docx::part_string(&written, "word/document.xml").unwrap());
        let same = word == ours;
        let known = KNOWN.iter().any(|(n, _)| *n == name);
        if same == known {
            let mut report = format!(
                "{name}: {}\n",
                if same {
                    "now round-trips: unlist it"
                } else {
                    "differs"
                }
            );
            for at in 0..word.len().max(ours.len()) {
                let (w, o) = (word.get(at), ours.get(at));
                if w != o {
                    let _ = writeln!(report, "  #{at} word {w:?}\n  #{at} ours {o:?}");
                }
            }
            failures.push(report);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The view's math, through a docx and back: each line that holds `$`.
fn round_trip(markdown: &str) -> (String, String) {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("write")
        .docx;
    common::validity::assert_word_valid_package(&docx);
    let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
    let lines = view(&docx)
        .lines()
        .filter(|line| line.contains('$'))
        .collect::<Vec<_>>()
        .join("\n");
    (lines, xml)
}

#[test]
fn an_equation_inserted_whole_is_one_change_around_it() {
    let (view, xml) = round_trip("Added {++$E=mc^2$++} here.\n");
    assert!(view.contains("{++$E=mc^2$++}{>>r"), "{view}");
    // Word marks inside the math, never around it.
    assert!(xml.contains("<m:r><w:ins "), "{xml}");
    assert!(!xml.contains("\"><m:oMath"), "{xml}");
}

#[test]
fn a_change_inside_an_equation_comes_back() {
    let (view, xml) = round_trip("$A={--π--}{++2π++}r^2$\n");
    assert!(view.contains("$A={--π--}{++2π++}r^2$"), "{view}");
    assert!(xml.contains("<m:r><w:del "), "{xml}");
    assert!(xml.contains("<m:r><w:ins "), "{xml}");
}

#[test]
fn display_lines_become_an_equation_array() {
    let (view, xml) = round_trip("$$a=1 \\\\ b=2$$\n");
    assert!(xml.contains("<m:eqArr>"), "{xml}");
    assert_eq!(xml.matches("<m:oMath>").count(), 1, "{xml}");
    assert!(
        view.contains(r"$$\begin{aligned}a=1 \\ b=2\end{aligned}$$"),
        "{view}"
    );
}
