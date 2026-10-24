// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Probes from the adversarial reviews of the Markdown round trip
//! (`tests/fixtures/md-roundtrip/review`): table rows wider than their grid,
//! an author's date range, the document owner; and the author and date a
//! change or comment that names none takes (flag, front matter, the
//! reference document's creator and creation time, else Redline and
//! now).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const REVIEW: &str = "tests/fixtures/md-roundtrip/review";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(REVIEW)
        .join(name)
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn jubarte(dir: &Path, args: &[&str]) -> (String, String) {
    let out = run(dir, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn read(dir: &Path, docx: &Path) -> String {
    jubarte(dir, &["read", docx.to_str().unwrap(), "--no-page-markers"]).0
}

/// `docx` read, written from that view, and read again.
fn round_trip(dir: &Path, docx: &Path) -> (String, String) {
    let first = read(dir, docx);
    let md = dir.join("view.md");
    let out = dir.join("view.docx");
    std::fs::write(&md, &first).unwrap();
    jubarte(
        dir,
        &[
            "convert",
            md.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
            // The epoch pinned: unpinned, an undated change is dated now.
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    (first, read(dir, &out))
}

/// The lines of `view` that tell a table's shape and an author's dates.
fn telling(view: &str) -> Vec<String> {
    view.lines()
        .filter(|l| l.starts_with("<!-- t") || l.starts_with("  A"))
        .map(str::to_string)
        .collect()
}

fn part(docx: &Path, name: &str) -> String {
    let file = std::fs::File::open(docx).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut entry = zip.by_name(name).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut entry, &mut text).unwrap();
    text
}

#[test]
fn a_row_wider_than_its_grid_keeps_its_cells_and_merges() {
    let dir = tempfile::tempdir().unwrap();
    for (name, line) in [
        // An empty cell past the grid.
        (
            "p19_wide_empty_extra.docx",
            "<!-- t0 2x3, cells t0.row0 p1-p3 t0.row1 p4-p7 -->",
        ),
        // A gap after over columns cells take.
        (
            "p16_after_overflows_cell.docx",
            "<!-- t0 2x4, cells t0.row0 p1-p3 t0.row1 p4, merged t0.row0.cell1 across 2, \
             gaps t0.row0 after 1; t0.row1 after 3 -->",
        ),
        // A merged pad cell longer than the room the gap leaves.
        (
            "p17_span_cut_by_after.docx",
            "<!-- t0 2x4, cells t0.row0 p1-p2 t0.row1 p3, merged t0.row0.cell1 across 3, \
             gaps t0.row0 after 2; t0.row1 after 3 -->",
        ),
    ] {
        let (first, second) = round_trip(dir.path(), &fixture(name));
        assert!(first.contains(line), "{name}: no {line:?} in\n{first}");
        assert_eq!(telling(&first), telling(&second), "{name}");
    }
}

#[test]
fn an_authors_range_comes_back_whatever_change_ends_it() {
    let dir = tempfile::tempdir().unwrap();
    // The range's last day on a formatting change, an inserted row, a
    // header's insertion.
    for name in ["A-fmt.docx", "B-row.docx", "C-header.docx"] {
        let (first, second) = round_trip(dir.path(), &fixture(name));
        let range = "# 2 revisions, 2026-03-02..2026-03-04";
        assert!(first.contains(range), "{name}: {first}");
        assert!(second.contains(range), "{name} written back:\n{second}");
    }
}

#[test]
fn a_date_the_view_writes_out_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    // A comment dated midnight on the first day stays so; the undated one
    // takes the last day.
    let out = dir.path().join("e.docx");
    jubarte(
        dir.path(),
        &[
            "convert",
            fixture("E-midnight.md").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
        ],
    );
    let comments = part(&out, "word/comments.xml");
    let at = |text: &str| comments.find(text).unwrap();
    assert!(at("2026-03-04T23:59:59Z") < at(">one<"), "{comments}");
    assert!(at("2026-03-02T00:00:00Z") > at(">one<"), "{comments}");
    // A comment dated on the first day, not at midnight, keeps the range.
    let out = dir.path().join("m.docx");
    jubarte(
        dir.path(),
        &[
            "convert",
            fixture("M-noon.md").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
        ],
    );
    assert!(
        read(dir.path(), &out).contains("2026-03-02..2026-03-04"),
        "M-noon"
    );
}

#[test]
fn a_bare_day_dates_and_a_backwards_range_warns() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("g.docx");
    jubarte(
        dir.path(),
        &[
            "convert",
            fixture("G-bare.md").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
        ],
    );
    assert!(part(&out, "word/document.xml").contains("w:date=\"2026-03-02T00:00:00Z\""));
    let out = dir.path().join("f.docx");
    let (_, warnings) = jubarte(
        dir.path(),
        &[
            "convert",
            fixture("F-reversed.md").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
        ],
    );
    assert!(warnings.contains("its days run backwards"), "{warnings}");
}

fn convert_core(dir: &Path, md: &str, reference: Option<&str>) -> (String, String) {
    let out = dir.join("owner.docx");
    let md = fixture(md);
    let reference = reference.map(fixture);
    let mut args = vec![
        "convert",
        md.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--force",
    ];
    if let Some(reference) = &reference {
        args.extend(["--reference-doc", reference.to_str().unwrap()]);
    }
    let (_, warnings) = jubarte(dir, &args);
    (part(&out, "docProps/core.xml"), warnings)
}

#[test]
fn an_empty_or_null_owner_leaves_the_documents_own() {
    let dir = tempfile::tempdir().unwrap();
    for md in ["m-empty-owner.md", "m-null-owner.md"] {
        let (core, warnings) = convert_core(dir.path(), md, Some("ref-lmb.docx"));
        assert!(core.contains("Mod Iris"), "{md}: {core}");
        assert!(!core.contains("<dc:creator>null"), "{md}: {core}");
        assert!(warnings.contains("names no one"), "{md}: {warnings}");
    }
}

#[test]
fn a_comment_on_the_authors_key_keeps_its_block() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = convert_core(dir.path(), "m-authors-comment.md", Some("ref-normal.docx"));
    assert!(core.contains("<dc:creator>Bob New</dc:creator>"), "{core}");
}

#[test]
fn core_properties_cut_short_are_left_as_they_are() {
    let dir = tempfile::tempdir().unwrap();
    let (core, _) = convert_core(dir.path(), "m-none.md", Some("ref-badcore.docx"));
    assert!(core.contains("<dc:creator>Ada</dc:creator>"), "{core}");
}

#[test]
fn the_owner_is_the_first_creator_named_with_its_spaces() {
    let dir = tempfile::tempdir().unwrap();
    assert!(read(dir.path(), &fixture("ref-dup-creator.docx")).contains("document_owner: Zed"));
    let (core, _) = convert_core(dir.path(), "m-padded-owner.md", None);
    assert!(core.contains("<dc:creator>  Bob  </dc:creator>"), "{core}");
    let out = dir.path().join("owner.docx");
    assert!(read(dir.path(), &out).contains("document_owner: \"  Bob  \""));
}

/// `{++x++}` untagged, converted with the front matter `front` and the
/// flags `flags`: the insertion's author and date.
fn stamped(front: &str, flags: &[&str]) -> String {
    let dir = tempfile::tempdir().unwrap();
    let md = dir.path().join("plain.md");
    std::fs::write(&md, format!("{front}Some {{++x++}} here.\n")).unwrap();
    let out = dir.path().join("plain.docx");
    let mut args = vec![
        "convert",
        md.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--force",
    ];
    args.extend(flags);
    jubarte(dir.path(), &args);
    let xml = part(&out, "word/document.xml");
    let at = xml.find("<w:ins ").unwrap();
    xml[at..at + xml[at..].find('>').unwrap()].to_string()
}

/// `w:date="…"` of `ins`.
fn date_of(ins: &str) -> String {
    let at = ins.find("w:date=\"").unwrap() + "w:date=\"".len();
    ins[at..at + ins[at..].find('"').unwrap()].to_string()
}

#[test]
fn an_unnamed_change_is_signed_redline_and_dated_now() {
    let before = jubarte::convert::utc_now_iso8601();
    let ins = stamped("", &[]);
    let after = jubarte::convert::utc_now_iso8601();
    assert!(ins.contains("w:author=\"Redline\""), "{ins}");
    // The same format sorts by time.
    let date = date_of(&ins);
    assert!(
        before <= date && date <= after,
        "{before} <= {date} <= {after}"
    );
}

/// Without a flag or front matter, the `--reference-doc`'s creator and
/// creation time sign and date what names no one; the front matter beats
/// them, and a reference without a creation time leaves now.
#[test]
fn the_reference_documents_creator_and_creation_time_come_next() {
    let stamp = fixture("ref-stamp.docx");
    let stamp = stamp.to_str().unwrap();
    let ins = stamped("", &["--reference-doc", stamp]);
    assert!(ins.contains("w:author=\"Cleo Core\""), "{ins}");
    assert_eq!(date_of(&ins), "2025-03-04T05:06:07Z", "{ins}");
    let front = "---\nauthor: Fran Front\nlast_modified: 2025-01-02T03:04:05Z\n---\n\n";
    let ins = stamped(front, &["--reference-doc", stamp]);
    assert!(ins.contains("w:author=\"Fran Front\""), "{ins}");
    assert_eq!(date_of(&ins), "2025-01-02T03:04:05Z", "{ins}");
    let normal = fixture("ref-normal.docx");
    let ins = stamped("", &["--reference-doc", normal.to_str().unwrap()]);
    assert!(ins.contains("w:author=\"Ada\""), "{ins}");
    assert!(date_of(&ins).starts_with("20"), "{ins}");
    assert_ne!(date_of(&ins), "2025-03-04T05:06:07Z", "{ins}");
}

#[test]
fn the_front_matter_beats_the_default_and_a_flag_beats_both() {
    let front = "---\nauthor: Fran Front\nlast_modified: 2025-01-02T03:04:05Z\n---\n\n";
    let ins = stamped(front, &[]);
    assert!(ins.contains("w:author=\"Fran Front\""), "{ins}");
    assert!(ins.contains("w:date=\"2025-01-02T03:04:05Z\""), "{ins}");
    let ins = stamped(
        front,
        &["--author", "Cli Name", "--datetime", "2024-02-03T04:05:06Z"],
    );
    assert!(ins.contains("w:author=\"Cli Name\""), "{ins}");
    assert!(ins.contains("w:date=\"2024-02-03T04:05:06Z\""), "{ins}");
}

#[test]
fn the_date_flag_is_gone_for_datetime() {
    let dir = tempfile::tempdir().unwrap();
    let md = dir.path().join("x.md");
    std::fs::write(&md, "x\n").unwrap();
    let out = run(
        dir.path(),
        &[
            "convert",
            md.to_str().unwrap(),
            "-o",
            "x.docx",
            "--date",
            "2024-02-03T04:05:06Z",
        ],
    );
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--datetime"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// What front matter cannot sign or date: a date out of the calendar, a
/// null or comment-only `author:`, keys after a thematic break that opens
/// no front matter (round-7 defaults review, findings 1, 4 and 5).
#[test]
fn front_matter_that_names_no_one_or_no_real_date_leaves_redline_and_now() {
    for front in [
        "---\ndate: 2026-02-30\n---\n\n",
        "---\nlast_modified: 2026-99-99T99:99:99Z\n---\n\n",
        "---\nauthor: null\n---\n\n",
        "---\nauthor: ~\n---\n\n",
        "---\nauthor:  # the legend names them\n---\n\n",
        "---\n\nIntro.\n\n```\nauthor: Mallory\ndate: 2025-01-01T00:00:00Z\n```\n\n",
    ] {
        let before = jubarte::convert::utc_now_iso8601();
        let ins = stamped(front, &[]);
        let after = jubarte::convert::utc_now_iso8601();
        assert!(ins.contains("w:author=\"Redline\""), "{front:?}: {ins}");
        let date = date_of(&ins);
        assert!(before <= date && date <= after, "{front:?}: {ins}");
    }
    // A quoted "null" is a name.
    let ins = stamped("---\nauthor: \"null\"\n---\n\n", &[]);
    assert!(ins.contains("w:author=\"null\""), "{ins}");
}

/// An author with a line break keeps it in `w:author` (a character
/// reference: a literal one reads back as a space).
#[test]
fn an_author_with_a_line_break_keeps_it() {
    let ins = stamped("---\nauthor: \"A\\nB\\tC\"\n---\n\n", &[]);
    assert!(ins.contains("w:author=\"A&#xA;B&#x9;C\""), "{ins}");
}

/// A library caller's date must be one Word takes as `w:date`.
#[test]
fn a_library_date_out_of_the_calendar_is_refused() {
    let options = jubarte::markdown::DocxOptions {
        date: "2026-02-30T00:00:00Z".into(),
        date_fixed: true,
        ..jubarte::markdown::DocxOptions::default()
    };
    let error =
        jubarte::markdown::markdown_to_docx("Some {++x++}.\n", &options).expect_err("refused");
    assert!(error.to_string().contains("2026-02-30"), "{error}");
}

/// A date key that does not read warns only when no other key dated the
/// file (finding 6).
#[test]
fn an_unread_date_key_warns_only_when_nothing_else_dates_the_file() {
    let write = |front: &str| {
        let options = jubarte::markdown::DocxOptions::default();
        jubarte::markdown::markdown_to_docx(&format!("{front}Some {{++x++}}.\n"), &options)
            .unwrap()
            .warnings
    };
    let warned = |warnings: &[String]| warnings.iter().any(|w| w.contains("is no timestamp"));
    assert!(warned(&write("---\nlast_modified: garbage\n---\n\n")));
    assert!(!warned(&write(
        "---\nlast_modified: garbage\nmodified: 2025-05-05T05:05:05Z\n---\n\n"
    )));
}

/// A range only headers or footers carry (round-7 dates review, finding
/// 1): the first story mark keeps the first day, the later ones take the
/// last.
#[test]
fn a_range_only_headers_or_footers_carry_comes_back() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["H-hdr-only.docx", "I-ftr-only.docx"] {
        let (first, second) = round_trip(dir.path(), &fixture(name));
        let range = "# 2 revisions, 2026-03-02..2026-03-04";
        assert!(first.contains(range), "{name}: {first}");
        assert!(second.contains(range), "{name} written back:\n{second}");
    }
}

/// A range written with timestamps reads as their days (finding 5), and
/// a tag dated outside the days warns (finding 3).
#[test]
fn a_timestamp_range_reads_as_days_and_a_tag_outside_it_warns() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("j.docx");
    let fixture_j = fixture("J-tz-range.md");
    jubarte(
        dir.path(),
        &[
            "convert",
            fixture_j.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let view = read(dir.path(), &out);
    assert!(
        view.contains("# 2 revisions, 2026-03-02..2026-03-04"),
        "{view}"
    );
    let out = dir.path().join("k.docx");
    let fixture_k = fixture("K-dated-out.md");
    let (_, stderr) = jubarte(
        dir.path(),
        &[
            "convert",
            fixture_k.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    assert!(
        stderr.contains("AL: a tag dated 2026-03-09 falls outside 2026-03-02..2026-03-04"),
        "{stderr}"
    );
}

/// A heading whose slug is longer than Word's 40-character bookmark name
/// keeps its bookmark through a round trip, and the link reaching it by
/// that name still lands (round-7 headings review, F1).
#[test]
fn a_long_headings_word_bookmark_survives_the_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let word_name = "aaaa-bbbb-cccc-dddd-eeee-ffff-gggg-hhhh-";
    let (first, second) = round_trip(dir.path(), &fixture("N-longslug.docx"));
    assert!(first.contains(&format!("(#{word_name})")), "{first}");
    assert_eq!(first.matches("<a id").count(), 0, "{first}");
    assert_eq!(second.matches("<a id").count(), 0, "{second}");
    let written = part(&dir.path().join("view.docx"), "word/document.xml");
    assert!(
        written.contains(&format!("w:name=\"{word_name}\"")),
        "{written}"
    );
}

/// An anchor that takes a long heading's Word name pushes the heading's
/// bookmark to `-1`: the reader still knows it as the heading's own, and
/// the link still lands (F2).
#[test]
fn a_long_headings_suffixed_bookmark_is_still_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.docx");
    let source = fixture("P-longslug.md");
    let (_, stderr) = jubarte(
        dir.path(),
        &[
            "convert",
            source.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--force",
        ],
    );
    assert!(!stderr.contains("names no heading"), "{stderr}");
    let view = read(dir.path(), &out);
    let heading = view.lines().find(|l| l.starts_with("# ")).unwrap();
    assert!(!heading.contains("<a id"), "{view}");
    // Written again, the link the view prints still lands.
    let md = dir.path().join("again.md");
    std::fs::write(&md, &view).unwrap();
    let (_, stderr) = jubarte(
        dir.path(),
        &[
            "convert",
            md.to_str().unwrap(),
            "-o",
            dir.path().join("again.docx").to_str().unwrap(),
            "--force",
        ],
    );
    assert!(!stderr.contains("names no heading"), "{stderr}");
}

/// A run its character style hides prints in no header, whatever the
/// view resolves (F3).
#[test]
fn a_style_hidden_header_run_stays_hidden_when_changes_resolve() {
    let dir = tempfile::tempdir().unwrap();
    let docx = fixture("O-hidden-header.docx");
    for resolve in [
        &[][..],
        &["--track-changes", "accept"],
        &["--track-changes", "reject"],
    ] {
        let mut args = vec!["read", docx.to_str().unwrap(), "--no-page-markers"];
        args.extend(resolve);
        let (view, _) = jubarte(dir.path(), &args);
        assert!(!view.contains("[style]"), "{resolve:?}: {view}");
        assert!(view.contains("Head [unhidden]"), "{resolve:?}: {view}");
    }
}
