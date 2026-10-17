// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The Markdown round trip, strict (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md §0: nothing with a short id is
//! lost). Every `.md` in `tests/fixtures/md-roundtrip` must come back
//! exactly after md → docx → md, and every docx listed in its `docx.txt`
//! must print the same Markdown after docx → md → docx → md. Only the
//! `source:`/`sha256:` lines, which name the file, are left out. Arthur,
//! 2026-10-13: "Start strict! Fail fail fail until we fix": there is no list
//! of known failures.
//!
//! Every case runs to the end, and the failure lists every short id it met
//! with a verdict: `✓ p3 correct`, `✗ p5 wrong` (with the first line that
//! changed), `✗ c3 lost`, `✗ r9 new`. Arthur, 2026-10-13: "Make it run until
//! the end and fail only then. I want the lots saying: this is wrong, that is
//! wrong, this is correct". The `ids-*.md` fixtures are views of documents
//! dense in one kind of id (`views.txt`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use regex::Regex;

const FIXTURES: &str = "tests/fixtures/md-roundtrip";

/// The page count and its source at the end of the `body:` line.
static PAGES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r", \d+ pages?\s+#.*$").unwrap());

/// The Markdown fixtures written by hand, in the view's dialect; every other
/// one is a view of a docx listed in `views.txt`.
const HAND_WRITTEN: &[&str] = &[
    "kitchen-sink.md",
    // Pandoc manuscripts, as their authors wrote them (2026-10-14).
    "dolphins.md",
    "eLife.00047.md",
    "git_manuscript.md",
    "list_of_figures.md",
    // A Google Docs export (2026-10-16). Arthur: "we will use its syntax for
    // rendering images": `![][image4]` in the text, `[image4]: <data:image/
    // png;base64,…>` defined at the end, a definition shared by every use.
    "docx-libraries-comparison.md",
];

fn jubarte(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("{args:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{args:?} exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("{args:?}: {e}"))
}

/// The view as the round trip compares it: page markers off, and without
/// the lines that name the file.
fn view(dir: &Path, docx: &str) -> Result<String, String> {
    let text = jubarte(dir, &["read", docx, "--no-page-markers"])?;
    Ok(comparable(&text))
}

fn comparable(markdown: &str) -> String {
    // Only the YAML header's own source:/sha256: lines name the file; a body
    // paragraph that happens to start with those words is content.
    let mut front_matter = false;
    let mut out = String::new();
    for (i, line) in markdown.lines().enumerate() {
        if line == "---" {
            front_matter = i == 0;
        }
        if front_matter && (line.starts_with("source: ") || line.starts_with("sha256: ")) {
            continue;
        }
        // The page count is measured, not written: Word's cached layout
        // counts the source, jubarte's layout counts what it wrote. Arthur,
        // 2026-10-13: "we don't force page count into the docx".
        if front_matter && let Some(pages) = PAGES.find(line).filter(|_| line.starts_with("body: "))
        {
            out.push_str(&line[..pages.start()]);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn to_docx(dir: &Path, markdown: &str, name: &str) -> Result<String, String> {
    let md = format!("{name}.md");
    let docx = format!("{name}.docx");
    std::fs::write(dir.join(&md), markdown).map_err(|e| e.to_string())?;
    // The epoch pinned: unpinned, an undated change is dated now.
    jubarte(
        dir,
        &[
            "convert",
            &md,
            "-o",
            &docx,
            "--force",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    )?;
    Ok(docx)
}

/// The first lines that differ, numbered, for the failure report.
fn difference(want: &str, got: &str) -> String {
    let (a, b): (Vec<&str>, Vec<&str>) = (want.lines().collect(), got.lines().collect());
    let mut out = Vec::new();
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied(), b.get(i).copied());
        if x != y {
            out.push(format!(
                "    line {}:\n      want {:?}\n      got  {:?}",
                i + 1,
                x.unwrap_or("<end>"),
                y.unwrap_or("<end>")
            ));
        }
        if out.len() == 8 {
            out.push("    …".to_string());
            break;
        }
    }
    format!("{} lines -> {} lines\n{}", a.len(), b.len(), out.join("\n"))
}

/// One round trip: the Markdown it started from and the Markdown it gave
/// back, or why it gave nothing back; and what the docx it wrote lost or
/// gained against the source docx.
struct Case {
    name: String,
    want: String,
    got: Result<String, String>,
    docx: Vec<String>,
}

/// What the docx jubarte wrote lost or gained against the source docx, as
/// `jubarte debug A B --check render` lists it (highlights, colours, fonts,
/// styles, tables, sections, parts). The Markdown fixed point cannot see a
/// loss in the first docx → md step: both views lose it alike. This can.
fn docx_verdicts(dir: &Path, source: &str, written: &str) -> Vec<String> {
    let out = match jubarte(dir, &["debug", source, written, "--check", "render"]) {
        Ok(out) => out,
        Err(e) => return vec![format!("  ✗ docx not compared: {e}")],
    };
    let mut part = String::new();
    let mut lines = Vec::new();
    // A lost line and a new line that say the same cancel each other.
    let mut lost: Vec<(String, String)> = Vec::new();
    let mut new: Vec<(String, String)> = Vec::new();
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("-A ") {
            if let Some(rest) = in_dialect(&part, rest.trim()) {
                lost.push((part.clone(), rest));
            }
        } else if let Some(rest) = line.strip_prefix("+B ") {
            if let Some(rest) = in_dialect(&part, rest.trim()) {
                new.push((part.clone(), rest));
            }
        } else if let Some(name) = line.strip_suffix(": only in A") {
            lines.push(format!("  ✗ docx {name} lost: the whole part"));
        } else if let Some(name) = line.strip_suffix(": only in B") {
            lines.push(format!("  ✗ docx {name} new: the whole part"));
        } else if let Some((name, _)) = line
            .split_once(": ")
            .filter(|(_, r)| r.ends_with("differ") || r.ends_with("differs"))
        {
            part = format!("{name} ");
        }
    }
    let (lost, new) = unmatched(lost, new);
    lines.extend(
        lost.iter()
            .map(|(part, rest)| format!("  ✗ docx {part}lost: {rest}")),
    );
    lines.extend(
        new.iter()
            .map(|(part, rest)| format!("  ✗ docx {part}new: {rest}")),
    );
    lines
}

/// The lost and the new lines left once the longest run of equal lines,
/// in order, cancels: a line that only moved (an insertion now after the
/// deletion it came before) is reported, not cancelled.
fn unmatched<T: PartialEq>(lost: Vec<T>, new: Vec<T>) -> (Vec<T>, Vec<T>) {
    let (n, m) = (lost.len(), new.len());
    let mut longest = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            longest[i][j] = if lost[i] == new[j] {
                longest[i + 1][j + 1] + 1
            } else {
                longest[i + 1][j].max(longest[i][j + 1])
            };
        }
    }
    let (mut kept_lost, mut kept_new) = (vec![false; n], vec![false; m]);
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if lost[i] == new[j] {
            (i, j) = (i + 1, j + 1);
        } else if longest[i + 1][j] >= longest[i][j + 1] {
            kept_lost[i] = true;
            i += 1;
        } else {
            kept_new[j] = true;
            j += 1;
        }
    }
    kept_lost[i..].iter_mut().for_each(|k| *k = true);
    kept_new[j..].iter_mut().for_each(|k| *k = true);
    let keep = |items: Vec<T>, kept: Vec<bool>| {
        items
            .into_iter()
            .zip(kept)
            .filter_map(|(item, kept)| kept.then_some(item))
            .collect()
    };
    (keep(lost, kept_lost), keep(new, kept_new))
}

#[test]
fn a_moved_render_line_is_reported_and_an_equal_one_cancels() {
    assert_eq!(
        unmatched(vec!["X", "A"], vec!["A", "X"]),
        (vec!["X"], vec!["X"])
    );
    assert_eq!(unmatched(vec!["A", "B"], vec!["A", "B"]), (vec![], vec![]));
    assert_eq!(
        unmatched(vec!["A", "B", "C"], vec!["B", "C", "A"]),
        (vec!["A"], vec!["A"])
    );
    assert_eq!(unmatched(vec!["A"], vec!["B"]), (vec!["A"], vec!["B"]));
}

/// What the Markdown view cannot say, so a docx it writes cannot hold it,
/// or says in another form Word shows alike: each pattern with why. Everything else `debug --check render` lists is
/// compared: the styles legend (each style's font, size, bold, spacing,
/// alignment), the page, highlights (their colour on the `<mark>`'s
/// `class`), tables, sections, fields, notes and parts.
static OUT_OF_DIALECT: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
    [
        // A font set on a run, not through a style: the view has no run font.
        (r#"^rfonts "[^"]*" ×"#, "direct run font"),
        // The style a font comes through (`via "Normal"`, or the document
        // defaults): the legend gives each style's effective font, which
        // the Markdown comparison checks; where it is set, it does not say.
        (
            r#"^rfonts "[^"]*" via "[^"]*" ×"#,
            "where a style's font is set",
        ),
        (r"^color ", "text colour, set on a run or by a style"),
        (r"^(para|run)-shd ", "shading"),
        // The package's font list and theme, the defaults under every
        // style, and the faces a renderer loads for them: the legend gives
        // each style's effective font instead.
        // Word draws a `HYPERLINK "url"` field and a `w:hyperlink` alike,
        // and the view prints both as `[text](url)`, whose text and target
        // the Markdown comparison checks. Arthur, 2026-10-13: treat them as
        // equal. A header's or footer's field is not a link the writer
        // writes, so it stays compared.
        (
            r"^word/(document|footnotes|endnotes|comments)\.xml field HYPERLINK ",
            "a HYPERLINK field and a link are the same link",
        ),
        (
            r"^\(layout\) (fontTable|theme|font|docDefaults) ",
            "fonts outside the legend",
        ),
        // A note is one Markdown definition: a table in it prints as its
        // cells, one `<br>` line each (their text is compared), and the
        // dialect has no table inside a note to write back.
        (
            r"^word/(footnotes|endnotes)\.xml table ",
            "a table inside a note",
        ),
    ]
    .into_iter()
    .map(|(pattern, why)| (Regex::new(pattern).unwrap(), why))
    .collect()
});

/// A `debug --check render` line as far as the view can say it, `None` when
/// it says nothing the view can: a table's cell shading is left out, so the
/// table itself is compared without it. A highlight's colour the view does
/// say (`<mark class="hl-…">`), so it is compared too.
fn in_dialect(part: &str, rest: &str) -> Option<String> {
    static CELLS_SHD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r" cells-shd\[[^\]]*\]").unwrap());
    let line = format!("{part}{rest}");
    if OUT_OF_DIALECT
        .iter()
        .any(|(pattern, _)| pattern.is_match(rest) || pattern.is_match(&line))
    {
        return None;
    }
    Some(CELLS_SHD.replace_all(rest, "").into_owned())
}

/// The docx a committed view was read from (`views.txt`), by file name.
fn view_sources() -> HashMap<String, PathBuf> {
    std::fs::read_to_string(Path::new(FIXTURES).join("views.txt"))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(' '))
        .map(|(md, docx)| (md.to_string(), PathBuf::from(docx.trim())))
        .collect()
}

/// md → docx → md must give the Markdown back; when the Markdown is the
/// view of a docx, the docx it writes must also hold what that docx held.
fn markdown_case(dir: &Path, path: &Path, docx_source: Option<&Path>) -> Case {
    let name = format!("md -> docx -> md   {}", path.display());
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return Case {
                name,
                want: String::new(),
                got: Err(e.to_string()),
                docx: Vec::new(),
            };
        }
    };
    let written = to_docx(dir, &source, "md");
    let got = written.clone().and_then(|docx| view(dir, &docx));
    let docx = match (docx_source, &written) {
        (Some(original), Ok(written)) => match std::fs::copy(original, dir.join("original.docx")) {
            Ok(_) => docx_verdicts(dir, "original.docx", written),
            Err(e) => vec![format!(
                "  ✗ docx not compared: {}: {e}",
                original.display()
            )],
        },
        _ => Vec::new(),
    };
    Case {
        name,
        want: comparable(&source),
        got,
        docx,
    }
}

/// docx → md → docx → md must print the same Markdown twice.
fn docx_case(dir: &Path, path: &Path) -> Case {
    let name = format!("docx -> md -> docx -> md   {}", path.display());
    // The source sits apart from the Markdown: once anchoring is automatic
    // (spec §5.3), a Markdown beside its source would keep everything and
    // measure nothing.
    let first = std::fs::create_dir_all(dir.join("source"))
        .and_then(|()| std::fs::copy(path, dir.join("source/source.docx")))
        .map_err(|e| format!("{}: {e}", path.display()))
        .and_then(|_| view(dir, "source/source.docx"));
    match first {
        Ok(first) => {
            let written = to_docx(dir, &first, "back");
            let docx = match &written {
                Ok(written) => docx_verdicts(dir, "source/source.docx", written),
                Err(_) => Vec::new(),
            };
            let got = written.and_then(|docx| view(dir, &docx));
            Case {
                name,
                want: first,
                got,
                docx,
            }
        }
        Err(e) => Case {
            name,
            want: String::new(),
            got: Err(e),
            docx: Vec::new(),
        },
    }
}

/// A note: `{>>r1+2 @AL<<}`, `{>>c5 @AL: text<<}`, `{>>footer2.r7 @LW<<}`.
static NOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\{>>([^\s<]+)(.*?)<<\}").unwrap());

/// What a view says under each short id, in the order it says it:
/// id-line blocks (`p3`, `t0`, `p10-p11`, `footnotes.p0`) up to the next id
/// line, notes by their first token with the CriticMarkup span they follow,
/// and YAML header entries (`yaml authors.AL`). A repeated id is kept as
/// `ID#2`, so neither copy hides the other.
fn ids(view: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut push = |key: String, value: String| {
        let n = seen.entry(key.clone()).or_insert(0);
        *n += 1;
        let key = if *n == 1 { key } else { format!("{key}#{n}") };
        out.push((key, value));
    };
    let mut lines = view.lines().peekable();
    if lines.peek() == Some(&"---") {
        lines.next();
        let mut parent: Option<String> = None;
        let mut entry: Option<(String, String)> = None;
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
            let indented = line.starts_with(' ');
            let key = line
                .trim_start()
                .split(':')
                .next()
                .unwrap_or("")
                .trim_matches('"');
            let value_less = line
                .split("  #")
                .next()
                .unwrap_or(line)
                .trim_end()
                .ends_with(':');
            if !indented || parent.is_some() && line.starts_with("  ") && !line.starts_with("   ") {
                if let Some((k, v)) = entry.take() {
                    push(k, v);
                }
                if !indented {
                    parent = value_less.then(|| key.to_string());
                    if parent.is_some() {
                        continue;
                    }
                }
                let name = match (&parent, indented) {
                    (Some(p), true) => format!("yaml {p}.{key}"),
                    _ => format!("yaml {key}"),
                };
                entry = Some((name, line.to_string()));
            } else if let Some((_, v)) = entry.as_mut() {
                v.push('\n');
                v.push_str(line);
            }
        }
        if let Some((k, v)) = entry.take() {
            push(k, v);
        }
    }
    let body: Vec<&str> = lines.collect();
    let mut block: Option<(String, Vec<&str>)> = None;
    let mut blocks: Vec<(String, String)> = Vec::new();
    for line in &body {
        // A nested list item's id line is indented with its item.
        let id = line
            .trim_start()
            .strip_prefix("<!-- ")
            .and_then(|rest| rest.split_whitespace().next());
        match id {
            Some(id) if id != "-->" => {
                if let Some((k, v)) = block.take() {
                    blocks.push((k, v.join("\n").trim_end().to_string()));
                }
                block = Some((id.to_string(), vec![line]));
            }
            _ => match block.as_mut() {
                Some((_, v)) => v.push(line),
                None if line.trim().is_empty() => {}
                None => block = Some(("(before the first id)".to_string(), vec![line])),
            },
        }
    }
    if let Some((k, v)) = block.take() {
        blocks.push((k, v.join("\n").trim_end().to_string()));
    }
    for (k, v) in blocks {
        push(k, v);
    }
    let body = body.join("\n");
    for m in NOTE.captures_iter(&body) {
        let whole = m.get(0).unwrap();
        push(
            m[1].to_string(),
            format!("{}{}", span_before(&body, whole.start()), whole.as_str()),
        );
    }
    out
}

/// The CriticMarkup span a note at `at` belongs to: skip the notes chained
/// before it (`{~~a~>b~~}{>>r0<<}{>>r1<<}`), then take the span ending there.
fn span_before(text: &str, mut at: usize) -> &str {
    while text[..at].ends_with("<<}") {
        match text[..at].rfind("{>>") {
            Some(i) => at = i,
            None => break,
        }
    }
    for (open, close) in [
        ("{++", "++}"),
        ("{--", "--}"),
        ("{~~", "~~}"),
        ("{==", "==}"),
    ] {
        if text[..at].ends_with(close)
            && let Some(i) = text[..at].rfind(open)
        {
            return &text[i..at];
        }
    }
    ""
}

/// The first line of a value that changed, for a `wrong` verdict.
fn first_change(want: &str, got: &str) -> String {
    let cut = |s: &str| {
        let s: String = s.chars().take(160).collect();
        format!("{s:?}")
    };
    let (a, b): (Vec<&str>, Vec<&str>) = (want.lines().collect(), got.lines().collect());
    let i = (0..a.len().max(b.len()))
        .find(|&i| a.get(i) != b.get(i))
        .unwrap_or(0);
    format!(
        "\n        want {}\n        got  {}",
        cut(a.get(i).copied().unwrap_or("<end>")),
        cut(b.get(i).copied().unwrap_or("<end>"))
    )
}

/// Verdicts for every short id of one case, and the counts.
struct Report {
    lines: Vec<String>,
    correct: usize,
    wrong: usize,
    lost: usize,
    new: usize,
}

fn verdicts(want: &str, got: &str) -> Report {
    let (want, got) = (ids(want), ids(got));
    let got_by_id: HashMap<&str, &str> =
        got.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let want_ids: HashMap<&str, ()> = want.iter().map(|(k, _)| (k.as_str(), ())).collect();
    let mut r = Report {
        lines: Vec::new(),
        correct: 0,
        wrong: 0,
        lost: 0,
        new: 0,
    };
    for (id, value) in &want {
        match got_by_id.get(id.as_str()) {
            Some(g) if g == value => {
                r.correct += 1;
                r.lines.push(format!("  ✓ {id} correct"));
            }
            Some(g) => {
                r.wrong += 1;
                r.lines
                    .push(format!("  ✗ {id} wrong{}", first_change(value, g)));
            }
            None => {
                r.lost += 1;
                r.lines.push(format!("  ✗ {id} lost"));
            }
        }
    }
    for (id, _) in got
        .iter()
        .filter(|(k, _)| !want_ids.contains_key(k.as_str()))
    {
        r.new += 1;
        r.lines.push(format!(
            "  ✗ {id} new: the round trip printed an id the source does not have"
        ));
    }
    r
}

fn docx_list() -> Vec<PathBuf> {
    std::fs::read_to_string(Path::new(FIXTURES).join("docx.txt"))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(PathBuf::from)
        .collect()
}

#[test]
fn markdown_and_docx_round_trip_exactly() {
    let mut cases: Vec<Case> = Vec::new();
    let mut markdown: Vec<PathBuf> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")))
        .collect();
    markdown.sort();
    assert!(!markdown.is_empty(), "no Markdown fixtures in {FIXTURES}");
    let sources = view_sources();
    let unlisted: Vec<String> = markdown
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .filter(|f| !sources.contains_key(f) && !HAND_WRITTEN.contains(&f.as_str()))
        .collect();
    assert!(
        unlisted.is_empty(),
        "every Markdown fixture is a view listed in views.txt or hand-written (HAND_WRITTEN): {unlisted:?}"
    );
    // `MD_FIXED_POINT_ONLY=ids-tables` replays only the fixtures whose path
    // holds that text, for a quick loop on one document; the gate itself
    // runs them all.
    let only = std::env::var("MD_FIXED_POINT_ONLY").ok();
    let picked = |path: &Path| {
        only.as_deref()
            .is_none_or(|only| path.to_string_lossy().contains(only))
    };
    for path in markdown.iter().filter(|p| picked(p)) {
        let dir = tempfile::tempdir().unwrap();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        cases.push(markdown_case(
            dir.path(),
            path,
            sources.get(&file).map(PathBuf::as_path),
        ));
    }
    let docx = docx_list();
    assert!(!docx.is_empty(), "no docx fixtures in {FIXTURES}/docx.txt");
    for path in docx.iter().filter(|p| picked(p)) {
        let dir = tempfile::tempdir().unwrap();
        cases.push(docx_case(dir.path(), path));
    }
    let mut summary = Vec::new();
    let mut failed = 0;
    let mut totals = [0usize; 4];
    let mut docx_lines = 0;
    for case in &cases {
        let line = match &case.got {
            Err(e) => {
                failed += 1;
                let lost = ids(&case.want).len();
                totals[2] += lost;
                println!("✗ {}\n  every id lost ({lost}): {e}\n", case.name);
                format!("✗ {}: no Markdown back, {lost} ids lost: {e}", case.name)
            }
            Ok(got) => {
                let r = verdicts(&case.want, got);
                for (t, n) in totals.iter_mut().zip([r.correct, r.wrong, r.lost, r.new]) {
                    *t += n;
                }
                let exact = case.want == *got && case.docx.is_empty();
                failed += usize::from(!exact);
                docx_lines += case.docx.len();
                let counts = format!(
                    "{} correct, {} wrong, {} lost, {} new; docx: {} differences",
                    r.correct,
                    r.wrong,
                    r.lost,
                    r.new,
                    case.docx.len()
                );
                println!("{} {}: {counts}", if exact { "✓" } else { "✗" }, case.name);
                for l in r.lines.iter().chain(&case.docx) {
                    println!("{l}");
                }
                if !exact {
                    println!("  text: {}", difference(&case.want, got));
                }
                println!();
                format!("{} {}: {counts}", if exact { "✓" } else { "✗" }, case.name)
            }
        };
        summary.push(line);
    }
    let [correct, wrong, lost, new] = totals;
    assert!(
        failed == 0,
        "{failed} of {} round trips are not exact; ids: {correct} correct, {wrong} wrong, {lost} lost, {new} new; docx: {docx_lines} differences\n{}\n(every verdict is printed above)",
        cases.len(),
        summary.join("\n")
    );
}

/// A committed view of a docx fixture is what `jubarte read` prints for it
/// today: the Markdown fixtures stay in the dialect the view writes.
/// (`views.txt` names them.)
#[test]
fn committed_views_match_the_view() {
    let mut stale = Vec::new();
    let mut sources: Vec<(String, PathBuf)> = view_sources().into_iter().collect();
    sources.sort();
    for (md, docx) in &sources {
        let dir = tempfile::tempdir().unwrap();
        std::fs::copy(docx, dir.path().join("source.docx")).unwrap();
        let got = view(dir.path(), "source.docx").unwrap();
        let want = comparable(&std::fs::read_to_string(Path::new(FIXTURES).join(md)).unwrap());
        if want != got {
            stale.push(format!("{md}: {}", difference(&want, &got)));
        }
    }
    assert!(stale.is_empty(), "stale views:\n{}", stale.join("\n"));
}

#[test]
fn ids_name_blocks_notes_and_yaml_entries() {
    let view = "---\nview: tracked  # c\nauthors:\n  AL: Ana Lima  # 1\n---\n\n<!-- p0 left -->\n\
                a {~~x~>y~~}{>>r0 @AL<<}{>>r1 @AL<<} b{== c==}{>>c2 @AL: hi<<}\n\n<!-- p1-p2 empty -->\n";
    let keys: Vec<String> = ids(view).into_iter().map(|(k, _)| k).collect();
    assert_eq!(
        keys,
        [
            "yaml view",
            "yaml authors.AL",
            "p0",
            "p1-p2",
            "r0",
            "r1",
            "c2"
        ]
    );
    let notes: HashMap<String, String> = ids(view).into_iter().collect();
    assert_eq!(notes["r1"], "{~~x~>y~~}{>>r1 @AL<<}");
    assert_eq!(notes["c2"], "{== c==}{>>c2 @AL: hi<<}");
    let r = verdicts(
        view,
        &view.replace("hi<<}", "ho<<}").replace("{>>r1 @AL<<}", ""),
    );
    assert_eq!(
        (r.correct, r.wrong, r.lost, r.new),
        (4, 2, 1, 0),
        "{:#?}",
        r.lines
    );
}
