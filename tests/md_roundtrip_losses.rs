// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What the Markdown round trip lost (tests/md_fixed_point.rs, run of
//! 2026-10-14), one shape each: the view written to Word and read back
//! prints the same line.

use std::path::Path;
use std::process::Command;

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

const HEADER: &str = "---\n\
view: tracked\n\
authors:\n  SW: S Wheeler                    # 2026-09-25T20:04:00Z\n\
---\n\n";

/// `body` under a view header, written to Word and read back: the body
/// lines of the second view, and the docx.
fn round_trip(body: &str) -> (String, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.md"), format!("{HEADER}{body}")).unwrap();
    jubarte(
        dir.path(),
        &[
            "convert",
            "in.md",
            "-o",
            "out.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let view = jubarte(dir.path(), &["read", "out.docx", "--no-page-markers"]);
    let body = view
        .splitn(3, "---\n")
        .nth(2)
        .unwrap_or_default()
        .to_string();
    (body, std::fs::read(dir.path().join("out.docx")).unwrap())
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut text).unwrap();
    text
}

/// A link inside one tracked change is one change: Word's own shape, a
/// HYPERLINK field inside the one `w:del`, not a `w:hyperlink` that parts
/// the mark in two (`r213` read back as `r213+361`).
#[test]
fn a_link_inside_a_change_stays_one_change() {
    let line =
        "{--Title: [Ida Greaves](https://eur.on.worldcat.org/oclc/1381732921)--}{>>r213 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    assert!(body.contains(&format!("{line}\n")), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    assert!(
        document.contains("HYPERLINK &quot;https://eur.on.worldcat.org/oclc/1381732921&quot;")
            || document.contains("HYPERLINK \"https://eur.on.worldcat.org/oclc/1381732921\""),
        "{document}"
    );
    assert!(document.contains("<w:delInstrText"), "{document}");
    // An inserted link the same; a plain link stays a hyperlink element.
    let inserted = "{++See [the site](https://example.com) now++}{>>r4 @SW<<}";
    let (body, docx) = round_trip(&format!(
        "<!-- p0 -->\n{inserted}\n\n<!-- p1 -->\n[plain](https://example.org)\n"
    ));
    assert!(body.contains(&format!("{inserted}\n")), "{body}");
    assert!(body.contains("[plain](https://example.org)\n"), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert!(document.contains("<w:instrText"), "{document}");
    assert_eq!(document.matches("<w:hyperlink ").count(), 1, "{document}");
}

/// A heading in a bulleted list prints its bullet before its text
/// (`## - Economics`); the bullet is the list's, not text to write.
#[test]
fn a_bulleted_heading_keeps_one_bullet() {
    let body = "<!-- p0 bullet -->\n## - {--Economics & business--}{>>r211 @SW<<}\n";
    let (got, docx) = round_trip(body);
    assert!(
        got.contains("## - {--Economics & business--}{>>r211 @SW<<}\n"),
        "{got}"
    );
    assert!(!got.contains("## - - "), "{got}");
    assert!(!part(&docx, "word/document.xml").contains(">- <"));
}
