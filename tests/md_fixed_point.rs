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

use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "tests/fixtures/md-roundtrip";

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
    markdown
        .lines()
        .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn to_docx(dir: &Path, markdown: &str, name: &str) -> Result<String, String> {
    let md = format!("{name}.md");
    let docx = format!("{name}.docx");
    std::fs::write(dir.join(&md), markdown).map_err(|e| e.to_string())?;
    jubarte(dir, &["convert", &md, "-o", &docx, "--force"])?;
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
    format!(
        "{} lines -> {} lines\n{}",
        a.len(),
        b.len(),
        out.join("\n")
    )
}

/// md → docx → md must give the Markdown back.
fn markdown_case(dir: &Path, path: &Path) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let want = comparable(&source);
    let docx = to_docx(dir, &source, "md")?;
    let got = view(dir, &docx)?;
    if want == got {
        Ok(())
    } else {
        Err(difference(&want, &got))
    }
}

/// docx → md → docx → md must print the same Markdown twice.
fn docx_case(dir: &Path, path: &Path) -> Result<(), String> {
    let copy = dir.join("source.docx");
    std::fs::copy(path, &copy).map_err(|e| format!("{}: {e}", path.display()))?;
    let first = view(dir, "source.docx")?;
    let docx = to_docx(dir, &first, "back")?;
    let second = view(dir, &docx)?;
    if first != second {
        return Err(difference(&first, &second));
    }
    Ok(())
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
    let mut cases: Vec<(String, Result<(), String>)> = Vec::new();
    let mut markdown: Vec<PathBuf> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    markdown.sort();
    assert!(!markdown.is_empty(), "no Markdown fixtures in {FIXTURES}");
    for path in &markdown {
        let dir = tempfile::tempdir().unwrap();
        cases.push((
            format!("md -> docx -> md   {}", path.display()),
            markdown_case(dir.path(), path),
        ));
    }
    let docx = docx_list();
    assert!(!docx.is_empty(), "no docx fixtures in {FIXTURES}/docx.txt");
    for path in &docx {
        let dir = tempfile::tempdir().unwrap();
        cases.push((
            format!("docx -> md -> docx -> md   {}", path.display()),
            docx_case(dir.path(), path),
        ));
    }
    let failed: Vec<String> = cases
        .iter()
        .filter_map(|(name, r)| r.as_ref().err().map(|e| format!("✗ {name}\n    {e}")))
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} round trips are not exact:\n\n{}",
        failed.len(),
        cases.len(),
        failed.join("\n\n")
    );
}

/// A committed view of a docx fixture is what `jubarte read` prints for it
/// today: the Markdown fixtures stay in the dialect the view writes.
#[test]
fn committed_views_match_the_view() {
    for (md, docx) in [
        ("received.md", "tests/fixtures/agent-view/received.docx"),
        (
            "potpourri.md",
            "tests/corpus/neurotic_docx_bench/corpus/word/tracking_without_comments/docx/32997dd3c9_potpourritest.docx",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::copy(docx, dir.path().join("source.docx")).unwrap();
        let got = view(dir.path(), "source.docx").unwrap();
        let want = comparable(&std::fs::read_to_string(Path::new(FIXTURES).join(md)).unwrap());
        assert!(want == got, "{md}: {}", difference(&want, &got));
    }
}
