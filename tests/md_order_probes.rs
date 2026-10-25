// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The revision-order probes (`tests/fixtures/md-roundtrip/order`, written
//! by its `make_probes.py`): something between, inside or across the two
//! marks of a substitution, in the body, a cell, a header and a footnote.
//! The text and the `revisions:` count pair the marks alike, and the view
//! comes back from the docx it writes.

use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

const PROBES: &str = "tests/fixtures/md-roundtrip/order";

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn probe(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(PROBES)
        .join(format!("{name}.docx"))
}

fn read(docx: &Path, extra: &[&str]) -> String {
    let path = docx.to_str().unwrap();
    let mut args = vec!["read", path, "--no-page-markers"];
    args.extend(extra);
    jubarte(Path::new(env!("CARGO_MANIFEST_DIR")), &args)
}

/// The view without the lines that name the file, and without the page
/// count, which is measured, not written.
fn comparable(view: &str) -> String {
    let pages = Regex::new(r", \d+ pages?\s+#.*$").unwrap();
    view.lines()
        .filter(|line| !line.starts_with("source: ") && !line.starts_with("sha256: "))
        .map(|line| {
            if line.starts_with("body: ") {
                pages.replace(line, "").into_owned()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The view of the docx `markdown` writes.
fn again(dir: &Path, markdown: &str, name: &str) -> String {
    let md = dir.join(format!("{name}.md"));
    let docx = dir.join(format!("{name}.docx"));
    std::fs::write(&md, markdown).unwrap();
    jubarte(
        dir,
        &[
            "convert",
            md.to_str().unwrap(),
            "-o",
            docx.to_str().unwrap(),
            "--force",
            // The epoch pinned: unpinned, an undated change is dated now.
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    read(&docx, &[])
}

/// What the view prints for each probe, and how its `revisions:` line pairs
/// the same marks.
const SEEN: &[(&str, &str, &str)] = &[
    // Nothing printed between the marks: one substitution.
    (
        "direct-vanish",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "style-vanish",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "style-basedon",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "style-vanish-tab",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "soft-hyphen",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    ("sym", "A {~~old~>new~~}{>>r2+1 @AL<<} Z", "1 substitution"),
    (
        "lrp-only",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "empty-run",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "page-br",
        "A {~~old~>new~~}{>>r2+1 @AL<<} Z",
        "1 substitution",
    ),
    (
        "t-style-vanish",
        "|{~~old~>new~~}{>>r2+1 @AL<<}|",
        "1 substitution",
    ),
    // A printed tab keeps them apart.
    (
        "empty-t-tab",
        "A {++new++}{>>r1 @AL<<}\t{--old--}{>>r2 @AL<<} Z",
        "1 insertion, 1 deletion",
    ),
    // A comment inside one side stays a substitution; one beside or across
    // the marks parts them.
    (
        "c-inside",
        "A {~~old~>{==new==}~~}{>>r2+1 @A<<}{>>c0 @A: note<<} Z",
        "1 substitution",
    ),
    (
        "c-between",
        "A {++new++}{>>r1 @A<<}{=={--old--}{>>r2 @A<<}==}{>>c0 @A: note<<} Z",
        "1 insertion, 1 deletion",
    ),
    (
        "c-cross",
        "A {++{==new==}++}{>>r1 @A<<}{=={--old--}{>>r2 @A<<}==}{>>c0 @A: note<<} Z",
        "1 insertion, 1 deletion",
    ),
    // A heading's own slug bookmark prints nothing and parts nothing; a
    // name the writer would not give it (`title-01`, a lone `title-1`) is
    // an anchor of its own.
    (
        "h-own-slug",
        "# {~~Old~>Title~~}{>>r2+1 @AL<<}",
        "1 substitution",
    ),
    (
        "h-two",
        "# {~~Old~>Title~~}{>>r2+1 @AL<<}",
        "1 substitution",
    ),
    (
        "h-suffix-01",
        "# {++Title++}{>>r1 @AL<<}<a id=\"title-01\"></a>{--Old--}{>>r2 @AL<<}",
        "1 insertion, 1 deletion",
    ),
    (
        "h-suffix-1",
        "# {++Title++}{>>r1 @AL<<}<a id=\"title-1\"></a>{--Old--}{>>r2 @AL<<}",
        "1 insertion, 1 deletion",
    ),
    (
        "h-suffix-1-1",
        "# {++Intro 1++}{>>r1 @AL<<}<a id=\"intro-1-1\"></a>{--Old--}{>>r2 @AL<<}",
        "1 insertion, 1 deletion",
    ),
    (
        "h-foreign",
        "# {++Title++}{>>r1 @AL<<}<a id=\"other\"></a>{--Old--}{>>r2 @AL<<}",
        "1 insertion, 1 deletion",
    ),
    // A cell writes no heading, so its bookmark is an anchor.
    (
        "t-heading",
        "|{++Title++}{>>r1 @AL<<}<a id=\"title\"></a>{--Old--}{>>r2 @AL<<}|",
        "1 insertion, 1 deletion",
    ),
    // Headers pair as the body does, and print no hidden run.
    (
        "hdr-direct-vanish",
        "text: \"{~~old~>new~~}{>>header1.r2+1 @AL<<}\"",
        "2 in header1",
    ),
    (
        "hdr-style-vanish",
        "text: \"{~~old~>new~~}{>>header1.r2+1 @AL<<}\"",
        "2 in header1",
    ),
    (
        "fn-style-vanish",
        "[^1]: {~~old~>new~~}{>>footnotes.r2+1 @AL<<}",
        "2 in footnotes",
    ),
    (
        "fn-soft",
        "[^1]: {~~old~>new~~}{>>footnotes.r2+1 @AL<<}",
        "2 in footnotes",
    ),
];

#[test]
fn the_text_and_the_count_pair_the_same_marks() {
    for (name, text, count) in SEEN {
        let view = read(&probe(name), &[]);
        assert!(
            view.lines().any(|line| line.contains(text)),
            "{name}: no line holds {text:?} in\n{view}"
        );
        let revisions = view
            .lines()
            .find(|line| line.starts_with("revisions:"))
            .unwrap_or_default();
        assert!(
            revisions.contains(count),
            "{name}: {revisions:?} does not say {count:?}"
        );
    }
}

/// Accepting names the change by the tag the tracked text gives it.
#[test]
fn the_accept_id_line_joins_what_the_text_joins() {
    for (name, id_line) in [
        ("h-own-slug", "<!-- p0 rev r2+1 @AL -->"),
        ("c-inside", "<!-- p0 rev r2+1 @A -->"),
        ("style-vanish", "<!-- p0 rev r2+1 @AL -->"),
    ] {
        let view = read(&probe(name), &["--track-changes", "accept"]);
        assert!(view.contains(id_line), "{name}: no {id_line:?} in\n{view}");
    }
}

/// A comment range that crosses from one mark into the other ends with the
/// accepted text, not at the paragraph's end.
#[test]
fn accepting_keeps_a_crossing_comment_on_its_text() {
    let view = read(&probe("c-cross"), &["--track-changes", "accept"]);
    assert!(
        view.contains("A {==new==}{>>c0 @A: note<<} Z"),
        "c-cross accepted:\n{view}"
    );
}

/// Probes whose first view the writer gives back with a known difference;
/// each is a fixed point from the docx it wrote on.
const SETTLED_LATER: &[(&str, &str)] = &[
    // A comment across the marks is written as one substitution under it.
    ("c-cross", "canonical substitution"),
    ("t-cross", "canonical substitution"),
    // Word asks a paragraph after a table that ends the body.
    ("t-cross", "trailing paragraph"),
    ("t-inside", "trailing paragraph"),
    ("t-plain", "trailing paragraph"),
    ("t-lrp", "trailing paragraph"),
    ("t-tab", "trailing paragraph"),
    ("t-style-vanish", "trailing paragraph"),
    ("t-heading", "trailing paragraph"),
    ("t-heading-1", "trailing paragraph"),
    // Word shows no comment in a header; one in a footnote moves to its
    // reference mark.
    ("hdr-cross", "header comment"),
    ("hdr-between", "header comment"),
    ("fn-cross", "footnote comment"),
    ("fn-inside", "footnote comment"),
];

#[test]
fn every_probe_comes_back_from_the_docx_its_view_writes() {
    let dir = tempfile::tempdir().unwrap();
    let mut names: Vec<String> =
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(PROBES))
            .unwrap()
            .filter_map(|e| {
                let path = e.unwrap().path();
                (path.extension()? == "docx")
                    .then(|| path.file_stem()?.to_str().map(str::to_string))?
            })
            .collect();
    names.sort();
    assert!(names.len() > 40, "probes missing: {names:?}");
    for name in &names {
        let first = read(&probe(name), &[]);
        let second = again(dir.path(), &first, &format!("{name}-2"));
        if SETTLED_LATER.iter().any(|(n, _)| n == name) {
            let third = again(dir.path(), &second, &format!("{name}-3"));
            assert_eq!(
                comparable(&second),
                comparable(&third),
                "{name}: the second view is no fixed point"
            );
        } else {
            assert_eq!(
                comparable(&first),
                comparable(&second),
                "{name}: the view does not come back"
            );
        }
    }
}
